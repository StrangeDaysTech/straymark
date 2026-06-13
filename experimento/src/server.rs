//! The axum server: read-only HTTP API + WebSocket stream + embedded SPA
//! (Spec 001 §4), loopback-only with anti-DNS-rebinding `Host` rejection
//! (FR7/NFR4).

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use rust_embed::RustEmbed;
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::snapshot::{ApiEdge, GraphFilters, Snapshot};

/// Frontend assets built by Vite into `web/dist/` and embedded at compile
/// time (NFR3 — adopters never run npm). Empty during development; use
/// `--assets-dir web/dist` (or the Vite dev server) instead.
#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub snapshot: Arc<RwLock<Arc<Snapshot>>>,
    pub events: broadcast::Sender<String>,
    pub assets_dir: Option<PathBuf>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/graph", get(api_graph))
        .route("/api/node/{id}", get(api_node))
        .route("/api/node/{id}/thread", get(api_thread))
        .route("/api/stats", get(api_stats))
        .route("/api/stream", get(api_stream))
        .route("/healthz", get(|| async { "ok" }))
        .fallback(get(serve_asset))
        .layer(middleware::from_fn(reject_non_loopback_host))
        .with_state(state)
}

/// Anti DNS-rebinding (FR7/NFR4): even though the listener binds 127.0.0.1,
/// a hostile page can point an attacker-controlled DNS name at 127.0.0.1 and
/// reach us through the victim's browser. Reject any `Host` that is not a
/// loopback name.
async fn reject_non_loopback_host(
    headers: HeaderMap,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let host_ok = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .map(is_loopback_host)
        .unwrap_or(false);
    if !host_ok {
        return (
            StatusCode::FORBIDDEN,
            "Loom only answers loopback hosts (localhost / 127.0.0.1 / [::1])",
        )
            .into_response();
    }
    next.run(request).await
}

/// `host` is the raw `Host` header value (possibly with a port).
fn is_loopback_host(host: &str) -> bool {
    // IPv6 literal: [::1] or [::1]:port
    if let Some(rest) = host.strip_prefix('[') {
        return matches!(rest.split(']').next(), Some("::1"));
    }
    let name = host.split(':').next().unwrap_or("");
    if name.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // Any 127.0.0.0/8 address is loopback.
    name.parse::<std::net::Ipv4Addr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

fn current(state: &AppState) -> Arc<Snapshot> {
    state
        .snapshot
        .read()
        .expect("snapshot lock poisoned")
        .clone()
}

async fn api_graph(State(state): State<AppState>, Query(filters): Query<GraphFilters>) -> Response {
    let snap = current(&state);
    axum::Json(snap.api.filtered(&filters)).into_response()
}

async fn api_stats(State(state): State<AppState>) -> Response {
    axum::Json(&current(&state).api.stats).into_response()
}

async fn api_node(State(state): State<AppState>, AxumPath(id): AxumPath<String>) -> Response {
    let snap = current(&state);
    let Some(node) = snap.graph.node(&id) else {
        return (StatusCode::NOT_FOUND, "unknown document id").into_response();
    };
    // Edge ids are indices into graph.edges — recover them by pointer-free
    // comparison against the precomputed API edges (same order).
    let out_edges: Vec<&ApiEdge> = snap
        .api
        .edges
        .iter()
        .filter(|e| e.edge.source == node.id)
        .collect();
    let in_edges: Vec<&ApiEdge> = snap
        .api
        .edges
        .iter()
        .filter(|e| e.edge.resolved && e.edge.target == node.id)
        .collect();
    axum::Json(serde_json::json!({
        "node": node,
        "out_edges": out_edges,
        "in_edges": in_edges,
        "excerpt": snap.excerpts.get(&id),
        "excerpt_truncated": snap.excerpt_truncated.get(&id).copied().unwrap_or(false),
    }))
    .into_response()
}

#[derive(Deserialize)]
struct ThreadParams {
    depth: Option<usize>,
}

async fn api_thread(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
    Query(params): Query<ThreadParams>,
) -> Response {
    let snap = current(&state);
    match snap.graph.thread(&id, params.depth) {
        Some(thread) => axum::Json(thread).into_response(),
        None => (StatusCode::NOT_FOUND, "unknown document id").into_response(),
    }
}

/// WS /api/stream — pushes the full current graph on connect, then a
/// `rebuild` event on every settled filesystem change (M1: full snapshots;
/// M3 will add deltas).
async fn api_stream(State(state): State<AppState>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| stream_events(socket, state))
}

async fn stream_events(mut socket: WebSocket, state: AppState) {
    let mut rx = state.events.subscribe();
    // Initial sync so a late-joining client doesn't wait for the next change.
    let initial = current(&state).rebuild_event();
    if socket.send(Message::Text(initial.into())).await.is_err() {
        return;
    }
    loop {
        tokio::select! {
            event = rx.recv() => match event {
                Ok(msg) => {
                    if socket.send(Message::Text(msg.into())).await.is_err() {
                        return; // client gone
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(_)) => continue,    // ignore client chatter (read-only server)
                _ => return,                // closed or errored
            },
        }
    }
}

/// Serve the SPA: embedded `web/dist/` by default, `--assets-dir` override
/// for frontend development (T1.8).
async fn serve_asset(State(state): State<AppState>, uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(dir) = &state.assets_dir {
        let file = dir.join(path);
        // Confine to the assets dir (no traversal).
        if file.starts_with(dir) {
            if let Ok(body) = tokio::fs::read(&file).await {
                return with_mime(path, body);
            }
        }
        // SPA fallback to index.html for client-side routes.
        if let Ok(body) = tokio::fs::read(dir.join("index.html")).await {
            return with_mime("index.html", body);
        }
        return (StatusCode::NOT_FOUND, "asset not found").into_response();
    }

    match Assets::get(path).or_else(|| Assets::get("index.html")) {
        Some(content) => with_mime(path, content.data.into_owned()),
        None => (
            StatusCode::NOT_FOUND,
            "Loom UI assets not embedded in this build — \
             use --assets-dir web/dist after `npm run build`, \
             or a release binary",
        )
            .into_response(),
    }
}

fn with_mime(path: &str, body: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    ([(header::CONTENT_TYPE, mime.as_ref().to_string())], body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_loopback_host() {
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("localhost:7700"));
        assert!(is_loopback_host("LOCALHOST:7700"));
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.0.0.1:7700"));
        assert!(is_loopback_host("127.0.0.53:80"));
        assert!(is_loopback_host("[::1]"));
        assert!(is_loopback_host("[::1]:7700"));
        // DNS-rebinding shapes must be refused.
        assert!(!is_loopback_host("evil.example.com"));
        assert!(!is_loopback_host("evil.example.com:7700"));
        assert!(!is_loopback_host("192.168.1.10:7700"));
        assert!(!is_loopback_host("localhost.evil.com"));
        assert!(!is_loopback_host(""));
    }
}
