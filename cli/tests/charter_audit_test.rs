//! Integration tests for `straymark charter audit` (v1 unified flow).

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

const AUDIT_PROMPT_UNIFIED: &str = include_str!(
    "../../dist/.straymark/audit-prompts/audit-prompt.md"
);
const AUDIT_PROMPT_ES: &str = include_str!(
    "../../dist/.straymark/audit-prompts/i18n/es/audit-prompt.md"
);
const AUDIT_OUTPUT_SCHEMA: &str = include_str!(
    "../../dist/.straymark/schemas/audit-output.schema.v0.json"
);

fn bash_available() -> bool {
    StdCommand::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn setup_straymark(dir: &Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("audit-prompts")).unwrap();
    std::fs::create_dir_all(straymark.join("schemas")).unwrap();
    std::fs::create_dir_all(straymark.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(
        straymark.join("audit-prompts/audit-prompt.md"),
        AUDIT_PROMPT_UNIFIED,
    )
    .unwrap();
    std::fs::write(
        straymark.join("schemas/audit-output.schema.v0.json"),
        AUDIT_OUTPUT_SCHEMA,
    )
    .unwrap();
}

/// Helper: returns the v1 canonical audit dir for a Charter under `dir`.
fn audit_dir(dir: &Path, charter_id: &str) -> std::path::PathBuf {
    dir.join(".straymark").join("audits").join(charter_id)
}

fn write_charter(dir: &Path) {
    let charters = dir.join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    let body = r#"---
charter_id: CHARTER-01
status: in-progress
effort_estimate: M
trigger: "test"
---

# Charter: Audit test

## Files to modify

| File | Change |
|---|---|
| `src/foo.rs` | edit |

## Tasks
1. Run.
"#;
    std::fs::write(charters.join("01-audit-test.md"), body).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let status = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .expect("git failed");
    assert!(status.success(), "git {} failed", args.join(" "));
}

fn init_repo_with_diff(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/foo.rs"), "// initial\n").unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "initial"]);
    std::fs::write(dir.join("src/foo.rs"), "// edited\n").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "edit"]);
}

#[test]
fn audit_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn audit_unknown_charter_fails() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-99", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("CHARTER-99 not found"));
}

#[test]
fn audit_prepare_writes_unified_prompt_to_canonical_location() {
    if !bash_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("PREPARE"))
        .stdout(predicate::str::contains("audit-prompt.md"))
        .stdout(predicate::str::contains("/straymark-audit-execute"))
        .stdout(predicate::str::contains("/straymark-audit-review"));

    // v1 canonical path: .straymark/audits/CHARTER-01/audit-prompt.md
    // (singular file, not a prompts/ subdirectory with two files).
    let resolved_path = audit_dir(dir.path(), "CHARTER-01").join("audit-prompt.md");
    let resolved = std::fs::read_to_string(&resolved_path)
        .unwrap_or_else(|_| panic!("expected resolved prompt at {}", resolved_path.display()));

    // Placeholder substitution happened in the body (outside HTML comments).
    assert!(resolved.contains("CHARTER-01"));
    assert!(resolved.contains(".straymark/charters/01-audit-test.md"));
    // Diff was inlined.
    assert!(resolved.contains("// edited") || resolved.contains("// initial"));

    // R10 (issue #102): the resolver must NOT expand placeholders inside
    // <!-- ... --> blocks. The unified template has a documentation header
    // that lists placeholders literally; the resolver preserves them.
    let header_end = resolved
        .find("-->")
        .expect("template should have an HTML comment header");
    let header = &resolved[..header_end];
    let body = &resolved[header_end..];
    assert!(
        header.contains("{{charter_id}}"),
        "documentation header must preserve {{{{charter_id}}}} literal"
    );
    assert!(
        header.contains("{{git_diff}}"),
        "documentation header must preserve {{{{git_diff}}}} literal"
    );
    assert!(
        !body.contains("{{charter_id}}"),
        "body (outside comment) must have {{{{charter_id}}}} replaced"
    );
    assert!(
        !body.contains("{{git_diff}}"),
        "body (outside comment) must have {{{{git_diff}}}} replaced"
    );

    // v1: the v0 paths under audit/charters/ must NOT be written by the
    // v1 CLI. Only the canonical .straymark/audits/<id>/audit-prompt.md is
    // produced.
    let v0_primary = dir
        .path()
        .join("audit")
        .join("charters")
        .join("CHARTER-01")
        .join("prompts")
        .join("auditor-primary.prompt.md");
    assert!(
        !v0_primary.exists(),
        "v0 path under audit/charters/ must NOT be written by the v1 CLI"
    );
    let v0_secondary = dir
        .path()
        .join("audit")
        .join("charters")
        .join("CHARTER-01")
        .join("prompts")
        .join("auditor-secondary.prompt.md");
    assert!(
        !v0_secondary.exists(),
        "v0 secondary prompt path must NOT be written by the v1 CLI"
    );
}

