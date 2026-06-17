//! Integration tests for `straymark approve`.

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn setup_straymark(dir: &Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("07-ai-audit/decisions")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();
}

fn write_aidec(dir: &Path, id: &str, review_required: bool) -> std::path::PathBuf {
    write_aidec_with_risk(dir, id, review_required, "medium")
}

fn write_aidec_with_risk(
    dir: &Path,
    id: &str,
    review_required: bool,
    risk_level: &str,
) -> std::path::PathBuf {
    let path = dir.join(format!(
        ".straymark/07-ai-audit/decisions/{}-test-decision.md",
        id
    ));
    let body = format!(
        r#"---
id: {id}
title: Test decision
status: accepted
created: 2026-04-23
agent: test-v1.0
confidence: high
review_required: {rq}
risk_level: {risk}
---

# AIDEC: Test decision

## Context

Body.

## References

- (none)

<!-- Template: StrayMark | https://strangedays.tech -->
"#,
        id = id,
        rq = if review_required { "true" } else { "false" },
        risk = risk_level,
    );
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn approve_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-001",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn approve_unknown_doc_fails_clearly() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-999",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "AIDEC-2026-05-02-999 not found",
        ));
}

#[test]
fn approve_writes_frontmatter_and_body() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    let aidec = write_aidec(dir.path(), "AIDEC-2026-05-02-001", true);

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-001",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--at",
            "2026-05-02",
            "--notes",
            "Looks good. Ship it.",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("approved"))
        .stdout(predicate::str::contains("pepe@example.com"));

    let content = std::fs::read_to_string(&aidec).unwrap();

    // Frontmatter has the three new fields.
    assert!(content.contains("reviewed_by: pepe@example.com"), "{content}");
    assert!(content.contains("reviewed_at: 2026-05-02"), "{content}");
    assert!(content.contains("review_outcome: approved"), "{content}");

    // Body has the section, before the template signature.
    assert!(content.contains("## Approval"), "{content}");
    assert!(
        content.contains("**Approved**: 2026-05-02 by `pepe@example.com`."),
        "{content}"
    );
    assert!(content.contains("Looks good. Ship it."), "{content}");

    // Approval section appears BEFORE the template signature line.
    let approval_pos = content.find("## Approval").unwrap();
    let signature_pos = content.find("<!-- Template: StrayMark").unwrap();
    assert!(approval_pos < signature_pos);

    // Original frontmatter and body are preserved.
    assert!(content.contains("review_required: true"));
    assert!(content.contains("risk_level: medium"));
    assert!(content.contains("# AIDEC: Test decision"));
    assert!(content.contains("## References"));
}

#[test]
fn approve_warns_when_review_not_required_but_succeeds() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec(dir.path(), "AIDEC-2026-05-02-002", false);

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-002",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--at",
            "2026-05-02",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("does not have `review_required: true`"));
}

#[test]
fn approve_re_application_without_force_is_idempotent_skip() {
    // F4 (cli-3.7.1): re-running approve on an already-approved doc no
    // longer silently overwrites frontmatter and appends a duplicate body
    // block. The default is now an idempotent skip with a clear message.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    let aidec = write_aidec(dir.path(), "AIDEC-2026-05-02-003", true);

    // First approval (legitimate).
    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-003",
            "--outcome",
            "revisions_requested",
            "--reviewer",
            "first@example.com",
            "--at",
            "2026-05-01",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let content_after_first = std::fs::read_to_string(&aidec).unwrap();

    // Re-approval WITHOUT --force is an idempotent skip.
    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-003",
            "--outcome",
            "approved",
            "--reviewer",
            "second@example.com",
            "--at",
            "2026-05-02",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("already has approval"))
        .stdout(predicate::str::contains("--force"));

    // Document is unchanged.
    let content_after_skip = std::fs::read_to_string(&aidec).unwrap();
    assert_eq!(
        content_after_first, content_after_skip,
        "doc should not change without --force"
    );
}

