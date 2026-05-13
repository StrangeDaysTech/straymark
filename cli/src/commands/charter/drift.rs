//! `straymark charter drift` — file-vs-commit drift check at Charter close.
//!
//! Wraps the framework's `.straymark/scripts/check-charter-drift.sh` (ported
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

use crate::ailog;
use crate::charter::{self, Charter, CharterStatus};
use crate::utils;

const DEFAULT_RANGE: &str = "HEAD~1..HEAD";

pub fn run(
    path: &str,
    charter_id: &str,
    range: Option<&str>,
    no_ailog_suppress: bool,
    no_batch_ledger_check: bool,
) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;
    let straymark_dir = project_root.join(".straymark");

    // Resolve the Charter file.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let charter = charter::find_by_id(&charters, charter_id)
        .ok_or_else(|| {
            anyhow!(
                "Charter {} not found in .straymark/charters/.\n  hint: run `straymark charter list` to see discovered Charters.",
                charter_id
            )
        })?
        .clone();

    let charter_path_rel = charter
        .path
        .strip_prefix(project_root)
        .unwrap_or(&charter.path)
        .to_path_buf();

    // Locate the drift script.
    let script_path = straymark_dir.join("scripts").join("check-charter-drift.sh");
    if !script_path.exists() {
        bail!(
            "Drift script not found at {}. Run `straymark repair` to restore framework files \
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

    // Final exit code from file-vs-commit drift: if AILOG-suppression cleared
    // all omitted paths, the script-reported exit is overridden to 0 (the user
    // did the right thing by documenting the risk in an AILOG).
    let drift_exit = if raw_exit == 1 && omitted_after.is_empty() && !suppressions.is_empty() {
        println!();
        println!(
            "{} all declared-omitted paths are documented in AILOGs — drift accepted.",
            "OK".green().bold()
        );
        0
    } else {
        raw_exit
    };

    // Batch Ledger gate (cli-3.13.0, GH #146): if the Charter is past `declared`
    // and any AILOG `### Batch N` is still `(pending)`, reject. Skipped when:
    //   - `--no-batch-ledger-check` flag is set
    //   - Charter status is `declared` (nothing has been executed yet)
    //   - the AILOG does not have a `## Batch Ledger` section (opt-in pattern)
    let mut ledger_failures: Vec<(String, Vec<u32>)> = Vec::new();
    if !no_batch_ledger_check
        && !matches!(charter.frontmatter.status, CharterStatus::Declared)
    {
        if let Some(ids) = &charter.frontmatter.originating_ailogs {
            ledger_failures = collect_pending_batch_failures(project_root, ids);
        }
    }

    if !ledger_failures.is_empty() {
        println!();
        println!(
            "{} {} AILOG(s) have `### Batch N` entries still marked `(pending)`:",
            "WARNING:".red().bold(),
            ledger_failures.len()
        );
        for (ailog_id, batches) in &ledger_failures {
            let list = batches
                .iter()
                .map(|n| format!("Batch {}", n))
                .collect::<Vec<_>>()
                .join(", ");
            println!("  - {}: {}", ailog_id, list);
        }
        println!();
        println!(
            "  Action: run `straymark charter batch-complete {} <N>` for each pending batch,\n          or pass `--no-batch-ledger-check` if the ledger is intentionally consolidated post-close.",
            charter.frontmatter.charter_id
        );
    }

    let final_exit = if !ledger_failures.is_empty() {
        1
    } else {
        drift_exit
    };

    if final_exit != 0 {
        std::process::exit(final_exit);
    }
    Ok(())
}

/// For each AILOG referenced by the Charter, parse its `## Batch Ledger` and
/// collect (ailog_id, [pending batch numbers]) when at least one batch is
/// pending. AILOGs without a ledger contribute nothing — the section is
/// opt-in.
fn collect_pending_batch_failures(
    project_root: &Path,
    ailog_ids: &[String],
) -> Vec<(String, Vec<u32>)> {
    let agent_logs_dir = ailog::agent_logs_dir(project_root);
    if !agent_logs_dir.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for ailog_id in ailog_ids {
        let Some(path) = ailog::find_ailog_file(&agent_logs_dir, ailog_id) else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let pending = ailog::pending_batches(&content);
        if !pending.is_empty() {
            out.push((ailog_id.clone(), pending));
        }
    }
    out
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

    let agent_logs_dir = ailog::agent_logs_dir(project_root);
    if !agent_logs_dir.exists() {
        return Ok(hits);
    }

    // Collect Risk sections from all referenced AILOGs once.
    let mut risk_blobs: Vec<(String, String)> = Vec::new();
    for ailog_id in &ailog_ids {
        if let Some(path) = ailog::find_ailog_file(&agent_logs_dir, ailog_id) {
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
  Charter: .straymark/charters/01-foo.md
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

    // Note: `find_ailog_file` tests now live in `crate::ailog` since the
    // function was promoted to the shared module for use by `batch_complete`.

    #[test]
    fn collect_pending_batch_failures_finds_pending_entries() {
        let tmp = tempfile::TempDir::new().unwrap();
        let agent_logs = tmp.path().join(".straymark/07-ai-audit/agent-logs");
        std::fs::create_dir_all(&agent_logs).unwrap();
        let ailog = agent_logs.join("AILOG-2026-05-13-001-test.md");
        std::fs::write(
            &ailog,
            r#"# AILOG
## Batch Ledger

### Batch 1 — Setup

Done.

### Batch 2 — Impl

(pending)

### Batch 3 — Tests

(pending)
"#,
        )
        .unwrap();

        let report =
            collect_pending_batch_failures(tmp.path(), &["AILOG-2026-05-13-001".to_string()]);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].0, "AILOG-2026-05-13-001");
        assert_eq!(report[0].1, vec![2, 3]);
    }

    #[test]
    fn collect_pending_batch_failures_skips_when_ledger_absent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let agent_logs = tmp.path().join(".straymark/07-ai-audit/agent-logs");
        std::fs::create_dir_all(&agent_logs).unwrap();
        let ailog = agent_logs.join("AILOG-2026-05-13-002-test.md");
        std::fs::write(&ailog, "# AILOG\n\n## Actions Performed\n\n1. Stuff.\n").unwrap();

        let report =
            collect_pending_batch_failures(tmp.path(), &["AILOG-2026-05-13-002".to_string()]);
        assert!(report.is_empty(), "no ledger → no gate");
    }

    #[test]
    fn collect_pending_batch_failures_passes_when_all_filled() {
        let tmp = tempfile::TempDir::new().unwrap();
        let agent_logs = tmp.path().join(".straymark/07-ai-audit/agent-logs");
        std::fs::create_dir_all(&agent_logs).unwrap();
        let ailog = agent_logs.join("AILOG-2026-05-13-003-test.md");
        std::fs::write(
            &ailog,
            r#"# AILOG
## Batch Ledger

### Batch 1 — Setup

Done.

### Batch 2 — Impl

Done too.
"#,
        )
        .unwrap();

        let report =
            collect_pending_batch_failures(tmp.path(), &["AILOG-2026-05-13-003".to_string()]);
        assert!(report.is_empty(), "all batches filled → no failures");
    }
}
