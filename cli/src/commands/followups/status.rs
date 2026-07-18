//! `straymark followups status` — registry pulse, or detail view of one entry.
//!
//! The pulse always shows counters **recomputed on the fly** from actual
//! entry statuses, flagging divergence from the (CLI-owned) frontmatter
//! counters when the file is stale. Schema validation against
//! `follow-ups-backlog.schema.v1.json` is advisory — issues print as
//! warnings, never failures.

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::followups::{self, Entry, FuStatus, Registry, Severity};
use crate::followups_schema;
use crate::utils::{self, pad_right_visual, visual_width};

pub fn run(path: &str, fu_id: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        println!(
            "No follow-ups registry yet. Copy `.straymark/templates/follow-ups-backlog.md` to \
             `.straymark/follow-ups-backlog.md` and run `straymark followups drift --scan-all --apply` \
             to seed it (see STRAYMARK.md §16)."
        );
        return Ok(());
    }

    let registry = followups::parse_registry(&registry_path)?;
    for w in &registry.warnings {
        utils::warn(w);
    }

    match fu_id {
        Some(id) => print_entry_detail(&registry, id),
        None => {
            print_pulse(&registry);
            // Advisory schema validation (warn-only; absent schema → dimmed hint).
            let straymark_dir = project_root.join(".straymark");
            match followups_schema::validate_frontmatter(&straymark_dir, &registry.frontmatter_raw)
            {
                Ok(issues) if issues.is_empty() => {}
                Ok(issues) => {
                    for issue in issues {
                        utils::warn(&format!("schema: {}", issue));
                    }
                }
                Err(e) => println!("  {}", format!("(schema check skipped: {})", e).dimmed()),
            }
            Ok(())
        }
    }
}

fn print_pulse(registry: &Registry) {
    let fm = &registry.frontmatter;
    let counters = followups::compute_counters(registry);

    println!();
    println!("  {}", "Follow-ups Backlog".bold().cyan());
    println!();
    println!(
        "  Schema: {}   Last scan: {}   AILOGs extracted: {}",
        fm.schema_version.as_deref().unwrap_or("v0"),
        fm.last_scan.as_deref().unwrap_or("—"),
        fm.fully_extracted_ailogs.len()
    );
    if registry.is_v0() {
        println!(
            "  {}",
            "v0 registry — the first `followups drift --apply` upgrades it to v1 in place."
                .dimmed()
        );
    }
    println!();

    // Status breakdown (recomputed — the trustworthy numbers).
    let rows: Vec<(&str, u32, colored::Color)> = vec![
        ("open", counters.open, colored::Color::White),
        ("in-progress", counters.in_progress, colored::Color::Yellow),
        (
            "suspected-closed",
            counters.suspected_closed,
            colored::Color::Magenta,
        ),
        (
            "closed + superseded",
            counters.closed_cumulative,
            colored::Color::Green,
        ),
        ("promoted", counters.promoted, colored::Color::Cyan),
    ];
    let label_w = rows
        .iter()
        .map(|(l, _, _)| visual_width(l))
        .max()
        .unwrap_or(10);
    let count_w = 5;

    println!(
        "  {} {} {}",
        pad_right_visual("Status", label_w).dimmed(),
        "│".dimmed(),
        pad_right_visual("Count", count_w).dimmed(),
    );
    println!(
        "  {}",
        format!("{}─┼─{}", "─".repeat(label_w), "─".repeat(count_w)).dimmed()
    );
    for (label, count, color) in &rows {
        let count_str = format!("{count:>count_w$}");
        let padded = pad_right_visual(label, label_w);
        if *count > 0 {
            println!("  {} │ {}", padded, count_str.color(*color).bold());
        } else {
            println!("  {} │ {}", padded.dimmed(), count_str.dimmed());
        }
    }
    println!(
        "  {} │ {}",
        pad_right_visual("TOTAL", label_w).bold(),
        format!("{:>count_w$}", counters.total).cyan().bold(),
    );
    println!();

    // Per-bucket open breakdown.
    let buckets: Vec<(&str, usize)> = registry
        .sections
        .iter()
        .filter(|s| s.is_bucket)
        .map(|s| {
            (
                s.name.as_str(),
                s.entries
                    .iter()
                    .filter(|e| matches!(e.status, FuStatus::Open | FuStatus::InProgress))
                    .count(),
            )
        })
        .collect();
    if !buckets.is_empty() {
        let bw = buckets
            .iter()
            .map(|(b, _)| visual_width(b))
            .max()
            .unwrap_or(10);
        println!("  {}", "Open by bucket".bold());
        for (bucket, n) in &buckets {
            let n_str = format!("{n:>count_w$}");
            if *n > 0 {
                println!("  {} │ {}", pad_right_visual(bucket, bw), n_str);
            } else {
                println!(
                    "  {} │ {}",
                    pad_right_visual(bucket, bw).dimmed(),
                    n_str.dimmed()
                );
            }
        }
        println!();
    }

    if counters.blocking_open > 0 {
        println!(
            "  {} {} open blocking entr{} — must land before production cutover. Run `straymark followups list --severity blocking`.",
            "!".red().bold(),
            counters.blocking_open,
            if counters.blocking_open == 1 { "y" } else { "ies" },
        );
        println!();
    }
    if counters.suspected_closed > 0 {
        println!(
            "  {} {} suspected-closed entr{} awaiting triage confirmation (closure marker in source AILOG).",
            "!".magenta().bold(),
            counters.suspected_closed,
            if counters.suspected_closed == 1 { "y" } else { "ies" },
        );
        println!();
    }

    // Flag stale frontmatter counters (informational — `recount` fixes them).
    let declared_open = fm.total_open;
    if let Some(declared) = declared_open {
        if declared != counters.open {
            println!(
                "  {} frontmatter says total_open: {} but the real count is {} — run `straymark followups recount` to reconcile (any write command also recomputes).",
                "!".yellow().bold(),
                declared,
                counters.open
            );
            println!();
        }
    }
}

