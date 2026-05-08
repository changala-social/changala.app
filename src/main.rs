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
///   CHANGALA_DATABASE_URL    →  [changala] database_url
///   CHANGALA_S3_ENDPOINT     →  [changala.s3] endpoint
///   CHANGALA_S3_BUCKET       →  [changala.s3] bucket
///   CHANGALA_S3_REGION       →  [changala.s3] region
///   CHANGALA_S3_ACCESS_KEY   →  [changala.s3] access_key
///   CHANGALA_S3_SECRET_KEY   →  [changala.s3] secret_key
///   CHANGALA_S3_PATH_STYLE   →  [changala.s3] path_style  (true/false)
///
/// Framework-level config ([app], [auth], [database]) is overridden
/// via ATRG_* env vars — see atrg-core's env_override module:
///   ATRG_APP__NAME           →  [app] name
///   ATRG_APP__HOST           →  [app] host
///   ATRG_APP__PORT           →  [app] port
///   ATRG_APP__SECRET_KEY     →  [app] secret_key
///   ATRG_APP__CORS_ORIGINS   →  [app] cors_origins  (comma-separated)
///   ATRG_APP__ENVIRONMENT    →  [app] environment
///   ATRG_AUTH__CLIENT_ID     →  [auth] client_id
///   ATRG_AUTH__REDIRECT_URI  →  [auth] redirect_uri
///   ATRG_AUTH__SCOPE         →  [auth] scope
///   ATRG_DATABASE__URL       →  [database] url
#[derive(Debug, serde::Deserialize)]
struct ChangalaConfig {
    database_url: String,
    s3: blob::S3Config,
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
        if overrides.is_empty() {
            tracing::info!("no env var overrides applied, using atrg.toml values");
        } else {
            tracing::info!(overrides = ?overrides, "applied env var overrides");
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

    // Run Changala business migrations (courses, sessions, notes, brain, etc.)
    //
    // Why a custom runner instead of `sqlx::migrate!()`?
    // ---------------------------------------------------
    // atrg-core runs its own internal SQLx migrations (atrg_sessions,
    // oauth_states) during `AtrgApp::run()`.  Both migrators share the same
    // Postgres database and the same `_sqlx_migrations` tracking table.
    // When atrg's migrator sees changala's version numbers in that table it
    // errors: "migration N was previously applied but is missing".
    //
    // sqlx 0.8 does not expose `set_migration_table_name`, so we run
    // changala's migrations ourselves against a separate tracking table
    // (`_changala_migrations`).  The migration directory lives at
    // `changala_migrations/` (not `migrations/`) so atrg's automatic
    // `run_user_migrations("./migrations")` call finds nothing to conflict
    // with.
    run_changala_migrations(&pg_pool)
        .await
        .context("Failed to run changala migrations")?;
    tracing::info!("applied changala migrations");

    // Initialize S3 blob store
    let blobs = blob::S3BlobStore::new(&config.s3).context("Failed to initialize S3 blob store")?;
    tracing::info!(endpoint = %config.s3.endpoint, bucket = %config.s3.bucket, "S3 blob store ready");

    // Initialize global Changala state
    state::init(state::Changala {
        db: pg_pool.clone(),
        blobs: Arc::new(blobs),
    });

    // Start the atrg server — shared PgPool, single database for everything.
    // with_db_pool() passes our pool to atrg so it runs its own internal
    // migrations (atrg_sessions, atrg_oauth_states) against the same Postgres.
    AtrgApp::new()
        .with_db_pool(pg_pool)
        .with_auth_routes(atrg_auth::routes::auth_router())
        .with_cleanup_task(atrg_auth::routes::spawn_cleanup_task)
        .mount(routes::api())
        .on_event(handlers::events::handle_event)
        .run()
        .await
}

/// Run changala's business-logic migrations using a private tracking table.
///
/// Migrations are embedded at compile time from `changala_migrations/` via
/// the `sqlx::migrate!()` macro.  We record applied versions in
/// `_changala_migrations` (not the default `_sqlx_migrations`) so that
/// atrg-core's internal migrator never sees changala-specific entries.
async fn run_changala_migrations(pool: &PgPool) -> anyhow::Result<()> {
    // Ensure the tracking table exists.
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS _changala_migrations (
            version  BIGINT      PRIMARY KEY,
            description TEXT     NOT NULL,
            checksum BYTEA       NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(pool)
    .await
    .context("creating _changala_migrations table")?;

    let migrator = sqlx::migrate!("./changala_migrations");

    for migration in migrator.migrations.iter() {
        let already_applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _changala_migrations WHERE version = $1)",
        )
        .bind(migration.version)
        .fetch_one(pool)
        .await?;

        if already_applied {
            continue;
        }

        sqlx::raw_sql(migration.sql.as_ref())
            .execute(pool)
            .await
            .with_context(|| {
                format!(
                    "migration {} ({})",
                    migration.version, migration.description
                )
            })?;

        sqlx::query(
            "INSERT INTO _changala_migrations (version, description, checksum) \
             VALUES ($1, $2, $3)",
        )
        .bind(migration.version)
        .bind(migration.description.as_ref())
        .bind(&*migration.checksum)
        .execute(pool)
        .await
        .with_context(|| {
            format!(
                "recording migration {} ({})",
                migration.version, migration.description
            )
        })?;

        tracing::info!(
            version = migration.version,
            name = %migration.description,
            "applied changala migration"
        );
    }

    Ok(())
}
