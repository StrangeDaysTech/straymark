//! `straymark status --where` (Loom A1.4, Spec 002 §8, §14) — the textual
//! "you are here" companion to Loom's visual Architecture Plan view.
//!
//! Two halves, mirroring the spec split:
//! - [`build_governance_state`] (T4.1) gathers the impure governance signals
//!   (charters, drift, on-disk inventory, open debt) into the pure
//!   [`GovernanceState`] data struct that `core` defines.
//! - [`run`] (T4.2) loads `architecture/model.yml`, calls the **pure**
//!   `core::architecture::project`, and renders the per-layer/per-component
//!   state with the active ("you are here") components highlighted, followed by
//!   the §8 "Where are we" summary (active charters + declared-vs-modified
//!   progress + recent AILOGs + open debt).
//!
//! The projection is shared with the Loom server (`/api/where`, A2): both build
//! a `GovernanceState` and call the same `project`, so the textual and visual
//! answers can never disagree (NFR3). The consistency gate (T4.3) asserts the
//! `active`/`in-progress`/`implemented` flags here line up with
//! `straymark charter list` + `charter drift`.
//!
//! `wiring-gap` (the sixth `ComponentState`) is intentionally **not** populated
//! here: `analyze declared-vs-wired` needs an explicit profile and is not part
//! of the §11.5 consistency gate, so feeding it would add noise without a
//! contract. It stays empty until a future increment wires a default profile.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

use straymark_core::architecture::{
    parse_model, project, ComponentProjection, ComponentState, GovernanceState, Projection,
};
use straymark_core::charter::{self, Charter, CharterStatus};
use straymark_core::charter_files::parse_files_to_modify;
use straymark_core::document::{detect_doc_type, discover_documents, parse_document, DocType};
use straymark_core::drift::compute_drift;

use super::common;
use crate::ailog;
use crate::utils;

/// `straymark status --where [path] [--out DIR]`.
pub fn run(path: &str, out: Option<&str>) -> Result<()> {
    let root = common::resolve_root(path);
    let (_out_dir, model_path, _drawio) = common::artifact_paths(&root, out);

    if !model_path.exists() {
        // Degrade gracefully (Spec 002 §14 / T4.2): this is a status view, not a
        // gate — no model just means "nothing to locate yet".
        utils::info(&format!(
            "No architecture model at {} yet.",
            model_path.display()
        ));
        utils::info(&format!(
            "Run {} to draft one from your codebase + ADRs.",
            "straymark architecture generate".cyan()
        ));
        return Ok(());
    }

    let model = parse_model(&model_path)
        .with_context(|| format!("parsing {}", model_path.display()))?;
    let state = build_governance_state(&root);
    let projection = project(&model, &state);

    render(&root, &model_path, &model, &projection, &state);
    Ok(())
}

/// Gather the governance-derived file sets (Spec 002 §4, T4.1) the pure
/// projection consumes. All impure work (git, fs walk, document parsing) lives
/// here; the projection itself stays pure.
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

    let on_disk_files: Vec<String> = common::collect_source_files(root)
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

/// Declared `## Files to Modify` paths across a set of Charters, sorted+deduped.
/// Reuses `charter_files::parse_files_to_modify` — the byte-for-byte extractor
/// `charter drift` uses — so declared sets match across commands.
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
                    let body = utils::split_frontmatter(&content)
                        .map(|(_, b)| b)
                        .unwrap_or(&content);
                    for f in straymark_core::ailog::parse_modified_files(body) {
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

/// Paths referenced by open `TDE` (technical-debt) documents — their `related`
/// frontmatter. A TDE counts as open unless its status reads as resolved.
fn open_tde_files(root: &Path) -> Vec<String> {
    let straymark_dir = root.join(".straymark");
    if !straymark_dir.is_dir() {
        return Vec::new();
    }
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
                    set.insert(r);
                }
            }
        }
    }
    set.into_iter().collect()
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

// ── Rendering ───────────────────────────────────────────────────────────────

fn render(
    root: &Path,
    model_path: &Path,
    model: &straymark_core::architecture::ArchModel,
    projection: &Projection,
    state: &GovernanceState,
) {
    println!();
    println!("  {}", "Where are we".bold().cyan());
    println!();
    println!(
        "  {} {}",
        "Model".dimmed(),
        model_path.display().to_string().dimmed()
    );
    println!();

    render_layers(model, projection);
    render_summary(root, projection, state);
}

