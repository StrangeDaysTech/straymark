use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_shows_error_when_already_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join(".straymark")).unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("init")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("StrayMark"));
}

#[test]
fn test_version_output() {
    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("straymark"));
}
