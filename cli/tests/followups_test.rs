//! Integration tests for `straymark followups` (cli-3.19.0,
//! ADR-2026-06-03-001): list / status / drift / promote against fixture
//! registries, including a Sentinel-style v0 registry (lenient parsing +
//! in-place upgrade) and the anti-noise / counter-recompute behaviors that
//! resolve issue #214 Signals 1-2.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Minimal StrayMark project scaffold: `.straymark/` with the directories
/// the followups commands touch.
fn scaffold(dir: &Path) -> PathBuf {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::create_dir_all(straymark.join("06-evolution/technical-debt")).unwrap();
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::create_dir_all(straymark.join("schemas")).unwrap();
    straymark
}

/// Sentinel-style v0 registry: no v1 fields, stale counters (#214 Signal 2 —
/// frontmatter claims total_open: 47, real count is 2).
const V0_REGISTRY: &str = r#"---
last_scan: 2026-05-06
schema_version: v0
total_open: 47
total_promoted: 0
total_closed_in_session: 0
total_phase_blocked: 0
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-04-11-001
---

# Follow-ups Backlog

## Bucket: ready

### FU-001 — Wire the retry budget into the sync loop
- **Origin**: AILOG-2026-04-11-001 §Follow-ups
- **Status**: open
- **Trigger**: ready
- **Destination**: operations
- **Cost**: S

## Bucket: charter-triggered

### FU-002 — Extend E2E coverage to the write paths
- **Origin**: AILOG-2026-04-11-001 §R3 (new, not in Charter)
- **Status**: open
- **Trigger**: next Charter touching writes
- **Destination**: TBD
- **Cost**: M

## Bucket: phase-blocked

## Bucket: operational
"#;

/// v1 registry exercising the new dimensions.
const V1_REGISTRY: &str = r#"---
schema_version: v1
last_scan: 2026-06-03
total_open: 2
total_promoted: 0
total_closed_in_session: 0
total_phase_blocked: 0
total_suspected_closed: 0
buckets:
  - ready
fully_extracted_ailogs: []
---

## Bucket: ready

### FU-010 — Harden staging probe
- **Origin**: AILOG-2026-06-01-002 §Follow-ups
- **Origin-class**: staging
- **Status**: open
- **Severity**: blocking
- **Trigger**: ready
- **Destination**: mini-charter
- **Cost**: M
- **Labels**: staging-hardening, reliability

### FU-011 — Document the rollout runbook
- **Origin**: AILOG-2026-06-01-002 §Follow-ups
- **Status**: open
- **Trigger**: ready
- **Destination**: chore
- **Cost**: S
"#;

const TDE_TEMPLATE: &str = r#"---
id: TDE-YYYY-MM-DD-NNN
title: [Technical debt title]
status: identified
created: YYYY-MM-DD
agent: [agent-name-v1.0]
confidence: high | medium | low
review_required: false
risk_level: low | medium | high
type: code | architecture | infrastructure | documentation | testing
impact: low | medium | high
effort: low | medium | high
iso_42001_clause: []
tags: []
related: []
priority: null
assigned_to: null
promoted_from_followup: null    # FU-NNN if promoted from .straymark/follow-ups-backlog.md
---

# TDE: [Technical Debt Title]

## Summary

[Brief description of the identified technical debt]
"#;

fn cmd() -> Command {
    Command::cargo_bin("straymark").unwrap()
}

fn write_registry(straymark: &Path, content: &str) {
    std::fs::write(straymark.join("follow-ups-backlog.md"), content).unwrap();
}

// ───────────────────────────── list / status ─────────────────────────────

#[test]
fn list_shows_entries_from_v0_registry() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY);

    cmd()
        .args(["followups", "list", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-001"))
        .stdout(predicate::str::contains("FU-002"))
        .stdout(predicate::str::contains("charter-triggered"));
}

#[test]
fn list_filters_by_severity_and_label() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args(["followups", "list"])
        .args(["--severity", "blocking"])
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-010"))
        .stdout(predicate::str::contains("FU-011").not());

    cmd()
        .args(["followups", "list"])
        .args(["--label", "reliability"])
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-010"))
        .stdout(predicate::str::contains("FU-011").not());
}

