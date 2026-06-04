//! `straymark followups list` — enumerate registry entries with filters.
//!
//! Output is a plain-text table to stdout. Parse warnings (malformed entry
//! headings) are reported to stderr but do not fail the command — list what
//! you can, surface what you can't (same contract as `charter list`).

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::followups::{self, Entry, FuStatus, Severity};
use crate::utils;

pub struct Filters<'a> {
    pub bucket: Option<&'a str>,
    pub status: Option<&'a str>,
    pub severity: Option<&'a str>,
    pub label: Option<&'a str>,
}

pub fn run(path: &str, filters: &Filters) -> Result<()> {
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

    let entries: Vec<&Entry> = registry
        .entries()
        .filter(|e| matches_filters(e, filters))
        .collect();

    if entries.is_empty() {
        if registry.entries().count() == 0 {
            println!("Registry is empty. Run `straymark followups drift --scan-all --apply` to seed it from existing AILOGs.");
        } else {
            println!("No entries match the given filter.");
        }
        return Ok(());
    }

    print_table(&entries);
    Ok(())
}

fn matches_filters(e: &Entry, f: &Filters) -> bool {
    if let Some(bucket) = f.bucket {
        if !e.bucket.eq_ignore_ascii_case(bucket) {
            return false;
        }
    }
    if let Some(status) = f.status {
        if e.status != FuStatus::from_str_loose(status) {
            return false;
        }
    }
    if let Some(severity) = f.severity {
        match Severity::from_str_loose(severity) {
            Some(want) => {
                if e.severity != Some(want) {
                    return false;
                }
            }
            None => return false,
        }
    }
    if let Some(label) = f.label {
        if !e.labels.iter().any(|l| l.eq_ignore_ascii_case(label)) {
            return false;
        }
    }
    true
}

/// Columns: FU, STATUS, SEV, BUCKET, DEST, DESCRIPTION.
fn print_table(entries: &[&Entry]) {
    const DEST_MAX: usize = 24;
    const BUCKET_MAX: usize = 18;

    let fu_w = entries
        .iter()
        .map(|e| e.fu_id.len())
        .max()
        .unwrap_or(2)
        .max("FU".len());
    let status_w = entries
        .iter()
        .map(|e| e.status.as_str().len())
        .max()
        .unwrap_or(0)
        .max("STATUS".len());
    let sev_w = "SEV".len().max("blocking".len());
    let bucket_w = entries
        .iter()
        .map(|e| utils::visual_width(&e.bucket).min(BUCKET_MAX))
        .max()
        .unwrap_or(0)
        .max("BUCKET".len());
    let dest_w = entries
        .iter()
        .map(|e| utils::visual_width(e.destination.as_deref().unwrap_or("—")).min(DEST_MAX))
        .max()
        .unwrap_or(0)
        .max("DEST".len());

    println!(
        "  {}  {}  {}  {}  {}  {}",
        utils::pad_right_visual("FU", fu_w).bold(),
        utils::pad_right_visual("STATUS", status_w).bold(),
        utils::pad_right_visual("SEV", sev_w).bold(),
        utils::pad_right_visual("BUCKET", bucket_w).bold(),
        utils::pad_right_visual("DEST", dest_w).bold(),
        "DESCRIPTION".bold(),
    );

    for e in entries {
        let sev = match e.severity {
            Some(Severity::Blocking) => "blocking",
            Some(Severity::Normal) => "normal",
            None => "—",
        };
        let dest = utils::truncate_visual(e.destination.as_deref().unwrap_or("—"), DEST_MAX);
        let bucket = utils::truncate_visual(&e.bucket, BUCKET_MAX);
        // Pad before coloring — escape codes don't contribute to visual width.
        let padded_status = utils::pad_right_visual(e.status.as_str(), status_w);
        let padded_sev = utils::pad_right_visual(sev, sev_w);

        println!(
            "  {}  {}  {}  {}  {}  {}",
            utils::pad_right_visual(&e.fu_id, fu_w),
            colorize_status(e.status, &padded_status),
            colorize_severity(e.severity, &padded_sev),
            utils::pad_right_visual(&bucket, bucket_w),
            utils::pad_right_visual(&dest, dest_w),
            e.description,
        );
    }
}

fn colorize_status(status: FuStatus, text: &str) -> colored::ColoredString {
    match status {
        FuStatus::Open => text.normal(),
        FuStatus::InProgress => text.yellow(),
        FuStatus::SuspectedClosed => text.magenta(),
        FuStatus::Closed | FuStatus::Superseded => text.green(),
        FuStatus::Promoted => text.cyan(),
        FuStatus::Unknown => text.dimmed(),
    }
}

fn colorize_severity(severity: Option<Severity>, text: &str) -> colored::ColoredString {
    match severity {
        Some(Severity::Blocking) => text.red().bold(),
        _ => text.normal(),
    }
}
