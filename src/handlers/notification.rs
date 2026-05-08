//! Notification handlers — read and mark notifications.

use axum::extract::Query;
use axum::{extract::State, Json};

use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use crate::generated::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Clamp a user-supplied limit.
fn clamp_limit(limit: Option<i64>, default: i64) -> i64 {
    limit.unwrap_or(default).min(100).max(1)
}

// ═══════════════════════════════════════════════════════════════════════════
// getNotifications — list notifications with optional unread filter
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getNotifications
///
/// MVP: returns all notifications (no recipient filtering without auth).
/// If `unread_only=true`, filters to unread notifications only.
pub async fn get_notifications(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetNotificationsParams>,
) -> Result<Json<AppChangalaGlobalviewGetNotificationsOutput>, XrpcError> {
    let limit = clamp_limit(params.limit, 50);
    let fetch_limit = limit + 1;

    let mut sql = String::from(
        "SELECT id, recipient_did, reason, subject_uri, read, created_at \
         FROM notifications WHERE 1=1",
    );
    let mut binds: Vec<String> = Vec::new();

    if params.unread_only.unwrap_or(false) {
        sql.push_str(" AND read = 0");
    }

    if let Some(ref cursor) = params.cursor {
        sql.push_str(" AND created_at < ?");
        binds.push(cursor.clone());
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ?");

    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    q = q.bind(fetch_limit);

    use sqlx::Row;
    let rows: Vec<sqlx::sqlite::SqliteRow> =
        q.fetch_all(&state.db).await.map_err(|e| XrpcError {
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
            let read_int: i64 = r.get("read");
            json!({
                "id": r.get::<String, _>("id"),
                "recipientDid": r.get::<String, _>("recipient_did"),
                "reason": r.get::<String, _>("reason"),
                "subjectUri": r.get::<String, _>("subject_uri"),
                "read": read_int != 0,
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    // Global unread count.
    let unread_count: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications WHERE read = 0")
            .fetch_one(&state.db)
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
    State(state): State<AppState>,
    Json(input): Json<AppChangalaGlobalviewMarkNotificationReadInput>,
) -> Result<Json<AppChangalaGlobalviewMarkNotificationReadOutput>, XrpcError> {
    let result = sqlx::query("UPDATE notifications SET read = 1 WHERE id = ?")
        .bind(&input.notification_id)
        .execute(&state.db)
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
    State(state): State<AppState>,
    Json(_input): Json<AppChangalaGlobalviewMarkAllReadInput>,
) -> Result<Json<AppChangalaGlobalviewMarkAllReadOutput>, XrpcError> {
    let result = sqlx::query("UPDATE notifications SET read = 1 WHERE read = 0")
        .execute(&state.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to mark all notifications as read: {e}"),
        })?;

    Ok(Json(AppChangalaGlobalviewMarkAllReadOutput {
        marked_count: result.rows_affected() as i64,
    }))
}
