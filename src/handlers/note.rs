//! Note service handlers — keywords, notes, collective notes, votes, labels.

use axum::extract::Query;
use axum::Json;

use atrg_auth::RequireAuth;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use super::auth;
use crate::generated::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Placeholder Ring DID used until real Ring identity is provisioned.
const RING_DID: &str = "did:web:ring.changala.local";

// ═══════════════════════════════════════════════════════════════════════════
// Keywords
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.addKeyword
///
/// Adds a keyword to a session while the keyword window is open.
/// The keyword window is open when `keyword_window_expires_at > now()` **or**
/// the session `status = 'live'`.
pub async fn add_keyword(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingAddKeywordInput>,
) -> Result<Json<AppChangalaRingAddKeywordOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Look up the session and check whether the keyword window is open.
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT status, keyword_window_expires_at FROM sessions WHERE uri = $1",
    )
    .bind(&input.session_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Session lookup failed: {e}"),
    })?;

    let (status, window_expires) = row.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Session not found: {}", input.session_uri),
    })?;

    let window_open = status == "live"
        || window_expires
            .as_deref()
            .map(|exp| exp > now.as_str())
            .unwrap_or(false);

    if !window_open {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: "Keyword window is closed for this session".to_string(),
        });
    }

    let rkey = atrg_repo::Tid::now().to_string();
    let keyword_uri = format!("at://{}/app.changala.keyword/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO keywords (session_uri, did, text, keyword_uri, created_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&input.session_uri)
    .bind(&session.did)
    .bind(&input.text)
    .bind(&keyword_uri)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| {
        // UNIQUE(session_uri, did, text) — duplicate keyword by this user
        if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate key") {
            XrpcError {
                name: XrpcErrorName::InvalidRequest,
                message: "You already submitted this keyword for this session".to_string(),
            }
        } else {
            XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to insert keyword: {e}"),
            }
        }
    })?;

    let effective_expires = window_expires.unwrap_or_else(|| now.clone());

    Ok(Json(AppChangalaRingAddKeywordOutput {
        keyword_uri,
        window_expires_at: effective_expires,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Notes
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.createNote
///
/// Creates a new note (version 1) for a session. Stores a blob reference on
/// the Ring and returns a `ring_ref` + `note_template` for the PDS record.
pub async fn create_note(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCreateNoteInput>,
) -> Result<Json<AppChangalaRingCreateNoteOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = input
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let cid = app
        .blobs
        .put(input.content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store content: {e}"),
        })?;
    let rkey = atrg_repo::Tid::now().to_string();
    let note_uri = format!("at://{}/app.changala.note/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO notes (uri, session_uri, author_did, format, ring_did, cid, version, \
         parent_note_uri, summary, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, 1, NULL, $7, $8)",
    )
    .bind(&note_uri)
    .bind(&input.session_uri)
    .bind(&session.did)
    .bind(&input.format)
    .bind(RING_DID)
    .bind(&cid)
    .bind(&input.summary)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to create note: {e}"),
    })?;

    let ring_ref = json!({
        "ringDid": RING_DID,
        "cid": cid,
    });

    let note_template = json!({
        "uri": note_uri,
        "sessionUri": input.session_uri,
        "format": input.format,
        "ringRef": ring_ref,
        "version": 1,
        "summary": input.summary,
        "createdAt": now,
    });

    Ok(Json(AppChangalaRingCreateNoteOutput {
        ring_ref,
        note_template,
    }))
}