fn print_entry_detail(registry: &Registry, id: &str) -> Result<()> {
    let entry = followups::find_entry(registry, id).ok_or_else(|| {
        anyhow!(
            "Entry {} not found in the registry.\n  hint: run `straymark followups list` to see entries.",
            id
        )
    })?;

    println!();
    println!("  {} — {}", entry.fu_id.bold().cyan(), entry.description.bold());
    println!();
    print_field("Bucket", Some(&entry.bucket));
    print_field("Status", Some(entry.status.as_str()));
    print_field(
        "Severity",
        entry.severity.map(|s| s.as_str()),
    );
    print_field("Premise", entry.premise.as_deref());
    print_field("Verified-at", entry.verified_at.as_deref());
    print_field("Origin", entry.origin.as_deref());
    print_field("Origin-class", entry.origin_class.as_deref());
    print_field("Trigger", entry.trigger.as_deref());
    print_field("Destination", entry.destination.as_deref());
    print_field("Cost", entry.cost.as_deref());
    if !entry.labels.is_empty() {
        print_field("Labels", Some(&entry.labels.join(", ")));
    }
    print_field("Notes", entry.notes.as_deref());
    print_field("Promoted to", entry.promoted_to.as_deref());
    println!();

    if matches!(entry.status, FuStatus::Open | FuStatus::InProgress)
        && entry.severity == Some(Severity::Blocking)
    {
        println!(
            "  {} blocking severity — must land before production cutover.",
            "!".red().bold()
        );
        println!();
    }
    print_premise_nudge(entry);
    let _ = print_promote_hint(entry);
    Ok(())
}

/// Follow-up entries are *dated hypotheses* (AIDEC-2026-07-18-001): the premise
/// they rest on may have been false at capture, or gone stale since. Nudge the
/// operator to re-verify at execution — the cheap moment — before acting. Fires
/// for actionable entries whose premise has not been re-verified.
fn print_premise_nudge(entry: &Entry) {
    if !matches!(entry.status, FuStatus::Open | FuStatus::InProgress) {
        return;
    }
    if entry.verified_at.is_some() {
        return;
    }
    let what = if entry.premise.is_some() {
        "Its premise hasn't been re-verified since capture."
    } else {
        "It carries no explicit premise — state and re-check the assumption it rests on."
    };
    println!(
        "  {} This is a dated hypothesis. {}",
        "?".yellow().bold(),
        what.dimmed()
    );
    println!(
        "  {}",
        format!(
            "Re-verify against the code before you build on it: `straymark followups verify {}`.",
            entry.fu_id
        )
        .dimmed()
    );
    println!();
}

fn print_field(label: &str, value: Option<&str>) {
    match value {
        Some(v) if !v.is_empty() => {
            println!("  {} {}", pad_right_visual(&format!("{label}:"), 14).dimmed(), v)
        }
        _ => println!(
            "  {} {}",
            pad_right_visual(&format!("{label}:"), 14).dimmed(),
            "—".dimmed()
        ),
    }
}

fn print_promote_hint(entry: &Entry) -> Result<()> {
    if matches!(entry.status, FuStatus::Open | FuStatus::InProgress) {
        println!(
            "  {}",
            format!(
                "If this entry meets the TDE criteria (AGENT-RULES.md §3), run `straymark followups promote {}`.",
                entry.fu_id
            )
            .dimmed()
        );
        println!();
    }
    Ok(())
}
