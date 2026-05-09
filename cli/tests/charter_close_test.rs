//! Integration tests for `straymark charter close`. Only the
//! `--from-template --non-interactive` path is testable without a TTY; the
//! interactive flow is exercised manually and via the unit tests in
//! `cli/src/commands/charter/close.rs`.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

const CHARTER_TEMPLATE: &str = r#"---
charter_id: CHARTER-NN
status: declared
effort_estimate: M
trigger: "[1-line]"
---

# Charter: [BRIEF TITLE]

> **Status (mirrored from frontmatter — source of truth is above):** declared. Effort: M (~1.5h).
>
> **Origin:** scaffolded.

## Files to modify

| File | Change |
|---|---|

## Tasks

1. Sync.
"#;

const TELEMETRY_TEMPLATE: &str = r#"# StrayMark Charter telemetry — fill at Charter close.
charter_telemetry:
  charter_id: "CHARTER-NN"
  charter_title: "<short title>"
  closed_at: "YYYY-MM-DD"
  effort:
    estimated_effort: "M (~1.5h)"
    actual_effort: "M (~1.5h)"
  outcome:
    completed_as_planned: true
    scope_changes: "ninguno"
"#;

const TELEMETRY_SCHEMA: &str = include_str!(
    "../../dist/.straymark/schemas/charter-telemetry.schema.v0.json"
);

/// Set up a minimal StrayMark installation with both the Charter template and
/// the telemetry template + schema. Mirrors what `straymark init` would produce
/// after fw-4.6.0 ships.
fn setup_straymark(dir: &Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::create_dir_all(straymark.join("schemas")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(
        straymark.join("templates").join("charter-template.md"),
        CHARTER_TEMPLATE,
    )
    .unwrap();
    std::fs::write(
        straymark
            .join("templates")
            .join("charter-telemetry-template.yaml"),
        TELEMETRY_TEMPLATE,
    )
    .unwrap();
    std::fs::write(
        straymark.join("schemas").join("charter-telemetry.schema.v0.json"),
        TELEMETRY_SCHEMA,
    )
    .unwrap();
}

fn create_charter(dir: &Path, title: &str) {
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg(title)
        .arg(dir.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn charter_close_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn charter_close_unknown_charter_fails_clearly() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-99",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("CHARTER-99 not found"));
}

#[test]
fn charter_close_from_template_non_interactive_writes_telemetry_file() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Test Charter");

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("closed"))
        .stdout(predicate::str::contains("Telemetry:"));

    let telemetry_path = dir
        .path()
        .join(".straymark/charters/CHARTER-01.telemetry.yaml");
    assert!(telemetry_path.exists(), "telemetry file should exist");

    let content = std::fs::read_to_string(&telemetry_path).unwrap();
    // Pre-fills applied.
    assert!(
        content.contains("charter_id: \"CHARTER-01\""),
        "should pre-fill charter_id, got:\n{content}"
    );
    assert!(
        content.contains("Test Charter"),
        "should pre-fill title, got:\n{content}"
    );
    assert!(
        !content.contains("YYYY-MM-DD"),
        "closed_at placeholder should be replaced with today's date, got:\n{content}"
    );
}

#[test]
fn charter_close_syncs_body_status_mirror_line() {
    // F6 (cli-3.7.1): the body's `> **Status (mirrored from frontmatter ...):**` line
    // must reflect the new status, not just the frontmatter. Before the fix,
    // frontmatter said `closed` while body still said `declared` — silent drift
    // between the document's own claim ("mirrored from frontmatter") and reality.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Mirror Sync");

    let charter_path = dir.path().join("docs/charters/01-mirror-sync.md");
    let before = std::fs::read_to_string(&charter_path).unwrap();
    assert!(before.contains(":** declared. Effort:"), "{before}");

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&charter_path).unwrap();

    // Frontmatter is closed.
    assert!(after.contains("status: closed"), "frontmatter not bumped:\n{after}");
    assert!(!after.contains("status: declared"));

    // Body mirror line is also closed (the actual F6 fix).
    assert!(
        after.contains(":** closed. Effort:"),
        "body mirror line not synced, got:\n{after}"
    );
    assert!(
        !after.contains(":** declared. Effort:"),
        "old body mirror still present, got:\n{after}"
    );
}

#[test]
fn charter_close_writes_closed_at_when_absent() {
    // F8 (cli-3.7.2): every closed Charter should carry a closed_at:
    // YYYY-MM-DD line in frontmatter. Sentinel CHARTER-02..05 telemetry had
    // to add it manually 4× consecutively. The CLI now does it on close.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Closed At Test");
    let charter_path = dir.path().join("docs/charters/01-closed-at-test.md");
    let before = std::fs::read_to_string(&charter_path).unwrap();
    assert!(!before.contains("closed_at:"), "scaffold should not pre-write closed_at");

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&charter_path).unwrap();
    assert!(after.contains("closed_at:"), "closed_at must be written on close, got:\n{after}");
    // Today's date.
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert!(
        after.contains(&format!("closed_at: {today}")),
        "closed_at should be today ({today}), got:\n{after}"
    );
    // Closed-at sits adjacent to the bumped status line for readability.
    let status_pos = after.find("status: closed").expect("status: closed");
    let closed_at_pos = after.find("closed_at:").expect("closed_at line");
    let between = &after[status_pos..closed_at_pos];
    assert!(
        between.lines().count() <= 2,
        "closed_at should be on the line right after status: closed, got intervening:\n{between}"
    );
}

