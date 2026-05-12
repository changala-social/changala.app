//! Session service handlers — lifecycle management.

use atrg_auth::RequireAuth;
use atrg_core::AppState;
use atrg_repo::Tid;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::{Query, State};
use axum::Json;
use chrono::{Duration, Utc};

use changala_shared::types::*;

use super::auth;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Keyword window duration after a session is closed (60 minutes).
const KEYWORD_WINDOW_MINS: i64 = 60;

/// Raw DB row for a session. Column order must match every SELECT used
/// in this module:
///   uri, course_uri, scheduled_at, duration_mins, status, created_by,
///   topic, opened_at, closed_at, keyword_window_expires_at,
///   rescheduled_to, created_at
type SessionRow = (
    String,         // 0  uri
    String,         // 1  course_uri
    String,         // 2  scheduled_at
    i32,            // 3  duration_mins
    String,         // 4  status
    String,         // 5  created_by
    Option<String>, // 6  topic
    Option<String>, // 7  opened_at
    Option<String>, // 8  closed_at
    Option<String>, // 9  keyword_window_expires_at
    Option<String>, // 10 rescheduled_to
    String,         // 11 created_at
);

/// The SELECT column list used by all session queries.
const SESSION_COLS: &str = "uri, course_uri, scheduled_at, duration_mins, status, \
                            created_by, topic, opened_at, closed_at, \
                            keyword_window_expires_at, rescheduled_to, created_at";

/// Compute whether the keyword window is currently open based on the
/// `keyword_window_expires_at` timestamp.
fn is_keyword_window_open(expires_at: &Option<String>) -> bool {
    match expires_at {
        None => false,
        Some(ts) => {
            let Ok(expires) = chrono::DateTime::parse_from_rfc3339(ts) else {
                return false;
            };
            Utc::now() < expires.with_timezone(&Utc)
        }
    }
}

/// Build a session view struct from a raw DB row. This is reused by every
/// handler that returns a session shape (CreateSessionOutput and friends all
/// have the same fields).
fn session_view(row: &SessionRow) -> SessionViewFields {
    SessionViewFields {
        uri: row.0.clone(),
        course_uri: row.1.clone(),
        scheduled_at: row.2.clone(),
        duration_mins: row.3 as i64,
        status: row.4.clone(),
        created_by: row.5.clone(),
        topic: row.6.clone(),
        opened_at: row.7.clone(),
        closed_at: row.8.clone(),
        keyword_window_expires_at: row.9.clone(),
        keyword_window_open: is_keyword_window_open(&row.9),
        rescheduled_to: row.10.clone(),
        created_at: row.11.clone(),
    }
}

/// Intermediate struct holding all fields shared across the various session
/// output types. Avoids repeating the mapping logic for every output struct.
struct SessionViewFields {
    uri: String,
    course_uri: String,
    scheduled_at: String,
    duration_mins: i64,
    status: String,
    created_by: String,
    topic: Option<String>,
    opened_at: Option<String>,
    closed_at: Option<String>,
    keyword_window_expires_at: Option<String>,
    keyword_window_open: bool,
    rescheduled_to: Option<String>,
    created_at: String,
}

/// Fetch a single session row by URI. Returns `None` when not found.
async fn fetch_session_row(db: &sqlx::PgPool, uri: &str) -> Result<Option<SessionRow>, XrpcError> {
    let sql = format!("SELECT {SESSION_COLS} FROM sessions WHERE uri = $1");
    sqlx::query_as::<_, SessionRow>(&sql)
        .bind(uri)
        .fetch_optional(db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })
}

/// Convenience: fetch a session row or return a `NotFound` error.
async fn require_session_row(db: &sqlx::PgPool, uri: &str) -> Result<SessionRow, XrpcError> {
    fetch_session_row(db, uri).await?.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("session not found: {uri}"),
    })
}

// ---------------------------------------------------------------------------
// Macro-free conversion helpers — one per output type
// ---------------------------------------------------------------------------

fn into_create_output(v: SessionViewFields) -> AppChangalaRingCreateSessionOutput {
    AppChangalaRingCreateSessionOutput {
        uri: v.uri,
        course_uri: v.course_uri,
        scheduled_at: v.scheduled_at,
        duration_mins: v.duration_mins,
        status: v.status,
        created_by: v.created_by,
        topic: v.topic,
        opened_at: v.opened_at,
        closed_at: v.closed_at,
        keyword_window_expires_at: v.keyword_window_expires_at,
        keyword_window_open: v.keyword_window_open,
        rescheduled_to: v.rescheduled_to,
        created_at: v.created_at,
    }
}

fn into_get_output(v: SessionViewFields) -> AppChangalaRingGetSessionOutput {
    AppChangalaRingGetSessionOutput {
        uri: v.uri,
        course_uri: v.course_uri,
        scheduled_at: v.scheduled_at,
        duration_mins: v.duration_mins,
        status: v.status,
        created_by: v.created_by,
        topic: v.topic,
        opened_at: v.opened_at,
        closed_at: v.closed_at,
        keyword_window_expires_at: v.keyword_window_expires_at,
        keyword_window_open: v.keyword_window_open,
        rescheduled_to: v.rescheduled_to,
        created_at: v.created_at,
    }
}

