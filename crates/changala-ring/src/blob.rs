//! S3-compatible blob store for content-addressed storage.
//!
//! Stores blobs keyed by their SHA-256 hash (content-addressed).
//! Works with any S3-compatible service (RustFS, MinIO, AWS S3).

use anyhow::Context;
use sha2::{Digest, Sha256};

/// S3-compatible blob store.
pub struct S3BlobStore {
    bucket: s3::Bucket,
}

impl std::fmt::Debug for S3BlobStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3BlobStore").finish()
    }
}

/// S3 configuration.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default = "default_true")]
    pub path_style: bool,
}

fn default_true() -> bool {
    true
}

impl S3BlobStore {
    /// Create a new S3 blob store from config.
    pub fn new(config: &S3Config) -> anyhow::Result<Self> {
        let region = s3::Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.clone(),
        };
        let credentials = s3::creds::Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )
        .context("Failed to create S3 credentials")?;

        let mut bucket = *s3::Bucket::new(&config.bucket, region, credentials)
            .context("Failed to create S3 bucket handle")?;

        if config.path_style {
            bucket = *bucket.with_path_style();
        }

        Ok(Self { bucket })
    }

    /// Store a blob and return its content-addressed key (CID).
    /// If the blob already exists (same content), this is a no-op due to content addressing.
    pub async fn put(&self, data: &[u8]) -> anyhow::Result<String> {
        let cid = compute_cid(data);
        self.bucket
            .put_object(&cid, data)
            .await
            .context("S3 put_object failed")?;
        Ok(cid)
    }

    /// Retrieve a blob by its CID.
    pub async fn get(&self, cid: &str) -> anyhow::Result<Vec<u8>> {
        let response = self
            .bucket
            .get_object(cid)
            .await
            .context("S3 get_object failed")?;
        Ok(response.bytes().to_vec())
    }

    /// Check if a blob exists.
    #[allow(dead_code)]
    pub async fn exists(&self, cid: &str) -> anyhow::Result<bool> {
        match self.bucket.head_object(cid).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Delete a blob by its CID.
    #[allow(dead_code)]
    pub async fn delete(&self, cid: &str) -> anyhow::Result<()> {
        self.bucket
            .delete_object(cid)
            .await
            .context("S3 delete_object failed")?;
        Ok(())
    }
}

/// Compute a content-addressed identifier from data bytes.
/// Uses SHA-256, hex-encoded with a `sha256-` prefix.
pub fn compute_cid(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("sha256-{}", hex::encode(hash))
}
