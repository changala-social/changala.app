//! Identity service handlers — email verification and membership management.

use crate::generated::types::*;
use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::Query;
use axum::{extract::State, Json};
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
    // Validate email domain — must be an institution email
    // For MVP, accept any email. Post-MVP: check against configured domains.

    match input.otp {
        None => {
            // Step 1: Generate and store OTP
            let code = generate_otp();
            let expires_at = chrono::Utc::now().timestamp() + 600; // 10 minutes

            sqlx::query("INSERT INTO otp_codes (did, email, code, expires_at) VALUES (?, ?, ?, ?)")
                .bind(&input.did)
                .bind(&input.email)
                .bind(&code)
                .bind(expires_at)
                .execute(&state.db)
                .await
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("Failed to store OTP: {e}"),
                })?;

            // In production, send email here. For MVP, log it.
            tracing::info!(did = %input.did, email = %input.email, otp = %code, "OTP generated (dev mode — logged, not emailed)");

            Ok(Json(AppChangalaRingVerifyEmailOutput {
                status: "otpSent".to_string(),
                membership_uri: None,
            }))
        }
        Some(otp) => {
            // Step 2: Verify OTP
            let now = chrono::Utc::now().timestamp();

            let valid = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM otp_codes WHERE did = ? AND email = ? AND code = ? AND expires_at > ? AND used = 0"
            )
            .bind(&input.did)
            .bind(&input.email)
            .bind(&otp)
            .bind(now)
            .fetch_one(&state.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("OTP lookup failed: {e}"),
            })?;

            if valid == 0 {
                return Err(XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: "Invalid or expired OTP".to_string(),
                });
            }

            // Mark OTP as used
            sqlx::query("UPDATE otp_codes SET used = 1 WHERE did = ? AND email = ? AND code = ?")
                .bind(&input.did)
                .bind(&input.email)
                .bind(&otp)
                .execute(&state.db)
                .await
                .ok();

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
                 VALUES (?, ?, ?, 'student', ?, ?) \
                 ON CONFLICT(did, institution_did) DO UPDATE SET verified_at = excluded.verified_at, verified_email = excluded.verified_email"
            )
            .bind(&input.did)
            .bind(&institution_did)
            .bind(&domain)
            .bind(&input.email)
            .bind(&verified_at)
            .execute(&state.db)
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
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
        "SELECT institution_did, institution_domain, role, verified_at, membership_uri FROM memberships WHERE did = ?"
    )
    .bind(&params.did)
    .fetch_all(&state.db)
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
    let role =
        sqlx::query_scalar::<_, String>("SELECT role FROM memberships WHERE did = ? LIMIT 1")
            .bind(&params.did)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to fetch role: {e}"),
            })?;

    match role {
        Some(r) => Ok(Json(AppChangalaRingGetRoleOutput { role: r })),
        None => Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: "No verified membership found for this DID".to_string(),
        }),
    }
}

/// Generate a 6-digit OTP code
fn generate_otp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:06}", seed % 1_000_000)
}
