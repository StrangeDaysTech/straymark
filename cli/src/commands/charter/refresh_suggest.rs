//! `straymark charter refresh-suggest <module>` — read-only heuristic
//! recommending a pre-declare SpecKit refresh when a multi-Charter module's
//! recent telemetry shows the chain has accumulated enough emergent risk that
//! the next Charter is likely to inherit stale spec premises.
//!
//! See `dist/.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md` Pattern 1
//! for the canonical framework guidance this command operationalizes, and
//! Issue #156 for the original RFC.
//!
//! v0.2 / fw-4.16.0:
//! - Module match is a case-insensitive substring scan on `charter_id`.
//! - Only `status: closed` Charters with a parseable
//!   `agent_quality.r_n_plus_one_emergent_count` are eligible.
//! - Ranking is by `closed_at` (descending), top 3 used for the rolling mean.
//! - Default threshold is 6; override with `--threshold N`.
//! - Detection of "has a refresh PR landed since the chain branch point" is
//!   deferred to a future iteration — out of scope for v0 (the rolling-mean
//!   signal carries the heuristic on its own per Sentinel telemetry).
//!
//! Exit code is always 0 — this is informational, never a CI gate.

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::charter::{discover_and_parse, Charter, CharterStatus};

const DEFAULT_THRESHOLD: u32 = 6;
const ROLLING_WINDOW: usize = 3;

pub fn run(project_path: &str, module: &str, threshold: Option<u32>) -> Result<()> {
    let project_root = Path::new(project_path);
    let module = module.trim();
    if module.is_empty() {
        anyhow::bail!("<module> argument must be non-empty");
    }

    let (charters, errors) = discover_and_parse(project_root);
    if !errors.is_empty() {
        eprintln!(
            "  {} {} Charter file(s) could not be parsed and were skipped.",
            "⚠".yellow().bold(),
            errors.len()
        );
        for (path, err) in &errors {
            eprintln!("    {} → {}", path.display(), err);
        }
        eprintln!();
    }

    let module_lower = module.to_ascii_lowercase();
    let mut module_charters: Vec<&Charter> = charters
        .iter()
        .filter(|c| matches!(c.frontmatter.status, CharterStatus::Closed))
        .filter(|c| {
            c.frontmatter
                .charter_id
                .to_ascii_lowercase()
                .contains(&module_lower)
        })
        .collect();

    if module_charters.is_empty() {
        println!(
            "  {} No closed Charters match module `{}` (substring on charter_id).",
            "ℹ".cyan().bold(),
            module
        );
        println!("  {}", "Recommendation: nothing to suggest yet.".dimmed());
        return Ok(());
    }

    let mut rows: Vec<CharterRow> = module_charters
        .drain(..)
        .map(|c| load_charter_row(project_root, c))
        .collect::<Result<Vec<_>>>()?;

    // Closed-at descending. Charters without a parseable closed_at sink to the
    // bottom so they are still visible in the table but never seed the rolling
    // mean.
    rows.sort_by(|a, b| b.closed_at.cmp(&a.closed_at));

    let threshold = threshold.unwrap_or(DEFAULT_THRESHOLD);
    let window: Vec<&CharterRow> = rows
        .iter()
        .filter(|r| r.r_n_plus_one.is_some())
        .take(ROLLING_WINDOW)
        .collect();

    print_table(&rows, &window);

    let rolling_mean = if window.is_empty() {
        None
    } else {
        let sum: u32 = window
            .iter()
            .map(|r| r.r_n_plus_one.unwrap_or(0))
            .sum();
        Some(sum as f64 / window.len() as f64)
    };

    print_recommendation(module, threshold, &window, rolling_mean);

    Ok(())
}

struct CharterRow {
    charter_id: String,
    closed_at: Option<String>,
    r_n_plus_one: Option<u32>,
    telemetry_path: Option<PathBuf>,
}

