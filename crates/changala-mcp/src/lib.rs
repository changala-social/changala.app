//! Changala MCP Server — AI-powered institution management.
//!
//! Provides MCP (Model Context Protocol) tools for managing a Changala Ring
//! via AI assistants. Can be mounted on the Ring's axum router when
//! `CHANGALA_MCP_ENABLED=true`.
//!
//! The MCP server calls Ring XRPC endpoints via HTTP (loopback) using an API key.
//!
//! # Security
//!
//! When mounted on the Ring, the `/mcp` endpoint is protected by the Ring's
//! `mcp_gate_middleware` which validates API keys against the `api_keys`
//! database table. No env var needed.
//!
//! The standalone binary uses [`standalone_auth_middleware`] which falls
//! back to the `CHANGALA_MCP_ACCESS_KEY` env var (no DB access).

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_router};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Parameter structs for each tool
// ---------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct CreateCourseParams {
    /// Course title
    title: String,
    /// Course code (e.g. CS301)
    code: String,
    /// Department name
    department: String,
    /// Semester (e.g. Fall 2026)
    semester: String,
    /// Course description
    description: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct ListCoursesParams {
    /// Filter by semester
    semester: Option<String>,
    /// Filter by department
    department: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct EnrollStudentParams {
    /// Course AT URI
    course_uri: String,
    /// Student DID to enroll
    target_did: String,
    /// Slot preference (e.g. "B1" for morning, "B2" for afternoon). Optional.
    slot: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct AssignClassRepParams {
    /// Course AT URI
    course_uri: String,
    /// Class rep DID
    class_rep_did: String,
}

#[derive(Deserialize, JsonSchema)]
struct CreateSessionParams {
    /// Course AT URI
    course_uri: String,
    /// Scheduled time (ISO 8601)
    scheduled_at: String,
    /// Duration in minutes
    duration_mins: i64,
    /// Session topic
    topic: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct SessionUriParams {
    /// Session AT URI
    session_uri: String,
}

#[derive(Deserialize, JsonSchema)]
struct BanUserParams {
    /// DID to ban
    target_did: String,
    /// Reason for ban
    reason: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct PromoteRoleParams {
    /// Target user DID
    target_did: String,
    /// New role: student, classRep, or admin
    role: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetMembershipsParams {
    /// User DID
    did: String,
}

#[derive(Deserialize, JsonSchema)]
struct FetchUrlParams {
    /// URL to fetch (must return JSON)
    url: String,
}

#[derive(Deserialize, JsonSchema)]
struct BulkCreateCoursesParams {
    /// Array of courses to create. Each object needs: code, title, department, semester.
    /// Optional: description, visibility (defaults to "institution").
    courses: Vec<BulkCourseEntry>,
}

#[derive(Deserialize, JsonSchema)]
struct BulkCourseEntry {
    /// Course code (e.g. EC2002E)
    code: String,
    /// Full title
    title: String,
    /// Department code
    department: String,
    /// Semester identifier (e.g. 2025-winter)
    semester: String,
    /// Optional description
    description: Option<String>,
    /// Visibility (defaults to "institution")
    visibility: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct LoadCalendarParams {
    /// Semester identifier (e.g. "2025-winter")
    semester: String,
    /// Calendar phases (instructional, exam, etc.) — array of objects with name, label, type, start, end
    phases: Vec<CalendarPhaseParam>,
    /// Holidays — array of objects with date and name
    holidays: Vec<HolidayParam>,
}

#[derive(Deserialize, JsonSchema)]
struct CalendarPhaseParam {
    /// Phase identifier
    name: String,
    /// Human-readable label
    label: String,
    /// Phase type: "instructional", "exam", "festival", "administrative"
    #[serde(rename = "type")]
    phase_type: String,
    /// Start date (YYYY-MM-DD)
    start: String,
    /// End date (YYYY-MM-DD)
    end: String,
}

#[derive(Deserialize, JsonSchema)]
struct HolidayParam {
    /// Holiday date (YYYY-MM-DD)
    date: String,
    /// Holiday name
    name: String,
}

#[derive(Deserialize, JsonSchema)]
struct LoadSlotsParams {
    /// Semester identifier
    semester: String,
    /// Slot definitions as a JSON object — keys are slot names, values have schedule arrays
    slots: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
struct ProvisionSessionsParams {
    /// AT URI of the course
    course_uri: String,
    /// Slot name (e.g. "B1")
    slot: String,
    /// Semester identifier
    semester: String,
}

// ---------------------------------------------------------------------------
// Server struct and XRPC helpers
// ---------------------------------------------------------------------------

pub struct ChangalaServer {
    client: reqwest::Client,
    ring_url: String,
    api_key: String,
}

impl ChangalaServer {
    fn new() -> anyhow::Result<Self> {
        // Ring URL: when running inside the Ring, defaults to loopback.
        // CHANGALA_RING_URL is only needed for standalone deployment.
        let ring_url = std::env::var("CHANGALA_RING_URL").unwrap_or_else(|_| {
            let port = std::env::var("ATRG_APP__PORT").unwrap_or_else(|_| "3000".to_string());
            format!("http://127.0.0.1:{}", port)
        });

        // API key: try CHANGALA_API_KEY first (explicit MCP key),
        // fall back to CHANGALA_BOOTSTRAP_API_KEY (the Ring's bootstrap key).
        let api_key = std::env::var("CHANGALA_API_KEY")
            .or_else(|_| std::env::var("CHANGALA_BOOTSTRAP_API_KEY"))
            .map_err(|_| {
                anyhow::anyhow!(
                    "Either CHANGALA_API_KEY or CHANGALA_BOOTSTRAP_API_KEY must be set for MCP"
                )
            })?;

        Ok(Self {
            client: reqwest::Client::new(),
            ring_url,
            api_key,
        })
    }

    async fn xrpc_get(
        &self,
        nsid: &str,
        params: &[(&str, &str)],
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/xrpc/{}", self.ring_url, nsid);
        let resp = self
            .client
            .get(&url)
            .query(params)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .error_for_status()?;
        let body: serde_json::Value = resp.json().await?;
        Ok(body)
    }

    async fn xrpc_post(
        &self,
        nsid: &str,
        body: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/xrpc/{}", self.ring_url, nsid);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let result: serde_json::Value = resp.json().await?;
        Ok(result)
    }
}

/// Helper to build a success result from a JSON value.
fn ok_result(value: &serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(value).unwrap_or_default(),
    )])
}

/// Helper to build an error result from an anyhow error.
fn err_result(e: anyhow::Error) -> CallToolResult {
    CallToolResult::error(vec![Content::text(format!("Error: {e}"))])
}

// ---------------------------------------------------------------------------
// Tool definitions
// ---------------------------------------------------------------------------

#[tool_router(server_handler)]
impl ChangalaServer {
    #[tool(description = "Create a new course")]
    async fn create_course(
        &self,
        Parameters(p): Parameters<CreateCourseParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut body = serde_json::json!({
            "title": p.title,
            "code": p.code,
            "department": p.department,
            "semester": p.semester,
            "visibility": "institution"
        });
        if let Some(desc) = p.description {
            body["description"] = serde_json::json!(desc);
        }
        match self.xrpc_post("app.changala.ring.createCourse", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "List all courses, optionally filtered by semester or department")]
    async fn list_courses(
        &self,
        Parameters(p): Parameters<ListCoursesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut params = vec![];
        let sem_str;
        let dept_str;
        if let Some(ref s) = p.semester {
            sem_str = s.clone();
            params.push(("semester", sem_str.as_str()));
        }
        if let Some(ref d) = p.department {
            dept_str = d.clone();
            params.push(("department", dept_str.as_str()));
        }
        match self
            .xrpc_get("app.changala.ring.listCourses", &params)
            .await
        {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Enroll a student (by DID) into a course")]
    async fn enroll_student(
        &self,
        Parameters(p): Parameters<EnrollStudentParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({"courseUri": p.course_uri, "targetDid": p.target_did, "slot": p.slot});
        match self
            .xrpc_post("app.changala.ring.enrollStudent", body)
            .await
        {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Assign a class representative to a course")]
    async fn assign_class_rep(
        &self,
        Parameters(p): Parameters<AssignClassRepParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({"courseUri": p.course_uri, "classRepDid": p.class_rep_did});
        match self
            .xrpc_post("app.changala.ring.assignClassRep", body)
            .await
        {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Create a new session for a course")]
    async fn create_session(
        &self,
        Parameters(p): Parameters<CreateSessionParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut body = serde_json::json!({
            "courseUri": p.course_uri,
            "scheduledAt": p.scheduled_at,
            "durationMins": p.duration_mins
        });
        if let Some(t) = p.topic {
            body["topic"] = serde_json::json!(t);
        }
        match self
            .xrpc_post("app.changala.ring.createSession", body)
            .await
        {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Open a session (transition to live)")]
    async fn open_session(
        &self,
        Parameters(p): Parameters<SessionUriParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({"sessionUri": p.session_uri});
        match self.xrpc_post("app.changala.ring.openSession", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Close a session (transition to ended, opens keyword window)")]
    async fn close_session(
        &self,
        Parameters(p): Parameters<SessionUriParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({"sessionUri": p.session_uri});
        match self.xrpc_post("app.changala.ring.closeSession", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Ban a user by DID")]
    async fn ban_user(
        &self,
        Parameters(p): Parameters<BanUserParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut body = serde_json::json!({"targetDid": p.target_did});
        if let Some(r) = p.reason {
            body["reason"] = serde_json::json!(r);
        }
        match self.xrpc_post("app.changala.ring.banDid", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "List all active bans")]
    async fn list_bans(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        match self.xrpc_get("app.changala.ring.listBans", &[]).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Promote or change a user's role (admin, classRep, student)")]
    async fn promote_role(
        &self,
        Parameters(p): Parameters<PromoteRoleParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({"targetDid": p.target_did, "role": p.role});
        match self.xrpc_post("app.changala.ring.promoteRole", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "View the admin audit log")]
    async fn get_audit_log(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        match self.xrpc_get("app.changala.ring.getAuditLog", &[]).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "Get user memberships and role")]
    async fn get_memberships(
        &self,
        Parameters(p): Parameters<GetMembershipsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match self
            .xrpc_get("app.changala.ring.getMemberships", &[("did", &p.did)])
            .await
        {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(description = "List API keys (admin only)")]
    async fn list_api_keys(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        match self.xrpc_get("app.changala.ring.listApiKeys", &[]).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(
        description = "Fetch a JSON URL. Use this to read curriculum data from the institution's data repo (e.g. courses.json, calendar, slots)."
    )]
    async fn fetch_url(
        &self,
        Parameters(p): Parameters<FetchUrlParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match self
            .client
            .get(&p.url)
            .header("User-Agent", "changala-mcp/0.1")
            .send()
            .await
        {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(json) => Ok(ok_result(&json)),
                Err(e) => Ok(err_result(anyhow::anyhow!("Failed to parse JSON: {e}"))),
            },
            Err(e) => Ok(err_result(anyhow::anyhow!("Fetch failed: {e}"))),
        }
    }

    #[tool(
        description = "Create multiple courses in one call. Provide an array of courses each with code, title, department, semester. Returns a summary of created/failed courses."
    )]
    async fn bulk_create_courses(
        &self,
        Parameters(p): Parameters<BulkCreateCoursesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let total = p.courses.len();
        let mut created = 0u32;
        let mut failed = Vec::new();

        for c in &p.courses {
            let body = serde_json::json!({
                "title": c.title,
                "code": c.code,
                "department": c.department,
                "semester": c.semester,
                "visibility": c.visibility.as_deref().unwrap_or("institution"),
                "description": c.description.as_deref().unwrap_or(""),
            });
            match self.xrpc_post("app.changala.ring.createCourse", body).await {
                Ok(_) => created += 1,
                Err(e) => failed.push(format!("{}: {}", c.code, e)),
            }
        }

        let summary = serde_json::json!({
            "total": total,
            "created": created,
            "failed_count": failed.len(),
            "failed": failed,
        });
        Ok(ok_result(&summary))
    }

    #[tool(
        description = "Load academic calendar (phases + holidays) for a semester into the Ring. Admin only. Provide phases with name/label/type/start/end and holidays with date/name."
    )]
    async fn load_calendar(
        &self,
        Parameters(p): Parameters<LoadCalendarParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut phases = Vec::new();
        for ph in &p.phases {
            phases.push(serde_json::json!({
                "name": ph.name,
                "label": ph.label,
                "type": ph.phase_type,
                "start": ph.start,
                "end": ph.end,
            }));
        }
        let mut holidays = Vec::new();
        for h in &p.holidays {
            holidays.push(serde_json::json!({
                "date": h.date,
                "name": h.name,
            }));
        }
        let body = serde_json::json!({
            "semester": p.semester,
            "phases": phases,
            "holidays": holidays,
        });
        match self.xrpc_post("app.changala.ring.loadCalendar", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(
        description = "Load timetable slot definitions for a semester into the Ring. Admin only. Provide slots as a JSON object where keys are slot names and values have 'schedule' arrays with day/start/end."
    )]
    async fn load_slots(
        &self,
        Parameters(p): Parameters<LoadSlotsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({
            "semester": p.semester,
            "slots": p.slots,
        });
        match self.xrpc_post("app.changala.ring.loadSlots", body).await {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }

    #[tool(
        description = "Provision sessions for a course based on its assigned slot. Uses the loaded calendar and slot definitions to compute all instructional dates, skipping holidays and exam periods. Admin only."
    )]
    async fn provision_sessions(
        &self,
        Parameters(p): Parameters<ProvisionSessionsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let body = serde_json::json!({
            "courseUri": p.course_uri,
            "slot": p.slot,
            "semester": p.semester,
        });
        match self
            .xrpc_post("app.changala.ring.provisionSessions", body)
            .await
        {
            Ok(result) => Ok(ok_result(&result)),
            Err(e) => Ok(err_result(e)),
        }
    }
}

// ---------------------------------------------------------------------------
// Auth middleware (standalone binary only)
// ---------------------------------------------------------------------------

/// Simple env-var auth middleware for the **standalone** MCP binary.
///
/// When the MCP server is mounted on the Ring, use the Ring's
/// `mcp_gate_middleware` instead — it validates against the `api_keys`
/// database table.
///
/// This middleware is only for the standalone `changala-mcp` binary which
/// has no database access. It gates on `CHANGALA_MCP_ACCESS_KEY` env var.
pub async fn standalone_auth_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let expected = std::env::var("CHANGALA_MCP_ACCESS_KEY").ok();

    let Some(expected_key) = expected else {
        tracing::warn!(
            "CHANGALA_MCP_ACCESS_KEY is not set \u{2014} MCP endpoint is unauthenticated!"
        );
        return next.run(req).await;
    };

    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(value)
            if value
                .strip_prefix("Bearer ")
                .is_some_and(|t| t == expected_key) =>
        {
            next.run(req).await
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Valid Bearer token required for MCP access"
            })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Service constructor
// ---------------------------------------------------------------------------

/// Create the MCP axum service that can be `route_service`'d onto a Router.
///
/// **When mounted on the Ring**, gate with the Ring's `mcp_gate_middleware`
/// which validates against the `api_keys` database table.
///
/// # Example
///
/// ```ignore
/// let mcp_router = axum::Router::new()
///     .route_service("/mcp", changala_mcp::mcp_service())
///     .layer(axum::middleware::from_fn(api_key_auth::mcp_gate_middleware));
/// ```
#[must_use]
pub fn mcp_service() -> rmcp::transport::streamable_http_server::StreamableHttpService<
    ChangalaServer,
    rmcp::transport::streamable_http_server::session::local::LocalSessionManager,
> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
    };

    // MCP host/origin allowlists from env vars.
    // Empty = disabled (allow all). Comma-separated for multiple values.
    //   CHANGALA_MCP_ALLOWED_HOSTS=changala-ring.example.com,localhost
    //   CHANGALA_MCP_ALLOWED_ORIGINS=https://changala-app.pages.dev
    let mut config = StreamableHttpServerConfig::default();

    match std::env::var("CHANGALA_MCP_ALLOWED_HOSTS") {
        Ok(v) if !v.is_empty() => {
            let hosts: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            config = config.with_allowed_hosts(hosts);
        }
        _ => {
            config = config.disable_allowed_hosts();
        }
    }

    match std::env::var("CHANGALA_MCP_ALLOWED_ORIGINS") {
        Ok(v) if !v.is_empty() => {
            let origins: Vec<String> = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            config = config.with_allowed_origins(origins);
        }
        _ => {
            config = config.disable_allowed_origins();
        }
    }

    StreamableHttpService::new(
        || ChangalaServer::new().map_err(std::io::Error::other),
        Default::default(),
        config,
    )
}
