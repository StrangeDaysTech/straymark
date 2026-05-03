//! `devtrail charter drift` — file-vs-commit drift check at Charter close.
//!
//! Wraps the framework's `.devtrail/scripts/check-charter-drift.sh` (ported
//! from Sentinel `scripts/check-plan-drift.sh`, validated empirically with
//! zero false positives across PLAN-05 retrospective + PLAN-06 prospective).
//! The CLI value-add over the raw script is **AILOG-awareness** (mitigation
//! R2 of the Sentinel experiment): suppresses alerts on paths already
//! documented as a risk in any AILOG referenced by the Charter's
//! `originating_ailogs` frontmatter.
//!
//! ## Scope (Phase 2)
//!
//! Bash delegation only. Windows native (no WSL, no Git Bash) currently has
//! no path; document that limitation in the user-facing message and direct
//! Windows users to WSL. A Rust-native fallback is feasible but deferred
//! until a real adopter reports the need.
//!
//! ## Exit codes
//!
//! - `0` — no drift, or only documented out-of-scope extras / AILOG-suppressed
//! - `1` — drift found that needs attention
//! - `2` — usage error (Charter not found, bash missing, etc.)

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::charter::{self, Charter};
use crate::utils;

const DEFAULT_RANGE: &str = "HEAD~1..HEAD";

