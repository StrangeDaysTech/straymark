//! `straymark charter` — Charter lifecycle subcommands.
//!
//! Phase 1 shipped `new` (scaffold from template), `list` (enumerate with filters),
//! and `status` (detail view).
//! Phase 2 adds `close` (interactive telemetry) and `drift` (file-vs-commit
//! drift check, in PR 3).
//! Phase 3 will add `audit` (multi-model external audit).

pub mod amend;
pub mod audit;
pub mod batch_complete;
pub mod close;
pub mod drift;
pub mod list;
pub mod new;
pub mod refresh_suggest;
pub mod status;

use std::path::{Path, PathBuf};

/// Canonical name of a Charter's telemetry sidecar: `CHARTER-NN.telemetry.yaml`.
///
/// Derived from `charter_id` with any slug suffix stripped, so the filename
/// survives a Charter rename (`CHARTER-01-foo` → `CHARTER-01.telemetry.yaml`).
///
/// **This is the single definition on purpose** (GH #416). `close` built this
/// name while `refresh-suggest` built `<NN-slug>.telemetry.yaml` from the
/// Charter's own filename, so the reader never found what the writer wrote and
/// the refresh heuristic was inert in every repo — reported as `(missing)` for
/// every Charter, indistinguishable from "not enough closed Charters yet".
pub fn canonical_telemetry_name(charter_id: &str) -> String {
    let canonical = charter_id
        .split_once('-')
        .and_then(|(prefix, rest)| {
            // CHARTER-NN[-slug] → CHARTER-NN
            let nn = rest.split('-').next()?;
            Some(format!("{prefix}-{nn}"))
        })
        .unwrap_or_else(|| charter_id.to_string());
    format!("{canonical}.telemetry.yaml")
}

/// Where `close` writes a Charter's telemetry.
pub fn canonical_telemetry_path(charters_state_dir: &Path, charter_id: &str) -> PathBuf {
    charters_state_dir.join(canonical_telemetry_name(charter_id))
}

/// Legacy sidecar name written before the canonical one: the Charter's own file
/// stem plus `.telemetry.yaml`. Read-only — never written any more, but adopters
/// may still have files under it.
pub fn legacy_telemetry_path(charter_path: &Path) -> Option<PathBuf> {
    let stem = charter_path.file_stem()?.to_str()?;
    let parent = charter_path.parent()?;
    Some(parent.join(format!("{stem}.telemetry.yaml")))
}

/// Locate a Charter's telemetry on disk: canonical name first, legacy second.
///
/// Returns the canonical path even when nothing exists, so callers can report a
/// meaningful "expected here" rather than `None`.
pub fn resolve_telemetry_path(charter_path: &Path, charter_id: &str) -> PathBuf {
    let dir = charter_path.parent().unwrap_or_else(|| Path::new("."));
    let canonical = canonical_telemetry_path(dir, charter_id);
    if canonical.exists() {
        return canonical;
    }
    if let Some(legacy) = legacy_telemetry_path(charter_path) {
        if legacy.exists() {
            return legacy;
        }
    }
    canonical
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_name_strips_the_slug() {
        assert_eq!(
            canonical_telemetry_name("CHARTER-22-fundacion-expedientes"),
            "CHARTER-22.telemetry.yaml"
        );
        assert_eq!(canonical_telemetry_name("CHARTER-02"), "CHARTER-02.telemetry.yaml");
    }

    #[test]
    fn canonical_name_passes_through_unrecognized_shapes() {
        assert_eq!(canonical_telemetry_name("WEIRD"), "WEIRD.telemetry.yaml");
    }

    /// GH #416: the writer and the reader disagreed for two releases and the
    /// only visible symptom was a reassuring "not enough Charters yet" message.
    /// What must never regress is not the name — it is that both sides derive
    /// it from the same place.
    #[test]
    fn writer_and_reader_agree_on_the_name() {
        let dir = std::path::Path::new("/p/.straymark/charters");
        let charter_path = dir.join("22-fundacion-expedientes.md");
        let id = "CHARTER-22-fundacion-expedientes";

        let written = canonical_telemetry_path(dir, id);
        let read = resolve_telemetry_path(&charter_path, id);
        assert_eq!(written, read);
        assert!(written.ends_with("CHARTER-22.telemetry.yaml"));
    }

    #[test]
    fn falls_back_to_the_legacy_name_when_only_that_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let charter_path = dir.join("07-old.md");
        std::fs::write(&charter_path, "x").unwrap();
        std::fs::write(dir.join("07-old.telemetry.yaml"), "y").unwrap();

        let resolved = resolve_telemetry_path(&charter_path, "CHARTER-07-old");
        assert!(resolved.ends_with("07-old.telemetry.yaml"));

        // Once the canonical file appears it wins, without touching the legacy one.
        std::fs::write(dir.join("CHARTER-07.telemetry.yaml"), "z").unwrap();
        let resolved = resolve_telemetry_path(&charter_path, "CHARTER-07-old");
        assert!(resolved.ends_with("CHARTER-07.telemetry.yaml"));
    }
}
