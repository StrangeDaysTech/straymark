//! `straymark followups merge-driver <base> <ours> <theirs>` — a git merge
//! driver for the follow-ups registry (GH #391).
//!
//! The registry is CLI-owned and every parallel PR that touches follow-ups
//! conflicts on it; resolving by taking one side and re-running `drift --apply`
//! silently reverted the other side's closures. Wired as a git merge driver,
//! the conflict disappears: git hands us the three file versions and this
//! command writes the structural three-way merge back into `ours`.
//!
//! Setup (once per clone):
//!
//! ```gitattributes
//! .straymark/follow-ups-backlog.md merge=straymark-followups
//! ```
//! ```sh
//! git config merge.straymark-followups.driver 'straymark followups merge-driver %O %A %B'
//! ```
//!
//! Exit codes follow git's merge-driver contract: 0 = merged (even with
//! reported soft conflicts), nonzero = unresolved, git falls back to marking
//! the file conflicted.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::followups;

pub fn run(base: &str, ours: &str, theirs: &str) -> Result<()> {
    let ours_path = Path::new(ours);
    let theirs_path = Path::new(theirs);
    let base_path = Path::new(base);

    let ours_reg = followups::parse_registry(ours_path)
        .with_context(|| format!("parse ours ({ours})"))?;
    let theirs_reg = followups::parse_registry(theirs_path)
        .with_context(|| format!("parse theirs ({theirs})"))?;

    // A missing/unparseable base (unborn branch history, force-pushed roots)
    // degrades to a two-way merge: ours is treated as the base, which only
    // disables deletion detection — statuses and additions still reconcile.
    let base_owned = match followups::parse_registry(base_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{} base ({}) unparseable — merging two-way, deletion detection disabled ({e})",
                "warn:".yellow().bold(),
                base
            );
            ours_reg.clone()
        }
    };
    let base_reg = &base_owned;

    for w in ours_reg
        .warnings
        .iter()
        .chain(theirs_reg.warnings.iter())
    {
        eprintln!("{} {w}", "warn:".yellow().bold());
    }

    let (merged, report) = followups::merge_registries(base_reg, &ours_reg, &theirs_reg)?;
    std::fs::write(ours_path, &merged)
        .with_context(|| format!("write merged registry to {ours}"))?;

    println!(
        "{} follow-ups registry merged structurally ({} → {}).",
        "✓".green().bold(),
        ours_reg.entries().count(),
        merged.matches("### FU-").count()
    );
    if report.statuses_preserved > 0 {
        println!(
            "  {} {} status(es) preserved from theirs (non-open beats open).",
            "→".blue().bold(),
            report.statuses_preserved
        );
    }
    if report.appended > 0 {
        println!(
            "  {} {} entr{} appended from theirs.",
            "→".blue().bold(),
            report.appended,
            if report.appended == 1 { "y" } else { "ies" }
        );
    }
    if report.deletions_respected > 0 {
        println!(
            "  {} {} deletion(s) from theirs respected.",
            "→".blue().bold(),
            report.deletions_respected
        );
    }
    for conflict in &report.conflicts {
        eprintln!("  {} {conflict}", "warn:".yellow().bold());
    }
    Ok(())
}
