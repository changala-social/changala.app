use anyhow::Context;
use atrg_core::AtrgApp;
use sqlx::postgres::PgPool;

mod handlers;
mod routes;
mod state;

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
    let toml_str = std::fs::read_to_string("atrg.toml").context("Failed to read atrg.toml")?;
    let toml_val: toml::Value = toml::from_str(&toml_str).context("Failed to parse atrg.toml")?;
    let changala_section = toml_val
        .get("changala")
        .context("Missing [changala] section in atrg.toml")?;
    let mut config: AggregatorConfig = changala_section
        .clone()
        .try_into()
        .context("Invalid [changala] config")?;

    config.apply_env_overrides();

    let pg_pool = PgPool::connect(&config.database_url)
        .await
        .context("Failed to connect to PostgreSQL")?;
    tracing::info!(url = %config.database_url, "connected to PostgreSQL");

    // Run aggregator migrations
    run_aggregator_migrations(&pg_pool)
        .await
        .context("Failed to run aggregator migrations")?;
    tracing::info!("applied aggregator migrations");

    state::init(state::Aggregator {
        db: pg_pool.clone(),
    });

    AtrgApp::new()
        .with_db_pool(pg_pool)
        .mount(routes::api())
        .on_event(handlers::events::handle_event)
        .run()
        .await
}

async fn run_aggregator_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS _changala_aggregator_migrations (
            version  BIGINT      PRIMARY KEY,
            description TEXT     NOT NULL,
            checksum BYTEA       NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )",
    )
    .execute(pool)
    .await
    .context("creating migration tracking table")?;

    let migrator = sqlx::migrate!("./aggregator_migrations");

    for migration in migrator.migrations.iter() {
        let already_applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _changala_aggregator_migrations WHERE version = $1)",
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
            "INSERT INTO _changala_aggregator_migrations (version, description, checksum) VALUES ($1, $2, $3)",
        )
        .bind(migration.version)
        .bind(migration.description.as_ref())
        .bind(&*migration.checksum)
        .execute(pool)
        .await?;

        tracing::info!(version = migration.version, name = %migration.description, "applied aggregator migration");
    }

    Ok(())
}
