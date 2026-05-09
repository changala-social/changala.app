//! Aggregator application state — Postgres pool only.

use once_cell::sync::OnceCell;
use sqlx::PgPool;
use std::sync::Arc;

/// Global Aggregator state.
#[derive(Clone, Debug)]
pub struct Aggregator {
    /// PostgreSQL connection pool for materialised views.
    pub db: PgPool,
}

static INSTANCE: OnceCell<Arc<Aggregator>> = OnceCell::new();

/// Initialize the global Aggregator state. Call once at startup.
pub fn init(aggregator: Aggregator) {
    INSTANCE
        .set(Arc::new(aggregator))
        .expect("Aggregator state already initialized");
}

/// Get the global Aggregator state. Panics if not initialized.
pub fn get() -> &'static Arc<Aggregator> {
    INSTANCE
        .get()
        .expect("Aggregator state not initialized — call state::init() in main")
}
