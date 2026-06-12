//! `straymark charter drift` — file-vs-commit drift check at Charter close.
//!
//! **Native Rust (cli-3.23.0, #237).** Replaces the bash delegation to
//! `.straymark/scripts/check-charter-drift.sh` (now deprecated, kept as a
//! reference prototype like `check-followups-drift.sh`). The declared-files
//! parser was already pure Rust (`charter_files.rs`, byte-for-byte equivalent
//! to the script's awk extraction); this module ports the remaining ~30% — the
//! git-range diff, the wildcard set-difference, and the report — so the command
//! works on Windows-native (no WSL, no Git Bash). Zero-false-positives property
//! (Sentinel PLAN-05/PLAN-06) preserved by the equivalence test suite.
//!
//! Catches drift of **omission** (declared in the Charter, never modified) and
//! **scope expansion** (modified but not declared). The CLI value-add over the
//! raw script is **AILOG-awareness**: it suppresses omission alerts on paths
//! already documented as a risk in any AILOG referenced by the Charter's
//! `originating_ailogs` frontmatter.
//!
//! ## Exit codes
//!
//! - `0` — no drift, or only documented out-of-scope extras / AILOG-suppressed
//! - `1` — drift found that needs attention
//! - `2` — usage error (Charter not found, etc.)

use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

use crate::ailog;
use crate::charter::{self, Charter, CharterStatus};
use crate::charter_files;
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

    let range_arg = range.unwrap_or(DEFAULT_RANGE);

    // Native drift computation (#237) — formerly a delegation to the bash
    // script `check-charter-drift.sh`. `declared_files` reuses the same parser
    // (`charter_files.rs`) the script's awk extraction was ported into.
    let declared = declared_files(&charter.body);
    let modified = modified_files(project_root, range_arg);
    let (omitted, extra) = compute_drift(&declared, &modified);

    // Render the script-equivalent report. Omitted/extra are shown in full;
    // AILOG-aware suppression is applied below as a separate, additive layer
    // (scope-expansion extras are informational and never suppressed — they
    // are not in the declared list and rarely overlap documented risks).
    render_drift_report(&charter_path_rel, range_arg, &declared, &modified, &omitted, &extra);

    // The script exited 1 iff there were declared-but-not-modified paths;
    // scope expansion is informational and does not change the exit code.
    let raw_exit = if omitted.is_empty() { 0 } else { 1 };

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

/// Declared files from the Charter's `## Files to modify` section: sorted,
/// deduplicated path list. Mirrors the script's `awk | grep | sort -u`,
/// reusing `charter_files::parse_files_to_modify` (the shared, byte-for-byte
/// port of the awk extraction). Wildcard paths (`...` / `*`) are preserved.
fn declared_files(charter_body: &str) -> Vec<String> {
    let mut v: Vec<String> = charter_files::parse_files_to_modify(charter_body)
        .into_iter()
        .map(|f| f.path)
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Files modified in the git range: sorted, deduplicated. Empty on git failure
/// or no output — the script treats both as "nothing modified" (WARN, exit 0).
fn modified_files(project_root: &Path, range: &str) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .args(["diff", "--name-only", range])
        .current_dir(project_root)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let mut v: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Glob match where `*` matches any (possibly empty) run of characters and
/// every other character is literal, anchored over the whole string. Mirrors
/// the script's `sed 's/\./\\./g; s/\*/.*/g'` + `^…$` for path-like inputs.
fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text; // no wildcard → literal equality
    }
    if !text.starts_with(parts[0]) {
        return false;
    }
    let mut pos = parts[0].len();
    for (i, part) in parts.iter().enumerate().skip(1) {
        if i == parts.len() - 1 {
            // Last segment must end the string (and not overlap consumed prefix).
            return text.len() >= pos && text[pos..].ends_with(part);
        }
        if part.is_empty() {
            continue;
        }
        match text[pos..].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }
    true
}

/// True when a declared path's wildcard form is satisfied by `target`.
/// Ellipsis `prefix...suffix` → any path with that prefix; glob `prefix*suffix`
/// → [`glob_match`]. Returns `None` when `decl` is a literal path.
fn wildcard_satisfied_by(decl: &str, target: &str) -> Option<bool> {
    if let Some(idx) = decl.rfind("...") {
        Some(target.starts_with(&decl[..idx]))
    } else if decl.contains('*') {
        Some(glob_match(decl, target))
    } else {
        None
    }
}

/// Set-difference at Charter close: `(declared_omitted, modified_extra)`.
///
/// - **Omitted**: declared but not modified. A wildcard declaration is
///   satisfied by any modified path it matches; a literal needs an exact match.
/// - **Extra** (scope expansion): modified but not declared. Charter-doc and
///   AILOG paths are always in scope; a modified path also matched by any
///   declared wildcard is not extra.
fn compute_drift(declared: &[String], modified: &[String]) -> (Vec<String>, Vec<String>) {
    let omitted: Vec<String> = declared
        .iter()
        .filter(|decl| {
            !modified
                .iter()
                .any(|m| wildcard_satisfied_by(decl, m).unwrap_or_else(|| m == *decl))
        })
        .cloned()
        .collect();

    let extra: Vec<String> = modified
        .iter()
        .filter(|m| {
            if m.starts_with(".straymark/charters/") || m.starts_with(".straymark/07-ai-audit/") {
                return false;
            }
            if declared.iter().any(|d| d == *m) {
                return false;
            }
            // Allowed when a declared wildcard prefix/glob matches it.
            !declared
                .iter()
                .any(|decl| wildcard_satisfied_by(decl, m).unwrap_or(false))
        })
        .cloned()
        .collect();

    (omitted, extra)
}

