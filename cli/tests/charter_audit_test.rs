//! Integration tests for `devtrail charter audit` (Phase 3 v0).

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

const AUDIT_PROMPT_PRIMARY: &str = include_str!(
    "../../dist/.devtrail/audit-prompts/auditor-primary.md"
);
const AUDIT_PROMPT_SECONDARY: &str = include_str!(
    "../../dist/.devtrail/audit-prompts/auditor-secondary.md"
);
const AUDIT_PROMPT_CALIBRATOR: &str = include_str!(
    "../../dist/.devtrail/audit-prompts/calibrator-reconciler.md"
);
const AUDIT_OUTPUT_SCHEMA: &str = include_str!(
    "../../dist/.devtrail/schemas/audit-output.schema.v0.json"
);

fn bash_available() -> bool {
    StdCommand::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn setup_devtrail(dir: &Path) {
    let devtrail = dir.join(".devtrail");
    std::fs::create_dir_all(devtrail.join("audit-prompts")).unwrap();
    std::fs::create_dir_all(devtrail.join("schemas")).unwrap();
    std::fs::create_dir_all(devtrail.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::create_dir_all(devtrail.join("templates")).unwrap();
    std::fs::write(devtrail.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(
        devtrail.join("audit-prompts/auditor-primary.md"),
        AUDIT_PROMPT_PRIMARY,
    )
    .unwrap();
    std::fs::write(
        devtrail.join("audit-prompts/auditor-secondary.md"),
        AUDIT_PROMPT_SECONDARY,
    )
    .unwrap();
    std::fs::write(
        devtrail.join("audit-prompts/calibrator-reconciler.md"),
        AUDIT_PROMPT_CALIBRATOR,
    )
    .unwrap();
    std::fs::write(
        devtrail.join("schemas/audit-output.schema.v0.json"),
        AUDIT_OUTPUT_SCHEMA,
    )
    .unwrap();
}

fn write_charter(dir: &Path) {
    let charters = dir.join("docs/charters");
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
fn audit_requires_devtrail_installed() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("devtrail")
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
    setup_devtrail(dir.path());
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-99", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("CHARTER-99 not found"));
}

#[test]
fn audit_prepare_writes_resolved_prompts() {
    if !bash_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("PREPARE"))
        .stdout(predicate::str::contains("auditor-primary.prompt.md"))
        .stdout(predicate::str::contains("auditor-secondary.prompt.md"))
        .stdout(predicate::str::contains("--calibrate"));

    let prompts = dir.path().join("audit/charters/CHARTER-01/prompts");
    let primary = std::fs::read_to_string(prompts.join("auditor-primary.prompt.md")).unwrap();
    // Placeholder substitution happened.
    assert!(primary.contains("CHARTER-01"));
    assert!(primary.contains("auditor-primary"));
    assert!(primary.contains("docs/charters/01-audit-test.md"));
    // Diff was inlined.
    assert!(primary.contains("// edited") || primary.contains("// initial"));
    // Unknown placeholder syntax is gone.
    assert!(!primary.contains("{{charter_id}}"));
    assert!(!primary.contains("{{git_diff}}"));

    let secondary = std::fs::read_to_string(prompts.join("auditor-secondary.prompt.md")).unwrap();
    assert!(secondary.contains("auditor-secondary"));
}

#[test]
fn audit_calibrate_requires_auditor_outputs() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    // Skip prepare; go directly to calibrate.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--calibrate", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("auditor-primary.md"))
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn audit_calibrate_validates_outputs_against_schema() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    let audit_dir = dir.path().join("audit/charters/CHARTER-01");
    std::fs::create_dir_all(&audit_dir).unwrap();

    // Write a malformed auditor-primary.md (missing required findings_total).
    std::fs::write(
        audit_dir.join("auditor-primary.md"),
        r#"---
audit_role: auditor-primary
auditor: copilot
charter_id: CHARTER-01
audited_at: "2026-05-03"
prompt_used: prompts/auditor-primary.prompt.md
---

# bad
"#,
    )
    .unwrap();
    std::fs::write(
        audit_dir.join("auditor-secondary.md"),
        r#"---
audit_role: auditor-secondary
auditor: gemini
charter_id: CHARTER-01
audited_at: "2026-05-03"
findings_total: 0
findings_by_category:
  hallucination: 0
  implementation_gap: 0
  real_debt: 0
  false_positive: 0
prompt_used: prompts/auditor-secondary.prompt.md
---

# good
"#,
    )
    .unwrap();

    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--calibrate", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema validation"));
}

