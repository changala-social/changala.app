//! Shared authentication and authorization helpers.
//!
//! Provides role checks, ban enforcement, and enrollment verification
//! used across all handler modules.

use atrg_xrpc::{XrpcError, XrpcErrorName};
use sqlx::PgPool;

/// Check if a DID is banned. Returns Err(Forbidden) if actively banned.
pub async fn check_not_banned(db: &PgPool, did: &str) -> Result<(), XrpcError> {
    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT 1 FROM bans WHERE target_did = $1 AND (permanent = TRUE OR expires_at > NOW()::TEXT)",
    )
    .bind(did)
    .fetch_optional(db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Ban check failed: {e}"),
    })?;

    if row.is_some() {
        Err(XrpcError {
            name: XrpcErrorName::Forbidden,
            message: "This account has been suspended".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Get the highest-privilege role for a DID. Returns None if no membership.
pub async fn get_role(db: &PgPool, did: &str) -> Result<Option<String>, XrpcError> {
    sqlx::query_scalar::<_, String>(
        "SELECT role FROM memberships WHERE did = $1 \
         ORDER BY CASE role WHEN 'admin' THEN 1 WHEN 'classRep' THEN 2 WHEN 'student' THEN 3 ELSE 4 END \
         LIMIT 1",
    )
    .bind(did)
    .fetch_optional(db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Role lookup failed: {e}"),
    })
}

/// Require at least the given role level. admin > classRep > student.
pub async fn require_role(db: &PgPool, did: &str, required: &str) -> Result<(), XrpcError> {
    let role = get_role(db, did).await?.ok_or_else(|| XrpcError {
        name: XrpcErrorName::Forbidden,
        message: "No verified membership found — verify your institution email first".to_string(),
    })?;

    if role_level(&role) > role_level(required) {
        Err(XrpcError {
            name: XrpcErrorName::Forbidden,
            message: format!("Requires {required} role or higher (you have {role})"),
        })
    } else {
        Ok(())
    }
}

/// Check if a DID is enrolled in a specific course.
#[allow(dead_code)]
pub async fn require_enrolled(db: &PgPool, did: &str, course_uri: &str) -> Result<(), XrpcError> {
    let row =
        sqlx::query_as::<_, (i64,)>("SELECT 1 FROM enrollments WHERE did = $1 AND course_uri = $2")
            .bind(did)
            .bind(course_uri)
            .fetch_optional(db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Enrollment check failed: {e}"),
            })?;

    if row.is_none() {
        Err(XrpcError {
            name: XrpcErrorName::Forbidden,
            message: "You must be enrolled in this course".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Require that the DID has a verified institution membership.
/// AT Protocol login alone is not enough — the user must have verified
/// an institution email. Returns Err(Forbidden) if no membership exists.
pub async fn require_institution_member(db: &PgPool, did: &str) -> Result<(), XrpcError> {
    let has_membership =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM memberships WHERE did = $1")
            .bind(did)
            .fetch_one(db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Membership check failed: {e}"),
            })?;

    if has_membership == 0 {
        Err(XrpcError {
            name: XrpcErrorName::Forbidden,
            message: "Institution email verification required. Verify your institution email to access this feature.".to_string(),
        })
    } else {
        Ok(())
    }
}

/// Check if a DID is the class rep for a course, or an admin.
pub async fn require_class_rep_or_admin(
    db: &PgPool,
    did: &str,
    course_uri: &str,
) -> Result<(), XrpcError> {
    // Check if class rep
    let is_rep =
        sqlx::query_as::<_, (i64,)>("SELECT 1 FROM courses WHERE uri = $1 AND class_rep_did = $2")
            .bind(course_uri)
            .bind(did)
            .fetch_optional(db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Class rep check failed: {e}"),
            })?;

    if is_rep.is_some() {
        return Ok(());
    }

    // Fall back to admin check
    let role = get_role(db, did).await?;
    if role.as_deref() == Some("admin") {
        return Ok(());
    }

    Err(XrpcError {
        name: XrpcErrorName::Forbidden,
        message: "Only the class rep or an admin can perform this action".to_string(),
    })
}

/// Look up the course_uri for a given session_uri.
pub async fn get_course_for_session(db: &PgPool, session_uri: &str) -> Result<String, XrpcError> {
    sqlx::query_scalar::<_, String>("SELECT course_uri FROM sessions WHERE uri = $1")
        .bind(session_uri)
        .fetch_optional(db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Session lookup failed: {e}"),
        })?
        .ok_or_else(|| XrpcError {
            name: XrpcErrorName::NotFound,
            message: "Session not found".to_string(),
        })
}

fn role_level(role: &str) -> u8 {
    match role {
        "admin" => 1,
        "classRep" => 2,
        "student" => 3,
        _ => 255,
    }
}