/// Render the report — byte-for-byte equivalent to `check-charter-drift.sh`
/// stdout/stderr. The header is printed only when both sides are non-empty
/// (the script's WARN-and-exit-0 paths emit only to stderr).
fn render_drift_report(
    charter_path: &Path,
    range: &str,
    declared: &[String],
    modified: &[String],
    omitted: &[String],
    extra: &[String],
) {
    if declared.is_empty() {
        eprintln!(
            "WARN: no files extracted from §Files to modify in {}",
            charter_path.display()
        );
        eprintln!("  Either the section is missing, the table format is unusual, or the");
        eprintln!("  declared paths don't have recognized extensions. Script can't help — exit clean.");
        return;
    }
    if modified.is_empty() {
        eprintln!(
            "WARN: no files modified in range {} — Charter may not have executed.",
            range
        );
        return;
    }

    println!("=== Charter drift check ===");
    println!("  Charter: {}", charter_path.display());
    println!("  Range:   {}", range);
    println!("  Declared: {} files", declared.len());
    println!("  Modified: {} files", modified.len());
    println!();

    if !omitted.is_empty() {
        println!(
            "WARNING: Declared in Charter but NOT modified ({} files):",
            omitted.len()
        );
        for p in omitted {
            println!("  - {}", p);
        }
        println!();
        println!("  Action: either complete the work, or document in AILOG under '## Risk'");
        println!("  as 'R<N+1> (new, not in Charter)' explaining why this file did not need");
        println!("  changes (Charter was wrong, scope simplified, etc.).");
        println!();
    }

    if !extra.is_empty() {
        println!(
            "INFO: Modified but NOT declared ({} files, scope expansion):",
            extra.len()
        );
        for p in extra {
            println!("  - {}", p);
        }
        println!();
        println!("  Action: if intentional, document the scope expansion in AILOG.");
        println!("  Common reasons: mock updates after interface change, generated");
        println!("  files (e.g. wire_gen.go), pre-existing drift fix needed to unblock work.");
        println!();
    }

    if omitted.is_empty() && extra.is_empty() {
        println!("OK No drift detected. Charter and execution are in sync.");
    }
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

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn glob_match_mirrors_script_semantics() {
        assert!(glob_match("AILOG-*.md", "AILOG-2026-06-12-001.md"));
        assert!(glob_match("src/*.rs", "src/main.rs"));
        assert!(glob_match("a*b*c", "axxbyyc"));
        assert!(glob_match("*.md", "README.md"));
        assert!(glob_match("dir/*", "dir/anything/deep.rs")); // `*` spans `/`, like `.*`
        assert!(!glob_match("AILOG-*.md", "AILOG-2026.txt"));
        assert!(!glob_match("src/*.rs", "src/main.go"));
        // `.` is literal, not a regex metachar.
        assert!(!glob_match("a.b", "axb"));
        assert!(glob_match("a.b", "a.b"));
        // No wildcard → exact equality.
        assert!(glob_match("src/main.rs", "src/main.rs"));
        assert!(!glob_match("src/main.rs", "src/main.rs.bak"));
    }

    #[test]
    fn compute_drift_literal_omission_and_scope_expansion() {
        let declared = s(&["src/a.rs", "src/b.rs", "src/c.rs"]);
        let modified = s(&["src/a.rs", "src/d.rs"]);
        let (omitted, extra) = compute_drift(&declared, &modified);
        assert_eq!(omitted, s(&["src/b.rs", "src/c.rs"])); // declared, not modified
        assert_eq!(extra, s(&["src/d.rs"])); // modified, not declared
    }

    #[test]
    fn compute_drift_wildcards_and_in_scope_paths() {
        // Ellipsis + glob declarations satisfied by a matching modified path;
        // charter-doc and AILOG paths are always in scope (never "extra").
        let declared = s(&[
            ".straymark/07-ai-audit/agent-logs/AILOG-...md",
            "src/gen/*.rs",
            "src/lit.rs",
        ]);
        let modified = s(&[
            ".straymark/07-ai-audit/agent-logs/AILOG-2026-06-12-001.md",
            ".straymark/charters/05-x.md",
            "src/gen/wire.rs",
            "src/lit.rs",
        ]);
        let (omitted, extra) = compute_drift(&declared, &modified);
        assert!(omitted.is_empty(), "all declared satisfied (2 wildcards + 1 literal)");
        assert!(extra.is_empty(), "charter/AILOG paths in scope; gen/wire matches glob");
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
