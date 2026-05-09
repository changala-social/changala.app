//! API key → session bridge middleware.
//!
//! `atrg-auth`'s `RequireAuth` extractor only knows about OAuth sessions
//! stored in `atrg_sessions`. Changala API keys (`chg_*`) live in a
//! separate `api_keys` table and are invisible to `RequireAuth`.
//!
//! This middleware intercepts `Authorization: Bearer chg_*` requests,
//! validates the key against `api_keys`, and injects a short-lived
//! synthetic session into `atrg_sessions`. The downstream `RequireAuth`
//! extractor then picks up this session transparently.
//!
//! The synthetic session is keyed by a deterministic ID derived from the
//! API key hash, so repeated calls with the same key reuse the same row
//! (upsert). Sessions expire after 5 minutes and are cleaned up by
//! atrg-auth's normal cleanup task.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};

/// Axum middleware that bridges `chg_*` API keys to atrg sessions.
///
/// Apply to the XRPC router so that all endpoints benefit:
///
/// ```ignore
/// let router = xrpc_routes()
///     .layer(axum::middleware::from_fn(api_key_auth::api_key_auth_middleware));
/// ```
pub async fn api_key_auth_middleware(req: Request<Body>, next: Next) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());

    // Only intercept chg_* keys — let everything else (JWTs, session
    // tokens, no-auth) pass through to RequireAuth unchanged.
    let Some(ref key) = auth_header else {
        return next.run(req).await;
    };
    if !key.starts_with("chg_") {
        return next.run(req).await;
    }

    let app = crate::state::get();

    // Validate the API key against the api_keys table.
    let key_result = crate::handlers::apikeys::find_api_key(&app.db, key).await;

    let (did, _scopes) = match key_result {
        Ok(Some(pair)) => pair,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": "InvalidApiKey",
                    "message": "API key is invalid or expired"
                })),
            )
                .into_response();
        }
        Err(_e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "error": "InternalServerError",
                    "message": "Failed to validate API key"
                })),
            )
                .into_response();
        }
    };

    // Create a deterministic session ID from the key hash so repeated
    // calls with the same key hit the same row (upsert).
    let session_id = {
        let hash = Sha256::digest(key.as_bytes());
        format!("apikey-{}", hex::encode(&hash[..16]))
    };

    // Upsert a short-lived synthetic session into atrg_sessions.
    // 5-minute TTL — long enough for a burst of MCP tool calls,
    // short enough to limit exposure if a key is leaked.
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        + 300;

    let upsert_result = sqlx::query(
        "INSERT INTO atrg_sessions (id, did, handle, access_token, refresh_token, expires_at)
         VALUES ($1, $2, '', $3, NULL, $4)
         ON CONFLICT (id) DO UPDATE SET expires_at = $4",
    )
    .bind(&session_id)
    .bind(&did)
    .bind(key) // access_token = the API key itself (not used downstream)
    .bind(expires_at)
    .execute(&app.db)
    .await;

    if let Err(e) = upsert_result {
        tracing::error!(error = %e, "failed to upsert API key session");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "error": "InternalServerError",
                "message": "Failed to create session for API key"
            })),
        )
            .into_response();
    }

    // Rewrite the Authorization header to carry the synthetic session ID
    // so RequireAuth's `resolve_bearer_token` → `resolve_atrg_session`
    // path picks it up.
    let mut req = req;
    req.headers_mut().insert(
        header::AUTHORIZATION,
        format!("Bearer {}", session_id)
            .parse()
            .expect("valid header value"),
    );

    next.run(req).await
}

/// Simpler gate middleware for the `/mcp` endpoint.
///
/// Unlike [`api_key_auth_middleware`] (which bridges keys to atrg sessions),
/// this just validates `Bearer chg_*` tokens against the `api_keys` table
/// and allows/denies. No session creation, no header rewriting.
///
/// This replaces the old env-var-based `CHANGALA_MCP_ACCESS_KEY` check —
/// API keys created via the UI now gate MCP access directly.
pub async fn mcp_gate_middleware(req: Request<Body>, next: Next) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string());

    let Some(ref key) = auth_header else {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Bearer token required for MCP access"
            })),
        )
            .into_response();
    };

    let app = crate::state::get();
    let key_result = crate::handlers::apikeys::find_api_key(&app.db, key).await;

    match key_result {
        Ok(Some(_)) => next.run(req).await,
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({
                "error": "InvalidApiKey",
                "message": "API key is invalid or expired"
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "error": "InternalServerError",
                "message": "Failed to validate API key"
            })),
        )
            .into_response(),
    }
}
