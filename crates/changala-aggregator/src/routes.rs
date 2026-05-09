//! Route wiring for the Changala Aggregator (Global View).

use atrg_core::AppState;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;

use crate::handlers;

pub fn api() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .merge(globalview_routes())
}

fn globalview_routes() -> Router<AppState> {
    atrg_xrpc::xrpc_router()
        // ── Feeds ───────────────────────────────────────────
        .route(
            "/xrpc/app.changala.globalview.getNotes",
            get(handlers::feed::get_notes),
        )
        .route(
            "/xrpc/app.changala.globalview.getCourseFeed",
            get(handlers::feed::get_course_feed),
        )
        .route(
            "/xrpc/app.changala.globalview.getSocialFeed",
            get(handlers::feed::get_social_feed),
        )
        .route(
            "/xrpc/app.changala.globalview.getBrainFeed",
            get(handlers::feed::get_brain_feed),
        )
        .route(
            "/xrpc/app.changala.globalview.getTrendingKeywords",
            get(handlers::feed::get_trending_keywords),
        )
        .route(
            "/xrpc/app.changala.globalview.getTrendingBrainTags",
            get(handlers::feed::get_trending_brain_tags),
        )
        .route(
            "/xrpc/app.changala.globalview.getFollowedEnrollments",
            get(handlers::feed::get_followed_enrollments),
        )
        .route(
            "/xrpc/app.changala.globalview.getGlobalArchiveFeed",
            get(handlers::feed::get_global_archive_feed),
        )
        // ── Search ──────────────────────────────────────────
        .route(
            "/xrpc/app.changala.globalview.searchNotes",
            get(handlers::search::search_notes),
        )
        .route(
            "/xrpc/app.changala.globalview.searchCourses",
            get(handlers::search::search_courses),
        )
        .route(
            "/xrpc/app.changala.globalview.searchArchive",
            get(handlers::search::search_archive),
        )
        .route(
            "/xrpc/app.changala.globalview.searchBrainNodes",
            get(handlers::search::search_brain_nodes),
        )
        // ── Histogram ───────────────────────────────────────
        .route(
            "/xrpc/app.changala.globalview.getKeywordHistogram",
            get(handlers::feed::get_keyword_histogram),
        )
        // ── Graph ───────────────────────────────────────────
        .route(
            "/xrpc/app.changala.globalview.getNodeGraph",
            get(handlers::graph::get_node_graph),
        )
        .route(
            "/xrpc/app.changala.globalview.getBacklinks",
            get(handlers::graph::get_backlinks),
        )
        .route(
            "/xrpc/app.changala.globalview.getNeighbours",
            get(handlers::graph::get_neighbours),
        )
        // ── Notifications ───────────────────────────────────
        .route(
            "/xrpc/app.changala.globalview.getNotifications",
            get(handlers::notification::get_notifications),
        )
        .route(
            "/xrpc/app.changala.globalview.markNotificationRead",
            post(handlers::notification::mark_notification_read),
        )
        .route(
            "/xrpc/app.changala.globalview.markAllRead",
            post(handlers::notification::mark_all_read),
        )
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "name": "changala-aggregator",
        "description": "Changala Global View — read-only aggregator",
        "status": "ok"
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "healthy": true }))
}
