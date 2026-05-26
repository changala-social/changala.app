//! Identity service handlers — email verification and membership management.

use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::{Query, State};
use axum::Json;
use changala_shared::types::*;
use serde_json::json;

/// POST /xrpc/app.changala.ring.verifyEmail
///
/// Two-step OTP flow:
/// - Call 1: {did, email} → generates OTP, stores in DB, returns {status: "otpSent"}
/// - Call 2: {did, email, otp} → verifies, creates membership, returns {status: "verified", membershipUri}
pub async fn verify_email(
    State(state): State<AppState>,
    Json(input): Json<AppChangalaRingVerifyEmailInput>,
) -> Result<Json<AppChangalaRingVerifyEmailOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    // Validate email domain — must be an institution email
    // For MVP, accept any email. Post-MVP: check against configured domains.

    match input.otp {
        None => {
            // Validate email domain against allowlist
            if let Err(msg) = atrg_email::validate_domain(&input.email, &app.allowed_email_domains)
            {
                return Err(XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: msg,
                });
            }

            // Generate OTP, store in DB, send via email (or log in dev mode)
            let db_pool = atrg_db::DbPool::Postgres(app.db.clone());
            if let Err(e) = atrg_email::send_otp(
                &db_pool,
                app.email_config.as_ref(),
                &input.did,
                &input.email,
            )
            .await
            {
                tracing::error!(email = %input.email, error = %e, "Failed to send OTP");
                return Err(XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: "Failed to send verification email. Please try again.".to_string(),
                });
            }

            Ok(Json(AppChangalaRingVerifyEmailOutput {
                status: "otpSent".to_string(),
                membership_uri: None,
            }))
        }
        Some(otp) => {
            // Validate domain
            if let Err(msg) = atrg_email::validate_domain(&input.email, &app.allowed_email_domains)
            {
                return Err(XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: msg,
                });
            }

            // Verify OTP
            let db_pool = atrg_db::DbPool::Postgres(app.db.clone());
            let valid = atrg_email::verify_otp(&db_pool, &input.did, &input.email, &otp)
                .await
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("OTP verification failed: {e}"),
                })?;

            if !valid {
                return Err(XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: "Invalid or expired OTP".to_string(),
                });
            }

            // Extract institution domain from email
            let domain = input
                .email
                .split('@')
                .nth(1)
                .unwrap_or("unknown")
                .to_string();

            // Create or update membership
            let verified_at = chrono::Utc::now().to_rfc3339();
            let institution_did = format!("did:web:{}", domain.replace('.', "-"));

            sqlx::query(
                "INSERT INTO memberships (did, institution_did, institution_domain, role, verified_email, verified_at) \
                 VALUES ($1, $2, $3, 'student', $4, $5) \
                 ON CONFLICT(did, institution_did) DO UPDATE SET verified_at = excluded.verified_at, verified_email = excluded.verified_email"
            )
            .bind(&input.did)
            .bind(&institution_did)
            .bind(&domain)
            .bind(&input.email)
            .bind(&verified_at)
            .execute(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to create membership: {e}"),
            })?;

            // Construct membership URI (convention: at://<did>/app.changala.membership/<rkey>)
            let rkey = atrg_repo::Tid::now().to_string();
            let membership_uri = format!("at://{}/app.changala.membership/{}", input.did, rkey);

            Ok(Json(AppChangalaRingVerifyEmailOutput {
                status: "verified".to_string(),
                membership_uri: Some(membership_uri),
            }))
        }
    }
}

/// GET /xrpc/app.changala.ring.getMemberships
pub async fn get_memberships(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingGetMembershipsParams>,
) -> Result<Json<AppChangalaRingGetMembershipsOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT institution_did, institution_domain, role, verified_at, membership_uri FROM memberships WHERE did = $1"
    )
    .bind(&params.did)
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch memberships: {e}"),
    })?;

    let memberships: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "institutionDid": r.0,
                "institutionDomain": r.1,
                "role": r.2,
                "verifiedAt": r.3,
                "membershipUri": r.4.as_deref().unwrap_or("")
            })
        })
        .collect();

    Ok(Json(AppChangalaRingGetMembershipsOutput { memberships }))
}

/// GET /xrpc/app.changala.ring.getRole
pub async fn get_role(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaRingGetRoleParams>,
) -> Result<Json<AppChangalaRingGetRoleOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    let role =
        sqlx::query_scalar::<_, String>(
            "SELECT role FROM memberships WHERE did = $1 \
             ORDER BY CASE role WHEN 'admin' THEN 1 WHEN 'classRep' THEN 2 WHEN 'student' THEN 3 ELSE 4 END \
             LIMIT 1",
        )
            .bind(&params.did)
            .fetch_optional(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to fetch role: {e}"),
            })?;

    match role {
        Some(r) => Ok(Json(AppChangalaRingGetRoleOutput { role: Some(r) })),
        None => Ok(Json(AppChangalaRingGetRoleOutput { role: None })),
    }
}