/// POST /xrpc/app.changala.ring.versionNote
///
/// Creates a new version of an existing note. Inherits session_uri and
/// author_did from the parent, increments the version counter.
pub async fn version_note(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingVersionNoteInput>,
) -> Result<Json<AppChangalaRingVersionNoteOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = input
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    // Fetch the parent note to inherit session + author and get current version.
    let parent = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT session_uri, author_did, version::BIGINT FROM notes WHERE uri = $1",
    )
    .bind(&input.parent_note_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Parent note lookup failed: {e}"),
    })?;

    let (session_uri, author_did, parent_version) = parent.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Parent note not found: {}", input.parent_note_uri),
    })?;

    let new_version = parent_version + 1;
    let cid = app
        .blobs
        .put(input.content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store content: {e}"),
        })?;
    let rkey = atrg_repo::Tid::now().to_string();
    let note_uri = format!("at://{}/app.changala.note/{}", author_did, rkey);

    sqlx::query(
        "INSERT INTO notes (uri, session_uri, author_did, format, ring_did, cid, version, \
         parent_note_uri, summary, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&note_uri)
    .bind(&session_uri)
    .bind(&author_did)
    .bind(&input.format)
    .bind(RING_DID)
    .bind(&cid)
    .bind(new_version)
    .bind(&input.parent_note_uri)
    .bind(&input.summary)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to version note: {e}"),
    })?;

    let ring_ref = json!({
        "ringDid": RING_DID,
        "cid": cid,
    });

    let note_template = json!({
        "uri": note_uri,
        "sessionUri": session_uri,
        "format": input.format,
        "ringRef": ring_ref,
        "version": new_version,
        "parentNote": input.parent_note_uri,
        "summary": input.summary,
        "createdAt": now,
    });

    Ok(Json(AppChangalaRingVersionNoteOutput {
        ring_ref,
        note_template,
    }))
}

/// GET /xrpc/app.changala.ring.getNoteContent
///
/// Retrieves note content from the Ring by CID using the S3 blob store.
pub async fn get_note_content(
    Query(params): Query<AppChangalaRingGetNoteContentParams>,
) -> Result<Json<AppChangalaRingGetNoteContentOutput>, XrpcError> {
    let app = crate::state::get();

    let content_bytes = app.blobs.get(&params.cid).await.map_err(|e| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Blob not found: {e}"),
    })?;
    let content = String::from_utf8_lossy(&content_bytes).to_string();

    let format = sqlx::query_scalar::<_, String>("SELECT format FROM notes WHERE cid = $1 LIMIT 1")
        .bind(&params.cid)
        .fetch_optional(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("DB error: {e}"),
        })?
        .unwrap_or_else(|| "plaintext".to_string());

    Ok(Json(AppChangalaRingGetNoteContentOutput {
        format,
        content,
    }))
}

/// GET /xrpc/app.changala.ring.getNoteHistory
///
/// Returns the full version chain of a note. All versions share the same
/// session_uri + author_did as the original and are ordered by version.
pub async fn get_note_history(
    Query(params): Query<AppChangalaRingGetNoteHistoryParams>,
) -> Result<Json<AppChangalaRingGetNoteHistoryOutput>, XrpcError> {
    let app = crate::state::get();

    // First, find the note itself so we can get session_uri + author_did.
    let root = sqlx::query_as::<_, (String, String)>(
        "SELECT session_uri, author_did FROM notes WHERE uri = $1",
    )
    .bind(&params.note_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Note lookup failed: {e}"),
    })?;

    let (session_uri, author_did) = root.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Note not found: {}", params.note_uri),
    })?;

    // Fetch all versions for this session + author, ordered chronologically.
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            i64,
            Option<String>,
            Option<String>,
            String,
        ),
    >(
        "SELECT uri, ring_did, cid, version::BIGINT, parent_note_uri, summary, created_at \
         FROM notes \
         WHERE session_uri = $1 AND author_did = $2 \
         ORDER BY version ASC",
    )
    .bind(&session_uri)
    .bind(&author_did)
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch note history: {e}"),
    })?;

    let versions: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "uri": r.0,
                "ringRef": {
                    "ringDid": r.1,
                    "cid": r.2,
                },
                "version": r.3,
                "parentNoteUri": r.4,
                "summary": r.5,
                "createdAt": r.6,
            })
        })
        .collect();

    Ok(Json(AppChangalaRingGetNoteHistoryOutput {
        session_uri,
        author_did,
        versions,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Collective Notes
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.proposeEdit
///
/// Proposes an edit (diff) to the collective note for a session.
/// Creates an `edit_proposals` row with status `pending`.
pub async fn propose_edit(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingProposeEditInput>,
) -> Result<Json<AppChangalaRingProposeEditOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let diff_cid = app
        .blobs
        .put(input.diff.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store content: {e}"),
        })?;
    let rkey = atrg_repo::Tid::now().to_string();
    let proposal_uri = format!(
        "at://{}/app.changala.collectivenote.proposal/{}",
        session.did, rkey
    );

    sqlx::query(
        "INSERT INTO edit_proposals \
         (proposal_uri, session_uri, proposer_did, diff_ring_did, diff_cid, status, summary, created_at) \
         VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7)",
    )
    .bind(&proposal_uri)
    .bind(&input.session_uri)
    .bind(&session.did)
    .bind(RING_DID)
    .bind(&diff_cid)
    .bind(&input.summary)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to create edit proposal: {e}"),
    })?;

    let diff_ring_ref = json!({
        "ringDid": RING_DID,
        "cid": diff_cid,
    });

    Ok(Json(AppChangalaRingProposeEditOutput {
        proposal_uri,
        diff_ring_ref,
    }))
}