fn load_charter_row(project_root: &Path, charter: &Charter) -> Result<CharterRow> {
    let telemetry_path = telemetry_path_for(project_root, charter);
    let (closed_at, r_n_plus_one) = match telemetry_path.as_ref() {
        Some(p) if p.exists() => parse_telemetry(p)
            .with_context(|| format!("Failed to parse telemetry at {}", p.display()))?,
        _ => (None, None),
    };

    Ok(CharterRow {
        charter_id: charter.frontmatter.charter_id.clone(),
        closed_at,
        r_n_plus_one,
        telemetry_path,
    })
}

/// Locate the telemetry sidecar for a Charter. Charters live at
/// `.straymark/charters/<NN-slug>.md`; telemetry sits next to them as
/// `<NN-slug>.telemetry.yaml` per the convention in charter.rs.
fn telemetry_path_for(_project_root: &Path, charter: &Charter) -> Option<PathBuf> {
    let stem = charter.path.file_stem()?.to_str()?;
    let parent = charter.path.parent()?;
    Some(parent.join(format!("{stem}.telemetry.yaml")))
}

fn parse_telemetry(path: &Path) -> Result<(Option<String>, Option<u32>)> {
    let content = std::fs::read_to_string(path)?;
    let value: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let ct = value
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("charter_telemetry".into())));
    let Some(ct) = ct else {
        return Ok((None, None));
    };
    let mapping = match ct.as_mapping() {
        Some(m) => m,
        None => return Ok((None, None)),
    };
    let closed_at = mapping
        .get(serde_yaml::Value::String("closed_at".into()))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let r_n_plus_one = mapping
        .get(serde_yaml::Value::String("agent_quality".into()))
        .and_then(|aq| aq.as_mapping())
        .and_then(|aq| aq.get(serde_yaml::Value::String("r_n_plus_one_emergent_count".into())))
        .and_then(|v| v.as_u64())
        .and_then(|n| u32::try_from(n).ok());
    Ok((closed_at, r_n_plus_one))
}

fn print_table(all: &[CharterRow], window: &[&CharterRow]) {
    println!();
    println!(
        "  {}",
        format!(
            "Closed Charters matched (window: top {} by closed_at)",
            ROLLING_WINDOW
        )
        .bold()
    );
    println!();
    println!(
        "    {:<40}  {:<12}  {:>10}  {}",
        "charter_id".dimmed(),
        "closed_at".dimmed(),
        "R<N+1>".dimmed(),
        "telemetry".dimmed()
    );
    println!(
        "    {:<40}  {:<12}  {:>10}  {}",
        "----------",
        "----------",
        "------",
        "---------"
    );
    for (idx, row) in all.iter().enumerate() {
        let in_window = window.iter().any(|w| w.charter_id == row.charter_id);
        let marker = if in_window { "●" } else { " " };
        let id_cell = if in_window {
            row.charter_id.as_str().green().bold().to_string()
        } else {
            row.charter_id.as_str().to_string()
        };
        let closed = row.closed_at.as_deref().unwrap_or("—").to_string();
        let r = row
            .r_n_plus_one
            .map(|n| n.to_string())
            .unwrap_or_else(|| "—".to_string());
        let tel = row
            .telemetry_path
            .as_ref()
            .map(|p| {
                if p.exists() {
                    p.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("(present)")
                        .to_string()
                } else {
                    "(missing)".to_string()
                }
            })
            .unwrap_or_else(|| "(no path)".to_string());
        let _ = idx;
        println!(
            "  {} {:<40}  {:<12}  {:>10}  {}",
            marker, id_cell, closed, r, tel.dimmed()
        );
    }
    println!();
}

