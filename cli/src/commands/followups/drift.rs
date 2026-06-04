//! `straymark followups drift` — keep the registry in sync with AILOGs.
//!
//! Native Rust replacement (cli-3.19.0) for the deprecated adopter-side
//! `check-followups-drift.sh`. Three modes, mirroring the v0 script:
//!
//! - **default** — scan AILOGs changed in `git diff origin/main..HEAD`
//!   (fallback `origin/master..HEAD`, then `HEAD~1..HEAD` with a warning).
//!   Warn + exit 1 when an AILOG with follow-up content is not in
//!   `fully_extracted_ailogs`.
//! - **`--apply`** — same scan + extract new entries into `## Bucket: ready`,
//!   append the AILOG ids to `fully_extracted_ailogs`, **recompute the
//!   `total_*` counters** (#214 Signal 2) and upgrade v0 → v1 in place.
//!   Entries whose AILOG text carries a closure marker are extracted as
//!   `suspected-closed` instead of `open` (#214 Signal 1 — the anti-noise
//!   refinement).
//! - **`--scan-all`** — sweep every AILOG in the project.
//!
//! Granularity is **per-AILOG**, never per-bullet — the empirically validated
//! design choice (0 false positives across 76 AILOGs in the reference
//! adopter). See `FOLLOW-UPS-BACKLOG-PATTERN.md` §Drift detection.
//!
//! ## Exit codes
//! - `0` — no drift (or `--apply` extracted everything)
//! - `1` — drift found (default mode only)

use anyhow::{anyhow, Result};
use chrono::Local;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ailog;
use crate::followups::{self, ExtractedFu};
use crate::utils;

pub fn run(path: &str, apply: bool, scan_all: bool, range: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        if apply {
            // Seed from the framework template so --apply works on first run.
            let template = project_root
                .join(".straymark")
                .join("templates")
                .join("follow-ups-backlog.md");
            if template.exists() {
                std::fs::copy(&template, &registry_path)?;
                utils::info(&format!(
                    "Created {} from the framework template.",
                    registry_path
                        .strip_prefix(project_root)
                        .unwrap_or(&registry_path)
                        .display()
                ));
            } else {
                return Err(anyhow!(
                    "No registry at {} and no template at .straymark/templates/follow-ups-backlog.md.\n  \
                     hint: run `straymark update-framework` (the template ships with fw-4.21.0+).",
                    registry_path.display()
                ));
            }
        } else {
            println!(
                "No follow-ups registry yet. Run `straymark followups drift --scan-all --apply` to create and seed it (see STRAYMARK.md §16)."
            );
            return Ok(());
        }
    }

    let registry = followups::parse_registry(&registry_path)?;
    for w in &registry.warnings {
        utils::warn(w);
    }

    // Candidate AILOG files.
    let agent_logs = ailog::agent_logs_dir(project_root);
    let candidates: Vec<PathBuf> = if scan_all {
        walk_ailogs(&agent_logs)
    } else {
        ailogs_in_git_range(project_root, &agent_logs, range)
    };

    // Drift = AILOG with follow-up content whose id is NOT in
    // fully_extracted_ailogs.
    let extracted_set: std::collections::HashSet<&str> = registry
        .frontmatter
        .fully_extracted_ailogs
        .iter()
        .map(|s| s.as_str())
        .collect();

    let mut drifted: Vec<(String, PathBuf, Vec<ExtractedFu>)> = Vec::new();
    for path in candidates {
        let Some(id) = followups::ailog_id_from_path(&path) else {
            continue;
        };
        if extracted_set.contains(id.as_str()) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let found = followups::extract_followups_from_ailog(&content);
        if !found.is_empty() {
            drifted.push((id, path, found));
        }
    }

    if drifted.is_empty() {
        println!(
            "{} registry in sync — no AILOGs with unextracted follow-up content.",
            "OK".green().bold()
        );
        // --apply always reconciles the CLI-owned counters, even with zero
        // extractions (#222 Finding 1): after a sanctioned manual-triage
        // session the statuses changed but nothing is left to extract, and
        // without this write the frontmatter stays knowingly stale.
        if apply {
            let counters = followups::compute_counters(&registry);
            let fm =
                followups::fm_apply_counters_and_v1(&registry.frontmatter_raw, &counters);
            if fm != registry.frontmatter_raw {
                let was_v0 = registry.is_v0();
                std::fs::write(&registry_path, followups::assemble(&fm, &registry.body))?;
                println!(
                    "  Counters recomputed: {} open / {} suspected-closed / {} promoted (total {}).",
                    counters.open, counters.suspected_closed, counters.promoted, counters.total
                );
                if was_v0 {
                    utils::info(
                        "Registry upgraded to schema v1 (non-destructive — counters are now CLI-owned).",
                    );
                }
            }
        }
        return Ok(());
    }

    if !apply {
        eprintln!(
            "{} {} AILOG(s) have follow-up content not yet in the registry:",
            "WARNING:".red().bold(),
            drifted.len()
        );
        for (id, path, found) in &drifted {
            eprintln!(
                "  - {} ({} entr{}) — {}",
                id,
                found.len(),
                if found.len() == 1 { "y" } else { "ies" },
                path.strip_prefix(project_root).unwrap_or(path).display()
            );
        }
        eprintln!();
        eprintln!("  Action: run `straymark followups drift --apply` to extract them.");
        std::process::exit(1);
    }

    // ── --apply: extract into the registry ──
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut body = registry.body.clone();
    let mut next_n = followups::next_fu_number(&registry);
    let mut added = 0usize;
    let mut suspected = 0usize;

    for (id, _path, found) in &drifted {
        for fu in found {
            // Re-parse spans against the evolving body so each insert lands
            // at the (possibly moved) end of the ready bucket.
            let current =
                followups::parse_registry_str(&registry_path, &followups::assemble(&registry.frontmatter_raw, &body))?;
            let block = followups::render_new_entry(next_n, fu, id, &today);
            body = followups::insert_into_bucket(&current, "ready", &block);
            if fu.suspected_closed {
                suspected += 1;
            }
            next_n += 1;
            added += 1;
        }
    }

    // Frontmatter: append AILOG ids, set last_scan, recompute counters + v1.
    let new_ids: Vec<String> = drifted.iter().map(|(id, _, _)| id.clone()).collect();
    let mut fm = followups::fm_append_list_items(
        &registry.frontmatter_raw,
        "fully_extracted_ailogs",
        &new_ids,
    );
    fm = followups::fm_set_scalar(&fm, "last_scan", &today);
    let final_registry =
        followups::parse_registry_str(&registry_path, &followups::assemble(&fm, &body))?;
    let counters = followups::compute_counters(&final_registry);
    fm = followups::fm_apply_counters_and_v1(&fm, &counters);

    let was_v0 = registry.is_v0();
    std::fs::write(&registry_path, followups::assemble(&fm, &body))?;

    utils::success(&format!(
        "Extracted {} entr{} from {} AILOG(s) into `## Bucket: ready`.",
        added,
        if added == 1 { "y" } else { "ies" },
        drifted.len()
    ));
    if suspected > 0 {
        println!(
            "  {} {} extracted as {} (closure marker in source AILOG) — confirm at the next triage.",
            "!".magenta().bold(),
            suspected,
            "suspected-closed".magenta()
        );
    }
    if was_v0 {
        utils::info("Registry upgraded to schema v1 (non-destructive — counters are now CLI-owned).");
    }
    println!(
        "  Counters recomputed: {} open / {} suspected-closed / {} promoted (total {}).",
        counters.open, counters.suspected_closed, counters.promoted, counters.total
    );
    println!("  Next: reclassify bucket/trigger/destination at triage (`straymark followups list --bucket ready`).");
    Ok(())
}

