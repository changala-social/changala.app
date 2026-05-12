//! Moderation service handlers — DID ban management.

use atrg_auth::RequireAuth;
use atrg_core::AppState;
use axum::extract::{Query, State};
use axum::Json;

use atrg_xrpc::{XrpcError, XrpcErrorName};

use changala_shared::types::*;

use super::auth;

/// POST /xrpc/app.changala.ring.banDid
pub async fn ban_did(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingBanDidInput>,
) -> Result<Json<AppChangalaRingBanDidOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::require_role(&app.db, &session.did, "admin").await?;

    let banned_at = chrono::Utc::now().to_rfc3339();
    let permanent = input.ttl_seconds.is_none();
    let expires_at = input
        .ttl_seconds
        .map(|ttl| (chrono::Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339());

    sqlx::query(
        "INSERT INTO bans (target_did, permanent, expires_at, reason, banned_at) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT(target_did) DO UPDATE SET permanent = excluded.permanent, expires_at = excluded.expires_at, reason = excluded.reason, banned_at = excluded.banned_at"
    )
    .bind(&input.target_did)
    .bind(permanent)
    .bind(&expires_at)
    .bind(&input.reason)
    .bind(&banned_at)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to ban DID: {e}"),
    })?;

    Ok(Json(AppChangalaRingBanDidOutput {
        target_did: input.target_did,
        permanent,
        expires_at,
        banned_at,
    }))
}

/// POST /xrpc/app.changala.ring.liftBan
pub async fn lift_ban(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingLiftBanInput>,
) -> Result<Json<AppChangalaRingLiftBanOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::require_role(&app.db, &session.did, "admin").await?;
    let result = sqlx::query("DELETE FROM bans WHERE target_did = $1")
        .bind(&input.target_did)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to lift ban: {e}"),
        })?;

    if result.rows_affected() == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: "No active ban found for this DID".to_string(),
        });
    }

    Ok(Json(AppChangalaRingLiftBanOutput {
        target_did: input.target_did,
        lifted_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// GET /xrpc/app.changala.ring.listBans
pub async fn list_bans(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Query(params): Query<AppChangalaRingListBansParams>,
) -> Result<Json<AppChangalaRingListBansOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::require_role(&app.db, &session.did, "admin").await?;
    let limit = params.limit.unwrap_or(50).min(100);

    let mut query_str =
        String::from("SELECT target_did, permanent, expires_at, reason, banned_at FROM bans");
    let mut conditions: Vec<String> = Vec::new();
    let mut param_idx = 0u32;

    if params.permanent_only == Some(true) {
        conditions.push("permanent = TRUE".to_string());
    }

    if params.cursor.is_some() {
        param_idx += 1;
        conditions.push(format!("banned_at < ${param_idx}"));
    }

    if !conditions.is_empty() {
        query_str.push_str(" WHERE ");
        query_str.push_str(&conditions.join(" AND "));
    }
    param_idx += 1;
    query_str.push_str(&format!(" ORDER BY banned_at DESC LIMIT ${param_idx}"));

    // Build query with dynamic bindings
    let mut q =
        sqlx::query_as::<_, (String, bool, Option<String>, Option<String>, String)>(&query_str);

    if let Some(ref cursor) = params.cursor {
        q = q.bind(cursor);
    }
    q = q.bind(limit);

    let rows = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to list bans: {e}"),
    })?;

    let next_cursor = rows.last().map(|r| r.4.clone());

    let bans: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "targetDid": r.0,
                "permanent": r.1,
                "expiresAt": r.2,
                "reason": r.3,
                "bannedAt": r.4
            })
        })
        .collect();

    Ok(Json(AppChangalaRingListBansOutput {
        bans,
        cursor: next_cursor,
    }))
}

/// GET /xrpc/app.changala.ring.isBanned
pub async fn is_banned(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingIsBannedParams>,
) -> Result<Json<AppChangalaRingIsBannedOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    let row = sqlx::query_as::<_, (bool, Option<String>)>(
        "SELECT 1::BIGINT, expires_at FROM bans WHERE target_did = $1 AND (permanent = TRUE OR expires_at > NOW()::TEXT)"
    )
    .bind(&params.did)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Ban check failed: {e}"),
    })?;

    match row {
        Some((_, expires_at)) => Ok(Json(AppChangalaRingIsBannedOutput {
            banned: true,
            expires_at,
        })),
        None => Ok(Json(AppChangalaRingIsBannedOutput {
            banned: false,
            expires_at: None,
        })),
    }
}