fn print_recommendation(
    module: &str,
    threshold: u32,
    window: &[&CharterRow],
    rolling_mean: Option<f64>,
) {
    println!("  {}", "Heuristic".bold());
    println!(
        "    Chain length (closed Charters in window): {}",
        window.len()
    );
    println!("    Threshold for rolling mean:               > {}", threshold);
    match rolling_mean {
        Some(m) => println!(
            "    Rolling mean of agent_quality.r_n_plus_one_emergent_count: {:.2}",
            m
        ),
        None => println!(
            "    Rolling mean: {} (insufficient telemetry)",
            "—".dimmed()
        ),
    }
    println!();

    let chain_ok = window.len() >= ROLLING_WINDOW;
    let mean_exceeds = rolling_mean.map(|m| m > threshold as f64).unwrap_or(false);

    if chain_ok && mean_exceeds {
        println!(
            "  {} {}",
            "▶".green().bold(),
            format!(
                "Recommend a pre-declare SpecKit refresh for `{}` before the next Charter.",
                module
            )
            .bold()
        );
        println!(
            "  {}",
            "  See .straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md Pattern 1 for mechanics."
                .dimmed()
        );
        println!(
            "  {}",
            "  Telemetry slot: `charter_telemetry.pre_declare_refresh` in the next Charter."
                .dimmed()
        );
    } else if !chain_ok {
        println!(
            "  {} Chain shorter than {} closed Charters with telemetry — heuristic not yet meaningful.",
            "ℹ".cyan().bold(),
            ROLLING_WINDOW
        );
        println!(
            "  {}",
            "  Recommendation: continue per-Charter pattern; revisit after the next close.".dimmed()
        );
    } else {
        println!(
            "  {} Rolling mean is within threshold — refresh not recommended.",
            "✔".green().bold()
        );
        println!(
            "  {}",
            "  Per-Charter pattern alone is doing its job for this module.".dimmed()
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_telemetry(dir: &std::path::Path, name: &str, closed_at: &str, r_n: u32) {
        let path = dir.join(format!("{name}.telemetry.yaml"));
        let body = format!(
            "charter_telemetry:\n  \
             charter_id: \"{name}\"\n  \
             charter_title: \"x\"\n  \
             closed_at: \"{closed_at}\"\n  \
             effort:\n    estimated_effort: \"M\"\n    actual_effort: \"M\"\n  \
             agent_quality:\n    r_n_plus_one_emergent_count: {r_n}\n  \
             outcome:\n    completed_as_planned: true\n    scope_changes: \"ninguno\"\n"
        );
        std::fs::write(&path, body).unwrap();
    }

    fn write_charter(dir: &std::path::Path, name: &str) {
        let path = dir.join(format!("{name}.md"));
        let body = format!(
            "---\ncharter_id: {name}\nstatus: closed\neffort_estimate: M\ntrigger: x\n---\nbody\n"
        );
        std::fs::write(&path, body).unwrap();
    }

    #[test]
    fn parse_telemetry_extracts_closed_at_and_r_n_plus_one() {
        let tmp = tempfile::tempdir().unwrap();
        write_telemetry(tmp.path(), "10-x", "2026-05-01", 7);
        let (closed, r_n) =
            parse_telemetry(&tmp.path().join("10-x.telemetry.yaml")).unwrap();
        assert_eq!(closed.as_deref(), Some("2026-05-01"));
        assert_eq!(r_n, Some(7));
    }

    #[test]
    fn placeholder_telemetry_returns_none_for_missing_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("11.telemetry.yaml");
        std::fs::write(&path, "charter_telemetry:\n  charter_id: x\n").unwrap();
        let (closed, r_n) = parse_telemetry(&path).unwrap();
        assert_eq!(closed, None);
        assert_eq!(r_n, None);
    }

    #[test]
    fn module_filter_is_case_insensitive_substring_on_charter_id() {
        let tmp = tempfile::tempdir().unwrap();
        let charters_dir = tmp.path().join(".straymark/charters");
        std::fs::create_dir_all(&charters_dir).unwrap();
        write_charter(&charters_dir, "12-commshub-foo");
        write_charter(&charters_dir, "13-other");
        write_telemetry(&charters_dir, "12-commshub-foo", "2026-05-10", 8);
        write_telemetry(&charters_dir, "13-other", "2026-05-12", 3);

        // Just ensure the run completes; the printed table is non-fatal.
        run(tmp.path().to_str().unwrap(), "CommsHub", Some(6)).unwrap();
    }
}