pub fn run(
    path: &str,
    charter_id: &str,
    range: Option<&str>,
    no_ailog_suppress: bool,
) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("DevTrail not installed. Run 'devtrail init' first."))?;
    let project_root = &resolved.path;
    let devtrail_dir = project_root.join(".devtrail");

    // Resolve the Charter file.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let charter = charter::find_by_id(&charters, charter_id)
        .ok_or_else(|| anyhow!("Charter {} not found in docs/charters/", charter_id))?
        .clone();

    let charter_path_rel = charter
        .path
        .strip_prefix(project_root)
        .unwrap_or(&charter.path)
        .to_path_buf();

    // Locate the drift script.
    let script_path = devtrail_dir.join("scripts").join("check-charter-drift.sh");
    if !script_path.exists() {
        bail!(
            "Drift script not found at {}. Run `devtrail repair` to restore framework files \
             (the script ships with fw-4.6.0 and later).",
            script_path.display()
        );
    }

    // Locate bash.
    let bash = which_bash().ok_or_else(|| {
        anyhow!(
            "`bash` not found in PATH. The drift check delegates to a bash script. \
             On Windows native, install Git Bash or WSL; a pure-Rust fallback is on \
             the roadmap but not in fw-4.6.0."
        )
    })?;

    let range_arg = range.unwrap_or(DEFAULT_RANGE);

    // Run the script. cwd = project_root so git ranges and relative Charter
    // paths work.
    let output = Command::new(&bash)
        .arg(&script_path)
        .arg(&charter_path_rel)
        .arg(range_arg)
        .current_dir(project_root)
        .output()
        .with_context(|| format!("Failed to invoke bash script at {}", script_path.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let raw_exit = output.status.code().unwrap_or(-1);

    // Parse the script output to identify declared-but-not-modified paths.
    // Scope-expansion ("Modified but NOT declared") is informational and not
    // suppressed — those paths are not in the Charter's declared list and
    // rarely overlap with documented risks.
    let omitted = extract_omitted_paths(&stdout);

    // O3 (cli-3.8.1, issue #91): always compute what AILOG-aware suppression
    // would have done, regardless of the flag. The flag only controls
    // whether to APPLY that suppression to the rendered output. This lets us
    // emit a confirming INFO line when --no-ailog-suppress is passed (the
    // operator opted into the diagnostic mode and deserves visible signal,
    // even when N=0).
    let would_have_suppressed: Vec<(String, String)> = if omitted.is_empty() {
        Vec::new()
    } else {
        compute_ailog_suppressions(project_root, &charter, &omitted)?
    };
    let suppressions: Vec<(String, String)> = if no_ailog_suppress {
        Vec::new()
    } else {
        would_have_suppressed.clone()
    };
    let suppressed_paths: std::collections::HashSet<String> =
        suppressions.iter().map(|(p, _)| p.clone()).collect();
    let omitted_after: Vec<String> = omitted
        .iter()
        .filter(|p| !suppressed_paths.contains(*p))
        .cloned()
        .collect();

    // Render the report. We re-render rather than passing raw stdout because
    // suppression can change the conclusion.
    print!("{}", stdout);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    if !suppressions.is_empty() {
        println!();
        println!(
            "{} {}",
            "AILOG-suppressed:".cyan().bold(),
            format!("{} path(s)", suppressions.len()).dimmed()
        );
        for (path, ailog_id) in &suppressions {
            println!("  - {} [documented in {}]", path, ailog_id.dimmed());
        }
    }

    // O3 (cli-3.8.1): when --no-ailog-suppress is passed, always emit at
    // least one line confirming dispatch — closes the "default and
    // --no-ailog-suppress produce byte-identical output at N=0" ambiguity
    // reported in Sentinel CHARTER-02 telemetry. Issue #91 vote: option
    // (c) "--no-ailog-suppress only", with explicit confirmation when N=0.
    if no_ailog_suppress {
        println!();
        let n = would_have_suppressed.len();
        if n == 0 {
            println!(
                "{} AILOG-aware suppression bypassed (would have suppressed: 0 paths)",
                "INFO:".cyan().bold()
            );
        } else {
            println!(
                "{} AILOG-aware suppression bypassed (would have suppressed: {} path(s) listed above as drift)",
                "INFO:".cyan().bold(),
                n
            );
            for (path, ailog_id) in &would_have_suppressed {
                println!(
                    "  - {} [would suppress: {}]",
                    path,
                    ailog_id.dimmed()
                );
            }
        }
    }

    // Final exit code: if AILOG-suppression cleared all omitted paths, the
    // script-reported exit is overridden to 0 (the user did the right thing
    // by documenting the risk in an AILOG).
    let final_exit = if raw_exit == 1 && omitted_after.is_empty() && !suppressions.is_empty() {
        println!();
        println!(
            "{} all declared-omitted paths are documented in AILOGs — drift accepted.",
            "OK".green().bold()
        );
        0
    } else {
        raw_exit
    };

    if final_exit != 0 {
        std::process::exit(final_exit);
    }
    Ok(())
}

/// Locate `bash` in PATH. Returns `None` if not found. We resolve it
/// explicitly (rather than letting `Command::new("bash")` do PATH lookup at
/// spawn time) so we can produce a friendlier error before spawning.
fn which_bash() -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_env) {
        for candidate in ["bash", "bash.exe"] {
            let p = dir.join(candidate);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Extract the list of declared-but-not-modified file paths from the drift
/// script's stdout. The script's relevant block looks like:
///
/// ```text
/// WARNING: Declared in Charter but NOT modified (3 files):
///   - path/one.go
///   - path/two.md
///   - path/three.yaml
///
///   Action: ...
/// ```
///
/// We scan from the WARNING line until a blank line or a non-bullet line
/// (the first sub-bullet that isn't a path is `  Action:` text).
fn extract_omitted_paths(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_section = false;
    for line in stdout.lines() {
        if line.starts_with("WARNING: Declared in Charter but NOT modified") {
            in_section = true;
            continue;
        }
        if in_section {
            // The script emits `  - <path>` bullets, then a blank line, then
            // an indented `  Action:` paragraph. We stop at the first line
            // that isn't a `  - ` bullet.
            if let Some(rest) = line.strip_prefix("  - ") {
                let path = rest.trim();
                if !path.is_empty() {
                    out.push(path.to_string());
                }
                continue;
            }
            // Any other line ends the section.
            in_section = false;
        }
    }
    out
}

/// For each declared-omitted path, check whether it appears in the `## Risk`
/// (EN), `## Riesgos` (ES), or `## 风险` (zh-CN) section of any AILOG referenced
/// by the Charter's `originating_ailogs`. Returns one (path, ailog_id) tuple
/// per path that was suppressed, with the first matching AILOG.
fn compute_ailog_suppressions(
    project_root: &Path,
    charter: &Charter,
    omitted: &[String],
) -> Result<Vec<(String, String)>> {
    let mut hits = Vec::new();
    let ailog_ids = match &charter.frontmatter.originating_ailogs {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => return Ok(hits),
    };

    let agent_logs_dir = project_root
        .join(".devtrail")
        .join("07-ai-audit")
        .join("agent-logs");
    if !agent_logs_dir.exists() {
        return Ok(hits);
    }

    // Collect Risk sections from all referenced AILOGs once.
    let mut risk_blobs: Vec<(String, String)> = Vec::new();
    for ailog_id in &ailog_ids {
        if let Some(path) = find_ailog_file(&agent_logs_dir, ailog_id) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(risk) = extract_risk_section(&content) {
                    risk_blobs.push((ailog_id.clone(), risk));
                }
            }
        }
    }

    for path in omitted {
        for (ailog_id, blob) in &risk_blobs {
            if blob.contains(path) {
                hits.push((path.clone(), ailog_id.clone()));
                break;
            }
        }
    }
    Ok(hits)
}

