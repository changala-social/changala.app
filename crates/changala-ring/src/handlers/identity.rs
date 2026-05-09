//! Identity service handlers — email verification and membership management.

use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::Query;
use axum::Json;
use changala_shared::types::*;
use serde_json::json;

/// POST /xrpc/app.changala.ring.verifyEmail
///
/// Two-step OTP flow:
/// - Call 1: {did, email} → generates OTP, stores in DB, returns {status: "otpSent"}
/// - Call 2: {did, email, otp} → verifies, creates membership, returns {status: "verified", membershipUri}
pub async fn verify_email(
    Json(input): Json<AppChangalaRingVerifyEmailInput>,
) -> Result<Json<AppChangalaRingVerifyEmailOutput>, XrpcError> {
    let app = crate::state::get();
    // Validate email domain — must be an institution email
    // For MVP, accept any email. Post-MVP: check against configured domains.

    match input.otp {
        None => {
            // Validate email domain against allowlist
            let domain = input.email.split('@').nth(1).unwrap_or("").to_lowercase();
            let allowed = &app.allowed_email_domains;
            if !allowed.is_empty() && !allowed.iter().any(|d| d == &domain) {
                return Err(XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: format!(
                        "Email domain '{}' is not allowed. Use your institution email (allowed: {}).",
                        domain,
                        allowed.join(", ")
                    ),
                });
            }

            // Step 1: Generate and store OTP
            let code = generate_otp();
            let expires_at = chrono::Utc::now().timestamp() + 600; // 10 minutes

            sqlx::query(
                "INSERT INTO otp_codes (did, email, code, expires_at) VALUES ($1, $2, $3, $4)",
            )
            .bind(&input.did)
            .bind(&input.email)
            .bind(&code)
            .bind(expires_at)
            .execute(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to store OTP: {e}"),
            })?;

            // Send OTP via email (or log in dev mode if SMTP not configured)
            if let Err(e) =
                crate::email::send_otp_email(app.smtp.as_ref(), &input.email, &code).await
            {
                tracing::error!(email = %input.email, error = %e, "Failed to send OTP email");
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
            // Validate email domain against allowlist (prevents submitting OTP for disallowed domain)
            let domain = input.email.split('@').nth(1).unwrap_or("").to_lowercase();
            let allowed = &app.allowed_email_domains;
            if !allowed.is_empty() && !allowed.iter().any(|d| d == &domain) {
                return Err(XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: format!(
                        "Email domain '{}' is not allowed. Use your institution email (allowed: {}).",
                        domain,
                        allowed.join(", ")
                    ),
                });
            }

            // Step 2: Verify OTP
            let now = chrono::Utc::now().timestamp();

            let valid = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM otp_codes WHERE did = $1 AND email = $2 AND code = $3 AND expires_at > $4 AND used = FALSE"
            )
            .bind(&input.did)
            .bind(&input.email)
            .bind(&otp)
            .bind(now)
            .fetch_one(&app.db)
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
            sqlx::query(
                "UPDATE otp_codes SET used = TRUE WHERE did = $1 AND email = $2 AND code = $3",
            )
            .bind(&input.did)
            .bind(&input.email)
            .bind(&otp)
            .execute(&app.db)
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
    Query(params): Query<AppChangalaRingGetMembershipsParams>,
) -> Result<Json<AppChangalaRingGetMembershipsOutput>, XrpcError> {
    let app = crate::state::get();
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
    Query(params): Query<AppChangalaRingGetRoleParams>,
) -> Result<Json<AppChangalaRingGetRoleOutput>, XrpcError> {
    let app = crate::state::get();
    let role =
        sqlx::query_scalar::<_, String>("SELECT role FROM memberships WHERE did = $1 LIMIT 1")
            .bind(&params.did)
            .fetch_optional(&app.db)
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
