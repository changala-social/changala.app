use anyhow::Context;
use atrg_core::AtrgApp;
use sqlx::postgres::PgPool;
use std::path::Path;
use std::sync::Arc;

mod api_key_auth;
mod handlers;
mod routes;

/// Changala application state — registered as an AppState extension.
///
/// Replaces the old `once_cell` singleton. Access in handlers via:
///   `let app = state.extension::<ChangalaState>();`
#[derive(Clone, Debug)]
pub struct ChangalaState {
    /// PostgreSQL connection pool for all business data.
    pub db: PgPool,
    /// S3-compatible blob store for content (notes, brain nodes, archives).
    pub blobs: Arc<atrg_blob::s3::S3BlobStore>,
    /// Allowed institution email domains for membership verification.
    pub allowed_email_domains: Vec<String>,
    /// Optional SMTP config — None = dev mode (log OTPs to stdout).
    pub email_config: Option<atrg_email::EmailConfig>,
}

/// Changala Ring config loaded from atrg.toml [changala] section.
///
/// Every field can be overridden by an environment variable:
///   CHANGALA_DATABASE_URL    →  [changala] database_url
///   CHANGALA_S3_ENDPOINT     →  [changala.s3] endpoint
///   CHANGALA_S3_BUCKET       →  [changala.s3] bucket
///   CHANGALA_S3_REGION       →  [changala.s3] region
///   CHANGALA_S3_ACCESS_KEY   →  [changala.s3] access_key
///   CHANGALA_S3_SECRET_KEY   →  [changala.s3] secret_key
///   CHANGALA_S3_PATH_STYLE   →  [changala.s3] path_style  (true/false)
///   CHANGALA_ALLOWED_EMAIL_DOMAINS → [changala] allowed_email_domains (comma-separated)
///   CHANGALA_ADMIN_DIDS      →  [changala] admin_dids  (comma-separated)
///   CHANGALA_SMTP_HOST       →  [changala.smtp] host
///   CHANGALA_SMTP_PORT       →  [changala.smtp] port
///   CHANGALA_SMTP_USERNAME   →  [changala.smtp] username
///   CHANGALA_SMTP_PASSWORD   →  [changala.smtp] password
///   CHANGALA_SMTP_FROM       →  [changala.smtp] from
///   CHANGALA_SMTP_ENCRYPTION →  [changala.smtp] encryption
#[derive(Debug, serde::Deserialize)]
struct ChangalaConfig {
    database_url: String,
    s3: atrg_blob::s3::S3Config,
    #[serde(default)]
    smtp: Option<atrg_email::EmailConfig>,
    #[serde(default)]
    allowed_email_domains: Vec<String>,
    #[serde(default)]
    admin_dids: Vec<String>,
}