#[test]
fn audit_merge_reports_with_no_reports_fails_helpfully() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    // Skip prepare and any report files; jump straight to merge-reports.
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--merge-reports", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No reports found"))
        .stderr(predicate::str::contains("report-*.md"));
}

#[test]
fn audit_merge_reports_validates_against_schema() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    let canonical = audit_dir(dir.path(), "CHARTER-01");
    std::fs::create_dir_all(&canonical).unwrap();

    // Malformed report (missing required findings_total).
    std::fs::write(
        canonical.join("report-claude-sonnet-4-6.md"),
        r#"---
audit_role: auditor
auditor: claude-sonnet-4-6
charter_id: CHARTER-01
audited_at: "2026-05-03"
prompt_used: audit-prompt.md
---

# bad
"#,
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--merge-reports", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema validation"));
}

#[test]
fn audit_merge_reports_handles_n_reports_with_unified_role() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    // Prepare (writes the unified prompt).
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--prepare", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Operator saves three reports under the canonical path with the v1
    // unified `audit_role: auditor`.
    let canonical = audit_dir(dir.path(), "CHARTER-01");
    for (slug, model, findings) in [
        ("claude-sonnet-4-6", "claude-sonnet-4-6", 2),
        ("gemini-2-5-pro", "gemini-2.5-pro", 1),
        ("gpt-5-3-codex", "gpt-5.3-codex", 0),
    ] {
        let body = format!(
            r#"---
audit_role: auditor
auditor: {model}
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: audit-prompt.md
audited_at: "2026-05-03"
findings_total: {findings}
findings_by_category:
  hallucination: 0
  implementation_gap: {findings}
  real_debt: 0
  false_positive: 0
audit_quality: high
evidence_citations: {findings}
---
# Body
"#,
            model = model,
            findings = findings
        );
        std::fs::write(canonical.join(format!("report-{slug}.md")), body).unwrap();
    }

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--merge-reports", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Audit cycle merge complete"))
        .stdout(predicate::str::contains("external_audit YAML"))
        .stdout(predicate::str::contains("claude-sonnet-4-6"))
        .stdout(predicate::str::contains("gemini-2.5-pro"))
        .stdout(predicate::str::contains("gpt-5.3-codex"))
        // No "warning: only one report" because we have three.
        .stderr(predicate::str::contains("only one report").not());
}

#[test]
fn audit_merge_reports_warns_on_single_report_but_proceeds() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    let canonical = audit_dir(dir.path(), "CHARTER-01");
    std::fs::create_dir_all(&canonical).unwrap();
    std::fs::write(
        canonical.join("report-claude-sonnet-4-6.md"),
        r#"---
audit_role: auditor
auditor: claude-sonnet-4-6
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: audit-prompt.md
audited_at: "2026-05-03"
findings_total: 0
findings_by_category:
  hallucination: 0
  implementation_gap: 0
  real_debt: 0
  false_positive: 0
---
# Body
"#,
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--merge-reports", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("only one report"))
        .stderr(predicate::str::contains("heterogeneity"));
}

#[test]
fn audit_deprecated_calibrate_emits_warning_and_exits() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--calibrate", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("v0 way"))
        .stderr(predicate::str::contains("/straymark-audit-review"))
        .stderr(predicate::str::contains("--merge-reports"));
}

