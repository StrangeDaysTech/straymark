use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_update_framework_shows_error_when_not_initialized() {
    let dir = tempfile::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.current_dir(dir.path())
        .arg("update-framework")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_update_in_empty_dir_does_not_fail() {
    let dir = tempfile::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.current_dir(dir.path())
        .arg("update")
        .assert()
        .success();
}

#[test]
fn test_remove_shows_error_when_not_initialized() {
    let dir = tempfile::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.current_dir(dir.path())
        .arg("remove")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}
