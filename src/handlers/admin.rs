//! Admin provisioning, role management, and audit log handlers.

use atrg_auth::RequireAuth;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::Query;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::handlers::auth;

// ═══════════════════════════════════════════════════════════════════════════
// Audit log helper
// ═══════════════════════════════════════════════════════════════════════════

/// Write an entry to the audit log.
pub async fn write_audit_log(
    db: &sqlx::PgPool,
    actor_did: &str,
    action: &str,
    target_did: Option<&str>,
    details: Option<serde_json::Value>,
) -> Result<(), XrpcError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO audit_log (actor_did, action, target_did, details, created_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(actor_did)
    .bind(action)
    .bind(target_did)
    .bind(details)
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to write audit log: {e}"),
    })?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// provisionAdmin — bootstrap first admin via shared secret
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct ProvisionAdminInput {
    did: String,
    secret: String,
    institution_domain: Option<String>,
}

/// POST /xrpc/app.changala.ring.provisionAdmin
///
/// Bootstrap an admin account using a shared secret (CHANGALA_ADMIN_SECRET env var).
/// This endpoint does NOT require authentication — it uses the shared secret instead.
/// This allows bootstrapping the very first admin without database access.
pub async fn provision_admin(
    Json(input): Json<ProvisionAdminInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let expected_secret = std::env::var("CHANGALA_ADMIN_SECRET").map_err(|_| XrpcError {
        name: XrpcErrorName::Forbidden,
        message: "Admin provisioning is not configured (CHANGALA_ADMIN_SECRET not set)".to_string(),
    })?;

    if input.secret != expected_secret {
        return Err(XrpcError {
            name: XrpcErrorName::Forbidden,
            message: "Invalid admin secret".to_string(),
        });
    }

    let app = crate::state::get();
    let now = chrono::Utc::now().to_rfc3339();
    let domain = input.institution_domain.as_deref().unwrap_or("system");
    let institution_did = format!("did:web:{}", domain.replace('.', "-"));

    sqlx::query(
        "INSERT INTO memberships (did, institution_did, institution_domain, role, verified_at) \
         VALUES ($1, $2, $3, 'admin', $4) \
         ON CONFLICT (did, institution_did) DO UPDATE SET role = 'admin'",
    )
    .bind(&input.did)
    .bind(&institution_did)
    .bind(domain)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to provision admin: {e}"),
    })?;

    write_audit_log(
        &app.db,
        "system",
        "provision_admin",
        Some(&input.did),
        Some(json!({"method": "shared_secret"})),
    )
    .await
    .ok();

    tracing::info!(did = %input.did, "admin provisioned via shared secret");

    Ok(Json(json!({
        "did": input.did,
        "role": "admin",
        "provisionedAt": now
    })))
}

// ═══════════════════════════════════════════════════════════════════════════
// promoteRole / demoteRole — admin-only role management
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct RoleChangeInput {
    target_did: String,
    role: String,
}

/// POST /xrpc/app.changala.ring.promoteRole
pub async fn promote_role(
    RequireAuth(session): RequireAuth,
    Json(input): Json<RoleChangeInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let valid_roles = ["student", "classRep", "admin"];
    if !valid_roles.contains(&input.role.as_str()) {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "Invalid role '{}'. Valid roles: student, classRep, admin",
                input.role
            ),
        });
    }

    let now = chrono::Utc::now().to_rfc3339();

    let rows_affected = sqlx::query(
        "UPDATE memberships SET role = $1, promoted_by = $2, promoted_at = $3 WHERE did = $4",
    )
    .bind(&input.role)
    .bind(&session.did)
    .bind(&now)
    .bind(&input.target_did)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to update role: {e}"),
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: "No membership found for the target DID".to_string(),
        });
    }

    write_audit_log(
        &app.db,
        &session.did,
        "promote_role",
        Some(&input.target_did),
        Some(json!({"new_role": input.role})),
    )
    .await
    .ok();

    Ok(Json(json!({
        "targetDid": input.target_did,
        "newRole": input.role,
        "promotedBy": session.did,
        "promotedAt": now
    })))
}

/// POST /xrpc/app.changala.ring.demoteRole
pub async fn demote_role(
    RequireAuth(session): RequireAuth,
    Json(input): Json<RoleChangeInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    // Reuse promote_role — same logic, different semantic name
    promote_role(RequireAuth(session), Json(input)).await
}

// ═══════════════════════════════════════════════════════════════════════════
// getAuditLog — admin-only audit log viewer
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct GetAuditLogParams {
    limit: Option<i64>,
    cursor: Option<String>,
}

/// GET /xrpc/app.changala.ring.getAuditLog
pub async fn get_audit_log(
    RequireAuth(session): RequireAuth,
    Query(params): Query<GetAuditLogParams>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = crate::state::get();
    auth::require_role(&app.db, &session.did, "admin").await?;

    let limit = params.limit.unwrap_or(50).clamp(1, 100);

    let rows: Vec<sqlx::postgres::PgRow> = if let Some(ref cursor) = params.cursor {
        sqlx::query(
            "SELECT actor_did, action, target_did, details, created_at \
             FROM audit_log WHERE created_at < $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(cursor)
        .bind(limit + 1)
        .fetch_all(&app.db)
        .await
    } else {
        sqlx::query(
            "SELECT actor_did, action, target_did, details, created_at \
             FROM audit_log ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit + 1)
        .fetch_all(&app.db)
        .await
    }
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch audit log: {e}"),
    })?;

    use sqlx::Row;
    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "actorDid": r.get::<String, _>("actor_did"),
                "action": r.get::<String, _>("action"),
                "targetDid": r.get::<Option<String>, _>("target_did"),
                "details": r.get::<Option<serde_json::Value>, _>("details"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    let cursor = if has_more {
        rows.last().map(|r| r.get::<String, _>("created_at"))
    } else {
        None
    };

    Ok(Json(json!({
        "entries": entries,
        "cursor": cursor
    })))
}