impl ChangalaConfig {
    /// Apply environment variable overrides. Env vars take precedence over atrg.toml.
    fn apply_env_overrides(&mut self) {
        let mut overrides = Vec::new();
        if let Ok(v) = std::env::var("CHANGALA_DATABASE_URL") {
            self.database_url = v;
            overrides.push("CHANGALA_DATABASE_URL");
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_ENDPOINT") {
            self.s3.endpoint = v;
            overrides.push("CHANGALA_S3_ENDPOINT");
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_BUCKET") {
            self.s3.bucket = v;
            overrides.push("CHANGALA_S3_BUCKET");
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_REGION") {
            self.s3.region = v;
            overrides.push("CHANGALA_S3_REGION");
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_ACCESS_KEY") {
            self.s3.access_key = v;
            overrides.push("CHANGALA_S3_ACCESS_KEY");
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_SECRET_KEY") {
            self.s3.secret_key = v;
            overrides.push("CHANGALA_S3_SECRET_KEY");
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_PATH_STYLE") {
            match v.to_lowercase().as_str() {
                "true" | "1" | "yes" => self.s3.path_style = true,
                "false" | "0" | "no" => self.s3.path_style = false,
                _ => tracing::warn!(
                    value = %v,
                    "ignoring invalid CHANGALA_S3_PATH_STYLE value (expected true/false)"
                ),
            }
            overrides.push("CHANGALA_S3_PATH_STYLE");
        }
        if let Ok(v) = std::env::var("CHANGALA_ALLOWED_EMAIL_DOMAINS") {
            self.allowed_email_domains = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            overrides.push("CHANGALA_ALLOWED_EMAIL_DOMAINS");
        }
        if let Ok(v) = std::env::var("CHANGALA_ADMIN_DIDS") {
            self.admin_dids = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            overrides.push("CHANGALA_ADMIN_DIDS");
        }
        // SMTP overrides
        if let Ok(v) = std::env::var("CHANGALA_SMTP_HOST") {
            let smtp = self.smtp.get_or_insert_with(|| atrg_email::EmailConfig {
                host: String::new(),
                port: 587,
                username: String::new(),
                password: String::new(),
                from: String::new(),
                encryption: "starttls".to_string(),
                otp_expiry_secs: 600,
            });
            smtp.host = v;
            overrides.push("CHANGALA_SMTP_HOST");
        }
        if let Ok(v) = std::env::var("CHANGALA_SMTP_PORT") {
            if let Some(ref mut smtp) = self.smtp {
                if let Ok(port) = v.parse::<u16>() {
                    smtp.port = port;
                }
            }
            overrides.push("CHANGALA_SMTP_PORT");
        }
        if let Ok(v) = std::env::var("CHANGALA_SMTP_USERNAME") {
            if let Some(ref mut smtp) = self.smtp {
                smtp.username = v;
            }
            overrides.push("CHANGALA_SMTP_USERNAME");
        }
        if let Ok(v) = std::env::var("CHANGALA_SMTP_PASSWORD") {
            if let Some(ref mut smtp) = self.smtp {
                smtp.password = v;
            }
            overrides.push("CHANGALA_SMTP_PASSWORD");
        }
        if let Ok(v) = std::env::var("CHANGALA_SMTP_FROM") {
            if let Some(ref mut smtp) = self.smtp {
                smtp.from = v;
            }
            overrides.push("CHANGALA_SMTP_FROM");
        }
        if let Ok(v) = std::env::var("CHANGALA_SMTP_ENCRYPTION") {
            if let Some(ref mut smtp) = self.smtp {
                smtp.encryption = v;
            }
            overrides.push("CHANGALA_SMTP_ENCRYPTION");
        }
        if overrides.is_empty() {
            tracing::info!("no env var overrides applied, using atrg.toml values");
        } else {
            tracing::info!(overrides = ?overrides, "applied env var overrides");
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load changala-specific config from atrg.toml [changala] section
    // atrg 0.2.0: use load_app_config() instead of manual TOML parsing
    let mut config: ChangalaConfig = atrg_core::config::load_app_config("changala")
        .context("Failed to load [changala] config from atrg.toml")?;

    // Env vars override atrg.toml — for k8s Secrets, docker .env, etc.
    config.apply_env_overrides();

    // Connect to PostgreSQL
    let pg_pool = PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;
    tracing::info!(url = %config.database_url, "connected to PostgreSQL");

    // Run Changala business migrations using isolated tracking table.
    // atrg 0.2.0: run_isolated_migrations() replaces the hand-rolled runner.
    // Uses "_ring_migrations" tracking table so it never conflicts with
    // atrg-core's internal "_atrg_migrations" table.
    atrg_db::run_isolated_migrations(
        &atrg_db::DbPool::Postgres(pg_pool.clone()),
        Path::new("./ring_migrations"),
        "_ring_migrations",
    )
    .await
    .context("Failed to run ring migrations")?;
    tracing::info!("applied ring migrations");

    // Initialize S3 blob store (atrg_blob — replaces the custom blob.rs module)
    let blobs = atrg_blob::s3::S3BlobStore::new(&config.s3)
        .map_err(|e| anyhow::anyhow!("Failed to initialize S3 blob store: {e}"))?;
    tracing::info!(endpoint = %config.s3.endpoint, bucket = %config.s3.bucket, "S3 blob store ready");

    // Build Changala state — registered as an AppState extension via with_extension()
    // atrg 0.2.0: replaces the once_cell singleton pattern
    let changala_state = ChangalaState {
        db: pg_pool.clone(),
        blobs: Arc::new(blobs),
        allowed_email_domains: config.allowed_email_domains.clone(),
        email_config: config.smtp.clone(),
    };

    // Auto-provision admin DIDs from config/env var
    if !config.admin_dids.is_empty() {
        for did in &config.admin_dids {
            let result = sqlx::query(
                "INSERT INTO memberships (did, institution_did, institution_domain, role, verified_email, verified_at) \
                 VALUES ($1, 'did:web:system', 'system', 'admin', 'system-provisioned', $2) \
                 ON CONFLICT (did, institution_did) DO UPDATE SET role = 'admin'"
            )
            .bind(did)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&pg_pool)
            .await;
            match result {
                Ok(_) => tracing::info!(did = %did, "auto-provisioned admin DID"),
                Err(e) => {
                    tracing::warn!(did = %did, error = %e, "failed to auto-provision admin DID")
                }
            }
        }
    }

    // Auto-provision bootstrap API key.
    // Set CHANGALA_BOOTSTRAP_API_KEY to any non-empty value (e.g. "generate")
    // to auto-create an admin API key on startup. The generated key is logged
    // once and cannot be recovered — save it immediately.
    // NOTE: atrg-auth generates the key material; the env var no longer
    //       supplies the key value directly (format changed to hex).
    if let Ok(val) = std::env::var("CHANGALA_BOOTSTRAP_API_KEY") {
        if !val.is_empty() {
            let db_pool = atrg_db::DbPool::Postgres(pg_pool.clone());
            let admin_did = config
                .admin_dids
                .first()
                .map(|s| s.as_str())
                .unwrap_or("did:web:system");
            match atrg_auth::api_keys::create_api_key(
                &db_pool,
                admin_did,
                "Bootstrap Key",
                &["admin:*".to_string()],
                "chg_",
            )
            .await
            {
                Ok((full_key, api_key)) => {
                    tracing::info!(
                        prefix = %api_key.key_prefix,
                        "bootstrap API key created — key: {}",
                        full_key
                    );
                }
                Err(e) => {
                    // Might fail if key already exists from a previous run
                    tracing::debug!(error = %e, "bootstrap API key creation skipped (may already exist)");
                }
            }
        }
    }

    // Mount MCP server on the Ring when enabled via env var
    let mcp_enabled = std::env::var("CHANGALA_MCP_ENABLED").unwrap_or_default() == "true";
    if mcp_enabled {
        tracing::info!("MCP server will be mounted at /mcp");
    }

    let app_router = routes::api();

    // Clone the pool before moving it into the builder — MCP middleware needs it.
    let mcp_db_pool = pg_pool.clone();

    let mut builder = AtrgApp::new()
        .with_db_pool(pg_pool)
        .with_extension(changala_state)
        .with_auth_routes(atrg_auth::routes::routes())
        .with_cleanup_task(atrg_auth::routes::spawn_cleanup_task)
        .mount(app_router);

    if mcp_enabled {
        let mcp_router = axum::Router::<atrg_core::AppState>::new()
            .route_service("/mcp", changala_mcp::mcp_service())
            .route_service("/mcp/", changala_mcp::mcp_service())
            .layer({
                let db = mcp_db_pool.clone();
                axum::middleware::from_fn(move |req, next| {
                    crate::api_key_auth::mcp_gate_middleware(db.clone(), req, next)
                })
            });
        builder = builder.mount(mcp_router);
    }

    builder.run().await
}