/// Find the AILOG file matching the given AILOG ID. Searches the agent-logs
/// directory recursively and matches by filename prefix (since AILOG files are
/// named `AILOG-YYYY-MM-DD-NNN-slug.md`).
fn find_ailog_file(agent_logs_dir: &Path, ailog_id: &str) -> Option<PathBuf> {
    // The id may be either bare (AILOG-2026-05-02-001) or include a slug
    // (AILOG-2026-05-02-001-foo). We match the bare prefix.
    let prefix: String = ailog_id
        .split('-')
        .take(5) // "AILOG", "YYYY", "MM", "DD", "NNN"
        .collect::<Vec<_>>()
        .join("-");
    walk_for_prefix(agent_logs_dir, &prefix)
}

fn walk_for_prefix(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = walk_for_prefix(&path, prefix) {
                return Some(found);
            }
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(prefix) && name.ends_with(".md") {
                return Some(path);
            }
        }
    }
    None
}

/// Extract the `## Risk` / `## Riesgos` / `## 风险` section body from an AILOG.
/// Returns `None` if no such section exists.
fn extract_risk_section(content: &str) -> Option<String> {
    let mut buf = String::new();
    let mut in_section = false;
    for line in content.lines() {
        if line.starts_with("## ")
            && (line.contains("Risk") || line.contains("Riesgo") || line.contains("风险"))
        {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with("## ") {
                break;
            }
            buf.push_str(line);
            buf.push('\n');
        }
    }
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_omitted_handles_typical_output() {
        let stdout = r#"=== Charter drift check ===
  Charter: docs/charters/01-foo.md
  Range:   HEAD~1..HEAD
  Declared: 5 files
  Modified: 3 files

WARNING: Declared in Charter but NOT modified (2 files):
  - src/services/policy/handler.go
  - src/services/policy/repository.go

  Action: either complete the work, or document in AILOG.
"#;
        let omitted = extract_omitted_paths(stdout);
        assert_eq!(
            omitted,
            vec![
                "src/services/policy/handler.go".to_string(),
                "src/services/policy/repository.go".to_string()
            ]
        );
    }

    #[test]
    fn extract_omitted_handles_empty_output() {
        let stdout = "OK No drift detected.\n";
        assert!(extract_omitted_paths(stdout).is_empty());
    }

    #[test]
    fn extract_risk_section_finds_english_section() {
        let ailog = r#"# AILOG-2026-04-28-024 — Test

## Context

Body.

## Risk

- **R1**: see `src/services/policy/handler.go` for context.
- **R2**: see `src/services/policy/repository.go`.

## Outcome

Done.
"#;
        let risk = extract_risk_section(ailog).unwrap();
        assert!(risk.contains("src/services/policy/handler.go"));
        assert!(risk.contains("src/services/policy/repository.go"));
        assert!(!risk.contains("Done."));
    }

    #[test]
    fn extract_risk_section_finds_spanish_section() {
        let ailog = r#"## Riesgos

- R1 — `src/foo.rs` puede fallar.

## Cierre

bla.
"#;
        let risk = extract_risk_section(ailog).unwrap();
        assert!(risk.contains("src/foo.rs"));
        assert!(!risk.contains("Cierre"));
    }

    #[test]
    fn extract_risk_section_returns_none_when_absent() {
        let ailog = "## Context\n\nNo risks here.\n";
        assert!(extract_risk_section(ailog).is_none());
    }

    #[test]
    fn find_ailog_file_matches_prefix_with_slug() {
        let tmp = tempfile::TempDir::new().unwrap();
        let agent_logs = tmp.path().join("agent-logs");
        std::fs::create_dir_all(&agent_logs).unwrap();
        let path = agent_logs.join("AILOG-2026-04-28-024-implement-baseline.md");
        std::fs::write(&path, "stub\n").unwrap();

        let found = find_ailog_file(&agent_logs, "AILOG-2026-04-28-024").unwrap();
        assert_eq!(found, path);

        // Also match when the full id with slug is given.
        let found2 = find_ailog_file(&agent_logs, "AILOG-2026-04-28-024-implement-baseline").unwrap();
        assert_eq!(found2, path);
    }

    #[test]
    fn find_ailog_file_recurses_into_subdirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        let nested = tmp.path().join("agent-logs").join("daemon");
        std::fs::create_dir_all(&nested).unwrap();
        let path = nested.join("AILOG-2026-05-01-002-foo.md");
        std::fs::write(&path, "stub\n").unwrap();

        let found = find_ailog_file(&tmp.path().join("agent-logs"), "AILOG-2026-05-01-002").unwrap();
        assert_eq!(found, path);
    }
}