/// POST /xrpc/app.changala.ring.acceptEdit
///
/// Accepts a pending edit proposal. Only the Class Rep should call this
/// (auth enforcement is deferred to middleware). Updates the proposal status
/// and upserts the collective note for the session.
pub async fn accept_edit(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingAcceptEditInput>,
) -> Result<Json<AppChangalaRingAcceptEditOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Fetch the proposal and verify it is pending.
    let proposal = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, session_uri, status FROM edit_proposals WHERE proposal_uri = $1",
    )
    .bind(&input.proposal_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Proposal lookup failed: {e}"),
    })?;

    let (proposal_id, session_uri, status) = proposal.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Proposal not found: {}", input.proposal_uri),
    })?;

    if status != "pending" {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!("Proposal is already {status}, cannot accept"),
        });
    }

    // Verify the caller is a Class Rep or Admin for this course.
    let course_uri = auth::get_course_for_session(&app.db, &session_uri).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &course_uri).await?;

    // Mark proposal accepted.
    sqlx::query(
        "UPDATE edit_proposals SET status = 'accepted', resolved_at = $1, resolved_by = $2 WHERE id = $3",
    )
    .bind(&now)
    .bind(&session.did)
    .bind(proposal_id)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to accept proposal: {e}"),
    })?;

    // Upsert collective note — increment version or insert first version.
    let content = format!("collective-{}-{}", session_uri, now);
    let new_cid = app
        .blobs
        .put(content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store archive bundle: {e}"),
        })?;

    // Try to fetch existing collective note to get current version.
    let existing_version = sqlx::query_scalar::<_, i64>(
        "SELECT version::BIGINT FROM collective_notes WHERE session_uri = $1",
    )
    .bind(&session_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Collective note lookup failed: {e}"),
    })?;

    let new_version = existing_version.unwrap_or(0) + 1;

    sqlx::query(
        "INSERT INTO collective_notes (session_uri, ring_did, cid, version, updated_at) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT(session_uri) DO UPDATE SET \
           cid = excluded.cid, \
           version = excluded.version, \
           updated_at = excluded.updated_at",
    )
    .bind(&session_uri)
    .bind(RING_DID)
    .bind(&new_cid)
    .bind(new_version)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to upsert collective note: {e}"),
    })?;

    let rkey = atrg_repo::Tid::now().to_string();
    let collective_note_uri = format!("at://{}/app.changala.collectivenote/{}", RING_DID, rkey);

    let new_collective_ring_ref = json!({
        "ringDid": RING_DID,
        "cid": new_cid,
    });

    Ok(Json(AppChangalaRingAcceptEditOutput {
        collective_note_uri,
        new_collective_ring_ref,
    }))
}