#[test]
fn audit_full_three_step_cycle_succeeds() {
    if !bash_available() {
        return;
    }
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());
    write_charter(dir.path());
    init_repo_with_diff(dir.path());

    // Step 1: prepare.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Simulate the operator pasting valid auditor responses.
    let audit_dir = dir.path().join("audit/charters/CHARTER-01");
    std::fs::write(
        audit_dir.join("auditor-primary.md"),
        r#"---
audit_role: auditor-primary
auditor: copilot-v1.0.37
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: prompts/auditor-primary.prompt.md
audited_at: "2026-05-03"
findings_total: 2
findings_by_category:
  hallucination: 0
  implementation_gap: 1
  real_debt: 1
  false_positive: 0
audit_quality: high
---

# Audit by copilot

## Findings

### F1 — minor gap — implementation_gap

Body.

### F2 — leak — real_debt

Body.
"#,
    )
    .unwrap();
    std::fs::write(
        audit_dir.join("auditor-secondary.md"),
        r#"---
audit_role: auditor-secondary
auditor: gemini-cli-v1.5
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: prompts/auditor-secondary.prompt.md
audited_at: "2026-05-03"
findings_total: 1
findings_by_category:
  hallucination: 0
  implementation_gap: 1
  real_debt: 0
  false_positive: 0
audit_quality: medium
---

# Audit by gemini

## Findings

### F1 — overlapping gap — implementation_gap

Body.
"#,
    )
    .unwrap();

    // Step 2: calibrate.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--calibrate", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("CALIBRATE"))
        .stdout(predicate::str::contains("calibrator-reconciler.prompt.md"))
        .stdout(predicate::str::contains("--finalize"));

    // The resolved calibrator prompt should embed both auditors' findings.
    let cal = std::fs::read_to_string(
        audit_dir.join("prompts/calibrator-reconciler.prompt.md"),
    )
    .unwrap();
    assert!(cal.contains("calibrator-reconciler"));
    assert!(cal.contains("copilot-v1.0.37"), "primary auditor body should be embedded");
    assert!(cal.contains("gemini-cli-v1.5"), "secondary auditor body should be embedded");

    // Simulate calibrator response.
    std::fs::write(
        audit_dir.join("calibrator-reconciler.md"),
        r#"---
audit_role: calibrator-reconciler
calibrator: claude-opus-4
charter_id: CHARTER-01
git_range: "HEAD~1..HEAD"
prompt_used: prompts/calibrator-reconciler.prompt.md
calibrated_at: "2026-05-03"
auditors_reconciled:
  - auditor-primary.md
  - auditor-secondary.md
findings_consolidated: 2
findings_by_status:
  agreed: 1
  disputed: 0
  unique_primary: 1
  unique_secondary: 0
  rejected: 0
---

# Calibration

## Reconciliation summary

Both auditors converged on the implementation_gap; primary added a real_debt
that secondary missed.
"#,
    )
    .unwrap();

    // Step 3: finalize.
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--finalize", "--path"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FINALIZE"))
        .stdout(predicate::str::contains("Charter audit complete"))
        .stdout(predicate::str::contains("external_audit YAML"))
        // Both auditors appear in the rendered YAML.
        .stdout(predicate::str::contains("copilot-v1.0.37"))
        .stdout(predicate::str::contains("gemini-cli-v1.5"));
}

#[test]
fn audit_calibrate_and_finalize_are_mutually_exclusive() {
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());

    Command::cargo_bin("devtrail")
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
