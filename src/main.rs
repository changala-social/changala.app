use anyhow::Context;
use atrg_core::AtrgApp;
use sqlx::postgres::PgPool;
use std::sync::Arc;

mod blob;
#[allow(dead_code)]
mod generated;
mod handlers;
mod routes;
mod state;

/// Changala-specific config loaded from atrg.toml [changala] section.
///
/// Every field can be overridden by an environment variable:
///   CHANGALA_DATABASE_URL  →  [changala] database_url
///   CHANGALA_S3_ENDPOINT   →  [changala.s3] endpoint
///   CHANGALA_S3_BUCKET     →  [changala.s3] bucket
///   CHANGALA_S3_REGION     →  [changala.s3] region
///   CHANGALA_S3_ACCESS_KEY →  [changala.s3] access_key
///   CHANGALA_S3_SECRET_KEY →  [changala.s3] secret_key
#[derive(Debug, serde::Deserialize)]
struct ChangalaConfig {
    database_url: String,
    s3: blob::S3Config,
}

impl ChangalaConfig {
    /// Apply environment variable overrides. Env vars take precedence.
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("CHANGALA_DATABASE_URL") {
            self.database_url = v;
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_ENDPOINT") {
            self.s3.endpoint = v;
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_BUCKET") {
            self.s3.bucket = v;
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_REGION") {
            self.s3.region = v;
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_ACCESS_KEY") {
            self.s3.access_key = v;
        }
        if let Ok(v) = std::env::var("CHANGALA_S3_SECRET_KEY") {
            self.s3.secret_key = v;
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load changala-specific config from atrg.toml
    let toml_str = std::fs::read_to_string("atrg.toml").context("Failed to read atrg.toml")?;
    let toml_val: toml::Value = toml::from_str(&toml_str).context("Failed to parse atrg.toml")?;
    let changala_section = toml_val
        .get("changala")
        .context("Missing [changala] section in atrg.toml")?;
    let mut config: ChangalaConfig = changala_section
        .clone()
        .try_into()
        .context("Invalid [changala] config")?;

    // Env vars override atrg.toml — for k8s Secrets, docker .env, etc.
    config.apply_env_overrides();

    // Connect to PostgreSQL
    let pg_pool = PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;
    tracing::info!(url = %config.database_url, "connected to PostgreSQL");

    // Run all migrations — business tables + atrg internals
    sqlx::migrate!("./pg_migrations")
        .run(&pg_pool)
        .await
        .context("Failed to run migrations")?;
    tracing::info!("applied all migrations");

    // Initialize S3 blob store
    let blobs = blob::S3BlobStore::new(&config.s3).context("Failed to initialize S3 blob store")?;
    tracing::info!(endpoint = %config.s3.endpoint, bucket = %config.s3.bucket, "S3 blob store ready");

    // Initialize global Changala state
    state::init(state::Changala {
        db: pg_pool,
        blobs: Arc::new(blobs),
    });

    // Start the atrg server — single Postgres DB for everything
    AtrgApp::new()
        .with_auth_routes(atrg_auth::routes::auth_router())
        .with_cleanup_task(atrg_auth::routes::spawn_cleanup_task)
        .mount(routes::api())
        .on_event(handlers::events::handle_event)
        .run()
        .await
}
