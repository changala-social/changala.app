//! Timetable handlers — calendar loading, slot loading, session provisioning.
//!
//! These endpoints load institutional scheduling data into the Ring and
//! use it to generate sessions for courses.

use atrg_auth::RequireAuth;
use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use axum::extract::State;
use axum::Json;
use chrono::{Datelike, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

use super::auth;

// ── Input/Output types ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoadCalendarInput {
    pub semester: String,
    pub phases: Vec<PhaseInput>,
    pub holidays: Vec<HolidayInput>,
}

#[derive(Deserialize)]
pub struct PhaseInput {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub phase_type: String,
    pub start: String, // YYYY-MM-DD
    pub end: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct HolidayInput {
    pub date: String, // YYYY-MM-DD
    pub name: String,
    #[serde(rename = "type")]
    pub holiday_type: Option<String>,
}

#[derive(Deserialize)]
pub struct LoadSlotsInput {
    pub semester: String,
    pub slots: std::collections::HashMap<String, SlotInput>,
}

#[derive(Deserialize)]
pub struct SlotInput {
    #[serde(rename = "type")]
    pub slot_type: Option<String>,
    pub slot_system: Option<serde_json::Value>, // can be int or string
    pub schedule: Vec<OccurrenceInput>,
}

#[derive(Deserialize)]
pub struct OccurrenceInput {
    pub day: String,   // "MON", "TUE", etc.
    pub start: String, // "09:00"
    pub end: String,   // "09:50"
}

#[derive(Deserialize)]
pub struct ProvisionSessionsInput {
    #[serde(rename = "courseUri")]
    pub course_uri: String,
    pub slot: String,
    pub semester: String,
}

#[derive(Serialize)]
pub struct ProvisionSessionsOutput {
    #[serde(rename = "courseUri")]
    pub course_uri: String,
    pub slot: String,
    #[serde(rename = "sessionsCreated")]
    pub sessions_created: i64,
    #[serde(rename = "firstSession")]
    pub first_session: Option<String>,
    #[serde(rename = "lastSession")]
    pub last_session: Option<String>,
}

// ── Handlers ────────────────────────────────────────────────────────────

/// POST /xrpc/app.changala.ring.loadCalendar
///
/// Admin only — loads semester phases and holidays into the Ring.
/// Replaces any existing data for the given semester.
pub async fn load_calendar(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<LoadCalendarInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    // Clear existing data for this semester
    sqlx::query("DELETE FROM semester_holidays WHERE semester = $1")
        .bind(&input.semester)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to clear holidays: {e}"),
        })?;
    sqlx::query("DELETE FROM semester_phases WHERE semester = $1")
        .bind(&input.semester)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to clear phases: {e}"),
        })?;

    // Insert phases
    let mut phase_count = 0i64;
    for p in &input.phases {
        sqlx::query(
            "INSERT INTO semester_phases \
                 (semester, name, label, phase_type, start_date, end_date, notes) \
             VALUES ($1, $2, $3, $4, $5::DATE, $6::DATE, $7) \
             ON CONFLICT (semester, name) DO NOTHING",
        )
        .bind(&input.semester)
        .bind(&p.name)
        .bind(&p.label)
        .bind(&p.phase_type)
        .bind(&p.start)
        .bind(&p.end)
        .bind(&p.notes)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to insert phase {}: {e}", p.name),
        })?;
        phase_count += 1;
    }

    // Insert holidays
    let mut holiday_count = 0i64;
    for h in &input.holidays {
        sqlx::query(
            "INSERT INTO semester_holidays \
                 (semester, holiday_date, name, holiday_type) \
             VALUES ($1, $2::DATE, $3, $4) \
             ON CONFLICT (semester, holiday_date) DO NOTHING",
        )
        .bind(&input.semester)
        .bind(&h.date)
        .bind(&h.name)
        .bind(&h.holiday_type)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to insert holiday {}: {e}", h.name),
        })?;
        holiday_count += 1;
    }

    Ok(Json(json!({
        "semester": input.semester,
        "phasesLoaded": phase_count,
        "holidaysLoaded": holiday_count,
    })))
}

