//! straymark-loom — Loom, StrayMark's EXPERIMENTAL knowledge-graph
//! visualization server (Spec 001, `CHARTER-01-loom-server`).
//!
//! Loopback-only, read-only: binds 127.0.0.1 exclusively, rejects
//! non-loopback `Host` headers, and never writes to the watched directory.

mod architecture;
mod server;
mod snapshot;
mod watcher;

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use tokio::sync::broadcast;

use server::AppState;
use snapshot::{ParseCache, Snapshot};

#[derive(Parser)]
#[command(
    name = "straymark-loom",
    version,
    about = "Loom — StrayMark's EXPERIMENTAL knowledge-graph visualization server"
)]
struct Args {
    /// Project directory (defaults to the current directory). Loom watches
    /// its `.straymark/` subdirectory if present, otherwise the directory
    /// itself.
    #[arg(default_value = ".")]
    path: String,

    /// Port on 127.0.0.1 to serve on.
    #[arg(long, default_value_t = 7700)]
    port: u16,

    /// Serve frontend assets from this directory instead of the embedded
    /// bundle (frontend development).
    #[arg(long)]
    assets_dir: Option<PathBuf>,

    /// Directory holding the architecture `model.yml` + `plan.drawio`. Defaults
    /// to `<path>/.straymark/architecture/` (or `<path>/architecture/`). Override
    /// when the model lives elsewhere — e.g. this repo dogfoods on
    /// `experimento/architecture/` while globs resolve against the repo root.
    #[arg(long)]
    arch_dir: Option<PathBuf>,
}

/// Returns `(project_root, watch_dir)`. The watch dir is `.straymark/` when the
/// project has one, else the directory itself (this source repo dogfoods on
/// `docs/` & co. without a root install). The project root is the directory
/// that may hold `.straymark/config.yml` (for the UI language).
fn resolve_paths(path: &str) -> Result<(PathBuf, PathBuf)> {
    let base = PathBuf::from(path)
        .canonicalize()
        .with_context(|| format!("cannot resolve project path {path}"))?;
    let straymark = base.join(".straymark");
    let watch_dir = if straymark.is_dir() {
        straymark
    } else {
        base.clone()
    };
    Ok((base, watch_dir))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let (project_root, watch_dir) = resolve_paths(&args.path)?;
    let arch_dir = match args.arch_dir {
        Some(d) => d.canonicalize().unwrap_or(d),
        None => architecture::default_arch_dir(&project_root),
    };

    // The project's configured UI language, resolved by the shared core logic
    // (same as `straymark explore`) so Loom's UI matches the CLI (T3.5/NFR5).
    let locale = straymark_core::config::resolve_language(&project_root);

    // Warm parse cache: the initial build seeds it so the first edit is
    // already incremental (T3.1).
    let mut cache = ParseCache::new();
    let initial = Arc::new(
        Snapshot::build_cached(&watch_dir, &mut cache)
            .with_context(|| format!("initial graph build failed for {}", watch_dir.display()))?,
    );
    let doc_count = initial.api.stats.total_docs;
    let edge_count = initial.api.stats.total_edges;

    let snapshot = Arc::new(RwLock::new(initial));
    let (events, _) = broadcast::channel::<String>(16);

    // Keep the handle alive for the lifetime of the server (drop = unwatch).
    let _debouncer = watcher::watch(&watch_dir, snapshot.clone(), events.clone(), cache)
        .context("failed to start the filesystem watcher")?;

    let state = AppState {
        snapshot,
        events,
        assets_dir: args.assets_dir,
        locale,
        project_root,
        arch_dir,
    };

    // FR7: loopback bind only. If 127.0.0.1 cannot be bound, refuse to start —
    // never fall back to a wider interface.
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, args.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("cannot bind {addr} (is another Loom running?)"))?;

    eprintln!(
        "{}",
        "⚠ Loom is EXPERIMENTAL (v0) — unstable, loopback-only, read-only".yellow().bold()
    );
    eprintln!(
        "loom: watching {} ({} docs, {} links)",
        watch_dir.display(),
        doc_count,
        edge_count
    );
    eprintln!("loom: serving http://{addr}");

    axum::serve(listener, server::router(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("\nloom: shutting down");
        })
        .await?;
    Ok(())
}