#[test]
fn audit_deprecated_finalize_redirects_to_merge_reports() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    let canonical = audit_dir(dir.path(), "CHARTER-01");
    std::fs::create_dir_all(&canonical).unwrap();
    std::fs::write(
        canonical.join("report-claude-sonnet-4-6.md"),
        r#"---
audit_role: auditor
auditor: claude-sonnet-4-6
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: audit-prompt.md
audited_at: "2026-05-03"
findings_total: 0
findings_by_category:
  hallucination: 0
  implementation_gap: 0
  real_debt: 0
  false_positive: 0
---
# Body
"#,
    )
    .unwrap();

    // --finalize should warn but proceed via the merge-reports path.
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--finalize", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("--finalize is the v0 name"))
        .stderr(predicate::str::contains("--merge-reports"))
        .stdout(predicate::str::contains("Audit cycle merge complete"));
}

#[test]
fn audit_action_flags_are_mutually_exclusive() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    // --prepare and --merge-reports must not co-occur (clap-enforced).
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--prepare",
            "--merge-reports",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure();

    // Deprecated flags also conflict with each other and with the new ones.
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--calibrate",
            "--finalize",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure();
}

// ── --merge-into: PR 2 of audit-skills rollout (updated for v1 paths) ─────

/// Set up a Charter with two valid v1 reports under .straymark/audits/CHARTER-01/,
/// so we can drive --merge-reports + --merge-into repeatedly.
fn setup_finalized_audit(dir: &Path) {
    setup_straymark(dir);
    write_charter(dir);
    init_repo_with_diff(dir);

    // PREPARE writes the unified audit-prompt.md to the canonical path.
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--prepare", "--path"])
        .arg(dir.to_str().unwrap())
        .assert()
        .success();

    let canonical = audit_dir(dir, "CHARTER-01");
    std::fs::write(
        canonical.join("report-copilot-v1-0-37.md"),
        r#"---
audit_role: auditor
auditor: copilot-v1.0.37
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: audit-prompt.md
audited_at: "2026-05-03"
findings_total: 2
findings_by_category:
  hallucination: 0
  implementation_gap: 1
  real_debt: 1
  false_positive: 0
audit_quality: high
---
# Body
"#,
    )
    .unwrap();
    std::fs::write(
        canonical.join("report-gemini-cli-v1-5.md"),
        r#"---
audit_role: auditor
auditor: gemini-cli-v1.5
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: audit-prompt.md
audited_at: "2026-05-03"
findings_total: 1
findings_by_category:
  hallucination: 0
  implementation_gap: 1
  real_debt: 0
  false_positive: 0
audit_quality: medium
---
# Body
"#,
    )
    .unwrap();
}

/// Build a minimal Charter telemetry file (the shape charter close emits).
fn write_minimal_telemetry(dir: &Path) -> std::path::PathBuf {
    let path = dir
        .join(".straymark/charters/CHARTER-01.telemetry.yaml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        r#"charter_telemetry:
  charter_id: "CHARTER-01"
  charter_title: "Audit test"
  closed_at: "2026-05-03"

  trigger:
    declared_kind: "manual"
    declared_description: "test"
    fired_at: "2026-05-03"
    fire_clarity: "clear"
    fire_clarity_notes: ""
"#,
    )
    .unwrap();
    path
}

#[test]
fn audit_merge_into_appends_external_audit_to_telemetry() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_finalized_audit(dir.path());
    let telemetry_path = write_minimal_telemetry(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--merge-reports",
            "--merge-into",
            telemetry_path.to_str().unwrap(),
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Merged external_audit"));

    let merged = std::fs::read_to_string(&telemetry_path).unwrap();
    assert!(
        merged.contains("\n  external_audit:\n"),
        "external_audit block must be appended at indent 2:\n{merged}"
    );
    assert!(
        merged.contains("    - auditor: \"copilot-v1.0.37\""),
        "first auditor present in merged output"
    );
    assert!(
        merged.contains("    - auditor: \"gemini-cli-v1.5\""),
        "second auditor present in merged output"
    );
    assert!(
        merged.contains("audit/charters/CHARTER-01/"),
        "audit_notes must reference real charter id (not <charter-id> placeholder)"
    );
    // Pre-existing keys preserved.
    assert!(merged.contains("charter_id: \"CHARTER-01\""));
    assert!(merged.contains("trigger:"));
}

#[test]
fn audit_merge_into_missing_telemetry_fails_with_helpful_message() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_finalized_audit(dir.path());
    let missing = dir.path().join(".straymark/charters/CHARTER-01.telemetry.yaml");

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--merge-reports",
            "--merge-into",
            missing.to_str().unwrap(),
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Telemetry file not found"))
        .stderr(predicate::str::contains("straymark charter close"));
}