#[test]
fn charter_close_replaces_existing_closed_at_with_today() {
    // If a Charter was previously closed with a stale closed_at (manual
    // edit, or a re-close after status got reverted), the date should be
    // refreshed to today rather than left stale.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let charter_path = dir.path().join("docs/charters/01-stale.md");
    let stale = r#"---
charter_id: CHARTER-01
status: in-progress
closed_at: 2020-01-01
effort_estimate: M
trigger: "test"
---

# Charter: Stale

> **Status (mirrored from frontmatter — source of truth is above):** in-progress. Effort: M.

## Files to modify

| File | Change |
|---|---|

## Tasks

1. ok.
"#;
    std::fs::create_dir_all(dir.path().join("docs/charters")).unwrap();
    std::fs::write(&charter_path, stale).unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&charter_path).unwrap();
    assert!(!after.contains("closed_at: 2020-01-01"), "stale date should be replaced");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert!(after.contains(&format!("closed_at: {today}")));
}

#[test]
fn charter_close_bumps_status_to_closed() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Status Bump");

    let charter_path = dir.path().join("docs/charters/01-status-bump.md");
    let before = std::fs::read_to_string(&charter_path).unwrap();
    assert!(before.contains("status: declared"));

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&charter_path).unwrap();
    assert!(
        after.contains("status: closed"),
        "status line should be bumped, got:\n{after}"
    );
    assert!(
        !after.contains("status: declared"),
        "old declared status should be gone, got:\n{after}"
    );
}

#[test]
fn charter_close_idempotent_under_non_interactive() {
    // Running --from-template --non-interactive twice should not clobber
    // edits the user made between runs (we re-read the existing file).
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Idempotent");

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Simulate the user editing the telemetry file.
    let telemetry_path = dir
        .path()
        .join(".straymark/charters/CHARTER-01.telemetry.yaml");
    let edited = std::fs::read_to_string(&telemetry_path)
        .unwrap()
        .replace("ninguno", "menor");
    std::fs::write(&telemetry_path, &edited).unwrap();

    // Second run: should NOT overwrite the edit.
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let after = std::fs::read_to_string(&telemetry_path).unwrap();
    assert!(
        after.contains("scope_changes: \"menor\""),
        "user edit should be preserved, got:\n{after}"
    );
}

// ── F7 (cli-3.8.0): output differentiation first-run vs subsequent-run ──

#[test]
fn charter_close_first_run_prints_template_created_message() {
    // F7: first --from-template --non-interactive invocation writes the
    // template skeleton; output should tell the operator to edit and re-run,
    // NOT pretend the close is finalized.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "F7 First Run");

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Telemetry template created"))
        .stdout(predicate::str::contains("Edit the YAML"))
        // The "finalized" message is only printed on subsequent runs.
        .stdout(predicate::str::contains("finalized").not());
}

#[test]
fn charter_close_subsequent_run_prints_finalized_message() {
    // F7: subsequent invocation (telemetry already exists, presumably edited)
    // should run schema validation and report "finalized" on success.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "F7 Subsequent Run");

    // First invocation: writes the template.
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Simulate the operator editing the telemetry file with valid content.
    let telemetry_path = dir
        .path()
        .join(".straymark/charters/CHARTER-01.telemetry.yaml");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let valid_yaml = format!(
        r#"charter_telemetry:
  charter_id: "CHARTER-01"
  charter_title: "F7 Subsequent Run"
  closed_at: "{today}"
  effort:
    estimated_effort: "M (~1.5h)"
    actual_effort: "M (~1.5h)"
  outcome:
    completed_as_planned: true
    scope_changes: "ninguno"
"#
    );
    std::fs::write(&telemetry_path, valid_yaml).unwrap();

    // Second invocation: schema validates, output says finalized.
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("validation passed"))
        .stdout(predicate::str::contains("finalized"))
        // The first-run "Edit the YAML" guidance should NOT appear here.
        .stdout(predicate::str::contains("Edit the YAML").not())
        .stdout(predicate::str::contains("Telemetry template created").not());
}

#[test]
fn charter_close_subsequent_run_with_invalid_yaml_fails_clearly() {
    // F7: subsequent invocation that fails schema validation should bail
    // with a clear message, not silently print "finalized".
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "F7 Invalid Telemetry");

    // First invocation drops the template.
    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Now break the telemetry file (invalid scope_changes value).
    let telemetry_path = dir
        .path()
        .join(".straymark/charters/CHARTER-01.telemetry.yaml");
    std::fs::write(
        &telemetry_path,
        r#"charter_telemetry:
  charter_id: "CHARTER-01"
  charter_title: "test"
  closed_at: "2026-05-03"
  effort:
    estimated_effort: "M"
    actual_effort: "M"
  outcome:
    completed_as_planned: true
    scope_changes: "MAJOR_INVALID"
"#,
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "charter",
            "close",
            "CHARTER-01",
            "--from-template",
            "--non-interactive",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("validation"));
}
