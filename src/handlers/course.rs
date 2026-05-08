//! Course service handlers — CRUD and enrollment.

use atrg_repo::Tid;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::Query;
use axum::Json;
use chrono::Utc;

use atrg_auth::RequireAuth;

use crate::generated::types::*;

use super::auth;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a full course view from a DB row + an enrollment count.
///
/// Column order must match the SELECT used by callers:
///   uri, title, code, department, semester, visibility, created_by,
///   class_rep_did, description, created_at
fn course_view(
    uri: String,
    title: String,
    code: String,
    department: String,
    semester: String,
    visibility: String,
    created_by: String,
    class_rep_did: Option<String>,
    description: Option<String>,
    created_at: String,
    enrolled_count: i64,
) -> AppChangalaRingCreateCourseOutput {
    AppChangalaRingCreateCourseOutput {
        uri,
        title,
        code,
        department,
        semester,
        visibility,
        created_by,
        class_rep_did,
        description,
        created_at,
        enrolled_count,
    }
}

/// Fetch a course from the DB and its enrollment count, returning a full
/// course view output. Returns `None` when no row matches.
async fn fetch_course_view(
    db: &sqlx::PgPool,
    uri: &str,
) -> Result<Option<AppChangalaRingCreateCourseOutput>, XrpcError> {
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
        ),
    >(
        "SELECT uri, title, code, department, semester, visibility, \
                created_by, class_rep_did, description, created_at \
         FROM courses WHERE uri = $1",
    )
    .bind(uri)
    .fetch_optional(db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("database error: {e}"),
    })?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let enrolled_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM enrollments WHERE course_uri = $1")
            .bind(uri)
            .fetch_one(db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("database error: {e}"),
            })?;

    Ok(Some(course_view(
        row.0,
        row.1,
        row.2,
        row.3,
        row.4,
        row.5,
        row.6,
        row.7,
        row.8,
        row.9,
        enrolled_count.0,
    )))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `app.changala.ring.createCourse` — POST procedure.
///
/// Generates a TID rkey, inserts a new course, and returns the full course
/// view with `enrolled_count: 0`.
pub async fn create_course(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingCreateCourseInput>,
) -> Result<Json<AppChangalaRingCreateCourseOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    let rkey = Tid::now().to_string();
    let uri = format!("at://changala.ring/app.changala.course/{rkey}");
    let now = Utc::now().to_rfc3339();
    let created_by = session.did.clone();

    sqlx::query(
        "INSERT INTO courses (uri, rkey, title, code, department, semester, \
                              visibility, created_by, class_rep_did, description, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, $9, $10)",
    )
    .bind(&uri)
    .bind(&rkey)
    .bind(&input.title)
    .bind(&input.code)
    .bind(&input.department)
    .bind(&input.semester)
    .bind(&input.visibility)
    .bind(&created_by)
    .bind(&input.description)
    .bind(&now)
    .execute(&app.db)
    .await
    .map_err(|e| {
        // UNIQUE constraint violation → duplicate course code+semester
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.message().contains("UNIQUE") {
                return XrpcError {
                    name: XrpcErrorName::InvalidRequest,
                    message: format!(
                        "course {code} already exists for semester {sem}",
                        code = input.code,
                        sem = input.semester,
                    ),
                };
            }
        }
        XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        }
    })?;

    Ok(Json(course_view(
        uri,
        input.title,
        input.code,
        input.department,
        input.semester,
        input.visibility,
        created_by,
        None,
        input.description,
        now,
        0,
    )))
}

/// `app.changala.ring.getCourse` — GET query.
///
/// Fetches a single course by AT URI. Returns `NotFound` if absent.
pub async fn get_course(
    Query(params): Query<AppChangalaRingGetCourseParams>,
) -> Result<Json<AppChangalaRingGetCourseOutput>, XrpcError> {
    let app = crate::state::get();
    let view = fetch_course_view(&app.db, &params.uri)
        .await?
        .ok_or_else(|| XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("course not found: {}", params.uri),
        })?;

    // GetCourseOutput has the exact same shape as CreateCourseOutput.
    Ok(Json(AppChangalaRingGetCourseOutput {
        uri: view.uri,
        title: view.title,
        code: view.code,
        department: view.department,
        semester: view.semester,
        visibility: view.visibility,
        created_by: view.created_by,
        class_rep_did: view.class_rep_did,
        description: view.description,
        created_at: view.created_at,
        enrolled_count: view.enrolled_count,
    }))
}