fn into_open_output(v: SessionViewFields) -> AppChangalaRingOpenSessionOutput {
    AppChangalaRingOpenSessionOutput {
        uri: v.uri,
        course_uri: v.course_uri,
        scheduled_at: v.scheduled_at,
        duration_mins: v.duration_mins,
        status: v.status,
        created_by: v.created_by,
        topic: v.topic,
        opened_at: v.opened_at,
        closed_at: v.closed_at,
        keyword_window_expires_at: v.keyword_window_expires_at,
        keyword_window_open: v.keyword_window_open,
        rescheduled_to: v.rescheduled_to,
        created_at: v.created_at,
    }
}

fn into_cancel_output(v: SessionViewFields) -> AppChangalaRingCancelSessionOutput {
    AppChangalaRingCancelSessionOutput {
        uri: v.uri,
        course_uri: v.course_uri,
        scheduled_at: v.scheduled_at,
        duration_mins: v.duration_mins,
        status: v.status,
        created_by: v.created_by,
        topic: v.topic,
        opened_at: v.opened_at,
        closed_at: v.closed_at,
        keyword_window_expires_at: v.keyword_window_expires_at,
        keyword_window_open: v.keyword_window_open,
        rescheduled_to: v.rescheduled_to,
        created_at: v.created_at,
    }
}

fn into_reschedule_output(v: SessionViewFields) -> AppChangalaRingRescheduleSessionOutput {
    AppChangalaRingRescheduleSessionOutput {
        uri: v.uri,
        course_uri: v.course_uri,
        scheduled_at: v.scheduled_at,
        duration_mins: v.duration_mins,
        status: v.status,
        created_by: v.created_by,
        topic: v.topic,
        opened_at: v.opened_at,
        closed_at: v.closed_at,
        keyword_window_expires_at: v.keyword_window_expires_at,
        keyword_window_open: v.keyword_window_open,
        rescheduled_to: v.rescheduled_to,
        created_at: v.created_at,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `app.changala.ring.createSession` — POST procedure.
///
/// Creates a new session for a course with status `scheduled`.
pub async fn create_session(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCreateSessionInput>,
) -> Result<Json<AppChangalaRingCreateSessionOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &input.course_uri).await?;

    // Verify the course exists
    let course_exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1::BIGINT FROM courses WHERE uri = $1")
            .bind(&input.course_uri)
            .fetch_optional(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("database error: {e}"),
            })?;

    if course_exists.is_none() {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("course not found: {}", input.course_uri),
        });
    }

    let rkey = Tid::now().to_string();
    let uri = format!("at://changala.ring/app.changala.session/{rkey}");
    let now = Utc::now().to_rfc3339();
    let created_by = session.did.clone();

    sqlx::query(
        "INSERT INTO sessions (uri, rkey, course_uri, scheduled_at, duration_mins, \
                               status, created_by, topic, created_at) \
         VALUES ($1, $2, $3, $4, $5, 'scheduled', $6, $7, $8)",
    )
    .bind(&uri)
    .bind(&rkey)
    .bind(&input.course_uri)
    .bind(&input.scheduled_at)
    .bind(input.duration_mins)
    .bind(&created_by)
    .bind(&input.topic)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("database error: {e}"),
    })?;

    Ok(Json(into_create_output(SessionViewFields {
        uri,
        course_uri: input.course_uri,
        scheduled_at: input.scheduled_at,
        duration_mins: input.duration_mins,
        status: "scheduled".to_string(),
        created_by,
        topic: input.topic,
        opened_at: None,
        closed_at: None,
        keyword_window_expires_at: None,
        keyword_window_open: false,
        rescheduled_to: None,
        created_at: now,
    })))
}

/// `app.changala.ring.getSession` — GET query.
///
/// Fetches a single session by AT URI. The `keyword_window_open` field is
/// computed dynamically from `keyword_window_expires_at` vs current time.
pub async fn get_session(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingGetSessionParams>,
) -> Result<Json<AppChangalaRingGetSessionOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    let row = require_session_row(&app.db, &params.uri).await?;
    Ok(Json(into_get_output(session_view(&row))))
}

/// `app.changala.ring.listSessions` — GET query.
///
/// Lists sessions for a course with optional status filtering and cursor-based
/// pagination. Default limit 50, max 100.
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingListSessionsParams>,
) -> Result<Json<AppChangalaRingListSessionsOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    let limit = params.limit.unwrap_or(50).clamp(1, 100);

    let mut sql = format!("SELECT {SESSION_COLS} FROM sessions WHERE course_uri = $1");
    let mut binds: Vec<String> = vec![params.course_uri.clone()];
    let mut param_idx = 2usize;

    // Status filtering — the lexicon type gives us Option<Vec<String>>
    if let Some(ref statuses) = params.statuses {
        if !statuses.is_empty() {
            let placeholders: Vec<String> = statuses
                .iter()
                .enumerate()
                .map(|(i, _)| format!("${}", param_idx + i))
                .collect();
            sql.push_str(&format!(" AND status IN ({})", placeholders.join(", ")));
            param_idx += statuses.len();
            for s in statuses {
                binds.push(s.clone());
            }
        }
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND created_at < ${}", param_idx));
        param_idx += 1;
        binds.push(cursor.clone());
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${}", param_idx));

    let fetch_limit = limit + 1;

    let mut query = sqlx::query_as::<_, SessionRow>(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(fetch_limit);

    let rows = query.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("database error: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows
    };

    let sessions: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let v = session_view(r);
            serde_json::to_value(into_get_output(v)).unwrap()
        })
        .collect();

    let cursor = if has_more {
        rows.last().map(|r| r.11.clone())
    } else {
        None
    };

    Ok(Json(AppChangalaRingListSessionsOutput { sessions, cursor }))
}

