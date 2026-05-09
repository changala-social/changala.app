//! Archive service handlers — end-of-semester archival pipeline.

use atrg_auth::RequireAuth;
use axum::extract::Query;
use axum::Json;

use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use changala_shared::types::*;

use super::auth;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Placeholder Ring DID used until real Ring identity is provisioned.
const RING_DID: &str = "did:web:ring.changala.local";

// ═══════════════════════════════════════════════════════════════════════════
// Archive Lifecycle
// ═══════════════════════════════════════════════════════════════════════════

/// POST /xrpc/app.changala.ring.initiateArchive
///
/// Initiates the archival process for a course + semester pair. Validates
/// that the course exists, counts sessions and notes, and creates an archive
/// record with status `initiated`.
pub async fn initiate_archive(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingInitiateArchiveInput>,
) -> Result<Json<AppChangalaRingInitiateArchiveOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Validate the course exists.
    let course_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM courses WHERE uri = $1")
        .bind(&input.course_uri)
        .fetch_one(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Course lookup failed: {e}"),
        })?;

    if course_exists == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("Course not found: {}", input.course_uri),
        });
    }

    // Count sessions for this course.
    let session_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions WHERE course_uri = $1")
            .bind(&input.course_uri)
            .fetch_one(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Session count failed: {e}"),
            })?;

    // Count notes for this course (notes joined through sessions).
    let note_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notes n \
         JOIN sessions s ON n.session_uri = s.uri \
         WHERE s.course_uri = $1",
    )
    .bind(&input.course_uri)
    .fetch_one(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Note count failed: {e}"),
    })?;

    // Insert the archive record.
    sqlx::query(
        "INSERT INTO archives \
         (course_uri, semester, session_count, note_count, status, initiated_at) \
         VALUES ($1, $2, $3, $4, 'initiated', $5)",
    )
    .bind(&input.course_uri)
    .bind(&input.semester)
    .bind(session_count)
    .bind(note_count)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate key") {
            XrpcError {
                name: XrpcErrorName::InvalidRequest,
                message: format!(
                    "Archive already initiated for course {} semester {}",
                    input.course_uri, input.semester
                ),
            }
        } else {
            XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to initiate archive: {e}"),
            }
        }
    })?;

    Ok(Json(AppChangalaRingInitiateArchiveOutput {
        course_uri: input.course_uri,
        semester: input.semester,
        initiated_at: now,
        session_count,
        note_count,
    }))
}

/// POST /xrpc/app.changala.ring.sealArchive
///
/// Seals a previously initiated archive, making it immutable. Generates a
/// bundle CID and assigns an AT URI to the sealed archive record.
pub async fn seal_archive(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingSealArchiveInput>,
) -> Result<Json<AppChangalaRingSealArchiveOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Validate the archive exists with status 'initiated'.
    let archive = sqlx::query_as::<_, (i64, i64)>(
        "SELECT session_count::BIGINT, note_count::BIGINT FROM archives \
         WHERE course_uri = $1 AND semester = $2 AND status = 'initiated'",
    )
    .bind(&input.course_uri)
    .bind(&input.semester)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Archive lookup failed: {e}"),
    })?;

    let (session_count, note_count) = archive.ok_or_else(|| XrpcError {
        name: XrpcErrorName::InvalidRequest,
        message: format!(
            "No initiated archive found for course {} semester {} \
             (it may already be sealed or does not exist)",
            input.course_uri, input.semester
        ),
    })?;

    let content = format!("{}:{}", input.course_uri, input.semester);
    let bundle_cid = app
        .blobs
        .put(content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store archive bundle: {e}"),
        })?;
    let rkey = atrg_repo::Tid::now().to_string();
    let archive_uri = format!("at://{}/app.changala.archive/{}", RING_DID, rkey);

    // Seal the archive.
    sqlx::query(
        "UPDATE archives \
         SET status = 'sealed', sealed_at = $1, sealed_by = $2, \
             ring_did = $3, cid = $4, archive_uri = $5 \
         WHERE course_uri = $6 AND semester = $7 AND status = 'initiated'",
    )
    .bind(&now)
    .bind(&session.did)
    .bind(RING_DID)
    .bind(&bundle_cid)
    .bind(&archive_uri)
    .bind(&input.course_uri)
    .bind(&input.semester)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to seal archive: {e}"),
    })?;

    let bundle_ref = json!({
        "ringDid": RING_DID,
        "cid": bundle_cid,
    });

    Ok(Json(AppChangalaRingSealArchiveOutput {
        archive_uri,
        bundle_ref,
        sealed_at: now,
        session_count,
        note_count,
    }))
}

