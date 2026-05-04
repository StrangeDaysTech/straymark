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

// ── --merge-into: PR 2 of audit-skills rollout ─────────────────────────────

/// Set up a Charter that has been fully audited (3 outputs present), so we
/// can drive --finalize repeatedly with different --merge-into targets.
fn setup_finalized_audit(dir: &Path) {
    setup_devtrail(dir);
    write_charter(dir);
    init_repo_with_diff(dir);

    // PREPARE
    Command::cargo_bin("devtrail")
        .unwrap()
        .args(["charter", "audit", "CHARTER-01", "--path"])
        .arg(dir.to_str().unwrap())
        .assert()
        .success();

    let audit_dir = dir.join("audit/charters/CHARTER-01");
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
# Body
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
# Body
"#,
    )
    .unwrap();

    // CALIBRATE (the CLI writes calibrator-reconciler.prompt.md but here we
    // just simulate the operator pasting the calibrator response directly).
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
# Body
"#,
    )
    .unwrap();
}

/// Build a minimal Charter telemetry file (the shape charter close emits).
fn write_minimal_telemetry(dir: &Path) -> std::path::PathBuf {
    let path = dir
        .join(".devtrail/charters/CHARTER-01.telemetry.yaml");
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

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--finalize",
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
        "primary auditor present in merged output"
    );
    assert!(
        merged.contains("    - auditor: \"gemini-cli-v1.5\""),
        "secondary auditor present in merged output"
    );
    assert!(
        merged.contains("audit/charters/CHARTER-01/auditor-primary.md"),
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
    let missing = dir.path().join(".devtrail/charters/CHARTER-01.telemetry.yaml");

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--finalize",
            "--merge-into",
            missing.to_str().unwrap(),
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Telemetry file not found"))
        .stderr(predicate::str::contains("devtrail charter close"));
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

    Command::cargo_bin("devtrail")
        .unwrap()
        .args([
            "charter",
            "audit",
            "CHARTER-01",
            "--finalize",
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
fn audit_merge_into_requires_finalize() {
    let dir = TempDir::new().unwrap();
    setup_devtrail(dir.path());

    // Without --finalize, clap should reject --merge-into.
    Command::cargo_bin("devtrail")
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
        .failure();
}