/// POST /xrpc/app.changala.ring.rejectEdit
///
/// Rejects a pending edit proposal (Class Rep action).
pub async fn reject_edit(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRejectEditInput>,
) -> Result<Json<AppChangalaRingRejectEditOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    let proposal = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, session_uri, status FROM edit_proposals WHERE proposal_uri = $1",
    )
    .bind(&input.proposal_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Proposal lookup failed: {e}"),
    })?;

    let (proposal_id, session_uri, status) = proposal.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Proposal not found: {}", input.proposal_uri),
    })?;

    if status != "pending" {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!("Proposal is already {status}, cannot reject"),
        });
    }

    // Verify the caller is a Class Rep or Admin for this course.
    let course_uri = auth::get_course_for_session(&app.db, &session_uri).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &course_uri).await?;

    sqlx::query(
        "UPDATE edit_proposals SET status = 'rejected', resolved_at = $1, resolved_by = $2 WHERE id = $3",
    )
    .bind(&now)
    .bind(&session.did)
    .bind(proposal_id)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to reject proposal: {e}"),
    })?;

    Ok(Json(AppChangalaRingRejectEditOutput {
        proposal_uri: input.proposal_uri,
    }))
}

/// GET /xrpc/app.changala.ring.getCollectiveNote
///
/// Returns the current collective note for a session, including the list
/// of contributor DIDs (authors of accepted proposals).
pub async fn get_collective_note(
    Query(params): Query<AppChangalaRingGetCollectiveNoteParams>,
) -> Result<Json<AppChangalaRingGetCollectiveNoteOutput>, XrpcError> {
    let app = crate::state::get();

    let row = sqlx::query_as::<_, (String, String, i64, String)>(
        "SELECT ring_did, cid, version::BIGINT, updated_at FROM collective_notes WHERE session_uri = $1",
    )
    .bind(&params.session_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Collective note lookup failed: {e}"),
    })?;

    let (ring_did, cid, version, updated_at) = row.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!(
            "No collective note exists for session: {}",
            params.session_uri
        ),
    })?;

    // Gather contributor DIDs from accepted proposals.
    let contributor_dids: Vec<String> = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT proposer_did FROM edit_proposals \
         WHERE session_uri = $1 AND status = 'accepted' \
         ORDER BY proposer_did",
    )
    .bind(&params.session_uri)
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch contributors: {e}"),
    })?;

    let ring_ref = json!({
        "ringDid": ring_did,
        "cid": cid,
    });

    Ok(Json(AppChangalaRingGetCollectiveNoteOutput {
        session_uri: params.session_uri,
        ring_ref,
        contributor_dids,
        version,
        updated_at,
    }))
}

