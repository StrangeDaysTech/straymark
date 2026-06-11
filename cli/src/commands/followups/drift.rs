//! `straymark followups drift` — keep the registry in sync with AILOGs.
//!
//! Native Rust replacement (cli-3.19.0) for the deprecated adopter-side
//! `check-followups-drift.sh`. Three modes, mirroring the v0 script:
//!
//! - **default** — scan AILOGs changed in `git diff origin/main..HEAD`
//!   (fallback `origin/master..HEAD`, then `HEAD~1..HEAD` with a warning)
//!   **unioned with the working tree** (`git status --porcelain`) so an
//!   uncommitted/untracked AILOG is visible to the documented pre-commit flow
//!   (#229). Warn + exit 1 when an AILOG carries follow-up content not yet in
//!   the registry.
//! - **`--apply`** — same scan + extract new entries into `## Bucket: ready`,
//!   append the AILOG ids to `fully_extracted_ailogs`, **recompute the
//!   `total_*` counters** (#214 Signal 2) and upgrade v0 → v1 in place.
//!   Entries whose AILOG text carries a closure marker are extracted as
//!   `suspected-closed` instead of `open` (#214 Signal 1 — the anti-noise
//!   refinement).
//! - **`--scan-all`** — sweep every AILOG in the project.
//!
//! Drift is detected **per follow-up by content hash** (`fu_content_hash`),
//! not by a whole-AILOG idempotency gate. Already-extracted AILOGs are
//! re-scanned and individual follow-ups deduped against the registry, so
//! content appended to a multi-batch AILOG after its first extraction is
//! caught instead of silently skipped (#231). The hash dedup preserves the
//! empirically validated zero-false-positive property (no AILOG re-flags once
//! its content is in the registry). See `FOLLOW-UPS-BACKLOG-PATTERN.md`
//! §Drift detection.
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

    // Candidate AILOG files. Default mode unions the committed git range with
    // the working tree (#229): at pre-commit time the just-written AILOG is
    // usually untracked, so a range-only scan is blind to exactly the file the
    // documented `drift --apply` flow targets.
    let agent_logs = ailog::agent_logs_dir(project_root);
    let candidates: Vec<PathBuf> = if scan_all {
        walk_ailogs(&agent_logs)
    } else {
        let mut set = ailogs_in_git_range(project_root, &agent_logs, range);
        for p in ailogs_in_working_tree(project_root, &agent_logs) {
            if !set.contains(&p) {
                set.push(p);
            }
        }
        set
    };

    // Drift = an AILOG carrying follow-up content whose content hash is NOT yet
    // in the registry (#231). Re-scanning already-extracted AILOGs and deduping
    // per follow-up by content hash — instead of skipping the whole file once
    // its id lands in `fully_extracted_ailogs` — is what catches follow-ups
    // appended to a multi-batch AILOG after its first extraction.
    let seen_hashes = followups::registry_extracted_hashes(&registry);

    let mut drifted: Vec<(String, PathBuf, Vec<ExtractedFu>)> = Vec::new();
    for path in candidates {
        let Some(id) = followups::ailog_id_from_path(&path) else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let new: Vec<ExtractedFu> = followups::extract_followups_from_ailog(&content)
            .into_iter()
            .filter(|fu| {
                !seen_hashes.contains(&followups::fu_content_hash(
                    &id,
                    &fu.origin_section,
                    &fu.description,
                ))
            })
            .collect();
        if !new.is_empty() {
            drifted.push((id, path, new));
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

    // Frontmatter: append AILOG ids (deduped — a re-scanned AILOG already in
    // the list must not be appended twice), set last_scan, recompute counters.
    let already: std::collections::HashSet<&str> = registry
        .frontmatter
        .fully_extracted_ailogs
        .iter()
        .map(|s| s.as_str())
        .collect();
    let mut new_ids: Vec<String> = Vec::new();
    for (id, _, _) in &drifted {
        if !already.contains(id.as_str()) && !new_ids.contains(id) {
            new_ids.push(id.clone());
        }
    }
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

/// AILOG files present in the working tree (staged, unstaged, or untracked),
/// via `git status --porcelain`. Unioned into the default scan so the
/// documented pre-commit `drift --apply` flow sees the just-written AILOG
/// before it is committed (#229) — mirroring the v0 reference script, which
/// folded `git status --porcelain` into its default scan set for exactly this
/// in-progress agent workflow.
fn ailogs_in_working_tree(project_root: &Path, agent_logs: &Path) -> Vec<PathBuf> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let agent_logs_rel = agent_logs
        .strip_prefix(project_root)
        .unwrap_or(agent_logs)
        .to_path_buf();
    parse_porcelain_ailogs(&String::from_utf8_lossy(&output.stdout), &agent_logs_rel)
        .into_iter()
        .map(|p| project_root.join(p))
        .filter(|p| p.exists())
        .collect()
}

/// Parse `git status --porcelain` (v1) output into AILOG file paths relative to
/// the repo root, filtered to the agent-logs dir. Handles the `XY <path>`
/// shape, untracked `?? <path>`, and rename/copy `R  <old> -> <new>` (takes the
/// destination). Pure for testability.
fn parse_porcelain_ailogs(stdout: &str, agent_logs_rel: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        // Porcelain v1: two status chars + a space, then the path (≥ 4 chars).
        if line.len() < 4 {
            continue;
        }
        let rest = &line[3..];
        // Rename/copy entries render as `old -> new`; the destination is what
        // exists on disk.
        let path_str = rest.rsplit(" -> ").next().unwrap_or(rest).trim();
        // Porcelain quotes paths containing special chars; strip the quotes.
        let path_str = path_str.trim_matches('"');
        let p = PathBuf::from(path_str);
        if p.starts_with(agent_logs_rel)
            && p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("AILOG-") && n.ends_with(".md"))
                .unwrap_or(false)
        {
            out.push(p);
        }
    }
    out
}