#[test]
fn audit_merge_into_rejects_existing_external_audit() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_finalized_audit(dir.path());
    let telemetry_path = write_minimal_telemetry(dir.path());

    // Pre-populate external_audit so re-audit guard fires.
    let mut existing = std::fs::read_to_string(&telemetry_path).unwrap();
    existing.push_str("\n  external_audit:\n    - auditor: \"old-auditor\"\n      findings_total: 0\n");
    std::fs::write(&telemetry_path, &existing).unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--merge-reports",
            "--merge-into",
            telemetry_path.to_str().unwrap(),
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already has an `external_audit:`"));
}

#[test]
fn audit_merge_into_requires_merge_reports_or_finalize() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    // Without --merge-reports (or deprecated --finalize), the CLI should
    // reject --merge-into with a clear error.
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--merge-into",
            "/tmp/whatever.yaml",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--merge-into is only valid with --merge-reports"));
}

// ── R11(A) regression tests (issue #102) ───────────────────────────────────
//
// Sentinel CHARTER-07 was implemented as 8 commits on a feature branch; the
// previous default `HEAD~1..HEAD` only sent the last (metadata-only) commit
// to the auditors, who converged on "0 substantive findings" vacuously. The
// fix prefers `origin/main..HEAD` (or `origin/master..HEAD` on legacy repos)
// when an upstream is reachable, falling back to `HEAD~1..HEAD` otherwise.
// The resolved Git range appears in the prompt body via `Git range: <range>`,
// which is what these tests assert against.

/// Set up a working tree where `origin/main` is reachable: a bare repo as
/// the remote (in its own tempdir to avoid collisions when tests run in
/// parallel), an initial commit on `main`, push to remote, then a feature
/// branch with two additional commits. The current branch is the feature
/// branch when this returns. The returned `TempDir` MUST be kept alive by
/// the caller for the duration of the test — dropping it removes the bare
/// remote and breaks subsequent git operations on the working tree.
fn init_repo_with_remote_main(dir: &Path) -> TempDir {
    let remote = TempDir::new().unwrap();
    let status = StdCommand::new("git")
        .args(["init", "--bare", "-q", "-b", "main"])
        .current_dir(remote.path())
        .status()
        .expect("git init --bare failed");
    assert!(status.success());

    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/foo.rs"), "// initial\n").unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "initial on main"]);
    git(dir, &["remote", "add", "origin", remote.path().to_str().unwrap()]);
    git(dir, &["push", "-q", "origin", "main"]);

    // Feature branch with multiple commits — this is what `origin/main..HEAD`
    // is supposed to capture in full.
    git(dir, &["checkout", "-q", "-b", "feature/multi"]);
    std::fs::write(dir.join("src/foo.rs"), "// edited 1\n").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "feature: first"]);
    std::fs::write(dir.join("src/foo.rs"), "// edited 2\n").unwrap();
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "feature: second"]);

    remote
}

#[test]
fn audit_default_range_uses_origin_main_when_available() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    let _remote = init_repo_with_remote_main(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let prompt = std::fs::read_to_string(
        dir.path()
            .join(".straymark/audits/CHARTER-01/audit-prompt.md"),
    )
    .unwrap();
    assert!(
        prompt.contains("origin/main..HEAD"),
        "default range must resolve to origin/main..HEAD when remote is reachable; \
         prompt did not contain that string. Excerpt:\n{}",
        prompt.lines().take(80).collect::<Vec<_>>().join("\n")
    );
    assert!(
        !prompt.contains("Git range: `HEAD~1..HEAD`"),
        "default range must NOT fall back to HEAD~1..HEAD when origin/main exists"
    );
}

#[test]
fn audit_default_range_falls_back_to_head_minus_one_without_remote() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path()); // no remote configured

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("no upstream branch reachable"))
        .stderr(predicate::str::contains("HEAD~1..HEAD"))
        .stderr(predicate::str::contains("--range"));

    let prompt = std::fs::read_to_string(
        dir.path()
            .join(".straymark/audits/CHARTER-01/audit-prompt.md"),
    )
    .unwrap();
    assert!(
        prompt.contains("HEAD~1..HEAD"),
        "fallback range must be HEAD~1..HEAD when no upstream is reachable"
    );
    assert!(
        !prompt.contains("origin/main..HEAD"),
        "no origin/main exists, fallback must not claim it does"
    );
}

