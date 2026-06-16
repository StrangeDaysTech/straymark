//! Filesystem watcher (Spec 001 §5 + Spec 002 FR6): `notify` with a ~250ms
//! debounce to coalesce editor save storms. On a settled change it rebuilds the
//! knowledge-graph snapshot and/or recomputes the architecture overlay, then
//! broadcasts to every `/api/stream` client:
//! - a graph `rebuild`/`delta` event when `.md` documents change (Spec 001);
//! - an `architecture` signal event when governance docs, `model.yml`, or
//!   `plan.drawio` change the projected overlay (Spec 002 A2.2).
//!
//! Both paths suppress no-ops (a file's mtime moved but the rendered result did
//! not) so open clients aren't churned for nothing.

use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Result;
use notify_debouncer_full::notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
use tokio::sync::broadcast;

use crate::architecture;
use crate::snapshot::{ParseCache, Snapshot};

pub const DEBOUNCE: Duration = Duration::from_millis(250);

/// The architecture overlay-changed signal. The client refetches
/// `/api/architecture` (+ `/api/where`) and re-applies the overlay; a tiny
/// signal (not the payload) keeps the watcher decoupled from the response shape.
const ARCH_EVENT: &str = r#"{"event":"architecture"}"#;

/// Spawns the watcher. Returns the debouncer handle — drop it and watching
/// stops, so the caller keeps it alive. `cache` is the warm parse cache from the
/// initial build (T3.1). `project_root`/`arch_dir` let the watcher recompute the
/// architecture overlay (Spec 002 §4) with the same inputs the API endpoints use.
pub fn watch(
    watch_dir: &Path,
    project_root: &Path,
    arch_dir: &Path,
    snapshot: Arc<RwLock<Arc<Snapshot>>>,
    events: broadcast::Sender<String>,
    mut cache: ParseCache,
) -> Result<impl Sized> {
    // Owned clones for the closure; the `&Path` params stay live for the
    // `watch()` calls below.
    let dir = watch_dir.to_path_buf();
    let proj = project_root.to_path_buf();
    let arch = arch_dir.to_path_buf();
    // Seed the no-op baseline with the initial overlay so the first real change
    // is compared against the served state, not a spurious empty.
    let mut last_arch_sig = architecture::projection_signature(&proj, &arch);

    let mut debouncer = new_debouncer(
        DEBOUNCE,
        None,
        move |result: DebounceEventResult| {
            let Ok(batch) = result else { return };
            let touched_md = batch.iter().any(|e| {
                e.paths
                    .iter()
                    .any(|p| p.extension().and_then(|x| x.to_str()) == Some("md"))
            });
            let touched_arch = batch
                .iter()
                .any(|e| e.paths.iter().any(|p| architecture::is_architecture_relevant(p)));
            if !touched_md && !touched_arch {
                return; // editor lockfiles, swap files, irrelevant yaml, etc.
            }

            // ── Knowledge graph (Spec 001): only `.md` churn rebuilds it ──
            if touched_md {
                match Snapshot::build_cached(&dir, &mut cache) {
                    Ok(next) => {
                        let next = Arc::new(next);
                        let prev_api =
                            snapshot.read().expect("snapshot lock poisoned").api.clone();
                        let delta = next.api.diff(&prev_api);
                        *snapshot.write().expect("snapshot lock poisoned") = next;
                        if !delta.is_noop(&prev_api) {
                            let _ = events.send(delta.to_event());
                        }
                    }
                    Err(err) => {
                        eprintln!("loom: rebuild failed (will retry on next change): {err:#}");
                    }
                }
            }

            // ── Architecture overlay (Spec 002 A2.2): governance / model / plan ──
            if touched_arch {
                let sig = architecture::projection_signature(&proj, &arch);
                if sig != last_arch_sig {
                    last_arch_sig = sig;
                    let _ = events.send(ARCH_EVENT.to_string());
                }
            }
        },
    )?;

    debouncer.watch(watch_dir, RecursiveMode::Recursive)?;
    // The model/plan may live outside the watch dir (e.g. `--arch-dir` pointing
    // at a dogfood tree). Watch it too so model/plan edits update the overlay.
    if !arch_dir.starts_with(watch_dir) && arch_dir.is_dir() {
        debouncer.watch(arch_dir, RecursiveMode::Recursive)?;
    }
    Ok(debouncer)
}
