//! Route wiring for Changala.
//!
//! All 62 XRPC endpoints wired to real handler implementations.

use atrg_core::AppState;
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

/// All XRPC routes — every endpoint wired to a real handler.
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
        // ── Archive ─────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.initiateArchive",
            post(handlers::archive::initiate_archive),
        )
        .route(
            "/xrpc/app.changala.ring.sealArchive",
            post(handlers::archive::seal_archive),
        )
        .route(
            "/xrpc/app.changala.ring.exportArchive",
            post(handlers::archive::export_archive),
        )
        .route(
            "/xrpc/app.changala.ring.uploadToInternetArchive",
            post(handlers::archive::upload_to_internet_archive),
        )
        .route(
            "/xrpc/app.changala.ring.getArchive",
            get(handlers::archive::get_archive),
        )
        // ── Brain ───────────────────────────────────────────
        .route(
            "/xrpc/app.changala.ring.createNode",
            post(handlers::brain::create_node),
        )
        .route(
            "/xrpc/app.changala.ring.versionNode",
            post(handlers::brain::version_node),
        )
        .route(
            "/xrpc/app.changala.ring.getNodeContent",
            get(handlers::brain::get_node_content),
        )
        .route(
            "/xrpc/app.changala.ring.createLink",
            post(handlers::brain::create_link),
        )
        .route(
            "/xrpc/app.changala.ring.deleteLink",
            post(handlers::brain::delete_link),
        )
        // ── Global View: Feeds ──────────────────────────────
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
        // ── Global View: Search ─────────────────────────────
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
        // ── Global View: Histogram ──────────────────────────
        .route(
            "/xrpc/app.changala.globalview.getKeywordHistogram",
            get(handlers::feed::get_keyword_histogram),
        )
        // ── Global View: Graph ──────────────────────────────
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
        // ── Global View: Notifications ──────────────────────
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
        "name": "changala",
        "description": "Federated social learning platform on AT Protocol",
        "status": "ok"
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "healthy": true }))
}