fn git_ref_exists(project_root: &Path, r: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", r])
        .current_dir(project_root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_porcelain_picks_ailogs_across_status_codes() {
        let rel = Path::new(".straymark/07-ai-audit/agent-logs");
        // Built via join so leading status spaces (` M`) survive verbatim — a
        // `\`-continued string literal would strip them.
        let stdout = [
            "?? .straymark/07-ai-audit/agent-logs/AILOG-2026-06-09-001-untracked.md",
            " M .straymark/07-ai-audit/agent-logs/AILOG-2026-06-09-002-modified.md",
            "A  .straymark/07-ai-audit/agent-logs/AILOG-2026-06-09-003-staged.md",
            "R  .straymark/07-ai-audit/agent-logs/AILOG-old.md -> .straymark/07-ai-audit/agent-logs/AILOG-2026-06-09-004-renamed.md",
            " M .straymark/07-ai-audit/agent-logs/notes.md",
            "?? src/main.rs",
            " M .straymark/07-ai-audit/decisions/AIDEC-2026-06-09-001.md",
        ]
        .join("\n");
        let got = parse_porcelain_ailogs(&stdout, rel);
        let names: Vec<String> = got
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            names,
            vec![
                "AILOG-2026-06-09-001-untracked.md",
                "AILOG-2026-06-09-002-modified.md",
                "AILOG-2026-06-09-003-staged.md",
                "AILOG-2026-06-09-004-renamed.md", // rename → destination path
            ]
        );
        // notes.md (not AILOG-*), src/main.rs (outside dir), and the AIDEC
        // (sibling dir) are all excluded.
    }

    #[test]
    fn parse_porcelain_tolerates_quoted_paths_and_short_lines() {
        let rel = Path::new(".straymark/07-ai-audit/agent-logs");
        let stdout = [
            "?? \".straymark/07-ai-audit/agent-logs/AILOG-2026-06-09-005-spaced name.md\"",
            "",
            "XX",
        ]
        .join("\n");
        let got = parse_porcelain_ailogs(&stdout, rel);
        assert_eq!(got.len(), 1);
        assert_eq!(
            got[0].file_name().unwrap().to_string_lossy(),
            "AILOG-2026-06-09-005-spaced name.md"
        );
    }
}