/// GET /xrpc/app.changala.ring.listEditProposals
///
/// Lists edit proposals for a session with optional status filter and
/// cursor-based pagination on `created_at`.
pub async fn list_edit_proposals(
    Query(params): Query<AppChangalaRingListEditProposalsParams>,
) -> Result<Json<AppChangalaRingListEditProposalsOutput>, XrpcError> {
    let app = crate::state::get();

    let limit = params.limit.unwrap_or(50).min(100);

    // Build query dynamically based on optional filters.
    // PostgreSQL uses numbered placeholders ($1, $2, ...) so we track the index.
    let mut sql = String::from(
        "SELECT proposal_uri, session_uri, proposer_did, diff_ring_did, diff_cid, \
                status, summary, resolved_at, resolved_by, created_at \
         FROM edit_proposals WHERE session_uri = $1",
    );
    let mut bind_values: Vec<String> = vec![params.session_uri.clone()];
    let mut param_idx = 2u32;

    if let Some(ref status_filter) = params.status_filter {
        sql.push_str(&format!(" AND status = ${param_idx}"));
        bind_values.push(status_filter.clone());
        param_idx += 1;
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND created_at > ${param_idx}"));
        bind_values.push(cursor.clone());
        param_idx += 1;
    }

    sql.push_str(&format!(" ORDER BY created_at ASC LIMIT ${param_idx}"));

    // sqlx requires static bind counts, so we build with the maximum shape
    // and use a raw query approach.
    let rows = sqlx::query_as::<
        _,
        (
            Option<String>, // proposal_uri
            String,         // session_uri
            String,         // proposer_did
            String,         // diff_ring_did
            String,         // diff_cid
            String,         // status
            Option<String>, // summary
            Option<String>, // resolved_at
            Option<String>, // resolved_by
            String,         // created_at
        ),
    >(&sql);

    // Bind values in order.
    let mut query = rows.bind(&bind_values[0]);
    for v in &bind_values[1..] {
        query = query.bind(v);
    }
    query = query.bind(limit);

    let results = query.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to list proposals: {e}"),
    })?;

    let proposals: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            json!({
                "proposalUri": r.0,
                "sessionUri": r.1,
                "proposerDid": r.2,
                "diffRingRef": {
                    "ringDid": r.3,
                    "cid": r.4,
                },
                "status": r.5,
                "summary": r.6,
                "resolvedAt": r.7,
                "resolvedBy": r.8,
                "createdAt": r.9,
            })
        })
        .collect();

    // Next cursor is the created_at of the last row, if we hit the limit.
    let next_cursor = if proposals.len() as i64 >= limit {
        results.last().map(|r| r.9.clone())
    } else {
        None
    };

    Ok(Json(AppChangalaRingListEditProposalsOutput {
        proposals,
        cursor: next_cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Votes
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.registerVote
///
/// Registers an upvote on a note or brain node. Each user may vote at most
/// once per subject (enforced by UNIQUE(subject_uri, voter_did)). Returns
/// the new total vote count.
pub async fn register_vote(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRegisterVoteInput>,
) -> Result<Json<AppChangalaRingRegisterVoteOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO votes (vote_uri, subject_uri, voter_did, created_at) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&input.vote_uri)
    .bind(&input.subject_uri)
    .bind(&session.did)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate key") {
            XrpcError {
                name: XrpcErrorName::InvalidRequest,
                message: "You have already voted on this subject".to_string(),
            }
        } else {
            XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to register vote: {e}"),
            }
        }
    })?;

    let total_votes =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM votes WHERE subject_uri = $1")
            .bind(&input.subject_uri)
            .fetch_one(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to count votes: {e}"),
            })?;

    Ok(Json(AppChangalaRingRegisterVoteOutput {
        subject_uri: input.subject_uri,
        total_votes,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Labels
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.applyLabel
///
/// Applies a quality/knowledge label to a note or brain node. Labels are
/// ATProto-native signals stored with `neg = FALSE` (positive assertion).
pub async fn apply_label(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingApplyLabelInput>,
) -> Result<Json<AppChangalaRingApplyLabelOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let rkey = atrg_repo::Tid::now().to_string();
    let label_uri = format!("at://{}/app.changala.label/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO labels (label_uri, subject_uri, val, src_did, neg, created_at) \
         VALUES ($1, $2, $3, $4, FALSE, $5)",
    )
    .bind(&label_uri)
    .bind(&input.subject_uri)
    .bind(&input.val)
    .bind(&session.did)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to apply label: {e}"),
    })?;

    Ok(Json(AppChangalaRingApplyLabelOutput { label_uri }))
}

/// POST /xrpc/app.changala.ring.retractLabel
///
/// Retracts a previously applied label by inserting a **negation** record
/// (`neg = TRUE`). The Global View materialises the effective label state by
/// checking for negation records.
pub async fn retract_label(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRetractLabelInput>,
) -> Result<Json<AppChangalaRingRetractLabelOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let rkey = atrg_repo::Tid::now().to_string();
    let label_uri = format!("at://{}/app.changala.label/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO labels (label_uri, subject_uri, val, src_did, neg, created_at) \
         VALUES ($1, $2, $3, $4, TRUE, $5)",
    )
    .bind(&label_uri)
    .bind(&input.subject_uri)
    .bind(&input.val)
    .bind(&session.did)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to retract label: {e}"),
    })?;

    Ok(Json(AppChangalaRingRetractLabelOutput { label_uri }))
}
