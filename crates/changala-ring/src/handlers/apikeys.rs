//! API key management handlers.

use atrg_auth::RequireAuth;
use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::State;
use axum::Json;
use base64::Engine;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::handlers::auth;

/// Generate a random API key with `chg_` prefix.
fn generate_api_key() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    format!("chg_{}", encoded)
}

/// Hash an API key with SHA-256 for storage.
fn hash_key(key: &str) -> String {
    let hash = Sha256::digest(key.as_bytes());
    format!("sha256-{}", hex::encode(hash))
}

/// Extract the prefix (first 12 chars) from a key for identification.
fn key_prefix(key: &str) -> String {
    key.chars().take(12).collect()
}

#[derive(Deserialize)]
pub struct CreateApiKeyInput {
    name: String,
    scopes: Option<Vec<String>>,
    expires_in_days: Option<i64>,
}

/// POST /xrpc/app.changala.ring.createApiKey
/// Admin only — creates a new API key.
pub async fn create_api_key(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<CreateApiKeyInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let key = generate_api_key();
    let hash = hash_key(&key);
    let prefix = key_prefix(&key);
    let scopes = serde_json::to_string(&input.scopes.unwrap_or_else(|| vec!["read:*".to_string()]))
        .unwrap_or_else(|_| "[]".to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let expires_at = input
        .expires_in_days
        .map(|days| (chrono::Utc::now() + chrono::Duration::days(days)).to_rfc3339());

    sqlx::query(
        "INSERT INTO api_keys (key_hash, key_prefix, did, name, scopes, expires_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&hash)
    .bind(&prefix)
    .bind(&session.did)
    .bind(&input.name)
    .bind(&scopes)
    .bind(&expires_at)
    .bind(&now)
    .execute(&app.db)
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
        Some(json!({"name": input.name, "prefix": prefix})),
    )
    .await;

    // Return the key ONCE — it cannot be retrieved again
    Ok(Json(json!({
        "key": key,
        "prefix": prefix,
        "name": input.name,
        "scopes": serde_json::from_str::<serde_json::Value>(&scopes).unwrap_or_default(),
        "expiresAt": expires_at,
        "createdAt": now
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

    use sqlx::Row;
    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        "SELECT key_prefix, did, name, scopes, expires_at, created_at, last_used_at \
         FROM api_keys ORDER BY created_at DESC",
    )
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to list API keys: {e}"),
    })?;

    let keys: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "prefix": r.get::<String, _>("key_prefix"),
                "did": r.get::<String, _>("did"),
                "name": r.get::<String, _>("name"),
                "scopes": serde_json::from_str::<serde_json::Value>(
                    &r.get::<String, _>("scopes")
                ).unwrap_or_default(),
                "expiresAt": r.get::<Option<String>, _>("expires_at"),
                "createdAt": r.get::<String, _>("created_at"),
                "lastUsedAt": r.get::<Option<String>, _>("last_used_at"),
            })
        })
        .collect();

    Ok(Json(json!({ "keys": keys })))
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

    let result = sqlx::query("DELETE FROM api_keys WHERE key_prefix = $1")
        .bind(&input.prefix)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to revoke API key: {e}"),
        })?;

    if result.rows_affected() == 0 {
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

/// Look up an API key by its hash. Returns (did, scopes) if valid.
/// Called by the auth middleware to authenticate API key requests.
#[allow(dead_code)]
pub async fn find_api_key(
    db: &sqlx::PgPool,
    key: &str,
) -> Result<Option<(String, String)>, XrpcError> {
    let hash = hash_key(key);
    let now = chrono::Utc::now().to_rfc3339();

    use sqlx::Row;
    let row: Option<sqlx::postgres::PgRow> = sqlx::query(
        "SELECT did, scopes FROM api_keys \
         WHERE key_hash = $1 AND (expires_at IS NULL OR expires_at > $2)",
    )
    .bind(&hash)
    .bind(&now)
    .fetch_optional(db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("API key lookup failed: {e}"),
    })?;

    if row.is_some() {
        // Update last_used_at
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = $1 WHERE key_hash = $2")
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(&hash)
            .execute(db)
            .await;
    }

    Ok(row.map(|r| (r.get("did"), r.get("scopes"))))
}
