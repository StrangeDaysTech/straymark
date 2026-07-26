//! `straymark followups verify FU-NNN` — re-verify a follow-up's premise at
//! execution time and record it.
//!
//! A follow-up entry is a **dated, decaying hypothesis** (AIDEC-2026-07-18-001,
//! from Weft field report #365): its premise may have been false at capture or
//! gone stale since, and the only real bug is acting on one *without* re-testing
//! that premise. The registry is a speculative buffer — cheap capture is its
//! value — so verification belongs at the cheap moment (acting on the entry),
//! not at capture.
//!
//! This verb covers the common case: an entry acted on as a chore that never
//! promotes (the promote-time reminder in `promote.rs` covers the graduation
//! path). It:
//!
//! - surfaces the entry's `Premise` (falling back to `Notes`);
//! - optionally records/updates the premise (`--premise "..."`);
//! - stamps `Verified-at: <today|--at>` when the operator confirms the re-check
//!   (`--verified`).
//!
//! Human judgment stays out of the CLI (as with `promote`): the command puts the
//! premise in front of the operator and records that the check happened — it
//! never decides truth itself.

use anyhow::{anyhow, bail, Result};
use chrono::Local;
use colored::Colorize;

use crate::followups;
use crate::utils;

pub fn run(
    path: &str,
    fu_id: &str,
    premise: Option<&str>,
    verified: bool,
    at: Option<&str>,
) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
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
    let entry = followups::find_entry(&registry, fu_id)
        .ok_or_else(|| {
            anyhow!(
                "Entry {} not found in the registry.\n  hint: run `straymark followups list` to see entries.",
                fu_id
            )
        })?
        .clone();

    // ── Surface the premise (the assumption to re-check) ──
    let effective_premise = premise
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .or_else(|| entry.premise.clone())
        .or_else(|| entry.notes.clone());

    println!();
    println!("  {} — {}", entry.fu_id.bold().cyan(), entry.description.bold());
    println!();
    println!("  {}", "Premise (re-check against the code):".bold());
    match &effective_premise {
        Some(text) if !text.trim().is_empty() => println!("    {}", text),
        _ => println!(
            "    {}",
            "(none recorded — pass --premise \"<the assumption this entry rests on>\")".dimmed()
        ),
    }
    println!();

    // ── Write-back: record premise and/or stamp verification ──
    let mut body = registry.body.clone();
    let mut wrote = false;

    if let Some(new_premise) = premise.filter(|s| !s.trim().is_empty()) {
        body = followups::set_entry_field(&body, &entry, "Premise", new_premise.trim());
        wrote = true;
    }

    let stamped = if verified {
        // Re-parse before the second edit so the span is current.
        let entry_now = if wrote {
            let reparsed = followups::parse_registry_str(
                &registry_path,
                &followups::assemble(&registry.frontmatter_raw, &body),
            )?;
            followups::find_entry(&reparsed, &entry.fu_id).unwrap().clone()
        } else {
            entry.clone()
        };
        let date = at
            .map(|s| s.to_string())
            .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
        body = followups::set_entry_field(&body, &entry_now, "Verified-at", &date);
        wrote = true;
        Some(date)
    } else {
        None
    };

    if wrote {
        // Recompute the CLI-owned counters + v0→v1 upgrade, like every write
        // command — even though status is unchanged, this keeps the registry
        // canonical and idempotent.
        followups::write_recounted(&registry_path, &registry.frontmatter_raw, &body)?;

        if premise.filter(|s| !s.trim().is_empty()).is_some() {
            utils::success(&format!("{} premise recorded.", entry.fu_id));
        }
        if let Some(date) = &stamped {
            utils::success(&format!("{} verified — Verified-at → {}.", entry.fu_id, date));
        }
        println!();
    } else {
        // Read-only surfacing: nudge toward recording the re-check.
        println!(
            "  {}",
            format!(
                "When you've re-checked it: `straymark followups verify {} --verified` (add `--premise \"...\"` to record the assumption).",
                entry.fu_id
            )
            .dimmed()
        );
        println!();
    }

    Ok(())
}
