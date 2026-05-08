//! Route wiring for Changala.
//!
//! Wires implemented XRPC handlers from `src/handlers/` and stubs for
//! endpoints not yet implemented. Uses `atrg_xrpc::xrpc_router()` as
//! the base router (provides JWT middleware + 501 fallback).

use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;

use crate::handlers;

pub fn api() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .merge(xrpc_routes())
}

/// All XRPC routes — implemented handlers + stubs.
fn xrpc_routes() -> Router<AppState> {
    atrg_xrpc::xrpc_router()
        // ── Identity ────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.verifyEmail",
            post(handlers::identity::verify_email),
        )
        .route(
            "/xrpc/app.changala.ring.getMemberships",
            get(handlers::identity::get_memberships),
        )
        .route(
            "/xrpc/app.changala.ring.getRole",
            get(handlers::identity::get_role),
        )
        // ── Course ──────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createCourse",
            post(handlers::course::create_course),
        )
        .route(
            "/xrpc/app.changala.ring.getCourse",
            get(handlers::course::get_course),
        )
        .route(
            "/xrpc/app.changala.ring.listCourses",
            get(handlers::course::list_courses),
        )
        .route(
            "/xrpc/app.changala.ring.enrollStudent",
            post(handlers::course::enroll_student),
        )
        .route(
            "/xrpc/app.changala.ring.getEnrollments",
            get(handlers::course::get_enrollments),
        )
        .route(
            "/xrpc/app.changala.ring.assignClassRep",
            post(handlers::course::assign_class_rep),
        )
        // ── Session ─────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createSession",
            post(handlers::session::create_session),
        )
        .route(
            "/xrpc/app.changala.ring.getSession",
            get(handlers::session::get_session),
        )
        .route(
            "/xrpc/app.changala.ring.listSessions",
            get(handlers::session::list_sessions),
        )
        .route(
            "/xrpc/app.changala.ring.openSession",
            post(handlers::session::open_session),
        )
        .route(
            "/xrpc/app.changala.ring.closeSession",
            post(handlers::session::close_session),
        )
        .route(
            "/xrpc/app.changala.ring.cancelSession",
            post(handlers::session::cancel_session),
        )
        .route(
            "/xrpc/app.changala.ring.rescheduleSession",
            post(handlers::session::reschedule_session),
        )
        // ── Notes & Keywords ────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.addKeyword",
            post(handlers::note::add_keyword),
        )
        .route(
            "/xrpc/app.changala.ring.createNote",
            post(handlers::note::create_note),
        )
        .route(
            "/xrpc/app.changala.ring.versionNote",
            post(handlers::note::version_note),
        )
        .route(
            "/xrpc/app.changala.ring.getNoteContent",
            get(handlers::note::get_note_content),
        )
        .route(
            "/xrpc/app.changala.ring.getNoteHistory",
            get(handlers::note::get_note_history),
        )
        // ── Collective Note ─────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.proposeEdit",
            post(handlers::note::propose_edit),
        )
        .route(
            "/xrpc/app.changala.ring.acceptEdit",
            post(handlers::note::accept_edit),
        )
        .route(
            "/xrpc/app.changala.ring.rejectEdit",
            post(handlers::note::reject_edit),
        )
        .route(
            "/xrpc/app.changala.ring.getCollectiveNote",
            get(handlers::note::get_collective_note),
        )
        .route(
            "/xrpc/app.changala.ring.listEditProposals",
            get(handlers::note::list_edit_proposals),
        )
        // ── Vote ────────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.registerVote",
            post(handlers::note::register_vote),
        )
        // ── Label ───────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.applyLabel",
            post(handlers::note::apply_label),
        )
        .route(
            "/xrpc/app.changala.ring.retractLabel",
            post(handlers::note::retract_label),
        )
        // ── Moderation ──────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.banDid",
            post(handlers::moderation::ban_did),
        )
        .route(
            "/xrpc/app.changala.ring.liftBan",
            post(handlers::moderation::lift_ban),
        )
        .route(
            "/xrpc/app.changala.ring.listBans",
            get(handlers::moderation::list_bans),
        )
        .route(
            "/xrpc/app.changala.ring.isBanned",
            get(handlers::moderation::is_banned),
        )
        // ── Archive (stubs) ─────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.initiateArchive",
            post(not_implemented),
        )
        .route("/xrpc/app.changala.ring.sealArchive", post(not_implemented))
        .route(
            "/xrpc/app.changala.ring.exportArchive",
            post(not_implemented),
        )
        .route(
            "/xrpc/app.changala.ring.uploadToInternetArchive",
            post(not_implemented),
        )
        .route("/xrpc/app.changala.ring.getArchive", get(not_implemented))
        // ── Brain (stubs) ───────────────────────────────────
        .route("/xrpc/app.changala.ring.createNode", post(not_implemented))
        .route("/xrpc/app.changala.ring.versionNode", post(not_implemented))
        .route(
            "/xrpc/app.changala.ring.getNodeContent",
            get(not_implemented),
        )
        .route("/xrpc/app.changala.ring.createLink", post(not_implemented))
        .route("/xrpc/app.changala.ring.deleteLink", post(not_implemented))
        // ── Global View: Feeds (stubs) ──────────────────────
        .route(
            "/xrpc/app.changala.globalview.getNotes",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getCourseFeed",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getSocialFeed",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getBrainFeed",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getTrendingKeywords",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getTrendingBrainTags",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getFollowedEnrollments",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getGlobalArchiveFeed",
            get(not_implemented),
        )
        // ── Global View: Search (stubs) ─────────────────────
        .route(
            "/xrpc/app.changala.globalview.searchNotes",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.searchCourses",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.searchArchive",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.searchBrainNodes",
            get(not_implemented),
        )
        // ── Global View: Histogram (stub) ───────────────────
        .route(
            "/xrpc/app.changala.globalview.getKeywordHistogram",
            get(not_implemented),
        )
        // ── Global View: Graph (stubs) ──────────────────────
        .route(
            "/xrpc/app.changala.globalview.getNodeGraph",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getBacklinks",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.getNeighbours",
            get(not_implemented),
        )
        // ── Global View: Notifications (stubs) ──────────────
        .route(
            "/xrpc/app.changala.globalview.getNotifications",
            get(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.markNotificationRead",
            post(not_implemented),
        )
        .route(
            "/xrpc/app.changala.globalview.markAllRead",
            post(not_implemented),
        )
}

async fn index() -> Json<serde_json::Value> {
    Json(json!({
        "name": "changala",
        "description": "Federated social learning platform on AT Protocol",
        "status": "ok"
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "healthy": true }))
}

/// Placeholder for unimplemented XRPC endpoints — returns 501.
async fn not_implemented() -> Result<Json<serde_json::Value>, XrpcError> {
    Err(XrpcError {
        name: XrpcErrorName::MethodNotImplemented,
        message: "This endpoint is not yet implemented".to_string(),
    })
}
