use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_status_not_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("status")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("not installed"));
}

#[test]
fn test_status_with_minimal_install() {
    let dir = TempDir::new().unwrap();
    let straymark = dir.path().join(".straymark");
    std::fs::create_dir_all(&straymark).unwrap();

    // Create minimal config and manifest
    std::fs::write(
        straymark.join("config.yml"),
        "language: es\n",
    )
    .unwrap();
    std::fs::write(
        straymark.join("dist-manifest.yml"),
        "version: \"2.1.0\"\ndescription: test\nfiles: []\n",
    )
    .unwrap();

    // Create some fake documents
    let reqs_dir = straymark.join("01-requirements");
    std::fs::create_dir_all(&reqs_dir).unwrap();
    std::fs::write(
        reqs_dir.join("REQ-2025-01-01-001-test.md"),
        "# Test requirement",
    )
    .unwrap();
    std::fs::write(
        reqs_dir.join("REQ-2025-01-02-002-another.md"),
        "# Another",
    )
    .unwrap();

    let logs_dir = straymark.join("07-ai-audit/agent-logs");
    std::fs::create_dir_all(&logs_dir).unwrap();
    std::fs::write(
        logs_dir.join("AILOG-2025-03-01-001-session.md"),
        "# Log",
    )
    .unwrap();

    // Create new Fase 1 directories
    let sec_dir = straymark.join("08-security");
    std::fs::create_dir_all(&sec_dir).unwrap();
    std::fs::write(sec_dir.join(".gitkeep"), "").unwrap();

    let models_dir = straymark.join("09-ai-models");
    std::fs::create_dir_all(&models_dir).unwrap();
    std::fs::write(models_dir.join(".gitkeep"), "").unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("status")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2.1.0")
                .and(predicate::str::contains("es"))
                .and(predicate::str::contains("REQ"))
                .and(predicate::str::contains("AILOG")),
        );
}

#[test]
fn test_status_recognizes_new_doc_types() {
    let dir = TempDir::new().unwrap();
    let straymark = dir.path().join(".straymark");
    std::fs::create_dir_all(&straymark).unwrap();

    std::fs::write(
        straymark.join("config.yml"),
        "language: en\n",
    )
    .unwrap();
    std::fs::write(
        straymark.join("dist-manifest.yml"),
        "version: \"3.0.0\"\ndescription: test\nfiles: []\n",
    )
    .unwrap();

    // Create documents of new types
    let sec_dir = straymark.join("08-security");
    std::fs::create_dir_all(&sec_dir).unwrap();
    std::fs::write(
        sec_dir.join("SEC-2026-03-25-001-threat-model.md"),
        "# Security Assessment",
    )
    .unwrap();

    let models_dir = straymark.join("09-ai-models");
    std::fs::create_dir_all(&models_dir).unwrap();
    std::fs::write(
        models_dir.join("MCARD-2026-03-25-001-classifier.md"),
        "# Model Card",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("status")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("SEC")
                .and(predicate::str::contains("MCARD")),
        );
}

#[test]
fn test_status_incomplete_structure() {
    let dir = TempDir::new().unwrap();
    let straymark = dir.path().join(".straymark");

    // Create only some directories
    std::fs::create_dir_all(straymark.join("00-governance")).unwrap();
    std::fs::create_dir_all(straymark.join("01-requirements")).unwrap();
    // Intentionally skip other directories

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("status")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("missing"));
}

#[test]
fn test_status_explicit_path_argument() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("status")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("not installed"));
}