/// POST /xrpc/app.changala.ring.loadSlots
///
/// Admin only — loads timetable slot definitions into the Ring.
/// Replaces any existing slots for the given semester.
pub async fn load_slots(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<LoadSlotsInput>,
) -> Result<Json<serde_json::Value>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    // Clear existing slots for this semester (cascade deletes occurrences)
    sqlx::query("DELETE FROM timetable_slots WHERE semester = $1")
        .bind(&input.semester)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to clear slots: {e}"),
        })?;

    let mut slot_count = 0i64;
    let mut occurrence_count = 0i64;

    for (slot_name, slot_def) in &input.slots {
        let slot_type = slot_def.slot_type.as_deref().unwrap_or("theory");
        let slot_system = slot_def.slot_system.as_ref().map(|v| match v {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s.clone(),
            _ => v.to_string(),
        });

        let slot_id: i64 = sqlx::query_scalar(
            "INSERT INTO timetable_slots (semester, slot_name, slot_type, slot_system) \
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(&input.semester)
        .bind(slot_name)
        .bind(slot_type)
        .bind(&slot_system)
        .fetch_one(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to insert slot {slot_name}: {e}"),
        })?;
        slot_count += 1;

        for occ in &slot_def.schedule {
            sqlx::query(
                "INSERT INTO slot_occurrences (slot_id, day_of_week, start_time, end_time) \
                 VALUES ($1, $2, $3::TIME, $4::TIME)",
            )
            .bind(slot_id)
            .bind(&occ.day)
            .bind(&occ.start)
            .bind(&occ.end)
            .execute(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to insert occurrence for {slot_name}: {e}"),
            })?;
            occurrence_count += 1;
        }
    }

    Ok(Json(json!({
        "semester": input.semester,
        "slotsLoaded": slot_count,
        "occurrencesLoaded": occurrence_count,
    })))
}

