//! Jetstream event handler — materialises firehose events into the
//! PostgreSQL database. Processes record creates/updates/deletes for all
//! app.changala.* collections.
//!
//! This module is wired into `main.rs` via `AtrgApp::on_event(handle_event)`.
//! The handler dispatches on `commit.collection` to specialised functions
//! that INSERT or UPDATE the materialised tables.

use atrg_core::AppState;
use atrg_stream::JetstreamEvent;
use sqlx::PgPool;

// ═══════════════════════════════════════════════════════════════════════════
// Top-level dispatcher
// ═══════════════════════════════════════════════════════════════════════════

/// Process a single Jetstream event.
///
/// This handler is called for every event matching the collections configured
/// in `atrg.toml` `[jetstream].collections`.
pub async fn handle_event(event: JetstreamEvent, _state: AppState) -> anyhow::Result<()> {
    let app = crate::state::get();

    let commit = match &event.commit {
        Some(c) => c,
        None => return Ok(()), // identity/account events — ignore for now
    };

    if commit.operation != "create" {
        // MVP: only handle creates. Updates and deletes deferred.
        return Ok(());
    }

    let record = match &commit.record {
        Some(r) => r,
        None => return Ok(()),
    };

    match commit.collection.as_str() {
        "app.changala.keyword" => handle_keyword(&event.did, record, &app.db).await?,
        "app.changala.vote" => handle_vote(&event.did, &commit.rkey, record, &app.db).await?,
        "app.changala.label" => handle_label(&event.did, &commit.rkey, record, &app.db).await?,
        "app.changala.brain.node" => {
            handle_brain_node(&event.did, &commit.rkey, record, &app.db).await?
        }
        "app.changala.brain.link" => {
            handle_brain_link(&event.did, &commit.rkey, record, &app.db).await?
        }
        _ => {
            tracing::debug!(collection = %commit.collection, "ignoring unhandled collection");
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Keyword
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a keyword record from the firehose.
///
/// Inserts into the `keywords` table. Duplicate (session_uri, did, text)
/// tuples are silently ignored via `ON CONFLICT DO NOTHING`.
async fn handle_keyword(did: &str, record: &serde_json::Value, db: &PgPool) -> anyhow::Result<()> {
    let session_uri = record["sessionUri"].as_str().unwrap_or_default();
    let text = record["text"].as_str().unwrap_or_default();
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if session_uri.is_empty() || text.is_empty() {
        tracing::warn!(did, "keyword event missing sessionUri or text — skipping");
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO keywords (session_uri, did, text, keyword_uri, created_at) \
         VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(session_uri)
    .bind(did)
    .bind(text)
    .bind("") // keyword_uri not available from firehose record
    .bind(created_at)
    .execute(db)
    .await?;

    tracing::info!(did, session_uri, text, "materialised keyword from firehose");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Vote
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a vote record from the firehose.
///
/// Constructs a canonical vote URI from the DID + rkey and inserts into the
/// `votes` table. Duplicates are silently ignored.
async fn handle_vote(
    did: &str,
    rkey: &str,
    record: &serde_json::Value,
    db: &PgPool,
) -> anyhow::Result<()> {
    let subject_uri = record["subjectUri"].as_str().unwrap_or_default();
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if subject_uri.is_empty() {
        tracing::warn!(did, "vote event missing subjectUri — skipping");
        return Ok(());
    }

    let vote_uri = format!("at://{did}/app.changala.vote/{rkey}");

    sqlx::query(
        "INSERT INTO votes (vote_uri, subject_uri, voter_did, created_at) \
         VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind(&vote_uri)
    .bind(subject_uri)
    .bind(did)
    .bind(created_at)
    .execute(db)
    .await?;

    tracing::info!(did, subject_uri, "materialised vote from firehose");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Label
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a label record from the firehose.
///
/// Labels carry a `neg` flag indicating whether this is an assertion (0)
/// or a negation/retraction (1). Both are stored; the Global View resolves
/// the effective state by checking for negation records.
async fn handle_label(
    did: &str,
    rkey: &str,
    record: &serde_json::Value,
    db: &PgPool,
) -> anyhow::Result<()> {
    let subject_uri = record["subjectUri"].as_str().unwrap_or_default();
    let val = record["val"].as_str().unwrap_or_default();
    let neg = record["neg"].as_i64().unwrap_or(0);
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if subject_uri.is_empty() || val.is_empty() {
        tracing::warn!(did, "label event missing subjectUri or val — skipping");
        return Ok(());
    }

    let label_uri = format!("at://{did}/app.changala.label/{rkey}");

    sqlx::query(
        "INSERT INTO labels (label_uri, subject_uri, val, src_did, neg, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&label_uri)
    .bind(subject_uri)
    .bind(val)
    .bind(did)
    .bind(neg)
    .bind(created_at)
    .execute(db)
    .await?;

    tracing::info!(
        did,
        subject_uri,
        val,
        neg,
        "materialised label from firehose"
    );
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Brain Node
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a brain node record from the firehose.
///
/// Brain nodes carry their content pointer (ringRef), metadata (title,
/// format, tags), and optional academic cross-references. The node URI is
/// constructed from the author DID + rkey.
///
/// Uses `INSERT ... ON CONFLICT (uri) DO UPDATE` so that versioned updates
/// (same URI, new version) overwrite the previous row.
async fn handle_brain_node(
    did: &str,
    rkey: &str,
    record: &serde_json::Value,
    db: &PgPool,
) -> anyhow::Result<()> {
    let title = record["title"].as_str().unwrap_or_default();
    let format = record["format"].as_str().unwrap_or("markdown");
    let version = record["version"].as_i64().unwrap_or(1);
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    // Ring reference — nested object with `ringDid` and `cid`
    let ring_did = record
        .get("ringRef")
        .and_then(|r| r.get("ringDid"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let cid = record
        .get("ringRef")
        .and_then(|r| r.get("cid"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    // Optional fields
    let summary = record.get("summary").and_then(|v| v.as_str());
    let academic_ref = record.get("academicRef").and_then(|v| v.as_str());

    // Tags — stored as JSON array string in the record, serialise back for storage
    let tags_json = match record.get("tags") {
        Some(v) if v.is_array() => v.to_string(),
        _ => "[]".to_string(),
    };

    let uri = format!("at://{did}/app.changala.brain.node/{rkey}");

    sqlx::query(
        "INSERT INTO brain_nodes \
         (uri, author_did, title, format, ring_did, cid, tags, academic_ref, version, summary, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
         ON CONFLICT (uri) DO UPDATE SET \
           title = EXCLUDED.title, format = EXCLUDED.format, ring_did = EXCLUDED.ring_did, \
           cid = EXCLUDED.cid, tags = EXCLUDED.tags, academic_ref = EXCLUDED.academic_ref, \
           version = EXCLUDED.version, summary = EXCLUDED.summary, created_at = EXCLUDED.created_at",
    )
    .bind(&uri)
    .bind(did)
    .bind(title)
    .bind(format)
    .bind(ring_did)
    .bind(cid)
    .bind(&tags_json)
    .bind(academic_ref)
    .bind(version)
    .bind(summary)
    .bind(created_at)
    .execute(db)
    .await?;

    tracing::info!(did, uri = %uri, title, "materialised brain node from firehose");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Brain Link
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a brain link record from the firehose.
///
/// Brain links are directed edges in the knowledge graph. The Global View
/// uses these to power backlink queries and graph traversal. Duplicate
/// (from_uri, to_uri) edges are silently ignored.
async fn handle_brain_link(
    did: &str,
    rkey: &str,
    record: &serde_json::Value,
    db: &PgPool,
) -> anyhow::Result<()> {
    let from_uri = record["fromUri"].as_str().unwrap_or_default();
    let to_uri = record["toUri"].as_str().unwrap_or_default();
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if from_uri.is_empty() || to_uri.is_empty() {
        tracing::warn!(did, "brain link event missing fromUri or toUri — skipping");
        return Ok(());
    }

    // Optional edge label (e.g. "see also", "contradicts", "expands")
    let label = record.get("label").and_then(|v| v.as_str());

    let link_uri = format!("at://{did}/app.changala.brain.link/{rkey}");

    sqlx::query(
        "INSERT INTO brain_links \
         (link_uri, from_uri, to_uri, label, created_by, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
    )
    .bind(&link_uri)
    .bind(from_uri)
    .bind(to_uri)
    .bind(label)
    .bind(did)
    .bind(created_at)
    .execute(db)
    .await?;

    tracing::info!(
        did,
        from_uri,
        to_uri,
        "materialised brain link from firehose"
    );
    Ok(())
}