#[test]
fn approve_re_application_with_force_replaces_and_appends_block() {
    // F4 (cli-3.7.1): with --force, the legitimate cycles work as before:
    //  - revisions_requested → approved (same reviewer iterating)
    //  - multi-reviewer hand-off (different reviewer adding to history)
    // Frontmatter is latest-wins; body preserves the chronological history.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    let aidec = write_aidec(dir.path(), "AIDEC-2026-05-02-003", true);

    // First approval.
    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-003",
            "--outcome",
            "revisions_requested",
            "--reviewer",
            "first@example.com",
            "--at",
            "2026-05-01",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Re-approval WITH --force.
    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-003",
            "--outcome",
            "approved",
            "--reviewer",
            "second@example.com",
            "--at",
            "2026-05-02",
            "--force",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let content = std::fs::read_to_string(&aidec).unwrap();

    // Frontmatter shows the LATEST approval only.
    assert!(content.contains("reviewed_by: second@example.com"), "{content}");
    assert!(content.contains("reviewed_at: 2026-05-02"), "{content}");
    assert!(content.contains("review_outcome: approved"), "{content}");
    assert!(!content.contains("reviewed_by: first@example.com"), "{content}");

    // Body contains BOTH approval blocks chronologically.
    let approval_count = content.matches("## Approval").count();
    assert_eq!(approval_count, 2, "expected 2 approval blocks, got:\n{content}");
}

#[test]
fn approve_invalid_outcome_rejected_by_clap() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec(dir.path(), "AIDEC-2026-05-02-004", true);

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-02-004",
            "--outcome",
            "totally-invalid",
            "--reviewer",
            "pepe@example.com",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ── F5 (cli-3.8.0): high-risk warning + --quiet ──────────────────────

#[test]
fn approve_warns_on_high_risk_document() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec_with_risk(dir.path(), "AIDEC-2026-05-03-001", true, "high");

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-001",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--at",
            "2026-05-03",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("WARNING"))
        .stderr(predicate::str::contains("risk_level: high"))
        .stderr(predicate::str::contains("thorough human review"));
}

#[test]
fn approve_warns_on_critical_risk_document() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec_with_risk(dir.path(), "AIDEC-2026-05-03-002", true, "critical");

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-002",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--at",
            "2026-05-03",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("risk_level: critical"));
}

#[test]
fn approve_no_warning_for_low_or_medium_risk() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec_with_risk(dir.path(), "AIDEC-2026-05-03-003", true, "low");

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-003",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("risk_level").not());
}

#[test]
fn approve_quiet_suppresses_normal_output_but_keeps_high_risk_warning() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec_with_risk(dir.path(), "AIDEC-2026-05-03-004", true, "high");

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-004",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--at",
            "2026-05-03",
            "--quiet",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        // Normal success message is suppressed.
        .stdout(predicate::str::contains("marked as").not())
        // But the high-risk warning still surfaces — that's the whole point
        // of F5 not being silenceable by --quiet.
        .stderr(predicate::str::contains("WARNING"))
        .stderr(predicate::str::contains("risk_level: high"));
}

#[test]
fn approve_quiet_suppresses_idempotent_skip_message() {
    // Re-approving an already-approved document under --quiet should still
    // be an idempotent no-op, but with no console output.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec(dir.path(), "AIDEC-2026-05-03-005", true);

    // First approval — verbose so we know the file is set up.
    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-005",
            "--outcome",
            "approved",
            "--reviewer",
            "first@example.com",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Second invocation — quiet, should produce no stdout.
    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-005",
            "--outcome",
            "approved",
            "--reviewer",
            "first@example.com",
            "--quiet",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn approve_quiet_suppresses_review_required_false_warning() {
    // The "document does not have review_required:true" info-warning IS
    // silenceable by --quiet (it documents an unusual-but-allowed pattern).
    // High-risk warning is NOT silenceable; this test only covers the
    // info-warning path.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_aidec(dir.path(), "AIDEC-2026-05-03-006", false);

    cargo_bin_cmd!("straymark")
        .args([
            "approve",
            "AIDEC-2026-05-03-006",
            "--outcome",
            "approved",
            "--reviewer",
            "pepe@example.com",
            "--quiet",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stderr(predicate::str::contains("review_required").not())
        .stdout(predicate::str::is_empty());
}
