//! Filesystem watcher (Spec 001 §5): `notify` with a ~250ms debounce to
//! coalesce editor save storms; on a settled change, rebuild the snapshot
//! and broadcast a `rebuild` event to every `/api/stream` client.

use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Result;
use notify_debouncer_full::notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use tokio::sync::broadcast;

use crate::snapshot::{ParseCache, Snapshot};

pub const DEBOUNCE: Duration = Duration::from_millis(250);

/// Spawns the watcher. Returns the debouncer handle — drop it and the
/// watching stops, so the caller must keep it alive. `cache` is the warm
/// parse cache from the initial build, threaded in so the first change is
/// already incremental (T3.1).
pub fn watch(
    watch_dir: &Path,
    snapshot: Arc<RwLock<Arc<Snapshot>>>,
    events: broadcast::Sender<String>,
    mut cache: ParseCache,
) -> Result<impl Sized> {
    let dir = watch_dir.to_path_buf();
    let mut debouncer = new_debouncer(
        DEBOUNCE,
        None,
        move |result: DebounceEventResult| {
            let Ok(batch) = result else { return };
            // Only .md churn matters; ignore editor lockfiles, swap files, etc.
            let relevant = batch.iter().any(|e| {
                e.paths
                    .iter()
                    .any(|p| p.extension().and_then(|x| x.to_str()) == Some("md"))
            });
            if !relevant {
                return;
            }
            match Snapshot::build_cached(&dir, &mut cache) {
                Ok(next) => {
                    let next = Arc::new(next);
                    // Diff against the previous snapshot so clients patch
                    // (preserve layout) instead of re-laying out the graph.
                    let prev_api = snapshot.read().expect("snapshot lock poisoned").api.clone();
                    let event = next.delta_event(&prev_api);
                    *snapshot.write().expect("snapshot lock poisoned") = next;
                    // No receivers (no open browser) is fine — ignore the error.
                    let _ = events.send(event);
                }
                Err(err) => {
                    eprintln!("loom: rebuild failed (will retry on next change): {err:#}");
                }
            }
        },
    )?;

    debouncer.watch(watch_dir, RecursiveMode::Recursive)?;
    Ok(debouncer)
}
