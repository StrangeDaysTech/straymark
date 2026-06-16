//! `straymark status --where` (Loom A1.4, Spec 002 §8, §14) — the textual
//! "you are here" companion to Loom's visual Architecture Plan view.
//!
//! [`run`] (T4.2) loads `architecture/model.yml`, builds the `GovernanceState`
//! via `core::architecture::build_governance_state` (the impure assembler moved
//! to `core` in A2.0 so the Loom server reuses it — one builder, NFR3), calls
//! the **pure** `core::architecture::project`, and renders the per-layer/
//! per-component state with the active ("you are here") components highlighted,
//! followed by the §8 "Where are we" summary (active charters +
//! declared-vs-modified progress + recent AILOGs + open debt).
//!
//! The consistency gate (T4.3) asserts the `active`/`in-progress`/`implemented`
//! flags here line up with `straymark charter list` + `charter drift`.

use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use straymark_core::architecture::{
    build_governance_state, parse_model, project, ComponentProjection, ComponentState,
    GovernanceState, Projection,
};
use straymark_core::charter::{self, Charter, CharterStatus};

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
    fn state_badges_orders_and_marks() {
        let badges = state_badges(&[ComponentState::Active, ComponentState::InProgress]);
        assert!(badges.contains("[active]"));
        assert!(badges.contains("[in-progress]"));
        assert_eq!(state_badges(&[]), "—".dimmed().to_string());
    }
}
