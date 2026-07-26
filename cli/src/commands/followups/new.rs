//! `straymark followups new` — mint an entry whose origin is a **Charter
//! declaration** (ex-ante), before any execution exists.
//!
//! Both pre-existing population paths assume an ex-post origin (Weft field
//! report #360): `drift --apply` extracts from AILOGs, and the hand-edit path
//! inherited that shape. But a deferral decided *at declaration time* — "the
//! Redis CI job is out of scope, register the coverage gap so it is deferred,
//! not silenced" — precedes any AILOG by design. Charters are ex-ante; AILOGs
//! are ex-post; the `declared → in-progress` window is days wide.
//!
//! The hazard this closes is not ergonomic, it is correctness. Lacking a
//! creation verb, the adopter forward-referenced `FU-011` in the Charter body
//! with nothing reserving it — and since ids are minted `max(existing) + 1` at
//! extraction time, the next unrelated `drift --apply` would hand `FU-011` to a
//! different entry, silently pointing the Charter's citations at the wrong
//! follow-up. Assigning the id here, atomically, removes the window: by the
//! time the Charter cites `FU-NNN`, the entry exists.
//!
//! Prioritization stays human, exactly as with `promote` and `verify`: this
//! verb records a decision the operator already made.

use anyhow::{anyhow, bail, Result};
use chrono::Local;
use colored::Colorize;

use crate::commands::followups::note::guard_parse_warnings;
use crate::followups::{self, FuStatus};
use crate::utils;

pub struct NewArgs<'a> {
    pub path: &'a str,
    pub title: &'a str,
    pub origin: &'a str,
    pub bucket: &'a str,
    pub status: &'a str,
    pub trigger: Option<&'a str>,
    pub destination: Option<&'a str>,
    pub cost: Option<&'a str>,
    pub premise: Option<&'a str>,
}

pub fn run(args: NewArgs<'_>) -> Result<()> {
    if args.title.trim().is_empty() {
        bail!("--title is empty. An entry whose title misrepresents it starts life already skewed.");
    }
    if args.origin.trim().is_empty() {
        bail!(
            "--origin is required (schema v1 requires it): the document that decided this \
             deferral, e.g. --origin \"CHARTER-06 §Scope\"."
        );
    }
    if !followups::CANONICAL_BUCKETS.contains(&args.bucket) {
        bail!(
            "Unknown bucket '{}'. Valid: {}.",
            args.bucket,
            followups::CANONICAL_BUCKETS.join(" | ")
        );
    }
    let status = FuStatus::from_str_loose(args.status);
    if status == FuStatus::Unknown {
        bail!(
            "Unknown status '{}'. Valid: open | in-progress | suspected-closed | closed | superseded.",
            args.status
        );
    }
    if status == FuStatus::Promoted {
        bail!("An entry cannot be created as `promoted` — use `followups promote` on an existing entry.");
    }

    let resolved = utils::resolve_project_root(args.path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: run `straymark followups drift --scan-all --apply` to create it, or copy the template from `.straymark/templates/follow-ups-backlog.md` (see STRAYMARK.md §16).",
            registry_path.display()
        );
    }
    let registry = followups::parse_registry(&registry_path)?;
    guard_parse_warnings(&registry)?;

    let fu_number = followups::next_fu_number(&registry);
    let today = Local::now().format("%Y-%m-%d").to_string();
    let notes = format!(
        "Created by `straymark followups new` {} — declared ex-ante, before execution.",
        today
    );
    let block = followups::render_declared_entry(
        fu_number,
        args.title,
        args.origin,
        status.as_str(),
        args.trigger,
        args.destination,
        args.cost,
        args.premise,
        &notes,
    );
    let body = followups::insert_into_bucket(&registry, args.bucket, &block);
    let counters = followups::write_recounted(&registry_path, &registry.frontmatter_raw, &body)?;

    let fu_id = format!("FU-{:03}", fu_number);
    utils::success(&format!(
        "{} created in bucket `{}` ({}).",
        fu_id.bold(),
        args.bucket,
        status.as_str()
    ));
    println!(
        "  Counters: {} open / {} in-progress / {} suspected-closed (total {}).",
        counters.open, counters.in_progress, counters.suspected_closed, counters.total
    );
    println!();
    println!(
        "  {} the id is assigned and written — cite {} in the Charter body now, not a reserved guess.",
        "Next:".bold(),
        fu_id.cyan()
    );
    if args.premise.map(str::trim).unwrap_or("").is_empty() {
        println!(
            "  {}",
            format!(
                "No premise recorded. An entry is a dated hypothesis — `straymark followups verify {} --premise \"...\"` states what it rests on, so acting on it later is a seconds-long re-check.",
                fu_id
            )
            .dimmed()
        );
    }
    Ok(())
}
