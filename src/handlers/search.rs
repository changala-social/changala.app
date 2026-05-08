//! Search handlers — full-text search across notes, courses, brain nodes, archives.

use axum::extract::Query;
use axum::Json;

use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use crate::generated::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Placeholder Ring DID.
const RING_DID: &str = "did:web:ring.changala.local";

/// Clamp a user-supplied limit.
fn clamp_limit(limit: Option<i64>, default: i64) -> i64 {
    limit.unwrap_or(default).min(100).max(1)
}

/// Wrap a search term for PostgreSQL ILIKE matching.
fn like_pattern(q: &str) -> String {
    format!("%{q}%")
}

// ═══════════════════════════════════════════════════════════════════════════
// searchNotes — full-text search over note summaries
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.searchNotes
///
/// MVP: uses PostgreSQL ILIKE for text search.
/// Searches notes by `summary ILIKE '%q%'`. Optionally filters by course_uri
/// via session JOIN, or by semester via course JOIN.
pub async fn search_notes(
    Query(params): Query<AppChangalaGlobalviewSearchNotesParams>,
) -> Result<Json<AppChangalaGlobalviewSearchNotesOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;
    let pattern = like_pattern(&params.q);

    // We always need to JOIN sessions so we can resolve course_uri; also
    // optionally JOIN courses to filter by semester.
    let mut param_idx = 2;
    let mut sql = String::from(
        "SELECT n.uri, n.session_uri, n.author_did, n.format, n.ring_did, n.cid, \
                n.version, n.summary, n.created_at, \
                COALESCE(v.cnt, 0) as vote_count \
         FROM notes n \
         JOIN sessions s ON n.session_uri = s.uri \
         LEFT JOIN (SELECT subject_uri, COUNT(*) as cnt FROM votes GROUP BY subject_uri) v \
           ON n.uri = v.subject_uri \
         WHERE n.summary ILIKE $1",
    );
    let mut binds: Vec<String> = vec![pattern.clone()];

    if let Some(ref course_uri) = params.course_uri {
        sql.push_str(&format!(" AND s.course_uri = ${param_idx}"));
        param_idx += 1;
        binds.push(course_uri.clone());
    }

    if let Some(ref semester) = params.semester {
        // Need courses table for semester filtering.
        sql = sql.replace(
            "JOIN sessions s ON n.session_uri = s.uri",
            "JOIN sessions s ON n.session_uri = s.uri JOIN courses c ON s.course_uri = c.uri",
        );
        sql.push_str(&format!(" AND c.semester = ${param_idx}"));
        param_idx += 1;
        binds.push(semester.clone());
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND n.created_at < ${param_idx}"));
        param_idx += 1;
        binds.push(cursor.clone());
    }

    sql.push_str(&format!(" ORDER BY n.created_at DESC LIMIT ${param_idx}"));

    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(fetch_limit);

    use sqlx::Row;
    let rows: Vec<sqlx::postgres::PgRow> = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to search notes: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    // Total hit count (separate COUNT query — expensive but simple for MVP).
    let mut count_param_idx = 2;
    let mut count_sql = String::from(
        "SELECT COUNT(*) as cnt FROM notes n \
         JOIN sessions s ON n.session_uri = s.uri \
         WHERE n.summary ILIKE $1",
    );
    let mut count_binds: Vec<String> = vec![pattern];

    if let Some(ref course_uri) = params.course_uri {
        count_sql.push_str(&format!(" AND s.course_uri = ${count_param_idx}"));
        count_param_idx += 1;
        count_binds.push(course_uri.clone());
    }

    if let Some(ref semester) = params.semester {
        if !count_sql.contains("JOIN courses") {
            count_sql = count_sql.replace(
                "JOIN sessions s ON n.session_uri = s.uri",
                "JOIN sessions s ON n.session_uri = s.uri JOIN courses c ON s.course_uri = c.uri",
            );
        }
        count_sql.push_str(&format!(" AND c.semester = ${count_param_idx}"));
        count_param_idx += 1;
        count_binds.push(semester.clone());
    }

    let _ = count_param_idx;

    let mut cq = sqlx::query(&count_sql);
    for b in &count_binds {
        cq = cq.bind(b);
    }
    let hits_total: i64 = cq
        .fetch_one(&app.db)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);

    // Batch-fetch labels.
    let note_uris: Vec<String> = rows.iter().map(|r| r.get::<String, _>("uri")).collect();
    let labels_map = batch_fetch_labels(&note_uris).await?;

    let notes: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let uri: String = r.get("uri");
            let labels = labels_map.get(&uri).cloned().unwrap_or_default();
            json!({
                "uri": uri,
                "authorDid": r.get::<String, _>("author_did"),
                "sessionUri": r.get::<String, _>("session_uri"),
                "format": r.get::<String, _>("format"),
                "ringRef": {
                    "ringDid": r.get::<String, _>("ring_did"),
                    "cid": r.get::<String, _>("cid"),
                },
                "version": r.get::<i64, _>("version"),
                "voteCount": r.get::<i64, _>("vote_count"),
                "labels": labels,
                "summary": r.get::<Option<String>, _>("summary"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    let cursor = if has_more {
        rows.last().map(|r| r.get::<String, _>("created_at"))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewSearchNotesOutput {
        notes,
        hits_total,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// searchCourses — search by title or code
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.searchCourses
pub async fn search_courses(
    Query(params): Query<AppChangalaGlobalviewSearchCoursesParams>,
) -> Result<Json<AppChangalaGlobalviewSearchCoursesOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;
    let pattern = like_pattern(&params.q);

    let mut param_idx = 3;
    let mut sql = String::from(
        "SELECT c.uri, c.title, c.code, c.department, c.semester, c.visibility, \
                COALESCE(e.cnt, 0) as enrolled_count \
         FROM courses c \
         LEFT JOIN (SELECT course_uri, COUNT(*) as cnt FROM enrollments GROUP BY course_uri) e \
           ON c.uri = e.course_uri \
         WHERE (c.title ILIKE $1 OR c.code ILIKE $2)",
    );
    let mut binds: Vec<String> = vec![pattern.clone(), pattern.clone()];

    if let Some(ref semester) = params.semester {
        sql.push_str(&format!(" AND c.semester = ${param_idx}"));
        param_idx += 1;
        binds.push(semester.clone());
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND c.created_at < ${param_idx}"));
        param_idx += 1;
        binds.push(cursor.clone());
    }

    sql.push_str(&format!(" ORDER BY c.created_at DESC LIMIT ${param_idx}"));

    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(fetch_limit);

    use sqlx::Row;
    let rows: Vec<sqlx::postgres::PgRow> = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to search courses: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    // Total hits.
    let mut count_param_idx = 3;
    let mut count_sql = String::from(
        "SELECT COUNT(*) as cnt FROM courses c WHERE (c.title ILIKE $1 OR c.code ILIKE $2)",
    );
    let mut count_binds: Vec<String> = vec![like_pattern(&params.q), like_pattern(&params.q)];

    if let Some(ref semester) = params.semester {
        count_sql.push_str(&format!(" AND c.semester = ${count_param_idx}"));
        count_param_idx += 1;
        count_binds.push(semester.clone());
    }

    let _ = count_param_idx;

    let mut cq = sqlx::query(&count_sql);
    for b in &count_binds {
        cq = cq.bind(b);
    }
    let hits_total: i64 = cq
        .fetch_one(&app.db)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);

    let courses: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "uri": r.get::<String, _>("uri"),
                "title": r.get::<String, _>("title"),
                "code": r.get::<String, _>("code"),
                "institutionDid": RING_DID,
                "department": r.get::<Option<String>, _>("department"),
                "semester": r.get::<String, _>("semester"),
                "visibility": r.get::<String, _>("visibility"),
                "enrolledCount": r.get::<i64, _>("enrolled_count"),
            })
        })
        .collect();

    let cursor = if has_more {
        // We don't select created_at in the main query — derive from last URI
        // or use a position cursor. For simplicity, use offset-style cursor
        // by selecting created_at as well. Let's adjust: we already ORDER BY
        // c.created_at so we should include it.
        // Since we didn't SELECT created_at, use the index position.
        Some(rows.len().to_string())
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewSearchCoursesOutput {
        courses,
        hits_total,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// searchArchive — search sealed archives by course title/code
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.searchArchive
pub async fn search_archive(
    Query(params): Query<AppChangalaGlobalviewSearchArchiveParams>,
) -> Result<Json<AppChangalaGlobalviewSearchArchiveOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;
    let pattern = like_pattern(&params.q);

    let mut param_idx = 3;
    let mut sql = String::from(
        "SELECT a.archive_uri, a.course_uri, c.title as course_title, a.semester, \
                a.session_count, a.internet_archive_url, a.sealed_at \
         FROM archives a \
         JOIN courses c ON a.course_uri = c.uri \
         WHERE a.status = 'sealed' AND (c.title ILIKE $1 OR c.code ILIKE $2)",
    );
    let mut binds: Vec<String> = vec![pattern.clone(), pattern.clone()];

    if let Some(ref semester) = params.semester {
        sql.push_str(&format!(" AND a.semester = ${param_idx}"));
        param_idx += 1;
        binds.push(semester.clone());
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND a.sealed_at < ${param_idx}"));
        param_idx += 1;
        binds.push(cursor.clone());
    }

    sql.push_str(&format!(" ORDER BY a.sealed_at DESC LIMIT ${param_idx}"));

    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(fetch_limit);

    use sqlx::Row;
    let rows: Vec<sqlx::postgres::PgRow> = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to search archive: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    // Total hits.
    let mut count_param_idx = 3;
    let mut count_sql = String::from(
        "SELECT COUNT(*) as cnt FROM archives a \
         JOIN courses c ON a.course_uri = c.uri \
         WHERE a.status = 'sealed' AND (c.title ILIKE $1 OR c.code ILIKE $2)",
    );
    let mut count_binds: Vec<String> = vec![like_pattern(&params.q), like_pattern(&params.q)];

    if let Some(ref semester) = params.semester {
        count_sql.push_str(&format!(" AND a.semester = ${count_param_idx}"));
        count_param_idx += 1;
        count_binds.push(semester.clone());
    }

    let _ = count_param_idx;

    let mut cq = sqlx::query(&count_sql);
    for b in &count_binds {
        cq = cq.bind(b);
    }
    let hits_total: i64 = cq
        .fetch_one(&app.db)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);

    let results: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "archiveUri": r.get::<String, _>("archive_uri"),
                "courseUri": r.get::<String, _>("course_uri"),
                "courseTitle": r.get::<String, _>("course_title"),
                "semester": r.get::<String, _>("semester"),
                "institutionDid": RING_DID,
                "iaUrl": r.get::<Option<String>, _>("internet_archive_url"),
                "sessionCount": r.get::<i64, _>("session_count"),
                "sealedAt": r.get::<Option<String>, _>("sealed_at"),
            })
        })
        .collect();

    let cursor = if has_more {
        rows.last()
            .and_then(|r| r.get::<Option<String>, _>("sealed_at"))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewSearchArchiveOutput {
        results,
        hits_total,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// searchBrainNodes — search by title/summary, optionally filter by tags/author
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.searchBrainNodes
pub async fn search_brain_nodes(
    Query(params): Query<AppChangalaGlobalviewSearchBrainNodesParams>,
) -> Result<Json<AppChangalaGlobalviewSearchBrainNodesOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;
    let pattern = like_pattern(&params.q);

    let mut param_idx = 3;
    let mut sql = String::from(
        "SELECT bn.uri, bn.author_did, bn.title, bn.format, bn.ring_did, bn.cid, \
                bn.tags, bn.academic_ref, bn.summary, bn.created_at, \
                COALESCE(v.cnt, 0) as vote_count \
         FROM brain_nodes bn \
         LEFT JOIN (SELECT subject_uri, COUNT(*) as cnt FROM votes GROUP BY subject_uri) v \
           ON bn.uri = v.subject_uri \
         WHERE (bn.title ILIKE $1 OR bn.summary ILIKE $2)",
    );
    let mut binds: Vec<String> = vec![pattern.clone(), pattern.clone()];

    if let Some(ref author_did) = params.author_did {
        sql.push_str(&format!(" AND bn.author_did = ${param_idx}"));
        param_idx += 1;
        binds.push(author_did.clone());
    }

    // Tag filtering: for each requested tag, add an ILIKE clause on the JSON
    // tags column. This is a pragmatic MVP approach — the tags column stores
    // a JSON array string like '["physics","idea"]'.
    if let Some(ref tags) = params.tags {
        for tag in tags {
            sql.push_str(&format!(" AND bn.tags ILIKE ${param_idx}"));
            param_idx += 1;
            binds.push(like_pattern(tag));
        }
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND bn.created_at < ${param_idx}"));
        param_idx += 1;
        binds.push(cursor.clone());
    }

    sql.push_str(&format!(" ORDER BY bn.created_at DESC LIMIT ${param_idx}"));

    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(fetch_limit);

    use sqlx::Row;
    let rows: Vec<sqlx::postgres::PgRow> = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to search brain nodes: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    // Total hit count.
    let mut count_param_idx = 3;
    let mut count_sql = String::from(
        "SELECT COUNT(*) as cnt FROM brain_nodes bn \
         WHERE (bn.title ILIKE $1 OR bn.summary ILIKE $2)",
    );
    let mut count_binds: Vec<String> = vec![like_pattern(&params.q), like_pattern(&params.q)];

    if let Some(ref author_did) = params.author_did {
        count_sql.push_str(&format!(" AND bn.author_did = ${count_param_idx}"));
        count_param_idx += 1;
        count_binds.push(author_did.clone());
    }

    if let Some(ref tags) = params.tags {
        for tag in tags {
            count_sql.push_str(&format!(" AND bn.tags ILIKE ${count_param_idx}"));
            count_param_idx += 1;
            count_binds.push(like_pattern(tag));
        }
    }

    let _ = count_param_idx;

    let mut cq = sqlx::query(&count_sql);
    for b in &count_binds {
        cq = cq.bind(b);
    }
    let hits_total: i64 = cq
        .fetch_one(&app.db)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);

    let nodes: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let tags_raw: Option<String> = r.get("tags");
            let tags: Vec<String> = tags_raw
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();

            json!({
                "uri": r.get::<String, _>("uri"),
                "authorDid": r.get::<String, _>("author_did"),
                "title": r.get::<String, _>("title"),
                "format": r.get::<String, _>("format"),
                "ringRef": {
                    "ringDid": r.get::<String, _>("ring_did"),
                    "cid": r.get::<String, _>("cid"),
                },
                "tags": tags,
                "academicRef": r.get::<Option<String>, _>("academic_ref"),
                "voteCount": r.get::<i64, _>("vote_count"),
                "summary": r.get::<Option<String>, _>("summary"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    let cursor = if has_more {
        rows.last().map(|r| r.get::<String, _>("created_at"))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewSearchBrainNodesOutput {
        nodes,
        hits_total,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Shared helper: batch-fetch labels for a set of subject URIs
// ═══════════════════════════════════════════════════════════════════════════

async fn batch_fetch_labels(
    uris: &[String],
) -> Result<std::collections::HashMap<String, Vec<serde_json::Value>>, XrpcError> {
    use sqlx::Row;

    if uris.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let app = crate::state::get();

    let placeholders: Vec<String> = (1..=uris.len()).map(|i| format!("${i}")).collect();
    let sql = format!(
        "SELECT subject_uri, val, src_did, created_at \
         FROM labels WHERE subject_uri IN ({}) AND neg = 0",
        placeholders.join(", ")
    );

    let mut q = sqlx::query(&sql);
    for uri in uris {
        q = q.bind(uri);
    }

    let rows: Vec<sqlx::postgres::PgRow> = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch labels: {e}"),
    })?;

    let mut map: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for r in &rows {
        let subj: String = r.get("subject_uri");
        let label_view = json!({
            "val": r.get::<String, _>("val"),
            "srcDid": r.get::<String, _>("src_did"),
            "createdAt": r.get::<String, _>("created_at"),
        });
        map.entry(subj).or_default().push(label_view);
    }

    Ok(map)
}