/// `app.changala.ring.listCourses` — GET query.
///
/// Lists courses with optional filters (semester, department) and cursor-based
/// pagination. Default limit 50, max 100. Cursor is the `created_at` of the
/// last item in the previous page.
pub async fn list_courses(
    Query(params): Query<AppChangalaRingListCoursesParams>,
) -> Result<Json<AppChangalaRingListCoursesOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = params.limit.unwrap_or(50).min(100).max(1);

    // Build dynamic query
    let mut sql = String::from(
        "SELECT uri, title, code, department, semester, visibility, \
                created_by, class_rep_did, description, created_at \
         FROM courses WHERE 1=1",
    );
    let mut binds: Vec<String> = Vec::new();
    let mut param_idx = 0u32;

    if let Some(ref sem) = params.semester {
        param_idx += 1;
        sql.push_str(&format!(" AND semester = ${param_idx}"));
        binds.push(sem.clone());
    }
    if let Some(ref dept) = params.department {
        param_idx += 1;
        sql.push_str(&format!(" AND department = ${param_idx}"));
        binds.push(dept.clone());
    }
    if let Some(ref cursor) = params.cursor {
        param_idx += 1;
        sql.push_str(&format!(" AND created_at < ${param_idx}"));
        binds.push(cursor.clone());
    }

    param_idx += 1;
    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${param_idx}"));

    // We need to fetch limit+1 to know if there's a next page
    let fetch_limit = limit + 1;

    // Build the query dynamically — sqlx doesn't support dynamic bind lists
    // with the compile-time checked API, so we use `query_as` with runtime binds.
    let mut query = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
        ),
    >(&sql);

    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(fetch_limit);

    let rows = query.fetch_all(&app.db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("database error: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows
    };

    let mut courses = Vec::with_capacity(rows.len());
    for row in rows {
        // Count enrollments per course
        let enrolled: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM enrollments WHERE course_uri = $1")
                .bind(&row.0)
                .fetch_one(&app.db)
                .await
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("database error: {e}"),
                })?;

        let view = course_view(
            row.0.clone(),
            row.1.clone(),
            row.2.clone(),
            row.3.clone(),
            row.4.clone(),
            row.5.clone(),
            row.6.clone(),
            row.7.clone(),
            row.8.clone(),
            row.9.clone(),
            enrolled.0,
        );
        courses.push(serde_json::to_value(view).unwrap());
    }

    let cursor = if has_more {
        rows.last().map(|r| r.9.clone())
    } else {
        None
    };

    Ok(Json(AppChangalaRingListCoursesOutput { courses, cursor }))
}

/// `app.changala.ring.enrollStudent` — POST procedure.
///
/// Enrolls a student in a course. Uses `target_did` if provided, otherwise
/// falls back to the authenticated user's DID (placeholder for now).
/// Returns 400 (InvalidRequest) on duplicate enrollment.
pub async fn enroll_student(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingEnrollStudentInput>,
) -> Result<Json<AppChangalaRingEnrollStudentOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;

    let did = input.target_did.unwrap_or_else(|| session.did.clone());
    let now = Utc::now().to_rfc3339();

    // Verify the course exists
    let course_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM courses WHERE uri = $1")
        .bind(&input.course_uri)
        .fetch_optional(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    if course_exists.is_none() {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("course not found: {}", input.course_uri),
        });
    }

    sqlx::query("INSERT INTO enrollments (course_uri, did, enrolled_at) VALUES ($1, $2, $3)")
        .bind(&input.course_uri)
        .bind(&did)
        .bind(&now)
        .execute(&app.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE") {
                    return XrpcError {
                        name: XrpcErrorName::InvalidRequest,
                        message: format!("{did} is already enrolled in {}", input.course_uri),
                    };
                }
            }
            XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("database error: {e}"),
            }
        })?;

    Ok(Json(AppChangalaRingEnrollStudentOutput {
        course_uri: input.course_uri,
        did,
        enrolled_at: now,
    }))
}