/// POST /xrpc/app.changala.ring.provisionSessions
///
/// Admin only — generates sessions for a course based on its slot and the
/// semester calendar. Computes all instructional dates for the slot, skips
/// holidays and exam periods.
pub async fn provision_sessions(
    State(state): State<AppState>,
    RequireAuth(session): RequireAuth,
    Json(input): Json<ProvisionSessionsInput>,
) -> Result<Json<ProvisionSessionsOutput>, XrpcError> {
    let app = state.extension::<crate::ChangalaState>();
    auth::check_not_banned(&app.db, &session.did).await?;
    auth::require_role(&app.db, &session.did, "admin").await?;

    // Verify course exists
    let course_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM courses WHERE uri = $1)")
            .bind(&input.course_uri)
            .fetch_one(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Course lookup failed: {e}"),
            })?;
    if !course_exists {
        return Err(XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!("Course not found: {}", input.course_uri),
        });
    }

    // Get slot occurrences (day + time)
    let slot_row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM timetable_slots WHERE semester = $1 AND slot_name = $2")
            .bind(&input.semester)
            .bind(&input.slot)
            .fetch_optional(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Slot lookup failed: {e}"),
            })?;

    let slot_id = slot_row
        .ok_or_else(|| XrpcError {
            name: XrpcErrorName::NotFound,
            message: format!(
                "Slot '{}' not found for semester {}",
                input.slot, input.semester
            ),
        })?
        .0;

    // Fetch as strings — sqlx doesn't have the `chrono` feature enabled
    let occ_rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT day_of_week, start_time::TEXT, end_time::TEXT FROM slot_occurrences WHERE slot_id = $1",
    )
    .bind(slot_id)
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch slot occurrences: {e}"),
    })?;

    if occ_rows.is_empty() {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!("Slot '{}' has no schedule occurrences", input.slot),
        });
    }

    // Parse occurrence strings into typed values
    let occurrences: Vec<(String, NaiveTime, NaiveTime)> = occ_rows
        .into_iter()
        .map(|(day, start_str, end_str)| {
            let start = NaiveTime::parse_from_str(&start_str, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(&start_str, "%H:%M"))
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("Invalid start_time '{start_str}': {e}"),
                })?;
            let end = NaiveTime::parse_from_str(&end_str, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(&end_str, "%H:%M"))
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("Invalid end_time '{end_str}': {e}"),
                })?;
            Ok((day, start, end))
        })
        .collect::<Result<Vec<_>, XrpcError>>()?;

    // Get instructional phases for this semester (dates as TEXT)
    let phase_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT start_date::TEXT, end_date::TEXT FROM semester_phases \
         WHERE semester = $1 AND phase_type = 'instructional'",
    )
    .bind(&input.semester)
    .fetch_all(&app.db)
    .await
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch phases: {e}"),
    })?;

    if phase_rows.is_empty() {
        return Err(XrpcError {
            name: XrpcErrorName::InvalidRequest,
            message: format!(
                "No instructional phases found for semester {}",
                input.semester
            ),
        });
    }

    // Parse phase date strings
    let phases: Vec<(NaiveDate, NaiveDate)> = phase_rows
        .into_iter()
        .map(|(s, e)| {
            let start = NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|err| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Invalid phase start_date '{s}': {err}"),
            })?;
            let end = NaiveDate::parse_from_str(&e, "%Y-%m-%d").map_err(|err| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Invalid phase end_date '{e}': {err}"),
            })?;
            Ok((start, end))
        })
        .collect::<Result<Vec<_>, XrpcError>>()?;

    // Get holidays for this semester (dates as TEXT)
    let holiday_rows: Vec<(String,)> =
        sqlx::query_as("SELECT holiday_date::TEXT FROM semester_holidays WHERE semester = $1")
            .bind(&input.semester)
            .fetch_all(&app.db)
            .await
            .map_err(|e| XrpcError {
                name: XrpcErrorName::InternalServerError,
                message: format!("Failed to fetch holidays: {e}"),
            })?;

    let holiday_set: HashSet<NaiveDate> = holiday_rows
        .into_iter()
        .filter_map(|(s,)| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .collect();

    // Expand all valid session dates
    let mut session_dates: Vec<(NaiveDate, NaiveTime, NaiveTime)> = Vec::new();

    for (phase_start, phase_end) in &phases {
        let mut date = *phase_start;
        while date <= *phase_end {
            // Skip holidays
            if !holiday_set.contains(&date) {
                // Check if any slot occurrence matches this day
                for (day_str, start_time, end_time) in &occurrences {
                    if let Some(weekday) = day_to_weekday(day_str) {
                        if date.weekday() == weekday {
                            session_dates.push((date, *start_time, *end_time));
                        }
                    }
                }
            }
            date += chrono::Duration::days(1);
        }
    }

    session_dates.sort();

    // Create sessions
    let mut created = 0i64;
    // TODO: read timezone from institution config / meta.json
    let timezone = "+05:30";

    for (date, start_time, end_time) in &session_dates {
        let scheduled_at = format!("{}T{}{}", date, start_time.format("%H:%M:%S"), timezone);
        let duration_mins = (*end_time - *start_time).num_minutes();
        let rkey = atrg_repo::Tid::now().to_string();
        let uri = format!("at://changala.ring/app.changala.session/{}", rkey);
        let now = chrono::Utc::now().to_rfc3339();
        let topic = format!("[{}] {}", input.slot, date.format("%a %b %d"));

        let result = sqlx::query(
            "INSERT INTO sessions \
                 (uri, rkey, course_uri, scheduled_at, duration_mins, \
                  status, created_by, topic, created_at) \
             VALUES ($1, $2, $3, $4, $5, 'scheduled', $6, $7, $8) \
             ON CONFLICT (uri) DO NOTHING",
        )
        .bind(&uri)
        .bind(&rkey)
        .bind(&input.course_uri)
        .bind(&scheduled_at)
        .bind(duration_mins as i32)
        .bind(&session.did)
        .bind(&topic)
        .bind(&now)
        .execute(&app.db)
        .await
        .map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to create session: {e}"),
        })?;

        if result.rows_affected() > 0 {
            created += 1;
        }
    }

    let first = session_dates
        .first()
        .map(|(d, t, _)| format!("{}T{}{}", d, t.format("%H:%M"), timezone));
    let last = session_dates
        .last()
        .map(|(d, t, _)| format!("{}T{}{}", d, t.format("%H:%M"), timezone));

    Ok(Json(ProvisionSessionsOutput {
        course_uri: input.course_uri,
        slot: input.slot,
        sessions_created: created,
        first_session: first,
        last_session: last,
    }))
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Map day-of-week abbreviation strings to chrono `Weekday` values.
fn day_to_weekday(day: &str) -> Option<chrono::Weekday> {
    match day.to_uppercase().as_str() {
        "MON" | "MONDAY" => Some(chrono::Weekday::Mon),
        "TUE" | "TUESDAY" => Some(chrono::Weekday::Tue),
        "WED" | "WEDNESDAY" => Some(chrono::Weekday::Wed),
        "THU" | "THURSDAY" => Some(chrono::Weekday::Thu),
        "FRI" | "FRIDAY" => Some(chrono::Weekday::Fri),
        "SAT" | "SATURDAY" => Some(chrono::Weekday::Sat),
        "SUN" | "SUNDAY" => Some(chrono::Weekday::Sun),
        _ => None,
    }
}
