//! Notification handlers — read and mark notifications.

use axum::extract::Query;
use axum::Json;

use atrg_auth::RequireAuth;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use changala_shared::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Clamp a user-supplied limit.
fn clamp_limit(limit: Option<i64>, default: i64) -> i64 {
    limit.unwrap_or(default).clamp(1, 100)
}

// ═══════════════════════════════════════════════════════════════════════════
// getNotifications — list notifications with optional unread filter
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getNotifications
///
/// Returns notifications for the authenticated user.
/// If `unread_only=true`, filters to unread notifications only.
pub async fn get_notifications(
    RequireAuth(session): RequireAuth,
    Query(params): Query<AppChangalaGlobalviewGetNotificationsParams>,
) -> Result<Json<AppChangalaGlobalviewGetNotificationsOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;

    let mut sql = String::from(
        "SELECT id, recipient_did, reason, subject_uri, read, created_at \
         FROM notifications WHERE recipient_did = $1",
    );
    let mut binds: Vec<String> = vec![session.did.clone()];

    if params.unread_only.unwrap_or(false) {
        sql.push_str(" AND read = FALSE");
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(&format!(" AND created_at < ${}", binds.len() + 1));
        binds.push(cursor.clone());
    }

    sql.push_str(&format!(
        " ORDER BY created_at DESC LIMIT ${}",
        binds.len() + 1
    ));

    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(fetch_limit);

    use sqlx::Row;
    let rows: Vec<sqlx::postgres::PgRow> = q.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch notifications: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows[..]
    };

    let notifications: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let read_val: bool = r.get("read");
            json!({
                "id": r.get::<String, _>("id"),
                "recipientDid": r.get::<String, _>("recipient_did"),
                "reason": r.get::<String, _>("reason"),
                "subjectUri": r.get::<String, _>("subject_uri"),
                "read": read_val,
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    // Unread count for the authenticated user.
    let unread_count: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE read = FALSE AND recipient_did = $1",
    )
    .bind(&session.did)
    .fetch_one(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to count unread notifications: {e}"),
    })?;

    let cursor = if has_more {
        rows.last().map(|r| r.get::<String, _>("created_at"))
    } else {
        None
    };

    Ok(Json(AppChangalaGlobalviewGetNotificationsOutput {
        notifications,
        unread_count,
        cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// markNotificationRead — mark a single notification as read
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.globalview.markNotificationRead
pub async fn mark_notification_read(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaGlobalviewMarkNotificationReadInput>,
) -> Result<Json<AppChangalaGlobalviewMarkNotificationReadOutput>, XrpcError> {
    let app = crate::state::get();

    let result =
        sqlx::query("UPDATE notifications SET read = TRUE WHERE id = $1 AND recipient_did = $2")
            .bind(&input.notification_id)
            .bind(&session.did)
            .execute(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to mark notification as read: {e}"),
            })?;

    if result.rows_affected() == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("Notification not found: {}", input.notification_id),
        });
    }

    Ok(Json(AppChangalaGlobalviewMarkNotificationReadOutput {
        notification_id: input.notification_id,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// markAllRead — mark all unread notifications as read
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.globalview.markAllRead
pub async fn mark_all_read(
    RequireAuth(session): RequireAuth,
    Json(_input): Json<AppChangalaGlobalviewMarkAllReadInput>,
) -> Result<Json<AppChangalaGlobalviewMarkAllReadOutput>, XrpcError> {
    let app = crate::state::get();

    let result = sqlx::query(
        "UPDATE notifications SET read = TRUE WHERE read = FALSE AND recipient_did = $1",
    )
    .bind(&session.did)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to mark all notifications as read: {e}"),
    })?;

    Ok(Json(AppChangalaGlobalviewMarkAllReadOutput {
        marked_count: result.rows_affected() as i64,
    }))
}