/// Per-layer / per-component states, in model layer order, with active
/// components marked "you are here".
fn render_layers(
    model: &straymark_core::architecture::ArchModel,
    projection: &Projection,
) {
    let any_active = projection
        .components
        .iter()
        .any(|c| c.states.contains(&ComponentState::Active));

    for layer in &model.layers {
        let comps: Vec<&ComponentProjection> = projection
            .components
            .iter()
            .filter(|c| c.layer == layer.id)
            .collect();
        if comps.is_empty() {
            continue;
        }
        println!("  {}", layer.label.bold());
        for cp in comps {
            let label = model
                .components
                .iter()
                .find(|c| c.id == cp.component_id)
                .map(|c| c.label.as_str())
                .unwrap_or(&cp.component_id);
            let here = cp.states.contains(&ComponentState::Active);
            let marker = if here { "▸".green().bold() } else { "·".dimmed() };
            let name = if here {
                label.green().bold()
            } else {
                label.normal()
            };
            let badges = state_badges(&cp.states);
            let you_are_here = if here {
                "  ← you are here".green()
            } else {
                "".normal()
            };
            println!("    {marker} {name}  {badges}{you_are_here}");
        }
        println!();
    }

    if !any_active {
        utils::info("No active component — no in-progress Charter declares files in any component.");
        println!();
    }
}

/// Colored, kebab-case badges for a component's states (projection order).
fn state_badges(states: &[ComponentState]) -> String {
    if states.is_empty() {
        return "—".dimmed().to_string();
    }
    states
        .iter()
        .map(|s| {
            let t = format!("[{}]", s.as_str());
            match s {
                ComponentState::Active => t.green().bold().to_string(),
                ComponentState::InProgress => t.yellow().bold().to_string(),
                ComponentState::Implemented => t.green().to_string(),
                ComponentState::HasDebt => t.magenta().to_string(),
                ComponentState::WiringGap => t.red().to_string(),
                ComponentState::Uncharted => t.dimmed().to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The §8 "Where are we" summary: active charters, declared-vs-modified
/// progress, recent AILOGs, open debt.
fn render_summary(root: &Path, projection: &Projection, state: &GovernanceState) {
    println!("  {}", "Summary".bold());

    // Active charters (in-progress).
    let (charters, _e) = charter::discover_and_parse(root);
    let active: Vec<&Charter> = charters
        .iter()
        .filter(|c| c.frontmatter.status == CharterStatus::InProgress)
        .collect();
    if active.is_empty() {
        println!(
            "    {} no in-progress Charter",
            "Active Charter:".dimmed()
        );
    } else {
        for c in &active {
            println!(
                "    {} {}",
                "Active Charter:".dimmed(),
                charter::display_title(c)
            );
        }
    }

    // Declared-vs-modified progress over the active declared set.
    let declared = &state.active_charter_files;
    if !declared.is_empty() {
        let touched = state.in_progress_files.len();
        let total = declared.len();
        let pct = (touched * 100) / total.max(1);
        println!(
            "    {} {}/{} declared files touched ({}%)",
            "Progress:".dimmed(),
            touched,
            total,
            pct
        );
    }

    // Recent AILOGs (most recent 3, date-prefixed filenames sort lexically).
    let recent = recent_ailogs(root, 3);
    if !recent.is_empty() {
        println!("    {} {}", "Recent AILOGs:".dimmed(), recent.join(", "));
    }

    // Open debt — components flagged has-debt.
    let debt = projection
        .components
        .iter()
        .filter(|c| c.states.contains(&ComponentState::HasDebt))
        .count();
    let uncharted = projection
        .components
        .iter()
        .filter(|c| c.states.contains(&ComponentState::Uncharted))
        .count();
    println!(
        "    {} {} component{} with open debt, {} uncharted",
        "Debt:".dimmed(),
        debt,
        common::plural(debt),
        uncharted
    );
    println!();
}

/// Filenames (without extension) of the most recent `n` AILOGs.
fn recent_ailogs(root: &Path, n: usize) -> Vec<String> {
    let dir = ailog::agent_logs_dir(root);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("AILOG-") && s.ends_with(".md"))
                    .unwrap_or(false)
            })
            .filter_map(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect(),
        Err(_) => return Vec::new(),
    };
    names.sort();
    names.into_iter().rev().take(n).collect()
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
    fn state_badges_orders_and_marks() {
        let badges = state_badges(&[ComponentState::Active, ComponentState::InProgress]);
        assert!(badges.contains("[active]"));
        assert!(badges.contains("[in-progress]"));
        assert_eq!(state_badges(&[]), "—".dimmed().to_string());
    }
}
