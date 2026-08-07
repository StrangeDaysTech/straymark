//! `straymark followups set-status FU-NNN <status>` — flip an entry's status
//! and recompute the CLI-owned counters in the same step.
//!
//! This closes the desync window `recount` exists to clean up after (Weft field
//! report #355). The two-step it replaces — hand-edit the `Status` bullet, then
//! remember to run `recount` — is desyncable by construction: forget the second
//! step and the frontmatter counters silently lie about the backlog, which is
//! the one thing a registry exists to be right about.
//!
//! `recount` stays as the escape hatch for a manual-triage session (several
//! statuses flipped by hand at once) and as the idempotent check that this verb
//! got the arithmetic right.

use anyhow::{anyhow, bail, Result};
use colored::Colorize;

use crate::commands::followups::note::guard_parse_warnings;
use crate::followups::{self, FuStatus};

pub fn run(path: &str, fu_id: &str, status_input: &str) -> Result<()> {
    // Validate against the canonical vocabulary rather than writing whatever
    // was typed: the parser is lenient (an unrecognized status degrades to
    // `Unknown` and counts toward no bucket), so a typo written here would not
    // fail — it would quietly remove the entry from every counter.
    let status = FuStatus::from_str_loose(status_input);
    if status == FuStatus::Unknown {
        bail!(
            "Unknown status '{}'. Valid: open | in-progress | suspected-closed | closed | \
             superseded | promoted.\n  note: an unrecognized status parses as `unknown` and \
             counts toward no bucket, so it would silently drop the entry from the counters.",
            status_input
        );
    }
    if status == FuStatus::Promoted {
        bail!(
            "Use `straymark followups promote {}` to reach `promoted` — it writes the TDE \
             document and the `Promoted-to` back-pointer that make the status meaningful. \
             Setting the status alone would claim a promotion with nothing to point at.",
            fu_id
        );
    }

    let resolved = crate::utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: see STRAYMARK.md §16 for the adoption walkthrough.",
            registry_path.display()
        );
    }
    let registry = followups::parse_registry(&registry_path)?;
    guard_parse_warnings(&registry)?;

    let entry = followups::find_entry_unique(&registry, fu_id)?.clone();

    let before = followups::compute_counters(&registry);
    let previous = entry.status_raw.clone().unwrap_or_else(|| entry.status.as_str().to_string());

    if entry.status == status {
        println!(
            "  {} {} is already `{}` — nothing to change.",
            "OK".green().bold(),
            entry.fu_id,
            status.as_str()
        );
        return Ok(());
    }

    let body = followups::set_entry_field(&registry.body, &entry, "Status", status.as_str());
    let after = followups::write_recounted(&registry_path, &registry.frontmatter_raw, &body)?;

    crate::utils::success(&format!(
        "{}: {} → {}",
        entry.fu_id,
        previous.trim().dimmed(),
        status.as_str().bold()
    ));
    println!(
        "  Counters: {} open / {} in-progress / {} suspected-closed / {} closed (was {} / {} / {} / {}).",
        after.open,
        after.in_progress,
        after.suspected_closed,
        after.closed_cumulative,
        before.open,
        before.in_progress,
        before.suspected_closed,
        before.closed_cumulative
    );
    Ok(())
}
