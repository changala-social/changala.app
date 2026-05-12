//! Jetstream event router — materialises firehose events into the
//! PostgreSQL database using atrg 0.2.0's EventRouterBuilder.
//!
//! This module builds a typed event dispatcher that routes events by
//! collection to specialised handler functions. Wired into main.rs via
//! `AtrgApp::on_event(events::event_router())`.

use atrg_core::AppState;
use atrg_stream::router::CommitEvent;
use atrg_stream::EventRouterBuilder;

// ═══════════════════════════════════════════════════════════════════════════
// Router builder
// ═══════════════════════════════════════════════════════════════════════════

/// Build the Jetstream event router.
///
/// Registers handlers for each `app.changala.*` collection the aggregator
/// cares about. Returns a closure compatible with `AtrgApp::on_event()`.
pub fn event_router() -> impl Fn(
    atrg_stream::JetstreamEvent,
    AppState,
) -> futures::future::BoxFuture<'static, anyhow::Result<()>>
       + Send
       + Sync {
    EventRouterBuilder::<AppState>::new()
        .on_create("app.changala.keyword", handle_keyword)
        .on_create("app.changala.vote", handle_vote)
        .on_create("app.changala.label", handle_label)
        .on_create("app.changala.brain.node", handle_brain_node)
        .on_create("app.changala.brain.link", handle_brain_link)
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════
// Keyword
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a keyword record from the firehose.
///
/// Inserts into the `keywords` table. Duplicate (session_uri, did, text)
/// tuples are silently ignored via `ON CONFLICT DO NOTHING`.
async fn handle_keyword(event: CommitEvent, state: AppState) -> anyhow::Result<()> {
    let app = state.extension::<crate::AggregatorState>();
    let record = match &event.record {
        Some(r) => r,
        None => return Ok(()),
    };

    let session_uri = record["sessionUri"].as_str().unwrap_or_default();
    let text = record["text"].as_str().unwrap_or_default();
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if session_uri.is_empty() || text.is_empty() {
        tracing::warn!(did = %event.did, "keyword event missing sessionUri or text — skipping");
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO keywords (session_uri, did, text, keyword_uri, created_at) \
         VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(session_uri)
    .bind(&event.did)
    .bind(text)
    .bind("") // keyword_uri not available from firehose record
    .bind(created_at)
    .execute(&app.db)
    .await?;

    tracing::info!(did = %event.did, session_uri, text, "materialised keyword from firehose");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Vote
// ═══════════════════════════════════════════════════════════════════════════

/// Materialise a vote record from the firehose.
///
/// Constructs a canonical vote URI from the DID + rkey and inserts into the
/// `votes` table. Duplicates are silently ignored.
async fn handle_vote(event: CommitEvent, state: AppState) -> anyhow::Result<()> {
    let app = state.extension::<crate::AggregatorState>();
    let record = match &event.record {
        Some(r) => r,
        None => return Ok(()),
    };

    let subject_uri = record["subjectUri"].as_str().unwrap_or_default();
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if subject_uri.is_empty() {
        tracing::warn!(did = %event.did, "vote event missing subjectUri — skipping");
        return Ok(());
    }

    let vote_uri = format!("at://{}/app.changala.vote/{}", event.did, event.rkey);

    sqlx::query(
        "INSERT INTO votes (vote_uri, subject_uri, voter_did, created_at) \
         VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind(&vote_uri)
    .bind(subject_uri)
    .bind(&event.did)
    .bind(created_at)
    .execute(&app.db)
    .await?;

    tracing::info!(did = %event.did, subject_uri, "materialised vote from firehose");
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
async fn handle_label(event: CommitEvent, state: AppState) -> anyhow::Result<()> {
    let app = state.extension::<crate::AggregatorState>();
    let record = match &event.record {
        Some(r) => r,
        None => return Ok(()),
    };

    let subject_uri = record["subjectUri"].as_str().unwrap_or_default();
    let val = record["val"].as_str().unwrap_or_default();
    let neg = record["neg"].as_i64().unwrap_or(0);
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if subject_uri.is_empty() || val.is_empty() {
        tracing::warn!(did = %event.did, "label event missing subjectUri or val — skipping");
        return Ok(());
    }

    let label_uri = format!("at://{}/app.changala.label/{}", event.did, event.rkey);

    sqlx::query(
        "INSERT INTO labels (label_uri, subject_uri, val, src_did, neg, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&label_uri)
    .bind(subject_uri)
    .bind(val)
    .bind(&event.did)
    .bind(neg)
    .bind(created_at)
    .execute(&app.db)
    .await?;

    tracing::info!(did = %event.did, subject_uri, val, neg, "materialised label from firehose");
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
async fn handle_brain_node(event: CommitEvent, state: AppState) -> anyhow::Result<()> {
    let app = state.extension::<crate::AggregatorState>();
    let record = match &event.record {
        Some(r) => r,
        None => return Ok(()),
    };

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

    let uri = format!("at://{}/app.changala.brain.node/{}", event.did, event.rkey);

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
    .bind(&event.did)
    .bind(title)
    .bind(format)
    .bind(ring_did)
    .bind(cid)
    .bind(&tags_json)
    .bind(academic_ref)
    .bind(version)
    .bind(summary)
    .bind(created_at)
    .execute(&app.db)
    .await?;

    tracing::info!(did = %event.did, uri = %uri, title, "materialised brain node from firehose");
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
async fn handle_brain_link(event: CommitEvent, state: AppState) -> anyhow::Result<()> {
    let app = state.extension::<crate::AggregatorState>();
    let record = match &event.record {
        Some(r) => r,
        None => return Ok(()),
    };

    let from_uri = record["fromUri"].as_str().unwrap_or_default();
    let to_uri = record["toUri"].as_str().unwrap_or_default();
    let created_at = record["createdAt"].as_str().unwrap_or_default();

    if from_uri.is_empty() || to_uri.is_empty() {
        tracing::warn!(did = %event.did, "brain link event missing fromUri or toUri — skipping");
        return Ok(());
    }

    // Optional edge label (e.g. "see also", "contradicts", "expands")
    let label = record.get("label").and_then(|v| v.as_str());

    let link_uri = format!("at://{}/app.changala.brain.link/{}", event.did, event.rkey);

    sqlx::query(
        "INSERT INTO brain_links \
         (link_uri, from_uri, to_uri, label, created_by, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
    )
    .bind(&link_uri)
    .bind(from_uri)
    .bind(to_uri)
    .bind(label)
    .bind(&event.did)
    .bind(created_at)
    .execute(&app.db)
    .await?;

    tracing::info!(did = %event.did, from_uri, to_uri, "materialised brain link from firehose");
    Ok(())
}