#[test]
fn audit_explicit_range_overrides_default_resolution() {
    // Backwards-compat sanity: --range still wins over the new defaulting
    // logic (no upstream probe attempted).
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--range",
            "HEAD~1..HEAD",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        // No fallback warning when --range was passed explicitly.
        .stderr(predicate::str::contains("no upstream branch reachable").not());
}

// ── i18n: localized audit-prompt resolution ─────────────────────────────
//
// fw-4.13.3 / cli-3.12.3 moved the ES prompt to the i18n overlay convention
// (EN canonical at `.straymark/audit-prompts/audit-prompt.md`, ES at
// `i18n/es/audit-prompt.md`). The CLI now reads `.straymark/config.yml`'s
// `language` field to decide which file to load, via
// `resolve_localized_path` (cli/src/utils.rs).
//
// These three tests pin the wiring: ES adopters get the ES prompt; unknown
// locales fall back to EN; explicit `language: en` always resolves to root.

/// Set up a project with both EN and ES audit-prompt files present. The
/// `language` field in config.yml decides which one the CLI uses.
fn setup_straymark_with_es_overlay(dir: &Path, language: &str) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("audit-prompts/i18n/es")).unwrap();
    std::fs::create_dir_all(straymark.join("schemas")).unwrap();
    std::fs::create_dir_all(straymark.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::write(
        straymark.join("config.yml"),
        format!("language: {}\n", language),
    )
    .unwrap();
    std::fs::write(
        straymark.join("audit-prompts/audit-prompt.md"),
        AUDIT_PROMPT_UNIFIED,
    )
    .unwrap();
    std::fs::write(
        straymark.join("audit-prompts/i18n/es/audit-prompt.md"),
        AUDIT_PROMPT_ES,
    )
    .unwrap();
    std::fs::write(
        straymark.join("schemas/audit-output.schema.v0.json"),
        AUDIT_OUTPUT_SCHEMA,
    )
    .unwrap();
}

#[test]
fn audit_prepare_uses_es_overlay_when_language_es() {
    if !bash_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark_with_es_overlay(dir.path(), "es");
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let resolved_path = audit_dir(dir.path(), "CHARTER-01").join("audit-prompt.md");
    let resolved = std::fs::read_to_string(&resolved_path).unwrap();

    // The ES prompt starts with "# Auditoría de Charter" (the EN one starts
    // with "# Charter audit"). Body sections like "## Tu rol" and "## Lo que
    // NO debes hacer" pin the language unambiguously.
    assert!(
        resolved.contains("# Auditoría de Charter"),
        "language: es should resolve the ES overlay, not the EN canonical"
    );
    assert!(resolved.contains("## Tu rol"));
    assert!(resolved.contains("## Lo que NO debes hacer"));
}

#[test]
fn audit_prepare_falls_back_to_en_when_locale_overlay_missing() {
    // `language: zh-CN` is configured but no i18n/zh-CN/ overlay exists in
    // this temp project. `resolve_localized_path` must fall back to the EN
    // canonical file at the root of audit-prompts/.
    if !bash_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark_with_es_overlay(dir.path(), "zh-CN");
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let resolved_path = audit_dir(dir.path(), "CHARTER-01").join("audit-prompt.md");
    let resolved = std::fs::read_to_string(&resolved_path).unwrap();

    assert!(
        resolved.contains("# Charter audit"),
        "language: zh-CN with no overlay should fall back to EN canonical"
    );
    assert!(resolved.contains("## Your role"));
    // Negative check: must NOT have leaked ES headings.
    assert!(!resolved.contains("## Tu rol"));
}

#[test]
fn audit_prepare_uses_en_canonical_when_language_en() {
    if !bash_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_straymark_with_es_overlay(dir.path(), "en");
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let resolved_path = audit_dir(dir.path(), "CHARTER-01").join("audit-prompt.md");
    let resolved = std::fs::read_to_string(&resolved_path).unwrap();

    // Explicit `language: en` must always pick the canonical EN file even if
    // an ES overlay exists.
    assert!(resolved.contains("# Charter audit"));
    assert!(!resolved.contains("## Tu rol"));
}

