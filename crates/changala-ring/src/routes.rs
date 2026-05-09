//! Route wiring for Changala Ring.
//!
//! All Ring XRPC endpoints wired to real handler implementations.
//! Global View endpoints live in the changala-globalview crate.

use atrg_core::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;

use crate::handlers;

pub fn api() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        // Custom client-metadata.json — derives client_uri from client_id
        // instead of using app.host:app.port (which produces http://0.0.0.0:3000).
        // We use atrg_auth::routes::routes() (not auth_router()) in main.rs
        // so there's no route conflict.
        .route("/client-metadata.json", get(client_metadata))
        .route(
            "/.well-known/oauth-protected-resource",
            get(well_known_oauth),
        )
        // Cross-origin session handoff: atrg-auth callback sets an HttpOnly
        // cookie and redirects here (same origin, so cookie is readable).
        // We look up the session and redirect to the frontend with token params.
        .route("/auth/complete", get(auth_complete))
        .merge(xrpc_routes())
}

/// All Ring XRPC routes — every endpoint wired to a real handler.
///
/// The `api_key_auth` middleware layer runs on every XRPC request. It
/// intercepts `Bearer chg_*` tokens, validates them against `api_keys`,
/// and injects a synthetic session so `RequireAuth` works transparently.
fn xrpc_routes() -> Router<AppState> {
    atrg_xrpc::xrpc_router()
        // ── Identity ────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.verifyEmail",
            post(handlers::identity::verify_email),
        )
        .route(
            "/xrpc/app.changala.ring.getMemberships",
            get(handlers::identity::get_memberships),
        )
        .route(
            "/xrpc/app.changala.ring.getRole",
            get(handlers::identity::get_role),
        )
        // ── Course ──────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createCourse",
            post(handlers::course::create_course),
        )
        .route(
            "/xrpc/app.changala.ring.getCourse",
            get(handlers::course::get_course),
        )
        .route(
            "/xrpc/app.changala.ring.listCourses",
            get(handlers::course::list_courses),
        )
        .route(
            "/xrpc/app.changala.ring.enrollStudent",
            post(handlers::course::enroll_student),
        )
        .route(
            "/xrpc/app.changala.ring.getEnrollments",
            get(handlers::course::get_enrollments),
        )
        .route(
            "/xrpc/app.changala.ring.assignClassRep",
            post(handlers::course::assign_class_rep),
        )
        // ── Session ─────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createSession",
            post(handlers::session::create_session),
        )
        .route(
            "/xrpc/app.changala.ring.getSession",
            get(handlers::session::get_session),
        )
        .route(
            "/xrpc/app.changala.ring.listSessions",
            get(handlers::session::list_sessions),
        )
        .route(
            "/xrpc/app.changala.ring.openSession",
            post(handlers::session::open_session),
        )
        .route(
            "/xrpc/app.changala.ring.closeSession",
            post(handlers::session::close_session),
        )
        .route(
            "/xrpc/app.changala.ring.cancelSession",
            post(handlers::session::cancel_session),
        )
        .route(
            "/xrpc/app.changala.ring.rescheduleSession",
            post(handlers::session::reschedule_session),
        )
        // ── Notes & Keywords ────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.addKeyword",
            post(handlers::note::add_keyword),
        )
        .route(
            "/xrpc/app.changala.ring.createNote",
            post(handlers::note::create_note),
        )
        .route(
            "/xrpc/app.changala.ring.versionNote",
            post(handlers::note::version_note),
        )
        .route(
            "/xrpc/app.changala.ring.getNoteContent",
            get(handlers::note::get_note_content),
        )
        .route(
            "/xrpc/app.changala.ring.getNoteHistory",
            get(handlers::note::get_note_history),
        )
        // ── Collective Note ─────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.proposeEdit",
            post(handlers::note::propose_edit),
        )
        .route(
            "/xrpc/app.changala.ring.acceptEdit",
            post(handlers::note::accept_edit),
        )
        .route(
            "/xrpc/app.changala.ring.rejectEdit",
            post(handlers::note::reject_edit),
        )
        .route(
            "/xrpc/app.changala.ring.getCollectiveNote",
            get(handlers::note::get_collective_note),
        )
        .route(
            "/xrpc/app.changala.ring.listEditProposals",
            get(handlers::note::list_edit_proposals),
        )
        // ── Vote ────────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.registerVote",
            post(handlers::note::register_vote),
        )
        // ── Label ───────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.applyLabel",
            post(handlers::note::apply_label),
        )
        .route(
            "/xrpc/app.changala.ring.retractLabel",
            post(handlers::note::retract_label),
        )
        // ── Moderation ──────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.banDid",
            post(handlers::moderation::ban_did),
        )
        .route(
            "/xrpc/app.changala.ring.liftBan",
            post(handlers::moderation::lift_ban),
        )
        .route(
            "/xrpc/app.changala.ring.listBans",
            get(handlers::moderation::list_bans),
        )
        .route(
            "/xrpc/app.changala.ring.isBanned",
            get(handlers::moderation::is_banned),
        )
        // ── Admin Provisioning ──────────────────────────────
        .route(
            "/xrpc/app.changala.ring.provisionAdmin",
            post(handlers::admin::provision_admin),
        )
        .route(
            "/xrpc/app.changala.ring.promoteRole",
            post(handlers::admin::promote_role),
        )
        .route(
            "/xrpc/app.changala.ring.demoteRole",
            post(handlers::admin::demote_role),
        )
        .route(
            "/xrpc/app.changala.ring.getAuditLog",
            get(handlers::admin::get_audit_log),
        )
        // ── API Keys ────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createApiKey",
            post(handlers::apikeys::create_api_key),
        )
        .route(
            "/xrpc/app.changala.ring.listApiKeys",
            get(handlers::apikeys::list_api_keys),
        )
        .route(
            "/xrpc/app.changala.ring.revokeApiKey",
            post(handlers::apikeys::revoke_api_key),
        )
        // ── Archive ─────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.initiateArchive",
            post(handlers::archive::initiate_archive),
        )
        .route(
            "/xrpc/app.changala.ring.sealArchive",
            post(handlers::archive::seal_archive),
        )
        .route(
            "/xrpc/app.changala.ring.exportArchive",
            post(handlers::archive::export_archive),
        )
        .route(
            "/xrpc/app.changala.ring.uploadToInternetArchive",
            post(handlers::archive::upload_to_internet_archive),
        )
        .route(
            "/xrpc/app.changala.ring.getArchive",
            get(handlers::archive::get_archive),
        )
        // ── Brain ───────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createNode",
            post(handlers::brain::create_node),
        )
        .route(
            "/xrpc/app.changala.ring.versionNode",
            post(handlers::brain::version_node),
        )
        .route(
            "/xrpc/app.changala.ring.getNodeContent",
            get(handlers::brain::get_node_content),
        )
        .route(
            "/xrpc/app.changala.ring.createLink",
            post(handlers::brain::create_link),
        )
        .route(
            "/xrpc/app.changala.ring.deleteLink",
            post(handlers::brain::delete_link),
        )
        // ── API key → session bridge ────────────────────────
        // Must be LAST: axum layers only wrap routes added before them.
        .layer(axum::middleware::from_fn(
            crate::api_key_auth::api_key_auth_middleware,
        ))
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "name": "changala-ring",
        "description": "Changala Ring — per-institution write server on AT Protocol",
        "status": "ok"
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "healthy": true }))
}

