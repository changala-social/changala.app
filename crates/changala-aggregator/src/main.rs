use anyhow::Context;
use atrg_core::AtrgApp;
use sqlx::postgres::PgPool;
use std::path::Path;

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
    atrg_db::run_isolated_migrations(
        &atrg_db::DbPool::Postgres(pg_pool.clone()),
        Path::new("./aggregator_migrations"),
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
