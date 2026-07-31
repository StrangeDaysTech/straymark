//! Integration tests for `straymark charter batch-complete` (#379).
//! Verifies that spec-originated Charters (with `execution_ailogs` instead of
//! `originating_ailogs`) can use batch-complete.

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn setup_straymark(dir: &Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::create_dir_all(straymark.join("charters")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();
}

/// Write a Charter with `execution_ailogs` (spec-originated, #379 case).
fn write_spec_originated_charter(dir: &Path, ailog_id: &str) {
    let charters_dir = dir.join(".straymark/charters");
    let content = format!(
        "---\ncharter_id: CHARTER-28-permiso-resuelto\nstatus: in-progress\neffort_estimate: L\ntrigger: \"test trigger\"\noriginating_spec: specs/005/spec.md\nexecution_ailogs:\n  - {}\n---\n\n# Charter: Permiso resuelto\n\n## Tasks\n\n1. Run.\n",
        ailog_id
    );
    std::fs::write(charters_dir.join("28-permiso-resuelto.md"), content).unwrap();
}

/// Write a Charter with `originating_ailogs` (AILOG-originated, existing case).
fn write_ailog_originated_charter(dir: &Path, ailog_id: &str) {
    let charters_dir = dir.join(".straymark/charters");
    let content = format!(
        "---\ncharter_id: CHARTER-01-origin\nstatus: in-progress\neffort_estimate: M\ntrigger: \"test trigger\"\noriginating_ailogs:\n  - {}\n---\n\n# Charter: Origin\n\n## Tasks\n\n1. Run.\n",
        ailog_id
    );
    std::fs::write(charters_dir.join("01-origin.md"), content).unwrap();
}

/// Write a Charter with neither field (error case).
fn write_bare_charter(dir: &Path) {
    let charters_dir = dir.join(".straymark/charters");
    let content =
        "---\ncharter_id: CHARTER-02-bare\nstatus: in-progress\neffort_estimate: S\ntrigger: \"test trigger\"\n---\n\n# Charter: Bare\n\n## Tasks\n\n1. Run.\n";
    std::fs::write(charters_dir.join("02-bare.md"), content).unwrap();
}

/// Create an AILOG file with a Batch Ledger containing a pending batch.
fn write_ailog_with_ledger(dir: &Path, ailog_id: &str) {
    let logs_dir = dir.join(".straymark/07-ai-audit/agent-logs");
    let content = format!(
        "---\nailog_id: {}\nstatus: draft\n---\n\n# AILOG\n\n## Batch Ledger\n\n### Batch 1 — setup\n\n(pending)\n\n### Batch 2 — impl\n\n(pending)\n",
        ailog_id
    );
    std::fs::write(logs_dir.join(format!("{}-test.md", ailog_id)), content).unwrap();
}

#[test]
fn batch_complete_resolves_execution_ailogs_for_spec_originated_charter() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_spec_originated_charter(dir.path(), "AILOG-2026-07-30-001");
    write_ailog_with_ledger(dir.path(), "AILOG-2026-07-30-001");

    cargo_bin_cmd!("straymark")
        .arg("charter")
        .arg("batch-complete")
        .arg("CHARTER-28")
        .arg("1")
        .arg("--note")
        .arg("Batch 1 done: migrations + handlers")
        .arg("--non-interactive")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch 1"))
        .stderr(predicate::str::contains("spec-originated"));
}

#[test]
fn batch_complete_still_works_with_originating_ailogs() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_ailog_originated_charter(dir.path(), "AILOG-2026-07-30-002");
    write_ailog_with_ledger(dir.path(), "AILOG-2026-07-30-002");

    cargo_bin_cmd!("straymark")
        .arg("charter")
        .arg("batch-complete")
        .arg("CHARTER-01")
        .arg("1")
        .arg("--note")
        .arg("Batch 1 done")
        .arg("--non-interactive")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch 1"));
}

#[test]
fn batch_complete_errors_when_neither_field_present() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    write_bare_charter(dir.path());

    cargo_bin_cmd!("straymark")
        .arg("charter")
        .arg("batch-complete")
        .arg("CHARTER-02")
        .arg("1")
        .arg("--note")
        .arg("x")
        .arg("--non-interactive")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("neither `originating_ailogs` nor `execution_ailogs`"));
}