/// Custom client-metadata.json that derives `client_uri` from `client_id`.
///
/// The AT Protocol OAuth spec requires `client_uri` to use the same hostname
/// as `client_id`, or localhost for development. atrg-auth's built-in handler
/// uses `http://{host}:{port}` which produces `http://0.0.0.0:3000` — rejected
/// by PDS validation. This override extracts the origin from `client_id`.
async fn client_metadata(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = &state.config.auth;

    // Derive client_uri from client_id: "https://example.com/client-metadata.json" → "https://example.com"
    let client_uri = {
        // Strip the path: find the third slash (after "https://host")
        let id = &config.client_id;
        match id.find("://") {
            Some(scheme_end) => {
                let after_scheme = &id[scheme_end + 3..];
                match after_scheme.find('/') {
                    Some(path_start) => id[..scheme_end + 3 + path_start].to_string(),
                    None => id.clone(),
                }
            }
            None => format!("http://{}:{}", state.config.app.host, state.config.app.port),
        }
    };

    Json(json!({
        "client_id": config.client_id,
        "client_name": state.config.app.name,
        "client_uri": client_uri,
        "redirect_uris": [config.redirect_uri],
        "scope": config.scope,
        "grant_types": ["authorization_code", "refresh_token"],
        "response_types": ["code"],
        "application_type": "web",
        "token_endpoint_auth_method": "none",
        "dpop_bound_access_tokens": true
    }))
}

