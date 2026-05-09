//! Changala MCP Server — AI-powered institution management.
//!
//! A standalone MCP server that translates tool calls into XRPC HTTP requests
//! to a changala-ring instance. Authenticated via API key.
//!
//! Usage:
//!   CHANGALA_API_KEY=chg_xxx CHANGALA_RING_URL=https://ring.changala.app changala-mcp

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::schemars::JsonSchema;
use rmcp::service::ServiceExt;
use rmcp::{tool, tool_router};
use serde::Deserialize;

mod tools;

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

// ---------------------------------------------------------------------------
// Server struct and XRPC helpers
// ---------------------------------------------------------------------------

struct ChangalaServer {
    client: reqwest::Client,
    ring_url: String,
    api_key: String,
}

impl ChangalaServer {
    fn new() -> anyhow::Result<Self> {
        let ring_url = std::env::var("CHANGALA_RING_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        let api_key = std::env::var("CHANGALA_API_KEY")
            .map_err(|_| anyhow::anyhow!("CHANGALA_API_KEY env var is required"))?;

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
        let mut url = format!("{}/xrpc/{}", self.ring_url, nsid);
        if !params.is_empty() {
            url.push('?');
            let pairs: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect();
            url.push_str(&pairs.join("&"));
        }
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;
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
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
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
        let body = serde_json::json!({"courseUri": p.course_uri, "targetDid": p.target_did});
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
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("changala_mcp=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("starting changala-mcp server");

    let server = ChangalaServer::new()?;

    // Run on stdio transport (for Claude Desktop / Cursor)
    let transport = rmcp::transport::io::stdio();

    let service = server.serve(transport).await?;

    service.waiting().await?;

    Ok(())
}
