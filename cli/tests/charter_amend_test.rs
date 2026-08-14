//! Integration smoke tests for `straymark charter amend` (fw-4.16.0+).
//!
//! See `cli/src/commands/charter/amend.rs` for unit coverage; this exercises
//! the end-to-end CLI surface.

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

fn write_closed_charter(dir: &Path) {
    let charters = dir.join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    let body = "---\n\
charter_id: CHARTER-18-commshub-us5\n\
status: closed\n\
effort_estimate: L\n\
trigger: \"x\"\n\
---\n\
Body.\n";
    std::fs::write(charters.join("18-commshub-us5.md"), body).unwrap();
}

fn write_prior_ailog(dir: &Path) {
    let logs = dir.join(".straymark/07-ai-audit/agent-logs");
    std::fs::create_dir_all(&logs).unwrap();
    let body = "---\n\
id: AILOG-2026-05-14-049\n\
type: ailog\n\
charter_id: CHARTER-18-commshub-us5\n\
---\n\
\n\
# Original CHARTER-18 AILOG\n\
\n\
Body referencing CHARTER-18-commshub-us5.\n";
    std::fs::write(logs.join("AILOG-2026-05-14-049-original.md"), body).unwrap();
}

fn write_minimal_telemetry(dir: &Path) -> std::path::PathBuf {
    let charters = dir.join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    let path = charters.join("18-commshub-us5.telemetry.yaml");
    std::fs::write(
        &path,
        "charter_telemetry:\n  \
         charter_id: \"CHARTER-18-commshub-us5\"\n  \
         charter_title: \"x\"\n  \
         closed_at: \"2026-05-15\"\n  \
         effort:\n    estimated_effort: \"L\"\n    actual_effort: \"L\"\n  \
         outcome:\n    completed_as_planned: true\n    scope_changes: \"ninguno\"\n",
    )
    .unwrap();
    path
}

#[test]
fn amend_creates_new_ailog_and_appends_historical_correction() {
    let dir = TempDir::new().unwrap();
    write_closed_charter(dir.path());
    write_prior_ailog(dir.path());

    cargo_bin_cmd!("straymark")
        .args([
            "charter",
            "amend",
            "CHARTER-18",
            "--trigger",
            "external_audit",
            "--findings-closed",
            "5",
            "--ailog-title",
            "post-close DI wiring",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("post_close_amendment YAML"))
        .stdout(predicate::str::contains("Created new AILOG"))
        .stdout(predicate::str::contains("Appended `## Historical correction"));

    // Original AILOG must now contain the Historical correction subsection.
    let original_path = dir
        .path()
        .join(".straymark/07-ai-audit/agent-logs/AILOG-2026-05-14-049-original.md");
    let original = std::fs::read_to_string(&original_path).unwrap();
    assert!(
        original.contains("## Historical correction ("),
        "original AILOG must carry Historical correction subsection:\n{original}"
    );
    assert!(
        original.contains("external_audit"),
        "trigger reason recorded in correction subsection"
    );

    // A new AILOG file must exist with today's date.
    let logs = dir.path().join(".straymark/07-ai-audit/agent-logs");
    let entries: Vec<String> = std::fs::read_dir(&logs)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.starts_with("AILOG-") && n != "AILOG-2026-05-14-049-original.md")
        .collect();
    assert_eq!(entries.len(), 1, "exactly one new AILOG must have been created: {entries:?}");
    assert!(
        entries[0].contains("post-close-di-wiring"),
        "new AILOG filename must carry the slugified title: {}",
        entries[0]
    );
}

#[test]
fn amend_fails_when_charter_not_closed() {
    let dir = TempDir::new().unwrap();
    let charters = dir.path().join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    std::fs::write(
        charters.join("19-still-running.md"),
        "---\ncharter_id: CHARTER-19-still-running\nstatus: in-progress\neffort_estimate: M\ntrigger: \"x\"\n---\nBody.\n",
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .args([
            "charter",
            "amend",
            "CHARTER-19",
            "--trigger",
            "external_audit",
            "--ailog-title",
            "should fail",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not closed"));
}

#[test]
fn amend_rejects_invalid_trigger() {
    let dir = TempDir::new().unwrap();
    write_closed_charter(dir.path());

    cargo_bin_cmd!("straymark")
        .args([
            "charter",
            "amend",
            "CHARTER-18",
            "--trigger",
            "totally-bogus",
            "--ailog-title",
            "x",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure();
}

#[test]
fn amend_merge_into_writes_post_close_amendment_to_telemetry() {
    let dir = TempDir::new().unwrap();
    write_closed_charter(dir.path());
    write_prior_ailog(dir.path());
    let telemetry_path = write_minimal_telemetry(dir.path());

    cargo_bin_cmd!("straymark")
        .args([
            "charter",
            "amend",
            "CHARTER-18",
            "--trigger",
            "external_audit",
            "--findings-closed",
            "3",
            "--ailog-title",
            "merge-into smoke",
            "--merge-into",
            telemetry_path.to_str().unwrap(),
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Merged `post_close_amendment:`"));

    let merged = std::fs::read_to_string(&telemetry_path).unwrap();
    assert!(
        merged.contains("post_close_amendment:"),
        "merged telemetry must carry the post_close_amendment block:\n{merged}"
    );
    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&merged).expect("merged YAML must parse");
    let trigger = parsed
        .get("charter_telemetry")
        .and_then(|v| v.get("post_close_amendment"))
        .and_then(|v| v.get("trigger"))
        .and_then(|v| v.as_str());
    assert_eq!(trigger, Some("external_audit"));
}

#[test]
fn amend_new_ailog_renders_guard_closure_placeholder() {
    let dir = TempDir::new().unwrap();
    write_closed_charter(dir.path());
    write_prior_ailog(dir.path());

    cargo_bin_cmd!("straymark")
        .args([
            "charter",
            "amend",
            "CHARTER-18",
            "--trigger",
            "external_audit",
            "--findings-closed",
            "2",
            "--ailog-title",
            "guard closure scaffold",
            "--path",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let logs = dir.path().join(".straymark/07-ai-audit/agent-logs");
    let new_ailog = std::fs::read_dir(&logs)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("AILOG-") && n.contains("guard-closure-scaffold"))
        })
        .expect("new AILOG exists");
    let content = std::fs::read_to_string(new_ailog).unwrap();
    assert!(
        content.contains("guard_closure:"),
        "remediation AILOG template renders the guard_closure field (#419):\n{content}"
    );
    assert!(
        content.contains("- finding: F1"),
        "placeholder item per finding:\n{content}"
    );

    // The placeholder frontmatter must parse as valid YAML carrying the field.
    let fm_text = content
        .strip_prefix("---\n")
        .and_then(|rest| rest.split("\n---\n").next())
        .expect("frontmatter delimiters");
    let parsed: serde_yaml::Value = serde_yaml::from_str(fm_text).expect("frontmatter parses");
    assert!(parsed.get("guard_closure").is_some());
}
