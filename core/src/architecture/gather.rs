//! The impure half of the "you are here" projection: gather the governance file
//! sets a [`GovernanceState`] needs from disk + git.
//!
//! [`super::projection::project`] is a **pure** function of `(model + state)`;
//! this module is the I/O that assembles the `state`. Moved out of the CLI in
//! Loom A2.0 (the A1.0 pattern) so the CLI (`status --where`) and the Loom
//! server (A2) build the projection input **one way** — the components they
//! flag `active`/`implemented` cannot disagree (NFR3).
//!
//! `wiring-gap` is intentionally left empty here: `analyze declared-vs-wired`
//! needs an explicit profile and is not part of the §11.5 consistency gate, so
//! feeding it would add noise without a contract.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ailog;
use crate::architecture::projection::GovernanceState;
use crate::charter::{self, Charter, CharterStatus};
use crate::charter_files::parse_files_to_modify;
use crate::architecture::scan::{resolve_scan_config, ScanConfig};
use crate::document::{detect_doc_type, discover_documents, parse_document, DocType};
use crate::drift::compute_drift;

/// Gather the governance-derived file sets (Spec 002 §4) the pure projection
/// consumes. All impure work (git, fs walk, document parsing) lives here; the
/// projection itself ([`super::projection::project`]) stays pure.
pub fn build_governance_state(root: &Path) -> GovernanceState {
    let (charters, _errors) = charter::discover_and_parse(root);

    let active_charter_files = declared_files(
        charters
            .iter()
            .filter(|c| c.frontmatter.status == CharterStatus::InProgress),
    );

    // Implemented = declared files of closed Charters, folding in the files
    // their originating AILOGs actually recorded as modified.
    let mut closed_charter_files: BTreeSet<String> = declared_files(
        charters
            .iter()
            .filter(|c| c.frontmatter.status == CharterStatus::Closed),
    )
    .into_iter()
    .collect();
    closed_charter_files.extend(closed_ailog_files(root, &charters));
    let closed_charter_files: Vec<String> = closed_charter_files.into_iter().collect();

    // in-progress = active declared files already touched in the working tree
    // (declared ∩ git-modified), computed with the same matcher as `charter
    // drift` so the two never disagree (NFR3).
    let modified = git_modified_files(root);
    let (_omitted, extra) = compute_drift(&active_charter_files, &modified);
    let in_progress_files: Vec<String> = modified
        .iter()
        .filter(|m| !extra.contains(m))
        .cloned()
        .collect();

    let on_disk_files: Vec<String> = collect_source_files(root)
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();

    GovernanceState {
        active_charter_files,
        in_progress_files,
        closed_charter_files,
        tde_files: open_tde_files(root),
        wiring_gap_files: Vec::new(),
        on_disk_files,
    }
}

/// Source files (paths relative to `root`), filtered by the project's
/// [`ScanConfig`] (source extensions + excluded dirs, defaults ∪ the
/// `architecture:` config section, #279). Used for the on-disk inventory (the
/// `uncharted` signal) and by the CLI generator to discover component dirs.
pub fn collect_source_files(root: &Path) -> Vec<PathBuf> {
    collect_source_files_with(root, &resolve_scan_config(root))
}

/// [`collect_source_files`] with an explicit config (so a caller that already
/// resolved one — e.g. the generator computing component dirs — reuses it).
pub fn collect_source_files_with(root: &Path, cfg: &ScanConfig) -> Vec<PathBuf> {
    fn walk(dir: &Path, root: &Path, cfg: &ScanConfig, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !cfg.is_excluded_dir(name) {
                        walk(&path, root, cfg, out);
                    }
                }
            } else if path.is_file() {
                let is_source = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| cfg.is_source_ext(ext));
                if is_source {
                    if let Ok(rel) = path.strip_prefix(root) {
                        out.push(rel.to_path_buf());
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, cfg, &mut out);
    out
}

/// Declared `## Files to Modify` paths across a set of Charters, sorted+deduped.
/// Reuses [`crate::charter_files::parse_files_to_modify`] — the byte-for-byte
/// extractor `charter drift` uses — so declared sets match across commands.
fn declared_files<'a>(charters: impl Iterator<Item = &'a Charter>) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for c in charters {
        for f in parse_files_to_modify(&c.body) {
            set.insert(f.path);
        }
    }
    set.into_iter().collect()
}

/// Files the originating AILOGs of closed Charters recorded as modified
/// (`## Modified Files`). Folds real implementation evidence into the
/// `implemented` signal.
fn closed_ailog_files(root: &Path, charters: &[Charter]) -> Vec<String> {
    let agent_logs_dir = ailog::agent_logs_dir(root);
    if !agent_logs_dir.exists() {
        return Vec::new();
    }
    let mut set: BTreeSet<String> = BTreeSet::new();
    for c in charters {
        if c.frontmatter.status != CharterStatus::Closed {
            continue;
        }
        let ids = match &c.frontmatter.originating_ailogs {
            Some(ids) => ids,
            None => continue,
        };
        for id in ids {
            if let Some(path) = ailog::find_ailog_file(&agent_logs_dir, id) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for f in ailog::modified_files(&content) {
                        set.insert(f);
                    }
                }
            }
        }
    }
    set.into_iter().collect()
}