/// OAuth protected resource metadata.
async fn well_known_oauth(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config = &state.config.auth;
    // Derive base URL from client_id origin (same logic as client_metadata)
    let base_url = {
        let id = &config.client_id;
        match id.find("://") {
            Some(scheme_end) => {
                let after_scheme = &id[scheme_end + 3..];
                match after_scheme.find('/') {
                    Some(path_start) => id[..scheme_end + 3 + path_start].to_string(),
                    None => id.clone(),
                }
            }
            None => format!("http://{}:{}", state.config.app.host, state.config.app.port),
        }
    };
    Json(json!({
        "resource": base_url,
        "authorization_servers": [],
        "scopes_supported": [config.scope],
        "bearer_methods_supported": ["header"]
    }))
}

/// `GET /auth/complete`
///
/// Cross-origin session handoff. After atrg-auth's `/auth/callback` sets the
/// `atrg_session` HttpOnly cookie and redirects here (same origin → cookie is
/// present), this handler:
/// 1. Reads the `atrg_session` cookie
/// 2. Looks up the session in the DB to get `did` and `handle`
/// 3. Redirects to the frontend with `?token=...&did=...&handle=...` in the URL
///
/// This avoids the cross-origin cookie problem: the cookie is only used on the
/// Ring's own domain, and the frontend receives the session info via URL params.
/// Query params for the session handoff endpoint.
#[derive(serde::Deserialize)]
struct AuthCompleteParams {
    /// The frontend URL to redirect to. Passed through the OAuth flow so
    /// /auth/complete knows where the user's browser came from.
    /// Falls back to `post_login_redirect` in atrg.toml.
    frontend: Option<String>,
}

async fn auth_complete(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<AuthCompleteParams>,
    headers: HeaderMap,
) -> Response {
    let frontend = params
        .frontend
        .filter(|f| !f.trim().is_empty())
        .unwrap_or_else(|| state.config.auth.post_login_redirect.clone());

    // 1. Extract atrg_session from the cookie header
    let session_id = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|c| {
                let c = c.trim();
                c.strip_prefix("atrg_session=")
            })
        });

    let Some(session_id) = session_id else {
        return axum::response::Redirect::temporary(&format!("{}?error=no_session", frontend))
            .into_response();
    };

    // 2. Look up session in DB
    let session = atrg_auth::session::find_session(&state.db, session_id).await;

    match session {
        Ok(Some(s)) => {
            let redirect_url = format!(
                "{}?token={}&did={}&handle={}",
                frontend,
                urlencoding::encode(session_id),
                urlencoding::encode(&s.did),
                urlencoding::encode(&s.handle),
            );
            axum::response::Redirect::temporary(&redirect_url).into_response()
        }
        _ => axum::response::Redirect::temporary(&format!("{}?error=invalid_session", frontend))
            .into_response(),
    }
}
