//! API key management handlers — delegates to atrg-auth's native API key module.

use atrg_auth::RequireAuth;
use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::handlers::auth;

#[derive(Deserialize)]
pub struct CreateApiKeyInput {
    name: String,
    scopes: Option<Vec<String>>,
    #[allow(dead_code)]
    expires_in_days: Option<i64>,
}

/// POST /xrpc/app.changala.ring.createApiKey
/// Admin only — creates a new API key using atrg-auth's native module.
pub async fn create_api_key(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<CreateApiKeyInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let scopes = input.scopes.unwrap_or_else(|| vec!["read:*".to_string()]);
    let db_pool = atrg_db::DbPool::Postgres(app.db.clone());

    let (full_key, api_key) =
        atrg_auth::api_keys::create_api_key(&db_pool, &session.did, &input.name, &scopes, "chg_")
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to create API key: {e}"),
            })?;

    // Log to audit
    let _ = crate::handlers::admin::write_audit_log(
        &app.db,
        &session.did,
        "create_api_key",
        None,
        Some(json!({"name": input.name, "prefix": api_key.key_prefix})),
    )
    .await;

    // Return the key ONCE — it cannot be retrieved again
    Ok(Json(json!({
        "key": full_key,
        "prefix": api_key.key_prefix,
        "name": api_key.name,
        "scopes": api_key.scopes,
        "expiresAt": api_key.expires_at,
        "createdAt": api_key.created_at
    })))
}

/// GET /xrpc/app.changala.ring.listApiKeys
/// Admin only — lists all API keys (prefix + name, never the full key).
pub async fn list_api_keys(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::require_role(&app.db, &session.did, "admin").await?;

    let db_pool = atrg_db::DbPool::Postgres(app.db.clone());
    let keys = atrg_auth::api_keys::list_api_keys(&db_pool, None)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to list API keys: {e}"),
        })?;

    let keys_json: Vec<serde_json::Value> = keys
        .iter()
        .map(|k| {
            json!({
                "prefix": k.key_prefix,
                "did": k.did,
                "name": k.name,
                "scopes": k.scopes,
                "expiresAt": k.expires_at,
                "createdAt": k.created_at,
                "lastUsedAt": k.last_used_at,
            })
        })
        .collect();

    Ok(Json(json!({ "keys": keys_json })))
}

#[derive(Deserialize)]
pub struct RevokeApiKeyInput {
    prefix: String,
}

/// POST /xrpc/app.changala.ring.revokeApiKey
/// Admin only — revokes an API key by its prefix.
pub async fn revoke_api_key(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<RevokeApiKeyInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let db_pool = atrg_db::DbPool::Postgres(app.db.clone());
    let revoked = atrg_auth::api_keys::revoke_api_key(&db_pool, &input.prefix)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to revoke API key: {e}"),
        })?;

    if !revoked {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("No API key found with prefix '{}'", input.prefix),
        });
    }

    let _ = crate::handlers::admin::write_audit_log(
        &app.db,
        &session.did,
        "revoke_api_key",
        None,
        Some(json!({"prefix": input.prefix})),
    )
    .await;

    Ok(Json(json!({ "revoked": true, "prefix": input.prefix })))
}
