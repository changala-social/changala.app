//! API key gate middleware for MCP endpoints.
//!
//! Validates `Authorization: Bearer chg_*` tokens against the `api_keys`
//! table and allows/denies access. Used to protect the `/mcp` endpoint.
//!
//! Regular XRPC endpoints no longer need a bridge middleware — atrg 0.2.0's
//! `RequireAuth` extractor handles API keys natively.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// Gate middleware for the `/mcp` endpoint.
///
/// Validates `Bearer chg_*` tokens against the `api_keys` table
/// and allows/denies. No session creation, no header rewriting.
///
/// API keys created via the UI gate MCP access directly.
pub async fn mcp_gate_middleware(db: sqlx::PgPool, req: Request<Body>, next: Next) -> Response {
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

    let key_result = crate::handlers::apikeys::find_api_key(&db, key).await;

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