/// `app.changala.ring.openSession` — POST procedure.
///
/// Transitions a session from `scheduled` → `live`. Sets `opened_at` to
/// the current time.
pub async fn open_session(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingOpenSessionInput>,
) -> Result<Json<AppChangalaRingOpenSessionOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    let course_uri = auth::get_course_for_session(&app.db, &input.session_uri).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &course_uri).await?;

    let row = require_session_row(&app.db, &input.session_uri).await?;

    // Validate current status
    if row.4 != "scheduled" {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "cannot open session with status '{}'; expected 'scheduled'",
                row.4
            ),
        });
    }

    let now = Utc::now().to_rfc3339();

    sqlx::query("UPDATE sessions SET status = 'live', opened_at = $1 WHERE uri = $2")
        .bind(&now)
        .bind(&input.session_uri)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    // Re-fetch the updated row
    let updated = require_session_row(&app.db, &input.session_uri).await?;
    Ok(Json(into_open_output(session_view(&updated))))
}

/// `app.changala.ring.closeSession` — POST procedure.
///
/// Transitions a session from `live` → `ended`. Sets `closed_at` and opens
/// a 60-minute keyword submission window.
pub async fn close_session(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCloseSessionInput>,
) -> Result<Json<AppChangalaRingCloseSessionOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    let course_uri = auth::get_course_for_session(&app.db, &input.session_uri).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &course_uri).await?;

    let row = require_session_row(&app.db, &input.session_uri).await?;

    if row.4 != "live" {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "cannot close session with status '{}'; expected 'live'",
                row.4
            ),
        });
    }

    let now = Utc::now();
    let closed_at = now.to_rfc3339();
    let keyword_expires = (now + Duration::minutes(KEYWORD_WINDOW_MINS)).to_rfc3339();

    sqlx::query(
        "UPDATE sessions SET status = 'ended', closed_at = $1, \
                keyword_window_expires_at = $2 WHERE uri = $3",
    )
    .bind(&closed_at)
    .bind(&keyword_expires)
    .bind(&input.session_uri)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("database error: {e}"),
    })?;

    // Re-fetch updated row and serialise the session as a JSON value
    let updated = require_session_row(&app.db, &input.session_uri).await?;
    let session_json = serde_json::to_value(into_get_output(session_view(&updated))).unwrap();

    Ok(Json(AppChangalaRingCloseSessionOutput {
        session: session_json,
        keyword_window_expires_at: keyword_expires,
    }))
}

/// `app.changala.ring.cancelSession` — POST procedure.
///
/// Cancels a session. Valid from `scheduled` or `live` status.
pub async fn cancel_session(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCancelSessionInput>,
) -> Result<Json<AppChangalaRingCancelSessionOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    let course_uri = auth::get_course_for_session(&app.db, &input.session_uri).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &course_uri).await?;

    let row = require_session_row(&app.db, &input.session_uri).await?;

    if row.4 != "scheduled" && row.4 != "live" {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "cannot cancel session with status '{}'; expected 'scheduled' or 'live'",
                row.4
            ),
        });
    }

    sqlx::query("UPDATE sessions SET status = 'cancelled' WHERE uri = $1")
        .bind(&input.session_uri)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    let updated = require_session_row(&app.db, &input.session_uri).await?;
    Ok(Json(into_cancel_output(session_view(&updated))))
}

/// `app.changala.ring.rescheduleSession` — POST procedure.
///
/// Reschedules a session. Valid only from `scheduled` status.
pub async fn reschedule_session(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingRescheduleSessionInput>,
) -> Result<Json<AppChangalaRingRescheduleSessionOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    let course_uri = auth::get_course_for_session(&app.db, &input.session_uri).await?;
    auth::require_class_rep_or_admin(&app.db, &session.did, &course_uri).await?;

    let row = require_session_row(&app.db, &input.session_uri).await?;

    if row.4 != "scheduled" {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "cannot reschedule session with status '{}'; expected 'scheduled'",
                row.4
            ),
        });
    }

    sqlx::query("UPDATE sessions SET status = 'rescheduled', rescheduled_to = $1 WHERE uri = $2")
        .bind(&input.rescheduled_to)
        .bind(&input.session_uri)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    let updated = require_session_row(&app.db, &input.session_uri).await?;
    Ok(Json(into_reschedule_output(session_view(&updated))))
}
