//! Integration tests for `straymark followups` (cli-3.19.0,
//! ADR-2026-06-03-001): list / status / drift / promote against fixture
//! registries, including a Sentinel-style v0 registry (lenient parsing +
//! in-place upgrade) and the anti-noise / counter-recompute behaviors that
//! resolve issue #214 Signals 1-2.

use assert_cmd::{cargo_bin_cmd, Command};
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
    cargo_bin_cmd!("straymark")
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
    // The AILOG's only follow-up is already in the registry (FU-001, same
    // origin + description) — drift dedups by content hash (#231), so it is in
    // sync even though the AILOG is re-scanned rather than skipped wholesale.
    write_ailog(
        &straymark,
        "AILOG-2026-04-11-001-first.md",
        "# AILOG\n\n## Follow-ups\n\n- Wire the retry budget into the sync loop\n",
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

#[test]
fn drift_apply_recomputes_counters_with_zero_extractions() {
    // #222 Finding 1 (second half): after a manual-triage session there is
    // nothing left to extract, but --apply must still reconcile the
    // CLI-owned counters instead of early-returning with a stale file.
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY); // claims total_open: 47, real count 2
    // The AILOG's follow-up already exists as FU-001 (same origin + description)
    // → nothing to extract; --apply still reconciles the counters.
    write_ailog(
        &straymark,
        "AILOG-2026-04-11-001-first.md",
        "# AILOG\n\n## Follow-ups\n\n- Wire the retry budget into the sync loop\n",
    );

    cmd()
        .args(["followups", "drift", "--scan-all", "--apply", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("registry in sync"))
        .stdout(predicate::str::contains("Counters recomputed: 2 open"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert!(updated.contains("total_open: 2"));
    assert!(updated.contains("schema_version: v1"));
}

#[test]
fn drift_apply_extracts_born_resolved_idiom_as_suspected_closed() {
    // #222 Finding 2: the exact lnxdrive phrasing ("updated atomically in
    // this PR") must land as suspected-closed, not open/TBD noise.
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    write_ailog(
        &straymark,
        "AILOG-2026-06-04-001-drift.md",
        "# AILOG\n\n## Drift\n\n- R1 (new, not in Charter): probe path normalization — Charter `## Files to modify` row updated atomically in this PR.\n",
    );

    cmd()
        .args(["followups", "drift", "--scan-all", "--apply", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("suspected-closed"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let idx = updated.find("probe path normalization").unwrap();
    let after = &updated[idx..];
    // Bound to this entry (next heading or EOF) on a char boundary — never a
    // fixed byte offset, which can split a multi-byte char like `—`.
    let end = after.find("\n###").unwrap_or(after.len());
    assert!(after[..end].contains("- **Status**: suspected-closed"));
}

// ───────────────────────────── recount (#222 Finding 1) ─────────────────────────────

#[test]
fn recount_reconciles_counters_after_manual_triage_and_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V0_REGISTRY); // claims total_open: 47, real count 2

    cmd()
        .args(["followups", "recount", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Counters recomputed: 2 open"))
        .stdout(predicate::str::contains("upgraded to schema v1"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert!(updated.contains("total_open: 2"));
    assert!(updated.contains("schema_version: v1"));
    // Entries and body untouched — counters only.
    assert!(updated.contains("### FU-001 — Wire the retry budget into the sync loop"));
    assert!(updated.contains("### FU-002 — Extend E2E coverage to the write paths"));

    // Second run: nothing to do.
    cmd()
        .args(["followups", "recount", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("already in sync"));
}

#[test]
fn recount_surfaces_invisible_glued_entry_instead_of_a_blind_in_sync(/* #253 */) {
    // Reproduces the #253 failure mode: a well-formed entry whose `### FU-`
    // heading is glued to the previous line is invisible to the counter parser,
    // so `recount` would otherwise report a clean "in sync" while the file holds
    // one more open entry than the counter says. The structural integrity check
    // must surface it.
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    let registry = "---\nschema_version: v1\nfully_extracted_ailogs: []\ntotal_open: 1\ntotal_promoted: 0\ntotal_closed_in_session: 0\ntotal_phase_blocked: 0\ntotal_suspected_closed: 0\n---\n\n# Follow-ups Backlog\n\n## Bucket: operational\n\n### FU-157 — first entry\n- **Status**: open\n- Notes: this line has no trailing blank line.### FU-158 — glued, invisible\n- **Status**: open\n";
    write_registry(&straymark, registry);

    cmd()
        .args(["followups", "recount", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("invisible to the counters"));
}

#[test]
fn recount_errors_when_no_registry() {
    let tmp = TempDir::new().unwrap();
    scaffold(tmp.path());

    cmd()
        .args(["followups", "recount", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No follow-ups registry"));
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

#[test]
fn promote_surfaces_premise_reminder_and_does_not_stamp_without_flag() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    std::fs::write(straymark.join("templates/TEMPLATE-TDE.md"), TDE_TEMPLATE).unwrap();

    cmd()
        .args(["followups", "promote", "FU-010", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Premise re-check"))
        .stdout(predicate::str::contains("Is this still true?"));

    // No --premise-verified → no Verified-at stamp.
    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert!(!updated.contains("- **Verified-at**"));
}

#[test]
fn promote_with_premise_verified_stamps_verified_at() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    std::fs::write(straymark.join("templates/TEMPLATE-TDE.md"), TDE_TEMPLATE).unwrap();

    cmd()
        .args([
            "followups", "promote", "FU-010", "--premise-verified",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("re-verification recorded"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let entry_block = &updated[updated.find("### FU-010").unwrap()..updated.find("### FU-011").unwrap()];
    assert!(entry_block.contains("- **Status**: promoted"));
    assert!(entry_block.contains("- **Verified-at**:"));
}

// ───────────────────────────── verify ─────────────────────────────

#[test]
fn verify_records_premise_and_stamps_verified_at() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args([
            "followups", "verify", "FU-011",
            "--premise", "the rollout runbook template already exists",
            "--verified", "--at", "2026-07-18",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("premise recorded"))
        .stdout(predicate::str::contains("Verified-at → 2026-07-18"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let entry_block = &updated[updated.find("### FU-011").unwrap()..];
    assert!(entry_block.contains("- **Premise**: the rollout runbook template already exists"));
    assert!(entry_block.contains("- **Verified-at**: 2026-07-18"));
    // Verify does not change status.
    assert!(entry_block.contains("- **Status**: open"));
}

#[test]
fn verify_read_only_surfacing_without_flags_does_not_write() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    let before = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();

    cmd()
        .args(["followups", "verify", "FU-010", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Premise"))
        .stdout(predicate::str::contains("re-checked"));

    // Read-only: registry untouched.
    let after = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn verify_unknown_entry_fails_with_hint() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args(["followups", "verify", "FU-404", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn status_detail_shows_premise_and_nudges_when_unverified() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    // Add a premise to FU-011 and re-check the nudge fires (no Verified-at).
    let reg = V1_REGISTRY.replace(
        "### FU-011 — Document the rollout runbook\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups",
        "### FU-011 — Document the rollout runbook\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Premise**: the runbook does not exist yet",
    );
    write_registry(&straymark, &reg);

    cmd()
        .args(["followups", "status", "FU-011", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("the runbook does not exist yet"))
        .stdout(predicate::str::contains("dated hypothesis"))
        .stdout(predicate::str::contains("Re-verify against the code"));
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

// ──────────────── note / set-status / new (CHARTER-01, #355 + #360) ────────────────

#[test]
fn note_appends_dated_annotation_with_source_and_preserves_status() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args([
            "followups", "note", "FU-011",
            "Part-a shipped (size cap in the codec); part-b deferred.",
            "--source", "CHARTER-04",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-011 annotated"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let block = &updated[updated.find("### FU-011").unwrap()..];
    assert!(block.contains("CHARTER-04"), "source must be recorded: {block}");
    assert!(block.contains("part-b deferred."));
    // The point of #355's case: annotate WITHOUT changing status.
    assert!(block.contains("- **Status**: open"));
}

#[test]
fn note_composes_onto_existing_notes_instead_of_replacing_them() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    let path = tmp.path().to_str().unwrap();

    cmd()
        .args(["followups", "note", "FU-011", "first", "--path", path])
        .assert()
        .success();
    cmd()
        .args(["followups", "note", "FU-011", "second", "--path", path])
        .assert()
        .success();

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let block = &updated[updated.find("### FU-011").unwrap()..];
    assert!(block.contains("first"), "earlier note must survive: {block}");
    assert!(block.contains("second"));
    // One Notes bullet, not two — the field is a single line by parser contract.
    assert_eq!(block.matches("- **Notes**").count(), 1);
}

#[test]
fn note_refuses_to_write_a_malformed_registry() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    // A `### FU-` heading the parser cannot read: a surgical edit against a
    // mis-read structure could corrupt neighbouring entries (CHARTER-01 R1).
    write_registry(
        &straymark,
        &format!("{V1_REGISTRY}\n### FU-malformed heading with no id\n- **Status**: open\n"),
    );
    let before = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();

    cmd()
        .args([
            "followups", "note", "FU-011", "must not be written",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Refusing to write"));

    let after = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert_eq!(before, after, "nothing may be written when the guard fires");
}

#[test]
fn set_status_flips_status_and_recomputes_counters_in_one_step() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    // v0 fixture: frontmatter claims total_open 47, reality is 2.
    write_registry(&straymark, V0_REGISTRY);
    let path = tmp.path().to_str().unwrap();

    cmd()
        .args(["followups", "set-status", "FU-002", "closed", "--path", path])
        .assert()
        .success()
        .stdout(predicate::str::contains("open"))
        .stdout(predicate::str::contains("closed"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert!(updated.contains("total_open: 1"), "counters must move with the status: {updated}");
    assert!(updated.contains("schema_version: v1"), "write commands upgrade v0 → v1");
    let block = &updated[updated.find("### FU-002").unwrap()..];
    assert!(block.contains("- **Status**: closed"));

    // The desync #355 describes cannot happen: `recount` has nothing left to do.
    cmd()
        .args(["followups", "recount", "--path", path])
        .assert()
        .success()
        .stdout(predicate::str::contains("already in sync"));
}

#[test]
fn set_status_rejects_unknown_status_and_redirects_promoted() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    let path = tmp.path().to_str().unwrap();
    let before = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();

    // A typo would parse as `unknown` and silently drop the entry from every
    // counter, so it is refused rather than written.
    cmd()
        .args(["followups", "set-status", "FU-011", "done", "--path", path])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown status"));

    // `promoted` requires the TDE that gives the status meaning.
    cmd()
        .args(["followups", "set-status", "FU-011", "promoted", "--path", path])
        .assert()
        .failure()
        .stderr(predicate::str::contains("followups promote"));

    let after = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn set_status_is_a_noop_when_already_in_that_status() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    let before = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();

    cmd()
        .args([
            "followups", "set-status", "FU-011", "open",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("already"));

    let after = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn new_mints_ex_ante_entry_with_assigned_id_and_no_source_hash() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    // V1_REGISTRY's highest entry is FU-011 → the next id is FU-012, assigned
    // and written here, so the Charter can cite an entry that exists (#360).
    cmd()
        .args([
            "followups", "new",
            "--title", "Redis CI job deferred",
            "--origin", "CHARTER-06 §Scope",
            "--cost", "S",
            "--trigger", "when the Actions budget resets",
            "--premise", "the Redis adapter has no CI coverage today",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-012"))
        .stdout(predicate::str::contains("charter-triggered"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let block = &updated[updated.find("### FU-012").unwrap()..];
    assert!(block.contains("- **Origin**: CHARTER-06 §Scope"));
    // The schema already had a name for this origin; only the creation path was missing.
    assert!(block.contains("- **Origin-class**: ex-ante-planning"));
    assert!(block.contains("- **Status**: open"));
    assert!(block.contains("- **Cost**: S"));
    assert!(block.contains("- **Premise**: the Redis adapter has no CI coverage today"));
    // No AILOG exists to hash: inventing a Source-hash would make a later
    // `drift --apply` believe it had extracted something it never saw.
    assert!(!block.contains("Source-hash"), "ex-ante entry must carry no Source-hash: {block}");
    assert!(updated.contains("total_open: 3"), "counters include the new entry: {updated}");
}

#[test]
fn new_lands_in_the_requested_bucket_and_defaults_unset_fields_to_tbd() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);

    cmd()
        .args([
            "followups", "new",
            "--title", "Revisit the probe interval",
            "--origin", "CHARTER-07 §Out of scope",
            "--bucket", "ready",
            "--path", tmp.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    let ready = &updated[updated.find("## Bucket: ready").unwrap()..];
    assert!(ready.contains("### FU-012 — Revisit the probe interval"));
    let block = &updated[updated.find("### FU-012").unwrap()..];
    assert!(block.contains("- **Trigger**: TBD"));
    assert!(block.contains("- **Cost**: TBD"));
}

#[test]
fn new_requires_origin_and_a_canonical_bucket() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    let path = tmp.path().to_str().unwrap();

    cmd()
        .args([
            "followups", "new", "--title", "No origin", "--origin", "   ",
            "--path", path,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--origin is required"));

    cmd()
        .args([
            "followups", "new", "--title", "Bad bucket",
            "--origin", "CHARTER-06 §Scope", "--bucket", "someday",
            "--path", path,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown bucket"));
}

#[test]
fn new_then_drift_does_not_re_extract_or_duplicate(/* CHARTER-01 R4 */) {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, V1_REGISTRY);
    let path = tmp.path().to_str().unwrap();

    cmd()
        .args([
            "followups", "new", "--title", "Redis CI job deferred",
            "--origin", "CHARTER-06 §Scope", "--path", path,
        ])
        .assert()
        .success();

    // A hash-less entry must not make drift misbehave (it scans AILOGs; there
    // are none here, and the ex-ante entry is not derived from one).
    cmd()
        .args(["followups", "drift", "--scan-all", "--path", path])
        .assert()
        .success();

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert_eq!(updated.matches("### FU-012").count(), 1, "no duplicate: {updated}");
}

// ───────────────────────── GH #391: registry merging ─────────────────────────

#[test]
fn drift_apply_does_not_duplicate_entry_declared_from_moved_section() {
    // GH #391: when a follow-up declaration moves section, its content hash
    // changes — the title match must keep it out of the registry instead of
    // re-adding it as a fresh `open` duplicate that shadows the operator's
    // status.
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    let registry = V1_REGISTRY
        .replace("Harden staging probe", "Harden staging probe.")
        .replace(
            "### FU-010 — Harden staging probe.\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Origin-class**: staging\n- **Status**: open",
            "### FU-010 — Harden staging probe.\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Origin-class**: staging\n- **Status**: in-progress",
        );
    write_registry(&straymark, &registry);
    // Same description, different heading variant → different origin
    // section → different content hash.
    write_ailog(
        &straymark,
        "AILOG-2026-06-01-002-x.md",
        "# AILOG\n\n## Follow-ups (auditoría)\n\n- Harden staging probe.\n",
    );

    cmd()
        .args(["followups", "drift", "--scan-all", "--apply", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("registry in sync"));

    let updated = std::fs::read_to_string(straymark.join("follow-ups-backlog.md")).unwrap();
    assert_eq!(
        updated.matches("Harden staging probe.").count(),
        1,
        "no duplicate entry: {updated}"
    );
    let idx = updated.find("Harden staging probe.").unwrap();
    assert!(
        updated[idx..idx + 400].contains("- **Status**: in-progress"),
        "operator status preserved: {updated}"
    );
}

const MERGE_BASE: &str = r#"---
schema_version: v1
last_scan: 2026-07-01
total_open: 2
buckets:
  - ready
fully_extracted_ailogs: []
---

## Bucket: ready

### FU-010 — Harden staging probe
- **Origin**: AILOG-2026-06-01-002 §Follow-ups
- **Status**: open

### FU-011 — Document the rollout runbook
- **Origin**: AILOG-2026-06-01-002 §Follow-ups
- **Status**: open
"#;

#[test]
fn merge_driver_preserves_closures_and_unions_entries() {
    // GH #391: the adopter scenario — the branch closed FU-010 and added a
    // new entry; ours added a different entry reusing FU-012's number. The
    // structural merge must keep the closure and union both additions.
    let tmp = TempDir::new().unwrap();
    let base_path = tmp.path().join("base.md");
    let ours_path = tmp.path().join("ours.md");
    let theirs_path = tmp.path().join("theirs.md");

    std::fs::write(&base_path, MERGE_BASE).unwrap();
    std::fs::write(
        &ours_path,
        format!(
            "{MERGE_BASE}\n### FU-012 — Add metrics for the sync loop\n- **Origin**: AILOG-2026-07-02-001 §Follow-ups\n- **Status**: open\n"
        ),
    )
    .unwrap();
    std::fs::write(
        &theirs_path,
        format!(
            "{}\n### FU-012 — Fix probe flake\n- **Origin**: AILOG-2026-07-03-001 §Follow-ups\n- **Status**: open\n",
            MERGE_BASE.replace(
                "### FU-010 — Harden staging probe\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: open",
                "### FU-010 — Harden staging probe\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: closed",
            )
        ),
    )
    .unwrap();

    cmd()
        .args([
            "followups", "merge-driver",
            base_path.to_str().unwrap(),
            ours_path.to_str().unwrap(),
            theirs_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 status(es) preserved from theirs"))
        .stdout(predicate::str::contains("1 entry appended from theirs"));

    let merged = std::fs::read_to_string(&ours_path).unwrap();
    // Closure from theirs survived (the regression this issue is about).
    let idx = merged.find("Harden staging probe").unwrap();
    assert!(merged[idx..idx + 300].contains("- **Status**: closed"), "{merged}");
    // Both additions present; theirs' colliding FU-012 was renumbered.
    assert!(merged.contains("Add metrics for the sync loop"), "{merged}");
    assert!(merged.contains("### FU-013 — Fix probe flake"), "{merged}");
    // Counters recomputed from the merged body: FU-011 + FU-012 + FU-013 open.
    assert!(merged.contains("total_open: 3"), "{merged}");
}

#[test]
fn merge_driver_respects_deletion_and_keeps_conflicts_visible() {
    let tmp = TempDir::new().unwrap();
    let base_path = tmp.path().join("base.md");
    let ours_path = tmp.path().join("ours.md");
    let theirs_path = tmp.path().join("theirs.md");

    std::fs::write(&base_path, MERGE_BASE).unwrap();
    // Ours unchanged vs base; theirs deletes FU-011 and sets FU-010 to
    // superseded while ours set it to closed — same rank, real conflict.
    std::fs::write(
        &ours_path,
        MERGE_BASE.replace(
            "### FU-010 — Harden staging probe\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: open",
            "### FU-010 — Harden staging probe\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: closed",
        ),
    )
    .unwrap();
    std::fs::write(
        &theirs_path,
        MERGE_BASE
            .replace(
                "### FU-010 — Harden staging probe\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: open",
                "### FU-010 — Harden staging probe\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: superseded",
            )
            .replace(
                "\n### FU-011 — Document the rollout runbook\n- **Origin**: AILOG-2026-06-01-002 §Follow-ups\n- **Status**: open\n",
                "\n",
            ),
    )
    .unwrap();

    cmd()
        .args([
            "followups", "merge-driver",
            base_path.to_str().unwrap(),
            ours_path.to_str().unwrap(),
            theirs_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("closed (ours) vs superseded (theirs)"));

    let merged = std::fs::read_to_string(&ours_path).unwrap();
    assert!(!merged.contains("rollout runbook"), "theirs' deletion respected: {merged}");
    let idx = merged.find("Harden staging probe").unwrap();
    assert!(merged[idx..].contains("- **Status**: closed"), "ours kept on same-rank conflict: {merged}");
}

// ───────────────────────────── verify --claims (cli-3.47.0, #419) ─────────────────────────────

/// Registry whose entries carry mechanical code claims: one good, one with a
/// phantom path, one with a phantom symbol, one with a stale "no callers"
/// claim, one closed entry whose phantom claim must be ignored by the batch.
const CLAIMS_REGISTRY: &str = r#"---
schema_version: v1
last_scan: 2026-08-13
total_open: 4
total_promoted: 0
total_closed_in_session: 1
total_phase_blocked: 0
total_suspected_closed: 0
buckets:
  - ready
fully_extracted_ailogs: []
---

## Bucket: ready

### FU-020 — Move the parser off `src/old/parser.rs`
- **Origin**: AILOG-2026-08-13-001 §Follow-ups
- **Status**: open
- **Trigger**: ready
- **Destination**: chore
- **Cost**: S

### FU-021 — Delete `definitely_gone_fn`
- **Origin**: AILOG-2026-08-13-001 §Follow-ups
- **Status**: open
- **Premise**: `definitely_gone_fn` is still referenced from the sync loop
- **Trigger**: ready
- **Destination**: chore
- **Cost**: S

### FU-022 — `wired_helper` has no callers and should be deleted
- **Origin**: AILOG-2026-08-13-001 §Follow-ups
- **Status**: in-progress
- **Trigger**: ready
- **Destination**: chore
- **Cost**: S

### FU-023 — Closed entry citing `ghost_fn`
- **Origin**: AILOG-2026-08-13-001 §Follow-ups
- **Status**: closed
- **Trigger**: ready
- **Destination**: chore
- **Cost**: S
"#;

#[test]
fn verify_claims_flags_phantom_path_and_symbol_but_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);

    cmd()
        .args(["followups", "verify", "--claims", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLAIM-PATH-GONE"))
        .stdout(predicate::str::contains("src/old/parser.rs"))
        .stdout(predicate::str::contains("CLAIM-SYMBOL-GONE"))
        .stdout(predicate::str::contains("definitely_gone_fn"));
}

#[test]
fn verify_claims_flags_stale_dead_claim_when_symbol_has_callers() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src/a.rs"), "fn wired_helper() {}\n").unwrap();
    std::fs::write(tmp.path().join("src/b.rs"), "use crate::a::wired_helper;\n").unwrap();

    cmd()
        .args(["followups", "verify", "--claims", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLAIM-STALE-DEAD"))
        .stdout(predicate::str::contains("wired_helper"));
}

#[test]
fn verify_claims_ignores_closed_entries_in_batch() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);

    // ghost_fn appears nowhere, but FU-023 is closed — the batch must not flag it.
    cmd()
        .args(["followups", "verify", "--claims", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-023").not());
}

#[test]
fn verify_claims_clean_tree_reports_success() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);
    std::fs::create_dir_all(tmp.path().join("src/old")).unwrap();
    std::fs::write(tmp.path().join("src/old/parser.rs"), "fn parse() {}\n").unwrap();
    std::fs::write(tmp.path().join("src/sync.rs"), "fn definitely_gone_fn() {}\nfn wired_helper() {}\n").unwrap();

    cmd()
        .args(["followups", "verify", "--claims", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLAIM-PATH-GONE").not())
        .stdout(predicate::str::contains("CLAIM-SYMBOL-GONE").not())
        .stdout(predicate::str::contains("re-derived clean"));
}

#[test]
fn verify_claims_with_fu_id_filters_to_that_entry() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);

    // FU-021's phantom symbol is flagged; FU-020's phantom path is not in scope.
    cmd()
        .args(["followups", "verify", "FU-021", "--claims", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("FU-021"))
        .stdout(predicate::str::contains("CLAIM-SYMBOL-GONE"))
        .stdout(predicate::str::contains("FU-020").not());
}

#[test]
fn verify_claims_conflicts_with_per_entry_flags() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);

    cmd()
        .args(["followups", "verify", "FU-020", "--claims", "--verified", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn verify_without_fu_id_or_claims_fails() {
    let tmp = TempDir::new().unwrap();
    let straymark = scaffold(tmp.path());
    write_registry(&straymark, CLAIMS_REGISTRY);

    cmd()
        .args(["followups", "verify", "--path", tmp.path().to_str().unwrap()])
        .assert()
        .failure();
}