/// All AILOG .md files under the agent-logs directory (recursive).
fn walk_ailogs(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_ailogs(&path));
        } else if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("AILOG-") && n.ends_with(".md"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// AILOG files touched in the given git range. Range resolution:
/// explicit `--range` > `origin/main..HEAD` > `origin/master..HEAD` >
/// `HEAD~1..HEAD` (with a warning).
fn ailogs_in_git_range(
    project_root: &Path,
    agent_logs: &Path,
    range: Option<&str>,
) -> Vec<PathBuf> {
    let range = match range {
        Some(r) => r.to_string(),
        None => {
            if git_ref_exists(project_root, "origin/main") {
                "origin/main..HEAD".to_string()
            } else if git_ref_exists(project_root, "origin/master") {
                "origin/master..HEAD".to_string()
            } else {
                utils::warn("No origin/main or origin/master — falling back to HEAD~1..HEAD.");
                "HEAD~1..HEAD".to_string()
            }
        }
    };

    let output = Command::new("git")
        .args(["diff", "--name-only", &range])
        .current_dir(project_root)
        .output();
    let Ok(output) = output else {
        utils::warn("git not available — nothing to scan (use --scan-all for a filesystem sweep).");
        return Vec::new();
    };
    if !output.status.success() {
        utils::warn(&format!(
            "`git diff --name-only {}` failed — nothing to scan (use --scan-all for a filesystem sweep).",
            range
        ));
        return Vec::new();
    }

    let agent_logs_rel = agent_logs
        .strip_prefix(project_root)
        .unwrap_or(agent_logs)
        .to_path_buf();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(PathBuf::from)
        .filter(|p| {
            p.starts_with(&agent_logs_rel)
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("AILOG-") && n.ends_with(".md"))
                    .unwrap_or(false)
        })
        .map(|p| project_root.join(p))
        .filter(|p| p.exists())
        .collect()
}

fn git_ref_exists(project_root: &Path, r: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", r])
        .current_dir(project_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
