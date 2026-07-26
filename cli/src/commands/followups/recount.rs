//! `straymark followups recount` — recompute the CLI-owned frontmatter
//! counters from actual entry statuses, without scanning AILOGs.
//!
//! Closes the manual-triage loop found in the first external adoption
//! (issue #222 Finding 1): the v1 lifecycle sanctions Triage and Consumption
//! as manual status edits, but until cli-3.20.0 the only commands that
//! recomputed the CLI-owned `total_*` counters were `drift --apply` and
//! `promote` — both no-ops after a pure-triage session, stranding the
//! frontmatter with knowingly stale counters and no §13-compliant fix.
//!
//! `recount` is the dedicated, idempotent verb: parse the registry, recompute
//! the counters from entry statuses (the same `compute_counters` the pulse
//! uses), rewrite the frontmatter (upgrading v0 → v1 in place, like every
//! other write command), and report. No AILOG scan, no extraction, no entry
//! mutation — counters only.

use anyhow::{anyhow, bail, Result};
use colored::Colorize;

use crate::followups;
use crate::utils;

pub fn run(path: &str) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: run `straymark followups drift --scan-all --apply` to create and seed it (see STRAYMARK.md §16).",
            registry_path.display()
        );
    }

    let registry = followups::parse_registry(&registry_path)?;
    for w in &registry.warnings {
        utils::warn(w);
    }

    // Same recompute path the mutating verbs use (`note` / `set-status` /
    // `new` / `verify`), so this stays the idempotent check on their arithmetic
    // rather than a second implementation that could disagree with them.
    let (fm, counters) = followups::recounted_frontmatter(
        &registry_path,
        &registry.frontmatter_raw,
        &registry.body,
    )?;

    if fm == registry.frontmatter_raw {
        println!(
            "{} counters already in sync: {} open / {} suspected-closed / {} promoted (total {}).",
            "OK".green().bold(),
            counters.open,
            counters.suspected_closed,
            counters.promoted,
            counters.total
        );
        return Ok(());
    }

    let was_v0 = registry.is_v0();
    std::fs::write(
        &registry_path,
        followups::assemble(&fm, &registry.body),
    )?;

    utils::success(&format!(
        "Counters recomputed: {} open / {} suspected-closed / {} promoted (total {}).",
        counters.open, counters.suspected_closed, counters.promoted, counters.total
    ));
    if was_v0 {
        utils::info("Registry upgraded to schema v1 (non-destructive — counters are now CLI-owned).");
    }
    Ok(())
}
