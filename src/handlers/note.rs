//! Note service handlers — keywords, notes, collective notes, votes, labels.

use axum::extract::Query;
use axum::{extract::State, Json};

use atrg_auth::RequireAuth;
use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use super::auth;
use crate::generated::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate a deterministic fake CID from content, for MVP blob-less operation.
/// Real CID generation will use multihash + multicodec once the Ring has blob storage.
fn fake_cid(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("bafyrei{:016x}", hasher.finish())
}

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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingAddKeywordInput>,
) -> Result<Json<AppChangalaRingAddKeywordOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Look up the session and check whether the keyword window is open.
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT status, keyword_window_expires_at FROM sessions WHERE uri = ?",
    )
    .bind(&input.session_uri)
    .fetch_optional(&state.db)
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
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&input.session_uri)
    .bind(&session.did)
    .bind(&input.text)
    .bind(&keyword_uri)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        // UNIQUE(session_uri, did, text) — duplicate keyword by this user
        if e.to_string().contains("UNIQUE") {
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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCreateNoteInput>,
) -> Result<Json<AppChangalaRingCreateNoteOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = input
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let cid = fake_cid(&input.content);
    let rkey = atrg_repo::Tid::now().to_string();
    let note_uri = format!("at://{}/app.changala.note/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO notes (uri, session_uri, author_did, format, ring_did, cid, version, \
         parent_note_uri, summary, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, 1, NULL, ?, ?)",
    )
    .bind(&note_uri)
    .bind(&input.session_uri)
    .bind(&session.did)
    .bind(&input.format)
    .bind(RING_DID)
    .bind(&cid)
    .bind(&input.summary)
    .bind(&now)
    .execute(&state.db)
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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingVersionNoteInput>,
) -> Result<Json<AppChangalaRingVersionNoteOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = input
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    // Fetch the parent note to inherit session + author and get current version.
    let parent = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT session_uri, author_did, version FROM notes WHERE uri = ?",
    )
    .bind(&input.parent_note_uri)
    .fetch_optional(&state.db)
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
    let cid = fake_cid(&input.content);
    let rkey = atrg_repo::Tid::now().to_string();
    let note_uri = format!("at://{}/app.changala.note/{}", author_did, rkey);

    sqlx::query(
        "INSERT INTO notes (uri, session_uri, author_did, format, ring_did, cid, version, \
         parent_note_uri, summary, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
    .execute(&state.db)
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
/// Retrieves note content from the Ring by CID. In MVP, the Ring does not
/// have actual blob storage, so this returns a stub indicating the CID that
/// should be fetched.
pub async fn get_note_content(
    State(_state): State<AppState>,
    Query(params): Query<AppChangalaRingGetNoteContentParams>,
) -> Result<Json<AppChangalaRingGetNoteContentOutput>, XrpcError> {
    // In production this would stream the blob from the Ring's content-
    // addressed store. For MVP we return a stub with the CID reference.
    Ok(Json(AppChangalaRingGetNoteContentOutput {
        format: "plaintext".to_string(),
        content: format!(
            "Content stored on Ring ({}) — fetch via CID: {}",
            params.ring_did, params.cid
        ),
    }))
}

/// GET /xrpc/app.changala.ring.getNoteHistory
///
/// Returns the full version chain of a note. All versions share the same
/// session_uri + author_did as the original and are ordered by version.
pub async fn get_note_history(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingGetNoteHistoryParams>,
) -> Result<Json<AppChangalaRingGetNoteHistoryOutput>, XrpcError> {
    // First, find the note itself so we can get session_uri + author_did.
    let root = sqlx::query_as::<_, (String, String)>(
        "SELECT session_uri, author_did FROM notes WHERE uri = ?",
    )
    .bind(&params.note_uri)
    .fetch_optional(&state.db)
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
        "SELECT uri, ring_did, cid, version, parent_note_uri, summary, created_at \
         FROM notes \
         WHERE session_uri = ? AND author_did = ? \
         ORDER BY version ASC",
    )
    .bind(&session_uri)
    .bind(&author_did)
    .fetch_all(&state.db)
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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingProposeEditInput>,
) -> Result<Json<AppChangalaRingProposeEditOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let diff_cid = fake_cid(&input.diff);
    let rkey = atrg_repo::Tid::now().to_string();
    let proposal_uri = format!(
        "at://{}/app.changala.collectivenote.proposal/{}",
        session.did, rkey
    );

    sqlx::query(
        "INSERT INTO edit_proposals \
         (proposal_uri, session_uri, proposer_did, diff_ring_did, diff_cid, status, summary, created_at) \
         VALUES (?, ?, ?, ?, ?, 'pending', ?, ?)",
    )
    .bind(&proposal_uri)
    .bind(&input.session_uri)
    .bind(&session.did)
    .bind(RING_DID)
    .bind(&diff_cid)
    .bind(&input.summary)
    .bind(&now)
    .execute(&state.db)
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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingAcceptEditInput>,
) -> Result<Json<AppChangalaRingAcceptEditOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Fetch the proposal and verify it is pending.
    let proposal = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, session_uri, status FROM edit_proposals WHERE proposal_uri = ?",
    )
    .bind(&input.proposal_uri)
    .fetch_optional(&state.db)
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
    let course_uri = auth::get_course_for_session(&state, &session_uri).await?;
    auth::require_class_rep_or_admin(&state, &session.did, &course_uri).await?;

    // Mark proposal accepted.
    sqlx::query(
        "UPDATE edit_proposals SET status = 'accepted', resolved_at = ?, resolved_by = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&session.did)
    .bind(proposal_id)
    .execute(&state.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to accept proposal: {e}"),
    })?;

    // Upsert collective note — increment version or insert first version.
    let new_cid = fake_cid(&format!("collective-{}-{}", session_uri, now));

    // Try to fetch existing collective note to get current version.
    let existing_version =
        sqlx::query_scalar::<_, i64>("SELECT version FROM collective_notes WHERE session_uri = ?")
            .bind(&session_uri)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Collective note lookup failed: {e}"),
            })?;

    let new_version = existing_version.unwrap_or(0) + 1;

    sqlx::query(
        "INSERT INTO collective_notes (session_uri, ring_did, cid, version, updated_at) \
         VALUES (?, ?, ?, ?, ?) \
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
    .execute(&state.db)
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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRejectEditInput>,
) -> Result<Json<AppChangalaRingRejectEditOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    let proposal = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, session_uri, status FROM edit_proposals WHERE proposal_uri = ?",
    )
    .bind(&input.proposal_uri)
    .fetch_optional(&state.db)
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
    let course_uri = auth::get_course_for_session(&state, &session_uri).await?;
    auth::require_class_rep_or_admin(&state, &session.did, &course_uri).await?;

    sqlx::query(
        "UPDATE edit_proposals SET status = 'rejected', resolved_at = ?, resolved_by = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&session.did)
    .bind(proposal_id)
    .execute(&state.db)
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
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingGetCollectiveNoteParams>,
) -> Result<Json<AppChangalaRingGetCollectiveNoteOutput>, XrpcError> {
    let row = sqlx::query_as::<_, (String, String, i64, String)>(
        "SELECT ring_did, cid, version, updated_at FROM collective_notes WHERE session_uri = ?",
    )
    .bind(&params.session_uri)
    .fetch_optional(&state.db)
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
         WHERE session_uri = ? AND status = 'accepted' \
         ORDER BY proposer_did",
    )
    .bind(&params.session_uri)
    .fetch_all(&state.db)
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
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingListEditProposalsParams>,
) -> Result<Json<AppChangalaRingListEditProposalsOutput>, XrpcError> {
    let limit = params.limit.unwrap_or(50).min(100);

    // Build query dynamically based on optional filters.
    let mut sql = String::from(
        "SELECT proposal_uri, session_uri, proposer_did, diff_ring_did, diff_cid, \
                status, summary, resolved_at, resolved_by, created_at \
         FROM edit_proposals WHERE session_uri = ?",
    );
    let mut bind_values: Vec<String> = vec![params.session_uri.clone()];

    if let Some(ref status_filter) = params.status_filter {
        sql.push_str(" AND status = ?");
        bind_values.push(status_filter.clone());
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(" AND created_at > ?");
        bind_values.push(cursor.clone());
    }

    sql.push_str(" ORDER BY created_at ASC LIMIT ?");

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

    let results = query.fetch_all(&state.db).await.map_err(|e| XrpcError {
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
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRegisterVoteInput>,
) -> Result<Json<AppChangalaRingRegisterVoteOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO votes (vote_uri, subject_uri, voter_did, created_at) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(&input.vote_uri)
    .bind(&input.subject_uri)
    .bind(&session.did)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
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
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM votes WHERE subject_uri = ?")
            .bind(&input.subject_uri)
            .fetch_one(&state.db)
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
/// ATProto-native signals stored with `neg = 0` (positive assertion).
pub async fn apply_label(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingApplyLabelInput>,
) -> Result<Json<AppChangalaRingApplyLabelOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let rkey = atrg_repo::Tid::now().to_string();
    let label_uri = format!("at://{}/app.changala.label/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO labels (label_uri, subject_uri, val, src_did, neg, created_at) \
         VALUES (?, ?, ?, ?, 0, ?)",
    )
    .bind(&label_uri)
    .bind(&input.subject_uri)
    .bind(&input.val)
    .bind(&session.did)
    .bind(&now)
    .execute(&state.db)
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
/// (`neg = 1`). The Global View materialises the effective label state by
/// checking for negation records.
pub async fn retract_label(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRetractLabelInput>,
) -> Result<Json<AppChangalaRingRetractLabelOutput>, XrpcError> {
    auth::check_not_banned(&state, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let rkey = atrg_repo::Tid::now().to_string();
    let label_uri = format!("at://{}/app.changala.label/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO labels (label_uri, subject_uri, val, src_did, neg, created_at) \
         VALUES (?, ?, ?, ?, 1, ?)",
    )
    .bind(&label_uri)
    .bind(&input.subject_uri)
    .bind(&input.val)
    .bind(&session.did)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to retract label: {e}"),
    })?;

    Ok(Json(AppChangalaRingRetractLabelOutput { label_uri }))
}
