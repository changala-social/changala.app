use anyhow::Context;
use atrg_core::AtrgApp;
use sqlx::postgres::PgPool;
use std::path::{Path, PathBuf};

mod handlers;
mod routes;

/// Aggregator application state — registered as an AppState extension.
///
/// Replaces the old `once_cell` singleton. Access in handlers via:
///   `let app = state.extension::<AggregatorState>();`
#[derive(Clone, Debug)]
pub struct AggregatorState {
    /// PostgreSQL connection pool for materialised views.
    pub db: PgPool,
}

/// Aggregator config from atrg.toml [changala] section.
#[derive(Debug, serde::Deserialize)]
struct AggregatorConfig {
    database_url: String,
}

impl AggregatorConfig {
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("CHANGALA_DATABASE_URL") {
            self.database_url = v;
            tracing::info!("applied CHANGALA_DATABASE_URL override");
        }
    }
}

/// Locate the directory containing the SQL migration files.
///
/// `atrg_db::run_isolated_migrations` takes a filesystem path. A bare
/// `./aggregator_migrations` only resolves when the process working directory
/// happens to contain it (the container image sets `WORKDIR /app` and copies
/// the migrations there). For `cargo run` from the workspace root — and any
/// deployment whose working directory differs — that path does not exist,
/// producing `migrations directory does not exist: ./aggregator_migrations`.
///
/// To be robust regardless of working directory, probe a list of candidates
/// and return the first that exists:
///   1. `$env_var` override (k8s / custom deployments),
///   2. `./<dir_name>` (container `WORKDIR`, or running from the crate dir),
///   3. `<exe_dir>/<dir_name>` (migrations shipped next to the binary),
///   4. `<CARGO_MANIFEST_DIR>/<dir_name>` (`cargo run` from anywhere).
fn resolve_migrations_dir(
    env_var: &str,
    dir_name: &str,
    manifest_dir: &str,
) -> anyhow::Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(p) = std::env::var(env_var) {
        if !p.trim().is_empty() {
            candidates.push(PathBuf::from(p));
        }
    }
    candidates.push(PathBuf::from(format!("./{dir_name}")));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(dir_name));
        }
    }
    candidates.push(Path::new(manifest_dir).join(dir_name));

    for candidate in &candidates {
        if candidate.is_dir() {
            tracing::info!(path = %candidate.display(), "resolved migrations directory");
            return Ok(candidate.clone());
        }
    }

    let tried = candidates
        .iter()
        .map(|c| c.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    anyhow::bail!(
        "could not locate the '{dir_name}' migrations directory (tried: {tried}). \
         Set {env_var} to point at it."
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // atrg 0.2.0: use load_app_config() instead of manual TOML parsing
    let mut config: AggregatorConfig = atrg_core::config::load_app_config("changala")
        .context("Failed to load [changala] config from atrg.toml")?;

    config.apply_env_overrides();

    let pg_pool = PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;
    tracing::info!(url = %config.database_url, "connected to PostgreSQL");

    // atrg 0.2.0: run_isolated_migrations() replaces the hand-rolled runner.
    // Uses "_aggregator_migrations" tracking table to avoid conflicts.
    let migrations_dir = resolve_migrations_dir(
        "CHANGALA_AGGREGATOR_MIGRATIONS_DIR",
        "aggregator_migrations",
        env!("CARGO_MANIFEST_DIR"),
    )?;
    atrg_db::run_isolated_migrations(
        &atrg_db::DbPool::Postgres(pg_pool.clone()),
        &migrations_dir,
        "_aggregator_migrations",
    )
    .await
    .context("Failed to run aggregator migrations")?;
    tracing::info!("applied aggregator migrations");

    // atrg 0.2.0: register app state as extension (replaces once_cell singleton)
    let aggregator_state = AggregatorState {
        db: pg_pool.clone(),
    };

    AtrgApp::new()
        .with_db_pool(pg_pool)
        .with_extension(aggregator_state)
        .mount(routes::api())
        .on_event(handlers::events::event_router())
        .run()
        .await
}