#[test]
fn list_filters_by_bucket_and_status() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY);

    cmd()
        .args(["followups", "list"])
        .args(["--bucket", "ready", "--status", "open"])
        .arg(tmp.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-001"))
        .stdout(predicate::str::contains("FU-002").not());
}

#[test]
fn list_without_registry_prints_adoption_hint() {
    let tmp = TempDir::new().unwrap();
    scaffold(tmp.path());

    cmd()
        .args(["followups", "list", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No follow-ups registry yet"));
}

#[test]
fn status_pulse_recomputes_and_flags_stale_counters() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY);

    cmd()
        .args(["followups", "status", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        // Recomputed truth: 2 open, despite frontmatter claiming 47.
        .stdout(predicate::str::contains("total_open: 47"))
        .stdout(predicate::str::contains("real count is 2"));
}

#[test]
fn status_detail_shows_v1_dimensions() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args(["followups", "status", "FU-010", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("blocking"))
        .stdout(predicate::str::contains("staging"))
        .stdout(predicate::str::contains("staging-hardening"));
}

#[test]
fn status_unknown_entry_fails_with_hint() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args(["followups", "status", "FU-999", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn malformed_entry_is_warning_not_failure() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    let broken = V1_REGISTRY.replace(
        "### FU-011 — Document the rollout runbook",
        "### FU- — heading without a number",
    );
    write_registry(&straymark, &broken);

    cmd()
        .args(["followups", "list", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-010"))
        .stdout(predicate::str::contains("Malformed"));
}

// ───────────────────────────── drift ─────────────────────────────

fn write_ailog(straymark: &Path, filename: &str, content: &str) {
    std::fs::write(
        straymark.join("07-ai-audit/agent-logs").join(filename),
        content,
    )
    .unwrap();
}

const AILOG_WITH_FOLLOWUPS: &str = r#"# AILOG-2026-06-03-003 — staging incident

## Risk

- **R3 (new, not in Charter)**: bus handler writes escape the unit suite.

## Follow-ups

- Extend NON-owner integration coverage to write-path-B
- Formal SC validation run — closed in-Charter (commit `ab12cd34ef`), 5/6 pass

## Outcome

Done.
"#;

#[test]
fn drift_scan_all_detects_unextracted_ailog_and_exits_1() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY);
    write_ailog(&straymark, "AILOG-2026-06-03-003-staging.md", AILOG_WITH_FOLLOWUPS);

    cmd()
        .args(["followups", "drift", "--scan-all", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("AILOG-2026-06-03-003"));
}

#[test]
fn drift_scan_all_clean_when_everything_extracted() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY);
    // The only AILOG is already in fully_extracted_ailogs.
    write_ailog(
        &straymark,
        "AILOG-2026-04-11-001-first.md",
        "# AILOG\n\n## Follow-ups\n\n- already extracted\n",
    );

    cmd()
        .args(["followups", "drift", "--scan-all", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("registry in sync"));
}

#[test]
fn drift_apply_extracts_marks_suspected_recomputes_and_upgrades_v1() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY);
    write_ailog(&straymark, "AILOG-2026-06-03-003-staging.md", AILOG_WITH_FOLLOWUPS);

    cmd()
        .args(["followups", "drift", "--scan-all", "--apply", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extracted 3"))
        .stdout(predicate::str::contains("suspected-closed"))
        .stdout(predicate::str::contains("upgraded to schema v1"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();

    // (1) New entries with sequential numbers landed in the ready bucket.
    assert!(updated.contains("### FU-003"));
    assert!(updated.contains("### FU-004"));
    assert!(updated.contains("### FU-005"));
    // (2) Anti-noise (#214 Signal 1): the bullet with the closure marker is
    // suspected-closed, the plain one is open.
    let closed_idx = updated.find("Formal SC validation run").unwrap();
    let closed_block = &updated[closed_idx..closed_idx + 400];
    assert!(closed_block.contains("- **Status**: suspected-closed"));
    let open_idx = updated.find("Extend NON-owner integration coverage").unwrap();
    let open_block = &updated[open_idx..open_idx + 400];
    assert!(open_block.contains("- **Status**: open"));
    // (3) AILOG registered as fully extracted.
    assert!(updated.contains("AILOG-2026-06-03-003"));
    // (4) v0 → v1 upgrade + counter recompute (#214 Signal 2): 47 → real.
    assert!(updated.contains("schema_version: v1"));
    assert!(updated.contains("total_open: 4")); // FU-001, FU-002 + 2 new open
    assert!(updated.contains("total_suspected_closed: 1"));

    // (5) Idempotent: a second run is clean.
    cmd()
        .args(["followups", "drift", "--scan-all", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("registry in sync"));
}

#[test]
fn drift_apply_seeds_registry_from_template_when_absent() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    // Ship the framework template (as fw-4.21.0 does).
    std::fs::write(
        straymark.join("templates/follow-ups-backlog.md"),
        "---\nlast_scan: YYYY-MM-DD\nschema_version: v1\ntotal_open: 0\nbuckets:\n  - ready\nfully_extracted_ailogs: []\n---\n\n## Bucket: ready\n",
    )
    .unwrap();
    write_ailog(&straymark, "AILOG-2026-06-03-001-x.md", "# A\n\n## Follow-ups\n\n- do the thing\n");

    cmd()
        .args(["followups", "drift", "--scan-all", "--apply", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("from the framework template"))
        .stdout(predicate::str::contains("Extracted 1"));

    let created = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert!(created.contains("### FU-001 — do the thing"));
}

// ───────────────────────────── promote ─────────────────────────────

#[test]
fn promote_creates_tde_with_backlink_and_updates_entry() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    std::fs::write(straymark.join("templates/TEMPLATE-TDE.md"), TDE_TEMPLATE).unwrap();

    cmd()
        .args(["followups", "promote", "FU-010", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("promoted"))
        .stdout(predicate::str::contains("TDE-"));

    // TDE exists with the traceability backlink.
    let tde_dir = straymark.join("06-evolution/technical-debt");
    let tde_files: Vec<_> = std::fs::read_dir(&tde_dir).unwrap().flatten().collect();
    assert_eq!(tde_files.len(), 1);
    let tde_content = std::fs::read_to_string(tde_files[0].path()).unwrap();
    assert!(tde_content.contains("promoted_from_followup: FU-010"));
    assert!(tde_content.contains("Harden staging probe"));

    // Registry entry flipped to promoted with the TDE pointer; counters
    // recomputed (1 open left, 1 promoted).
    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let entry_idx = updated.find("### FU-010").unwrap();
    let entry_block = &updated[entry_idx..updated.find("### FU-011").unwrap()];
    assert!(entry_block.contains("- **Status**: promoted"));
    assert!(entry_block.contains("- **Promoted to**: TDE-"));
    assert!(entry_block.contains("- **Destination**: TDE-"));
    assert!(updated.contains("total_open: 1"));
    assert!(updated.contains("total_promoted: 1"));
}

#[test]
fn promote_rejects_already_promoted_entry() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    std::fs::write(straymark.join("templates/TEMPLATE-TDE.md"), TDE_TEMPLATE).unwrap();

    cmd()
        .args(["followups", "promote", "FU-010", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success();

    cmd()
        .args(["followups", "promote", "FU-010", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already promoted"));
}

#[test]
fn promote_unknown_entry_fails_with_hint() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args(["followups", "promote", "FU-404", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ───────────────────────────── status command block ─────────────────────────────

#[test]
fn project_status_shows_followups_block() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    // status needs the manifest to exist for the version row; tolerate absence.
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args(["status", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Follow-ups"))
        .stdout(predicate::str::contains("blocking"));
}

#[test]
fn project_status_hints_when_no_registry() {
    let tmp = TempDir::new().unwrap();
    scaffold(tmp.path());

    cmd()
        .args(["status", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No follow-ups registry yet"));
}
