//! Feed handlers — course feeds, social feeds, brain feeds, trending, archives.

use axum::extract::{Query, State};
use axum::Json;

use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use changala_shared::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Placeholder Ring DID used until real Ring identity is provisioned.
const RING_DID: &str = "did:web:ring.changala.local";

/// Clamp a user-supplied limit into a sane range.
fn clamp_limit(limit: Option<i64>, default: i64) -> i64 {
    limit.unwrap_or(default).clamp(1, 100)
}

/// Map a session `status` column value to a courseFeedItem `eventType`.
fn session_status_to_event_type(status: &str) -> &str {
    match status {
        "live" => "sessionOpened",
        "ended" => "sessionEnded",
        "cancelled" => "sessionCancelled",
        "rescheduled" => "sessionRescheduled",
        "scheduled" => "sessionScheduled",
        _ => "sessionUpdated",
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// getNotes — notes for a session, ranked by votes or recency
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getNotes
pub async fn get_notes(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetNotesParams>,
) -> Result<Json<AppChangalaGlobalviewGetNotesOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = clamp_limit(params.limit, 50);
    let sort_by = params.sort_by.as_deref().unwrap_or("votes");

    // Decide ORDER BY clause and cursor column.
    let (cursor_col, order_clause) = match sort_by {
        "created_at" | "createdAt" => ("n.created_at", "ORDER BY n.created_at DESC"),
        _ => ("vote_count", "ORDER BY vote_count DESC, n.created_at DESC"),
    };

    let fetch_limit = limit + 1;

    let mut sql = "SELECT n.uri, n.session_uri, n.author_did, n.format, n.ring_did, n.cid, \
                n.version::BIGINT, n.summary, n.created_at, \
                COALESCE(v.cnt, 0) as vote_count \
         FROM notes n \
         LEFT JOIN (SELECT subject_uri, COUNT(*) as cnt FROM votes GROUP BY subject_uri) v \
           ON n.uri = v.subject_uri \
         WHERE n.session_uri = $1"
        .to_string();

    let mut binds: Vec<String> = vec![params.session_uri.clone()];
    let mut param_idx: i32 = 2;

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND {cursor_col} < ${param_idx}"));
        binds.push(cursor.clone());
        param_idx += 1;
    }

    sql.push_str(&format!(" {order_clause} LIMIT ${param_idx}"));

    // We cannot use query_as with a dynamic column list, so use query() + Row.
    let mut query = sqlx::query(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(fetch_limit);

    let rows: Vec<sqlx::postgres::PgRow> =
        query.fetch_all(&app.db).await.map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to fetch notes: {e}"),
        })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    // Batch-fetch labels for all returned notes.
    use sqlx::Row;

    let note_uris: Vec<String> = rows.iter().map(|r| r.get::<String, _>("uri")).collect();

    let labels_map = if note_uris.is_empty() {
        std::collections::HashMap::new()
    } else {
        let placeholders: Vec<String> = (1..=note_uris.len()).map(|i| format!("${i}")).collect();
        let labels_sql = format!(
            "SELECT subject_uri, val, src_did, created_at \
             FROM labels WHERE subject_uri IN ({}) AND neg = FALSE",
            placeholders.join(", ")
        );
        let mut lq = sqlx::query(&labels_sql);
        for uri in &note_uris {
            lq = lq.bind(uri);
        }
        let label_rows: Vec<sqlx::postgres::PgRow> =
            lq.fetch_all(&app.db).await.map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to fetch labels: {e}"),
            })?;

        let mut map: std::collections::HashMap<String, Vec<serde_json::Value>> =
            std::collections::HashMap::new();
        for lr in &label_rows {
            let subj: String = lr.get("subject_uri");
            let label_view = json!({
                "val": lr.get::<String, _>("val"),
                "srcDid": lr.get::<String, _>("src_did"),
                "createdAt": lr.get::<String, _>("created_at"),
            });
            map.entry(subj).or_default().push(label_view);
        }
        map
    };

    let mut notes: Vec<serde_json::Value> = Vec::with_capacity(rows.len());
    for r in rows {
        let uri: String = r.get("uri");
        let labels = labels_map.get(&uri).cloned().unwrap_or_default();
        let vote_count: i64 = r.get("vote_count");

        notes.push(json!({
            "uri": uri,
            "authorDid": r.get::<String, _>("author_did"),
            "sessionUri": r.get::<String, _>("session_uri"),
            "format": r.get::<String, _>("format"),
            "ringRef": {
                "ringDid": r.get::<String, _>("ring_did"),
                "cid": r.get::<String, _>("cid"),
            },
            "version": r.get::<i64, _>("version"),
            "voteCount": vote_count,
            "labels": labels,
            "summary": r.get::<Option<String>, _>("summary"),
            "createdAt": r.get::<String, _>("created_at"),
        }));
    }

    // Build cursor from the last item's sort key.
    let cursor = if has_more {
        let last = rows.last().unwrap();
        Some(match sort_by {
            "created_at" | "createdAt" => last.get::<String, _>("created_at"),
            _ => last.get::<i64, _>("vote_count").to_string(),
        })
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewGetNotesOutput { notes, cursor }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getCourseFeed — combined session events + note activity for a course
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getCourseFeed
pub async fn get_course_feed(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetCourseFeedParams>,
) -> Result<Json<AppChangalaGlobalviewGetCourseFeedOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;

    // ----- session events -----
    let mut session_sql = String::from(
        "SELECT uri, status, created_by, created_at \
         FROM sessions WHERE course_uri = $1",
    );
    let mut session_binds: Vec<String> = vec![params.course_uri.clone()];

    if let Some(ref cursor) = params.cursor {
        session_sql.push_str(" AND created_at < $2");
        session_binds.push(cursor.clone());
    }
    session_sql.push_str(" ORDER BY created_at DESC");

    let mut sq = sqlx::query(&session_sql);
    for b in &session_binds {
        sq = sq.bind(b);
    }
    let session_rows: Vec<sqlx::postgres::PgRow> =
        sq.fetch_all(&app.db).await.map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to fetch session events: {e}"),
        })?;

    // ----- note events -----
    let mut note_sql = String::from(
        "SELECT n.uri, n.author_did, n.session_uri, n.created_at \
         FROM notes n \
         JOIN sessions s ON n.session_uri = s.uri \
         WHERE s.course_uri = $1",
    );
    let mut note_binds: Vec<String> = vec![params.course_uri.clone()];

    if let Some(ref cursor) = params.cursor {
        note_sql.push_str(" AND n.created_at < $2");
        note_binds.push(cursor.clone());
    }
    note_sql.push_str(" ORDER BY n.created_at DESC");

    let mut nq = sqlx::query(&note_sql);
    for b in &note_binds {
        nq = nq.bind(b);
    }
    let note_rows: Vec<sqlx::postgres::PgRow> =
        nq.fetch_all(&app.db).await.map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to fetch note events: {e}"),
        })?;

    use sqlx::Row;

    // Combine into a single feed, sorted by created_at DESC.
    let mut feed_items: Vec<serde_json::Value> = Vec::new();

    for r in &session_rows {
        let status: String = r.get("status");
        feed_items.push(json!({
            "eventType": session_status_to_event_type(&status),
            "actorDid": r.get::<String, _>("created_by"),
            "subjectUri": r.get::<String, _>("uri"),
            "sessionUri": r.get::<String, _>("uri"),
            "createdAt": r.get::<String, _>("created_at"),
        }));
    }

    for r in &note_rows {
        feed_items.push(json!({
            "eventType": "noteCreated",
            "actorDid": r.get::<String, _>("author_did"),
            "subjectUri": r.get::<String, _>("uri"),
            "sessionUri": r.get::<String, _>("session_uri"),
            "createdAt": r.get::<String, _>("created_at"),
        }));
    }

    // Sort combined items by createdAt DESC.
    feed_items.sort_by(|a, b| {
        let ta = a["createdAt"].as_str().unwrap_or("");
        let tb = b["createdAt"].as_str().unwrap_or("");
        tb.cmp(ta)
    });

    // Paginate.
    let has_more = feed_items.len() as i64 > fetch_limit - 1;
    feed_items.truncate(limit as usize);

    let cursor = if has_more {
        feed_items
            .last()
            .and_then(|v| v["createdAt"].as_str().map(String::from))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewGetCourseFeedOutput {
        feed: feed_items,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getSocialFeed — recent activity, filterable by mode
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getSocialFeed
pub async fn get_social_feed(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetSocialFeedParams>,
) -> Result<Json<AppChangalaGlobalviewGetSocialFeedOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;
    let mode = params.mode.as_deref().unwrap_or("all");

    use sqlx::Row;
    let mut feed_items: Vec<serde_json::Value> = Vec::new();

    // --- academic notes ---
    if mode == "all" || mode == "academic" {
        let mut sql = String::from(
            "SELECT n.uri, n.author_did, n.session_uri, n.created_at, s.course_uri \
             FROM notes n \
             JOIN sessions s ON n.session_uri = s.uri",
        );
        let mut binds: Vec<String> = Vec::new();
        let mut param_idx: i32 = 1;

        if let Some(ref cursor) = params.cursor {
            sql.push_str(&format!(" WHERE n.created_at < ${param_idx}"));
            binds.push(cursor.clone());
            param_idx += 1;
        }
        sql.push_str(&format!(" ORDER BY n.created_at DESC LIMIT ${param_idx}"));

        let mut q = sqlx::query(&sql);
        for b in &binds {
            q = q.bind(b);
        }
        q = q.bind(fetch_limit);

        let rows: Vec<sqlx::postgres::PgRow> =
            q.fetch_all(&app.db).await.map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to fetch notes for social feed: {e}"),
            })?;

        for r in &rows {
            feed_items.push(json!({
                "eventType": "noteCreated",
                "actorDid": r.get::<String, _>("author_did"),
                "subjectUri": r.get::<String, _>("uri"),
                "courseUri": r.get::<String, _>("course_uri"),
                "sessionUri": r.get::<String, _>("session_uri"),
                "createdAt": r.get::<String, _>("created_at"),
            }));
        }
    }

    // --- brain nodes ---
    if mode == "all" || mode == "brain" {
        let mut sql = String::from(
            "SELECT uri, author_did, academic_ref, created_at \
             FROM brain_nodes",
        );
        let mut binds: Vec<String> = Vec::new();
        let mut param_idx: i32 = 1;

        if let Some(ref cursor) = params.cursor {
            sql.push_str(&format!(" WHERE created_at < ${param_idx}"));
            binds.push(cursor.clone());
            param_idx += 1;
        }
        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${param_idx}"));

        let mut q = sqlx::query(&sql);
        for b in &binds {
            q = q.bind(b);
        }
        q = q.bind(fetch_limit);

        let rows: Vec<sqlx::postgres::PgRow> =
            q.fetch_all(&app.db).await.map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to fetch brain nodes for social feed: {e}"),
            })?;

        for r in &rows {
            feed_items.push(json!({
                "eventType": "brainNodeCreated",
                "actorDid": r.get::<String, _>("author_did"),
                "subjectUri": r.get::<String, _>("uri"),
                "createdAt": r.get::<String, _>("created_at"),
            }));
        }
    }

    // Sort combined by createdAt DESC.
    feed_items.sort_by(|a, b| {
        let ta = a["createdAt"].as_str().unwrap_or("");
        let tb = b["createdAt"].as_str().unwrap_or("");
        tb.cmp(ta)
    });

    let has_more = feed_items.len() as i64 > limit;
    feed_items.truncate(limit as usize);

    let cursor = if has_more {
        feed_items
            .last()
            .and_then(|v| v["createdAt"].as_str().map(String::from))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewGetSocialFeedOutput {
        feed: feed_items,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getBrainFeed — brain nodes with vote counts
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getBrainFeed
pub async fn get_brain_feed(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetBrainFeedParams>,
) -> Result<Json<AppChangalaGlobalviewGetBrainFeedOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;

    let mut sql = String::from(
        "SELECT bn.uri, bn.author_did, bn.title, bn.format, bn.ring_did, bn.cid, \
                bn.tags, bn.academic_ref, bn.summary, bn.created_at, \
                COALESCE(v.cnt, 0) as vote_count \
         FROM brain_nodes bn \
         LEFT JOIN (SELECT subject_uri, COUNT(*) as cnt FROM votes GROUP BY subject_uri) v \
           ON bn.uri = v.subject_uri",
    );

    let mut binds: Vec<String> = Vec::new();
    let mut param_idx: i32 = 1;

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" WHERE bn.created_at < ${param_idx}"));
        binds.push(cursor.clone());
        param_idx += 1;
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
        message: format!("Failed to fetch brain feed: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    let mut nodes: Vec<serde_json::Value> = Vec::with_capacity(rows.len());
    for r in rows {
        // Parse tags from JSON string — stored as e.g. '["physics","idea"]'.
        let tags_raw: Option<String> = r.get("tags");
        let tags: Vec<String> = tags_raw
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        nodes.push(json!({
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
        }));
    }

    let cursor = if has_more {
        rows.last().map(|r| r.get::<String, _>("created_at"))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewGetBrainFeedOutput {
        nodes,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getKeywordHistogram — per-session keyword histogram
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getKeywordHistogram
///
/// Materialises the keyword histogram for a single session. Returns keyword
/// texts with submission counts, total submission count, and window state.
pub async fn get_keyword_histogram(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetKeywordHistogramParams>,
) -> Result<Json<AppChangalaGlobalviewGetKeywordHistogramOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    use sqlx::Row;

    // Look up session to determine window state.
    let session_row =
        sqlx::query("SELECT status, keyword_window_expires_at FROM sessions WHERE uri = $1")
            .bind(&params.session_uri)
            .fetch_optional(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Session lookup failed: {e}"),
            })?
            .ok_or_else(|| XrpcError {
                name: XrpcErrorName::NotFound,
                message: format!("Session not found: {}", params.session_uri),
            })?;

    let status: String = session_row.get("status");
    let window_expires: Option<String> = session_row.get("keyword_window_expires_at");
    let now = chrono::Utc::now().to_rfc3339();

    let window_open = status == "live"
        || window_expires
            .as_deref()
            .map(|exp| exp > now.as_str())
            .unwrap_or(false);

    // Aggregate keywords for this session.
    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        "SELECT text, COUNT(*) as cnt \
         FROM keywords WHERE session_uri = $1 \
         GROUP BY text ORDER BY cnt DESC",
    )
    .bind(&params.session_uri)
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to build keyword histogram: {e}"),
    })?;

    let total_submissions: i64 = rows.iter().map(|r| r.get::<i64, _>("cnt")).sum();

    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "text": r.get::<String, _>("text"),
                "count": r.get::<i64, _>("cnt"),
            })
        })
        .collect();

    Ok(Json(AppChangalaGlobalviewGetKeywordHistogramOutput {
        session_uri: params.session_uri,
        entries,
        total_submissions,
        window_open,
        window_expires_at: if window_open { window_expires } else { None },
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getTrendingKeywords — keyword histogram across active sessions
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getTrendingKeywords
pub async fn get_trending_keywords(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetTrendingKeywordsParams>,
) -> Result<Json<AppChangalaGlobalviewGetTrendingKeywordsOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let within_hours = params.within_hours.unwrap_or(24);
    let limit = clamp_limit(params.limit, 20);

    let cutoff = (chrono::Utc::now() - chrono::Duration::hours(within_hours)).to_rfc3339();

    let sql = "SELECT k.text, COUNT(*) as cnt, k.session_uri, s.course_uri \
               FROM keywords k \
               JOIN sessions s ON k.session_uri = s.uri \
               WHERE k.created_at > $1 \
               GROUP BY k.text, k.session_uri, s.course_uri \
               ORDER BY cnt DESC \
               LIMIT $2";

    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(sql)
        .bind(&cutoff)
        .bind(limit)
        .fetch_all(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to fetch trending keywords: {e}"),
        })?;

    use sqlx::Row;
    let keywords: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "text": r.get::<String, _>("text"),
                "count": r.get::<i64, _>("cnt"),
                "sessionUri": r.get::<String, _>("session_uri"),
                "courseUri": r.get::<String, _>("course_uri"),
            })
        })
        .collect();

    Ok(Json(AppChangalaGlobalviewGetTrendingKeywordsOutput {
        keywords,
        computed_at: chrono::Utc::now().to_rfc3339(),
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getTrendingBrainTags — most-used tags across recent brain nodes
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getTrendingBrainTags
pub async fn get_trending_brain_tags(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetTrendingBrainTagsParams>,
) -> Result<Json<AppChangalaGlobalviewGetTrendingBrainTagsOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let within_days = params.within_days.unwrap_or(7);
    let limit = clamp_limit(params.limit, 20) as usize;

    let cutoff = (chrono::Utc::now() - chrono::Duration::days(within_days)).to_rfc3339();

    let sql = "SELECT uri, tags FROM brain_nodes WHERE created_at > $1";

    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(sql)
        .bind(&cutoff)
        .fetch_all(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to fetch brain nodes for trending tags: {e}"),
        })?;

    use sqlx::Row;

    // Count tag occurrences in-memory, keep a sample node URI per tag.
    let mut tag_counts: std::collections::HashMap<String, (i64, String)> =
        std::collections::HashMap::new();

    for r in &rows {
        let tags_raw: Option<String> = r.get("tags");
        let node_uri: String = r.get("uri");

        let tags: Vec<String> = tags_raw
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        for tag in tags {
            let entry = tag_counts
                .entry(tag)
                .or_insert_with(|| (0, node_uri.clone()));
            entry.0 += 1;
        }
    }

    // Sort by count DESC, take top `limit`.
    let mut sorted: Vec<(String, i64, String)> = tag_counts
        .into_iter()
        .map(|(tag, (count, sample))| (tag, count, sample))
        .collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
    sorted.truncate(limit);

    let tags: Vec<serde_json::Value> = sorted
        .into_iter()
        .map(|(tag, count, sample_uri)| {
            json!({
                "tag": tag,
                "count": count,
                "sampleNodeUri": sample_uri,
            })
        })
        .collect();

    Ok(Json(AppChangalaGlobalviewGetTrendingBrainTagsOutput {
        tags,
        computed_at: chrono::Utc::now().to_rfc3339(),
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getFollowedEnrollments — MVP: most popular courses by enrollment
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getFollowedEnrollments
///
/// MVP: Since we cannot read the ATProto social graph yet, returns the
/// most popular courses by total enrollment count as a proxy.
pub async fn get_followed_enrollments(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetFollowedEnrollmentsParams>,
) -> Result<Json<AppChangalaGlobalviewGetFollowedEnrollmentsOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = clamp_limit(params.limit, 20);

    let sql = "SELECT e.course_uri, c.title, COUNT(*) as cnt \
               FROM enrollments e \
               JOIN courses c ON e.course_uri = c.uri \
               GROUP BY e.course_uri, c.title \
               ORDER BY cnt DESC \
               LIMIT $1";

    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(sql)
        .bind(limit)
        .fetch_all(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to fetch followed enrollments: {e}"),
        })?;

    use sqlx::Row;
    let enrollments: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "courseUri": r.get::<String, _>("course_uri"),
                "courseTitle": r.get::<String, _>("title"),
                "institutionDid": RING_DID,
                "followedCount": r.get::<i64, _>("cnt"),
            })
        })
        .collect();

    Ok(Json(AppChangalaGlobalviewGetFollowedEnrollmentsOutput {
        enrollments,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// getGlobalArchiveFeed — sealed archives, optionally filtered by institution
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getGlobalArchiveFeed
pub async fn get_global_archive_feed(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetGlobalArchiveFeedParams>,
) -> Result<Json<AppChangalaGlobalviewGetGlobalArchiveFeedOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;

    let mut sql = String::from(
        "SELECT a.archive_uri, a.course_uri, c.title as course_title, a.semester, \
                a.session_count::BIGINT, a.internet_archive_url, a.sealed_at \
         FROM archives a \
         JOIN courses c ON a.course_uri = c.uri \
         WHERE a.status = 'sealed'",
    );

    let mut binds: Vec<String> = Vec::new();
    let mut param_idx: i32 = 1;

    // institution_did filtering is a placeholder — in federated mode this
    // would filter by Ring DID. For single-instance MVP it's a no-op.
    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND a.sealed_at < ${param_idx}"));
        binds.push(cursor.clone());
        param_idx += 1;
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
        message: format!("Failed to fetch archive feed: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    let items: Vec<serde_json::Value> = rows
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

    Ok(Json(AppChangalaGlobalviewGetGlobalArchiveFeedOutput {
        items,
        cursor,
    }))
}