/// POST /xrpc/app.changala.ring.exportArchive
///
/// Exports a sealed archive in the requested format. Generates a real
/// bundle blob via the S3 blob store and returns an export reference.
pub async fn export_archive(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingExportArchiveInput>,
) -> Result<Json<AppChangalaRingExportArchiveOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Validate the archive exists with status 'sealed'.
    let sealed_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM archives \
         WHERE course_uri = $1 AND semester = $2 AND status = 'sealed'",
    )
    .bind(&input.course_uri)
    .bind(&input.semester)
    .fetch_one(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Archive lookup failed: {e}"),
    })?;

    if sealed_exists == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "No sealed archive found for course {} semester {} \
                 (archive must be sealed before export)",
                input.course_uri, input.semester
            ),
        });
    }

    let content = format!(
        "export:{}:{}:{}",
        input.course_uri, input.semester, input.format
    );
    let export_cid = app
        .blobs
        .put(content.as_bytes())
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to store archive bundle: {e}"),
        })?;

    let export_ref = json!({
        "ringDid": RING_DID,
        "cid": export_cid,
        "format": input.format,
    });

    Ok(Json(AppChangalaRingExportArchiveOutput {
        export_ref,
        format: input.format,
        size_bytes: 0, // TODO: compute actual bundle size
        created_at: now,
    }))
}

/// POST /xrpc/app.changala.ring.uploadToInternetArchive
///
/// Uploads a sealed archive to the Internet Archive. In MVP, actual upload
/// is not performed — this generates a fake IA identifier and URL. Real
/// upload requires an S3-compatible API key for `archive.org`.
pub async fn upload_to_internet_archive(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingUploadToInternetArchiveInput>,
) -> Result<Json<AppChangalaRingUploadToInternetArchiveOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let now = chrono::Utc::now().to_rfc3339();

    // Validate the archive is sealed.
    let sealed_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM archives \
         WHERE course_uri = $1 AND semester = $2 AND status = 'sealed'",
    )
    .bind(&input.course_uri)
    .bind(&input.semester)
    .fetch_one(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Archive lookup failed: {e}"),
    })?;

    if sealed_exists == 0 {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "No sealed archive found for course {} semester {} \
                 (archive must be sealed before IA upload)",
                input.course_uri, input.semester
            ),
        });
    }

    // Generate a fake IA identifier from the course + semester.
    let ia_identifier = format!(
        "changala-{}-{}",
        input.course_uri.rsplit('/').next().unwrap_or("unknown"),
        input.semester.replace(' ', "-").to_lowercase()
    );
    let ia_url = format!("https://archive.org/details/{}", ia_identifier);

    // Update the archive record with the IA URL.
    sqlx::query(
        "UPDATE archives SET internet_archive_url = $1 \
         WHERE course_uri = $2 AND semester = $3 AND status = 'sealed'",
    )
    .bind(&ia_url)
    .bind(&input.course_uri)
    .bind(&input.semester)
    .execute(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to update archive with IA URL: {e}"),
    })?;

    Ok(Json(AppChangalaRingUploadToInternetArchiveOutput {
        ia_identifier,
        ia_url,
        uploaded_at: now,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Archive Query
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.ring.getArchive
///
/// Retrieves an archive record by course URI and semester. Returns the
/// archive regardless of status so callers can check progress.
pub async fn get_archive(
    Query(params): Query<AppChangalaRingGetArchiveParams>,
) -> Result<Json<AppChangalaRingGetArchiveOutput>, XrpcError> {
    let app = crate::state::get();

    let row = sqlx::query_as::<
        _,
        (
            Option<String>, // archive_uri
            String,         // course_uri
            String,         // semester
            Option<String>, // sealed_by
            Option<String>, // ring_did
            Option<String>, // cid
            Option<String>, // internet_archive_url
            i64,            // session_count
            i64,            // note_count
            String,         // status
            String,         // initiated_at
            Option<String>, // sealed_at
        ),
    >(
        "SELECT archive_uri, course_uri, semester, sealed_by, ring_did, cid, \
                internet_archive_url, session_count::BIGINT, note_count::BIGINT, status, \
                initiated_at, sealed_at \
         FROM archives \
         WHERE course_uri = $1 AND semester = $2",
    )
    .bind(&params.course_uri)
    .bind(&params.semester)
    .fetch_optional(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Archive lookup failed: {e}"),
    })?;

    let (
        archive_uri,
        course_uri,
        semester,
        sealed_by,
        ring_did,
        cid,
        internet_archive_url,
        session_count,
        note_count,
        _status,
        initiated_at,
        sealed_at,
    ) = row.ok_or_else(|| XrpcError {
        name: XrpcErrorName::NotFound,
        message: format!(
            "Archive not found for course {} semester {}",
            params.course_uri, params.semester
        ),
    })?;

    let bundle_ref = match (ring_did.as_deref(), cid.as_deref()) {
        (Some(r), Some(c)) => json!({ "ringDid": r, "cid": c }),
        _ => json!(null),
    };

    Ok(Json(AppChangalaRingGetArchiveOutput {
        archive_uri: archive_uri.unwrap_or_default(),
        course_uri,
        semester,
        sealed_by: sealed_by.unwrap_or_default(),
        bundle_ref,
        internet_archive_url,
        session_count,
        note_count,
        sealed_at: sealed_at.unwrap_or(initiated_at),
    }))
}