/// `app.changala.ring.getEnrollments` — GET query.
///
/// Lists enrolled student DIDs for a course, with cursor-based pagination
/// and a total count.
pub async fn get_enrollments(
    Query(params): Query<AppChangalaRingGetEnrollmentsParams>,
) -> Result<Json<AppChangalaRingGetEnrollmentsOutput>, XrpcError> {
    let app = crate::state::get();
    let limit = params.limit.unwrap_or(50).min(100).max(1);

    // Total enrolled count (regardless of pagination)
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM enrollments WHERE course_uri = $1")
        .bind(&params.course_uri)
        .fetch_one(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    let fetch_limit = limit + 1;

    let rows = if let Some(ref cursor) = params.cursor {
        sqlx::query_as::<_, (String, String)>(
            "SELECT did, enrolled_at FROM enrollments \
             WHERE course_uri = $1 AND enrolled_at < $2 \
             ORDER BY enrolled_at DESC LIMIT $3",
        )
        .bind(&params.course_uri)
        .bind(cursor)
        .bind(fetch_limit)
        .fetch_all(&app.db)
        .await
    } else {
        sqlx::query_as::<_, (String, String)>(
            "SELECT did, enrolled_at FROM enrollments \
             WHERE course_uri = $1 \
             ORDER BY enrolled_at DESC LIMIT $2",
        )
        .bind(&params.course_uri)
        .bind(fetch_limit)
        .fetch_all(&app.db)
        .await
    }
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("database error: {e}"),
    })?;

    let has_more = rows.len() as i64 > limit;
    let rows = if has_more {
        &rows[..limit as usize]
    } else {
        &rows
    };

    let dids: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let cursor = if has_more {
        rows.last().map(|r| r.1.clone())
    } else {
        None
    };

    Ok(Json(AppChangalaRingGetEnrollmentsOutput {
        course_uri: params.course_uri,
        dids,
        total: total.0,
        cursor,
    }))
}

/// `app.changala.ring.assignClassRep` — POST procedure.
///
/// Updates the class representative for a course and promotes the target
/// DID's role in the memberships table to `classRep`. Returns the updated
/// course view.
pub async fn assign_class_rep(
    RequireAuth(session): RequireAuth,
    Json(input): Json<AppChangalaRingAssignClassRepInput>,
) -> Result<Json<AppChangalaRingAssignClassRepOutput>, XrpcError> {
    let app = crate::state::get();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    // Verify the course exists
    let course_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM courses WHERE uri = $1")
        .bind(&input.course_uri)
        .fetch_optional(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    if course_exists.is_none() {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("course not found: {}", input.course_uri),
        });
    }

    // Update the course's class_rep_did
    sqlx::query("UPDATE courses SET class_rep_did = $1 WHERE uri = $2")
        .bind(&input.class_rep_did)
        .bind(&input.course_uri)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    // Promote the DID's role in the memberships table to 'classRep'.
    // This is a best-effort update — the DID may not have a membership row
    // yet (e.g. not verified), but we still record the course assignment.
    sqlx::query("UPDATE memberships SET role = 'classRep' WHERE did = $1")
        .bind(&input.class_rep_did)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("database error: {e}"),
        })?;

    // Fetch the updated course view
    let view = fetch_course_view(&app.db, &input.course_uri)
        .await?
        .ok_or_else(|| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: "course disappeared after update".to_string(),
        })?;

    Ok(Json(AppChangalaRingAssignClassRepOutput {
        uri: view.uri,
        title: view.title,
        code: view.code,
        department: view.department,
        semester: view.semester,
        visibility: view.visibility,
        created_by: view.created_by,
        class_rep_did: view.class_rep_did,
        description: view.description,
        created_at: view.created_at,
        enrolled_count: view.enrolled_count,
    }))
}