/// Files modified in the working tree relative to `HEAD`, sorted+deduped. Empty
/// on git failure (treated as "nothing modified"), matching `charter drift`.
fn git_modified_files(root: &Path) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .current_dir(root)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let mut v: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Source paths attributable to open `TDE` (technical-debt) documents — the
/// `has-debt` overlay input (§4). A TDE counts as open unless its status reads
/// as resolved.
///
/// A TDE's `related` frontmatter lists *governance docs* (AILOGs, audit reviews,
/// Charters) far more often than source paths, so matching `related` directly
/// against component globs almost never lights up a component (#273). We bridge
/// the gap: for each `related` entry that names an **AILOG**, resolve it to its
/// file and pull that AILOG's `## Modified Files` — the code the debt actually
/// lives in — and feed those source paths to the projection. Raw `related`
/// entries are kept too (a TDE may list a source path directly).
fn open_tde_files(root: &Path) -> Vec<String> {
    let straymark_dir = root.join(".straymark");
    if !straymark_dir.is_dir() {
        return Vec::new();
    }
    let logs_dir = ailog::agent_logs_dir(root);
    let mut set: BTreeSet<String> = BTreeSet::new();
    for p in discover_documents(&straymark_dir) {
        let is_tde = p
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(detect_doc_type)
            == Some(DocType::Tde);
        if !is_tde {
            continue;
        }
        if let Ok(doc) = parse_document(&p) {
            if !tde_is_open(doc.frontmatter.status.as_deref()) {
                continue;
            }
            if let Some(related) = doc.frontmatter.related {
                for r in related {
                    // Transitively resolve an AILOG reference to the files it
                    // modified, so the debt maps onto the components that own
                    // those files.
                    if let Some(files) = ailog_modified_files(&logs_dir, &r) {
                        set.extend(files);
                    }
                    set.insert(r);
                }
            }
        }
    }
    set.into_iter().collect()
}

/// If `related` names an AILOG (by id, filename, or path), resolve it to its
/// file and return the source paths it recorded as modified (frontmatter
/// `files_modified` ∪ the `## Modified Files` table). Returns `None` when
/// `related` is not an AILOG reference or the file can't be found.
fn ailog_modified_files(logs_dir: &Path, related: &str) -> Option<Vec<String>> {
    let base = Path::new(related)
        .file_name()
        .and_then(|n| n.to_str())?
        .trim_end_matches(".md");
    if !base.starts_with("AILOG-") {
        return None;
    }
    let path = ailog::find_ailog_file(logs_dir, base)?;
    let content = std::fs::read_to_string(&path).ok()?;
    Some(ailog::modified_files(&content))
}

/// A TDE is open unless its status reads as a resolved/closed terminal state.
/// Absent status defaults to open (debt is open until explicitly retired).
fn tde_is_open(status: Option<&str>) -> bool {
    match status {
        None => true,
        Some(s) => !matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "resolved" | "closed" | "mitigated" | "done" | "fixed"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tde_open_unless_resolved() {
        assert!(tde_is_open(None));
        assert!(tde_is_open(Some("open")));
        assert!(tde_is_open(Some("accepted")));
        assert!(!tde_is_open(Some("resolved")));
        assert!(!tde_is_open(Some(" Closed ")));
        assert!(!tde_is_open(Some("MITIGATED")));
    }

    #[test]
    fn collect_source_files_skips_excluded_and_keeps_source() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::create_dir_all(root.join("node_modules")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(root.join("README.md"), "# doc\n").unwrap();
        std::fs::write(root.join("target/out.rs"), "// built\n").unwrap();
        std::fs::write(root.join("node_modules/dep.js"), "//dep\n").unwrap();

        let files: Vec<String> = collect_source_files(root)
            .iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect();
        assert_eq!(files, vec!["src/main.rs"]); // README.md (not source), target/, node_modules/ excluded
    }

    #[test]
    fn build_state_empty_on_bare_dir() {
        // No .straymark, no git, no charters → every set empty, no panic.
        let tmp = tempfile::TempDir::new().unwrap();
        let state = build_governance_state(tmp.path());
        assert!(state.active_charter_files.is_empty());
        assert!(state.closed_charter_files.is_empty());
        assert!(state.tde_files.is_empty());
        assert!(state.wiring_gap_files.is_empty());
    }

    #[test]
    fn open_tde_maps_to_source_via_related_ailog_modified_files() {
        // #273 part 4: a TDE's `related` lists an AILOG (a doc, not source), so
        // the debt must reach the code transitively — through that AILOG's
        // `## Modified Files`. Here the open TDE references an AILOG that touched
        // `internal/modules/commshub/http.go`, so that path must end up in
        // `tde_files` (where component globs can then match it).
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let logs = root.join(".straymark/07-ai-audit/agent-logs");
        let tdes = root.join(".straymark/06-evolution/technical-debt");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::create_dir_all(&tdes).unwrap();

        // This AILOG carries its modified files ONLY in the frontmatter list
        // (no `## Modified Files` table) — the etapa-log shape that used to be
        // missed, so the debt would never reach the component (#273).
        std::fs::write(
            logs.join("AILOG-2026-05-07-041-commshub.md"),
            "---\nid: AILOG-2026-05-07-041\nfiles_modified:\n  \
             - internal/modules/commshub/http.go\n---\n\n## Summary\n\nno table.\n",
        )
        .unwrap();
        std::fs::write(
            tdes.join("TDE-2026-05-11-002-commshub-coverage.md"),
            "---\nid: TDE-2026-05-11-002\nstatus: identified\nrelated:\n  \
             - AILOG-2026-05-07-041-commshub.md\n---\n\n# debt\n",
        )
        .unwrap();

        let state = build_governance_state(root);
        assert!(
            state
                .tde_files
                .iter()
                .any(|f| f == "internal/modules/commshub/http.go"),
            "expected the AILOG's modified file in tde_files, got: {:?}",
            state.tde_files
        );
    }
}
