//! Graph handlers — brain node graph traversal, backlinks, neighbours.
//!
//! These handlers power the Global View's `GraphService`:
//! - `get_node_graph` — BFS-based local subgraph around a brain node
//! - `get_backlinks` — nodes that link TO a given node (cursor-paginated)
//! - `get_neighbours` — flat list of nodes reachable within N hops

use std::collections::{HashMap, HashSet, VecDeque};

use axum::extract::{Query, State};
use axum::Json;

use atrg_core::AppState;
use atrg_xrpc::{XrpcError, XrpcErrorName};
use serde_json::json;

use changala_shared::types::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Hard ceiling on nodes returned to prevent graph explosion.
const MAX_GRAPH_NODES: usize = 100;

/// Maximum allowed traversal depth.
const MAX_DEPTH: i64 = 3;

/// Default traversal depth for get_node_graph.
const DEFAULT_DEPTH: i64 = 2;

/// Default hop count for get_neighbours.
const DEFAULT_HOPS: i64 = 1;

/// Default limit for get_neighbours.
const DEFAULT_NEIGHBOURS_LIMIT: i64 = 50;

/// Default limit for get_backlinks.
const DEFAULT_BACKLINKS_LIMIT: i64 = 50;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a JSON-encoded tags string (e.g. `["physics","math"]`) into a Vec.
/// Returns an empty vec on parse failure.
fn parse_tags(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

/// Build a PostgreSQL `IN ($1, $2, ...)` placeholder clause for `count` parameters.
///
/// Each call generates placeholders starting from `$1`. The caller is responsible
/// for binding the corresponding values in order.
fn build_in_clause(count: usize) -> String {
    (1..=count)
        .map(|i| format!("${i}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Fetch brain_node metadata for a set of URIs and return a map of uri → node JSON.
async fn fetch_node_metadata(
    uris: &HashSet<String>,
    db: &sqlx::PgPool,
) -> Result<HashMap<String, serde_json::Value>, XrpcError> {
    if uris.is_empty() {
        return Ok(HashMap::new());
    }

    let uris_vec: Vec<&str> = uris.iter().map(|s| s.as_str()).collect();
    let in_clause = build_in_clause(uris_vec.len());
    let query = format!(
        "SELECT uri, title, author_did, tags, summary FROM brain_nodes WHERE uri IN ({in_clause})"
    );

    let mut q = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(&query);
    for uri in &uris_vec {
        q = q.bind(uri);
    }
    let rows = q.fetch_all(db).await.map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to fetch node metadata: {e}"),
    })?;

    let mut map = HashMap::new();
    for (uri, title, author_did, tags_raw, summary) in rows {
        let tags = parse_tags(&tags_raw);
        map.insert(
            uri.clone(),
            json!({
                "uri": uri,
                "title": title,
                "authorDid": author_did,
                "tags": tags,
                "summary": summary,
            }),
        );
    }
    Ok(map)
}

// ═══════════════════════════════════════════════════════════════════════════
// get_node_graph
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getNodeGraph
///
/// Returns the local subgraph around a brain node, traversed via BFS up to
/// `depth` hops (default 2, max 3). Both outbound and inbound edges are
/// followed at each level. Total nodes are capped at 100.
pub async fn get_node_graph(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetNodeGraphParams>,
) -> Result<Json<AppChangalaGlobalviewGetNodeGraphOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let depth = params.depth.unwrap_or(DEFAULT_DEPTH).clamp(1, MAX_DEPTH) as usize;

    // BFS state
    let mut visited: HashSet<String> = HashSet::new();
    let mut frontier: HashSet<String> = HashSet::new();
    let mut all_edges: Vec<(String, String, Option<String>)> = Vec::new();

    // Seed with the center node
    visited.insert(params.node_uri.clone());
    frontier.insert(params.node_uri.clone());

    for _level in 0..depth {
        if frontier.is_empty() || visited.len() >= MAX_GRAPH_NODES {
            break;
        }

        let frontier_vec: Vec<&str> = frontier.iter().map(|s| s.as_str()).collect();
        let in_clause = build_in_clause(frontier_vec.len());

        // Outbound edges: current frontier → neighbours
        let outbound_query = format!(
            "SELECT from_uri, to_uri, label FROM brain_links WHERE from_uri IN ({in_clause})"
        );
        let mut outbound_q = sqlx::query_as::<_, (String, String, Option<String>)>(&outbound_query);
        for uri in &frontier_vec {
            outbound_q = outbound_q.bind(uri);
        }
        let outbound_rows = outbound_q.fetch_all(&app.db).await.map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to query outbound links: {e}"),
        })?;

        // Inbound edges: neighbours → current frontier
        let inbound_query = format!(
            "SELECT from_uri, to_uri, label FROM brain_links WHERE to_uri IN ({in_clause})"
        );
        let mut inbound_q = sqlx::query_as::<_, (String, String, Option<String>)>(&inbound_query);
        for uri in &frontier_vec {
            inbound_q = inbound_q.bind(uri);
        }
        let inbound_rows = inbound_q.fetch_all(&app.db).await.map_err(|e| XrpcError {
            name: XrpcErrorName::InternalServerError,
            message: format!("Failed to query inbound links: {e}"),
        })?;

        let mut next_frontier: HashSet<String> = HashSet::new();

        for (from_uri, to_uri, label) in outbound_rows.iter().chain(inbound_rows.iter()) {
            all_edges.push((from_uri.clone(), to_uri.clone(), label.clone()));

            // Add newly discovered nodes to the next frontier
            if !visited.contains(from_uri) {
                visited.insert(from_uri.clone());
                next_frontier.insert(from_uri.clone());
            }
            if !visited.contains(to_uri) {
                visited.insert(to_uri.clone());
                next_frontier.insert(to_uri.clone());
            }

            // Stop expanding if we've hit the node cap
            if visited.len() >= MAX_GRAPH_NODES {
                break;
            }
        }

        frontier = next_frontier;
    }

    // Fetch metadata for all discovered nodes
    let node_metadata = fetch_node_metadata(&visited, &app.db).await?;

    // Build output nodes — include a minimal entry even if the node isn't in
    // brain_nodes (it may be an external URI referenced by a link).
    let nodes: Vec<serde_json::Value> = visited
        .iter()
        .map(|uri| {
            node_metadata.get(uri).cloned().unwrap_or_else(|| {
                json!({
                    "uri": uri,
                    "title": "",
                    "authorDid": "",
                    "tags": [],
                    "summary": null,
                })
            })
        })
        .collect();

    // Deduplicate edges by (from, to)
    let mut seen_edges: HashSet<(String, String)> = HashSet::new();
    let edges: Vec<serde_json::Value> = all_edges
        .into_iter()
        .filter(|(f, t, _)| seen_edges.insert((f.clone(), t.clone())))
        .map(|(from_uri, to_uri, label)| {
            json!({
                "fromUri": from_uri,
                "toUri": to_uri,
                "label": label,
            })
        })
        .collect();

    Ok(Json(AppChangalaGlobalviewGetNodeGraphOutput {
        nodes,
        edges,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// get_backlinks
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getBacklinks
///
/// Returns all nodes that link TO the given brain node, with cursor
/// pagination on `created_at`. Each backlink includes the source node's
/// title and author DID for display without a second round-trip.
pub async fn get_backlinks(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetBacklinksParams>,
) -> Result<Json<AppChangalaGlobalviewGetBacklinksOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let limit = params
        .limit
        .unwrap_or(DEFAULT_BACKLINKS_LIMIT)
        .clamp(1, 100);

    // Build the query with optional cursor filter
    let (query, has_cursor) = if params.cursor.is_some() {
        (
            "SELECT bl.from_uri, bn.title, bn.author_did, bl.label, bl.created_at \
             FROM brain_links bl \
             JOIN brain_nodes bn ON bl.from_uri = bn.uri \
             WHERE bl.to_uri = $1 AND bl.created_at < $2 \
             ORDER BY bl.created_at DESC \
             LIMIT $3"
                .to_string(),
            true,
        )
    } else {
        (
            "SELECT bl.from_uri, bn.title, bn.author_did, bl.label, bl.created_at \
             FROM brain_links bl \
             JOIN brain_nodes bn ON bl.from_uri = bn.uri \
             WHERE bl.to_uri = $1 \
             ORDER BY bl.created_at DESC \
             LIMIT $2"
                .to_string(),
            false,
        )
    };

    let rows = if has_cursor {
        let cursor_val = params.cursor.as_deref().unwrap_or_default();
        sqlx::query_as::<_, (String, String, String, Option<String>, String)>(&query)
            .bind(&params.node_uri)
            .bind(cursor_val)
            .bind(limit)
            .fetch_all(&app.db)
            .await
    } else {
        sqlx::query_as::<_, (String, String, String, Option<String>, String)>(&query)
            .bind(&params.node_uri)
            .bind(limit)
            .fetch_all(&app.db)
            .await
    }
    .map_err(|e| XrpcError {
        name: XrpcErrorName::InternalServerError,
        message: format!("Failed to query backlinks: {e}"),
    })?;

    // Determine the next cursor from the last row's created_at
    let next_cursor = if rows.len() == limit as usize {
        rows.last()
            .map(|(_, _, _, _, created_at)| created_at.clone())
    } else {
        None
    };

    let backlinks: Vec<serde_json::Value> = rows
        .into_iter()
        .map(
            |(from_uri, from_title, from_author_did, label, created_at)| {
                json!({
                    "fromUri": from_uri,
                    "fromTitle": from_title,
                    "fromAuthorDid": from_author_did,
                    "label": label,
                    "createdAt": created_at,
                })
            },
        )
        .collect();

    Ok(Json(AppChangalaGlobalviewGetBacklinksOutput {
        backlinks,
        cursor: next_cursor,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// get_neighbours
// ═══════════════════════════════════════════════════════════════════════════

/// GET /xrpc/app.changala.globalview.getNeighbours
///
/// Returns a flat list of brain nodes reachable within N hops from the given
/// node (default 1, max 3). Each neighbour includes its hop distance from
/// the center. The center node itself is excluded from results.
pub async fn get_neighbours(
    State(state): State<AppState>,
    Query(params): Query<AppChangalaGlobalviewGetNeighboursParams>,
) -> Result<Json<AppChangalaGlobalviewGetNeighboursOutput>, XrpcError> {
    let app = state.extension::<crate::AggregatorState>();
    let hops = params.hops.unwrap_or(DEFAULT_HOPS).clamp(1, MAX_DEPTH) as usize;
    let limit = params
        .limit
        .unwrap_or(DEFAULT_NEIGHBOURS_LIMIT)
        .clamp(1, 100) as usize;

    // BFS with distance tracking
    let mut visited: HashMap<String, usize> = HashMap::new(); // uri → distance
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    visited.insert(params.node_uri.clone(), 0);
    queue.push_back((params.node_uri.clone(), 0));

    while let Some((current_uri, current_dist)) = queue.pop_front() {
        if current_dist >= hops {
            continue;
        }

        // Outbound neighbours
        let outbound_rows =
            sqlx::query_as::<_, (String,)>("SELECT to_uri FROM brain_links WHERE from_uri = $1")
                .bind(&current_uri)
                .fetch_all(&app.db)
                .await
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("Failed to query outbound neighbours: {e}"),
                })?;

        // Inbound neighbours
        let inbound_rows =
            sqlx::query_as::<_, (String,)>("SELECT from_uri FROM brain_links WHERE to_uri = $1")
                .bind(&current_uri)
                .fetch_all(&app.db)
                .await
                .map_err(|e| XrpcError {
                    name: XrpcErrorName::InternalServerError,
                    message: format!("Failed to query inbound neighbours: {e}"),
                })?;

        let next_dist = current_dist + 1;
        for (neighbour_uri,) in outbound_rows.into_iter().chain(inbound_rows) {
            if !visited.contains_key(&neighbour_uri) {
                visited.insert(neighbour_uri.clone(), next_dist);
                queue.push_back((neighbour_uri, next_dist));

                // Early exit if we've collected enough neighbours
                // (subtract 1 because the center node is in visited but excluded)
                if visited.len() > limit {
                    break;
                }
            }
        }

        if visited.len() > limit {
            break;
        }
    }

    // Remove center node from results
    visited.remove(&params.node_uri);

    // Fetch metadata for all discovered neighbour URIs
    let neighbour_uris: HashSet<String> = visited.keys().cloned().collect();
    let node_metadata = fetch_node_metadata(&neighbour_uris, &app.db).await?;

    // Build output, capped to limit
    let mut neighbours: Vec<serde_json::Value> = visited
        .iter()
        .take(limit)
        .map(|(uri, distance)| {
            if let Some(meta) = node_metadata.get(uri) {
                json!({
                    "uri": uri,
                    "title": meta["title"],
                    "authorDid": meta["authorDid"],
                    "tags": meta["tags"],
                    "distance": distance,
                    "summary": meta["summary"],
                })
            } else {
                // Node not in brain_nodes — external or deleted reference
                json!({
                    "uri": uri,
                    "title": "",
                    "authorDid": "",
                    "tags": [],
                    "distance": distance,
                    "summary": null,
                })
            }
        })
        .collect();

    // Sort by distance (closest first), then by URI for stability
    neighbours.sort_by(|a, b| {
        let da = a["distance"].as_i64().unwrap_or(0);
        let db = b["distance"].as_i64().unwrap_or(0);
        da.cmp(&db).then_with(|| {
            let ua = a["uri"].as_str().unwrap_or("");
            let ub = b["uri"].as_str().unwrap_or("");
            ua.cmp(ub)
        })
    });

    Ok(Json(AppChangalaGlobalviewGetNeighboursOutput {
        neighbours,
    }))
}
