//! Shared authentication and authorization helpers.
//!
//! Provides role checks, ban enforcement, and enrollment verification
//! used across all handler modules.

use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};

/// Check if a DID is banned. Returns Err(Forbidden) if actively banned.
pub async fn check_not_banned(state: &AppState, did: &str) -> Result<(), XrpcError> {
    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT 1 FROM bans WHERE target_did = ? AND (permanent = 1 OR expires_at > datetime('now'))",
    )
    .bind(did)
    .fetch_optional(&state.db)
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
pub async fn get_role(state: &AppState, did: &str) -> Result<Option<String>, XrpcError> {
    sqlx::query_scalar::<_, String>(
        "SELECT role FROM memberships WHERE did = ? \
         ORDER BY CASE role WHEN 'admin' THEN 1 WHEN 'classRep' THEN 2 WHEN 'student' THEN 3 ELSE 4 END \
         LIMIT 1",
    )
    .bind(did)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Role lookup failed: {e}"),
    })
}

/// Require at least the given role level. admin > classRep > student.
pub async fn require_role(state: &AppState, did: &str, required: &str) -> Result<(), XrpcError> {
    let role = get_role(state, did).await?.ok_or_else(|| XrpcError {
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
pub async fn require_enrolled(
    state: &AppState,
    did: &str,
    course_uri: &str,
) -> Result<(), XrpcError> {
    let row =
        sqlx::query_as::<_, (i64,)>("SELECT 1 FROM enrollments WHERE did = ? AND course_uri = ?")
            .bind(did)
            .bind(course_uri)
            .fetch_optional(&state.db)
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

/// Check if a DID is the class rep for a course, or an admin.
pub async fn require_class_rep_or_admin(
    state: &AppState,
    did: &str,
    course_uri: &str,
) -> Result<(), XrpcError> {
    // Check if class rep
    let is_rep =
        sqlx::query_as::<_, (i64,)>("SELECT 1 FROM courses WHERE uri = ? AND class_rep_did = ?")
            .bind(course_uri)
            .bind(did)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Class rep check failed: {e}"),
            })?;

    if is_rep.is_some() {
        return Ok(());
    }

    // Fall back to admin check
    let role = get_role(state, did).await?;
    if role.as_deref() == Some("admin") {
        return Ok(());
    }

    Err(XrpcError {
        name: XrpcErrorName::Forbidden,
        message: "Only the class rep or an admin can perform this action".to_string(),
    })
}

/// Look up the course_uri for a given session_uri.
pub async fn get_course_for_session(
    state: &AppState,
    session_uri: &str,
) -> Result<String, XrpcError> {
    sqlx::query_scalar::<_, String>("SELECT course_uri FROM sessions WHERE uri = ?")
        .bind(session_uri)
        .fetch_optional(&state.db)
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
