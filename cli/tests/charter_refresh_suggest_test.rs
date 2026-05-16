//! Integration smoke tests for `straymark charter refresh-suggest` (fw-4.16.0+).
//!
//! See `cli/src/commands/charter/refresh_suggest.rs` for unit coverage of the
//! parsing helpers; this file exercises the assembled binary end-to-end.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn write_charter(dir: &Path, charter_id: &str) {
    let charters = dir.join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    let stem = charter_id.strip_prefix("CHARTER-").unwrap_or(charter_id);
    let body = format!(
        "---\ncharter_id: {charter_id}\nstatus: closed\neffort_estimate: M\ntrigger: \"x\"\n---\nBody.\n"
    );
    std::fs::write(charters.join(format!("{stem}.md")), body).unwrap();
}

fn write_telemetry(dir: &Path, charter_id: &str, closed_at: &str, r_n: u32) {
    let charters = dir.join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    let stem = charter_id.strip_prefix("CHARTER-").unwrap_or(charter_id);
    let body = format!(
        "charter_telemetry:\n  \
         charter_id: \"{charter_id}\"\n  \
         charter_title: \"x\"\n  \
         closed_at: \"{closed_at}\"\n  \
         effort:\n    estimated_effort: \"M\"\n    actual_effort: \"M\"\n  \
         agent_quality:\n    r_n_plus_one_emergent_count: {r_n}\n  \
         outcome:\n    completed_as_planned: true\n    scope_changes: \"ninguno\"\n"
    );
    std::fs::write(
        charters.join(format!("{stem}.telemetry.yaml")),
        body,
    )
    .unwrap();
}

#[test]
fn refresh_suggest_recommends_when_rolling_mean_exceeds_threshold() {
    let dir = TempDir::new().unwrap();
    write_charter(dir.path(), "CHARTER-10-commshub-us1");
    write_charter(dir.path(), "CHARTER-11-commshub-us2");
    write_charter(dir.path(), "CHARTER-12-commshub-us3");
    write_telemetry(dir.path(), "CHARTER-10-commshub-us1", "2026-04-01", 7);
    write_telemetry(dir.path(), "CHARTER-11-commshub-us2", "2026-04-15", 8);
    write_telemetry(dir.path(), "CHARTER-12-commshub-us3", "2026-05-01", 9);

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "refresh-suggest",
            "commshub",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Recommend a pre-declare SpecKit refresh"));
}

#[test]
fn refresh_suggest_holds_when_rolling_mean_within_threshold() {
    let dir = TempDir::new().unwrap();
    write_charter(dir.path(), "CHARTER-20-commshub-us1");
    write_charter(dir.path(), "CHARTER-21-commshub-us2");
    write_charter(dir.path(), "CHARTER-22-commshub-us3");
    write_telemetry(dir.path(), "CHARTER-20-commshub-us1", "2026-04-01", 2);
    write_telemetry(dir.path(), "CHARTER-21-commshub-us2", "2026-04-15", 1);
    write_telemetry(dir.path(), "CHARTER-22-commshub-us3", "2026-05-01", 3);

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "refresh-suggest",
            "commshub",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("refresh not recommended"));
}

#[test]
fn refresh_suggest_reports_insufficient_when_chain_too_short() {
    let dir = TempDir::new().unwrap();
    write_charter(dir.path(), "CHARTER-30-commshub-us1");
    write_telemetry(dir.path(), "CHARTER-30-commshub-us1", "2026-05-01", 99);

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "refresh-suggest",
            "commshub",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Chain shorter than 3 closed Charters"));
}

#[test]
fn refresh_suggest_zero_matches_prints_nothing_to_suggest() {
    let dir = TempDir::new().unwrap();
    write_charter(dir.path(), "CHARTER-40-something-else");
    write_telemetry(dir.path(), "CHARTER-40-something-else", "2026-05-01", 99);

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "refresh-suggest",
            "commshub",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No closed Charters match module `commshub`"));
}

#[test]
fn refresh_suggest_threshold_flag_overrides_default() {
    let dir = TempDir::new().unwrap();
    // r_n_plus_one = 5 each → rolling mean = 5, below default 6 but above 3.
    write_charter(dir.path(), "CHARTER-50-commshub-us1");
    write_charter(dir.path(), "CHARTER-51-commshub-us2");
    write_charter(dir.path(), "CHARTER-52-commshub-us3");
    write_telemetry(dir.path(), "CHARTER-50-commshub-us1", "2026-04-01", 5);
    write_telemetry(dir.path(), "CHARTER-51-commshub-us2", "2026-04-15", 5);
    write_telemetry(dir.path(), "CHARTER-52-commshub-us3", "2026-05-01", 5);

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "refresh-suggest",
            "commshub",
            "--threshold",
            "3",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Recommend a pre-declare SpecKit refresh"));
}
