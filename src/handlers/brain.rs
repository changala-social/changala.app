//! Brain service handlers — node CRUD and linking.

use axum::extract::Query;
use axum::Json;

use atrg_auth::RequireAuth;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use super::auth;
use crate::generated::types::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Placeholder Ring DID used until real Ring identity is provisioned.
const RING_DID: &str = "did:web:ring.changala.local";

// ═══════════════════════════════════════════════════════════════════════════
// Nodes
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.createNode
///
/// Creates a new brain node (version 1). Stores content in the S3 blob store
/// and returns a `ring_ref` + `node_template` for the PDS record.
pub async fn create_node(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCreateNodeInput>,
) -> Result<Json<AppChangalaRingCreateNodeOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = input
        .created_at
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let cid = app
        .blobs
        .put(input.content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store content: {e}"),
        })?;
    let rkey = atrg_repo::Tid::now().to_string();
    let node_uri = format!("at://{}/app.changala.brain.node/{}", session.did, rkey);

    let tags_json = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default());

    sqlx::query(
        "INSERT INTO brain_nodes \
         (uri, author_did, title, format, ring_did, cid, tags, academic_ref, \
          version, parent_node_uri, summary, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, NULL, $9, $10)",
    )
    .bind(&node_uri)
    .bind(&session.did)
    .bind(&input.title)
    .bind(&input.format)
    .bind(RING_DID)
    .bind(&cid)
    .bind(&tags_json)
    .bind(&input.academic_ref)
    .bind(&input.summary)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to create brain node: {e}"),
    })?;

    let ring_ref = json!({
        "ringDid": RING_DID,
        "cid": cid,
    });

    let node_template = json!({
        "uri": node_uri,
        "title": input.title,
        "format": input.format,
        "ringRef": ring_ref,
        "tags": input.tags.unwrap_or_default(),
        "academicRef": input.academic_ref,
        "version": 1,
        "summary": input.summary,
        "createdAt": now,
    });

    Ok(Json(AppChangalaRingCreateNodeOutput {
        ring_ref,
        node_template,
    }))
}

/// POST /xrpc/app.changala.ring.versionNode
///
/// Creates a new version of an existing brain node. Looks up the parent node
/// to inherit author identity and increments the version counter.
pub async fn version_node(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingVersionNodeInput>,
) -> Result<Json<AppChangalaRingVersionNodeOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Fetch the parent node to inherit author_did and get current version.
    let parent = sqlx::query_as::<_, (String, i64)>(
        "SELECT author_did, version FROM brain_nodes WHERE uri = $1",
    )
    .bind(&input.parent_node_uri)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Parent node lookup failed: {e}"),
    })?;

    let (author_did, parent_version) = parent.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Parent brain node not found: {}", input.parent_node_uri),
    })?;

    let new_version = parent_version + 1;
    let cid = app
        .blobs
        .put(input.content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store content: {e}"),
        })?;
    let rkey = atrg_repo::Tid::now().to_string();
    let node_uri = format!("at://{}/app.changala.brain.node/{}", author_did, rkey);

    let tags_json = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default());

    sqlx::query(
        "INSERT INTO brain_nodes \
         (uri, author_did, title, format, ring_did, cid, tags, academic_ref, \
          version, parent_node_uri, summary, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(&node_uri)
    .bind(&author_did)
    .bind(&input.title)
    .bind(&input.format)
    .bind(RING_DID)
    .bind(&cid)
    .bind(&tags_json)
    .bind(&input.academic_ref)
    .bind(new_version)
    .bind(&input.parent_node_uri)
    .bind(&input.summary)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to version brain node: {e}"),
    })?;

    let ring_ref = json!({
        "ringDid": RING_DID,
        "cid": cid,
    });

    let node_template = json!({
        "uri": node_uri,
        "title": input.title,
        "format": input.format,
        "ringRef": ring_ref,
        "tags": input.tags.unwrap_or_default(),
        "academicRef": input.academic_ref,
        "version": new_version,
        "parentNode": input.parent_node_uri,
        "summary": input.summary,
        "createdAt": now,
    });

    Ok(Json(AppChangalaRingVersionNodeOutput {
        ring_ref,
        node_template,
    }))
}

/// GET /xrpc/app.changala.ring.getNodeContent
///
/// Retrieves brain node content from the Ring's S3 blob store by CID.
pub async fn get_node_content(
    Query(params): Query<AppChangalaRingGetNodeContentParams>,
) -> Result<Json<AppChangalaRingGetNodeContentOutput>, XrpcError> {
    let app = crate::state::get();
    let content_bytes = app.blobs.get(&params.cid).await.map_err(|e| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!("Blob not found: {e}"),
    })?;
    let content = String::from_utf8_lossy(&content_bytes).to_string();
    let format =
        sqlx::query_scalar::<_, String>("SELECT format FROM brain_nodes WHERE cid = $1 LIMIT 1")
            .bind(&params.cid)
            .fetch_optional(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("DB error: {e}"),
            })?
            .unwrap_or_else(|| "markdown".to_string());
    Ok(Json(AppChangalaRingGetNodeContentOutput {
        format,
        content,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Links
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.createLink
///
/// Creates a directed link between two brain nodes (or from a brain node to
/// any AT URI / external target). The Global View maintains a bidirectional
/// graph index from these records.
pub async fn create_link(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCreateLinkInput>,
) -> Result<Json<AppChangalaRingCreateLinkOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let rkey = atrg_repo::Tid::now().to_string();
    let link_uri = format!("at://{}/app.changala.brain.link/{}", session.did, rkey);

    sqlx::query(
        "INSERT INTO brain_links (link_uri, from_uri, to_uri, label, created_by, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&link_uri)
    .bind(&input.from_uri)
    .bind(&input.to_uri)
    .bind(&input.label)
    .bind(&session.did)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate key") {
            XrpcError {
                name: XrpcErrorName::InvalidRequest,
                message: "Link already exists between these nodes".to_string(),
            }
        } else {
            XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to create brain link: {e}"),
            }
        }
    })?;

    Ok(Json(AppChangalaRingCreateLinkOutput {
        link_uri,
        created_at: now,
    }))
}

/// POST /xrpc/app.changala.ring.deleteLink
///
/// Deletes a brain link by its AT URI. The Global View will pick up the
/// deletion via the firehose and update its graph index accordingly.
pub async fn delete_link(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingDeleteLinkInput>,
) -> Result<Json<AppChangalaRingDeleteLinkOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let result = sqlx::query("DELETE FROM brain_links WHERE link_uri = $1")
        .bind(&input.link_uri)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to delete brain link: {e}"),
        })?;

    if result.rows_affected() == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("Brain link not found: {}", input.link_uri),
        });
    }

    Ok(Json(AppChangalaRingDeleteLinkOutput {
        link_uri: input.link_uri,
    }))
}
