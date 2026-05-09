//! Changala application state — Postgres pool + S3 blob store.
//!
//! Uses `once_cell::sync::OnceCell` for global access from handlers
//! and the Jetstream event processor without needing Axum state extractors.

use once_cell::sync::OnceCell;
use sqlx::PgPool;
use std::sync::Arc;

use crate::blob::S3BlobStore;
use crate::email::SmtpConfig;

/// Global Changala application state.
#[derive(Clone, Debug)]
pub struct Changala {
    /// PostgreSQL connection pool for all business data.
    pub db: PgPool,
    /// S3-compatible blob store for content (notes, brain nodes, archives).
    pub blobs: Arc<S3BlobStore>,
    /// Allowed institution email domains for membership verification.
    pub allowed_email_domains: Vec<String>,
    /// Optional SMTP config — None = dev mode (log OTPs to stdout).
    pub smtp: Option<SmtpConfig>,
    /// DIDs to auto-provision as admin on startup (consumed in main, not read via state).
    #[allow(dead_code)]
    pub admin_dids: Vec<String>,
}

static INSTANCE: OnceCell<Arc<Changala>> = OnceCell::new();

/// Initialize the global Changala state. Call once at startup.
pub fn init(changala: Changala) {
    INSTANCE
        .set(Arc::new(changala))
        .expect("Changala state already initialized");
}

/// Get the global Changala state. Panics if not initialized.
pub fn get() -> &'static Arc<Changala> {
    INSTANCE
        .get()
        .expect("Changala state not initialized — call state::init() in main")
}
