//! Integration tests for `straymark analyze declared-vs-wired` (finding #209,
//! POLISH-CHARTER-PATTERN.md sub-class 5).

#![cfg(feature = "analyze")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

/// Lay out a client proxy (two declared methods) and a daemon interface (one
/// implemented), reproducing the LNXDrive regression: the client still declares
/// a method the daemon removed.
fn setup_proxy_vs_interface(dir: &Path) {
    std::fs::create_dir_all(dir.join("client/src")).unwrap();
    std::fs::create_dir_all(dir.join("daemon/src")).unwrap();
    std::fs::write(
        dir.join("client/src/proxy.rs"),
        "fn complete_auth_via_goa() {}\nfn complete_auth_with_tokens() {}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("daemon/src/interface.rs"),
        "fn complete_auth_via_goa() {}\n",
    )
    .unwrap();
}

fn run_inline(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.args(["analyze", "declared-vs-wired"])
        .arg(dir.to_str().unwrap())
        .args(["--declared-glob", "client/**/*.rs"])
        .args(["--declared-pattern", r"fn (\w+)"])
        .args(["--wired-glob", "daemon/**/*.rs"])
        .args(["--wired-pattern", r"fn (\w+)"]);
    cmd
}

#[test]
fn flags_declared_symbol_with_no_wiring() {
    let dir = TempDir::new().unwrap();
    setup_proxy_vs_interface(dir.path());

    run_inline(dir.path())
        .assert()
        .failure() // exit 1 — a finding
        .stdout(predicate::str::contains("complete_auth_with_tokens"))
        .stdout(predicate::str::contains("declared symbol(s) with NO wiring"));
}

#[test]
fn clean_when_every_declared_symbol_is_wired() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("client/src")).unwrap();
    std::fs::create_dir_all(dir.path().join("daemon/src")).unwrap();
    std::fs::write(dir.path().join("client/src/proxy.rs"), "fn ping() {}\n").unwrap();
    std::fs::write(dir.path().join("daemon/src/interface.rs"), "fn ping() {}\n").unwrap();

    run_inline(dir.path())
        .assert()
        .success() // exit 0 — clean
        .stdout(predicate::str::contains("Every declared symbol has a wiring counterpart"));
}

#[test]
fn json_output_lists_declared_not_wired() {
    let dir = TempDir::new().unwrap();
    setup_proxy_vs_interface(dir.path());

    run_inline(dir.path())
        .args(["--output", "json"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"declared_not_wired\""))
        .stdout(predicate::str::contains("\"complete_auth_with_tokens\""));
}

#[test]
fn errors_without_profile_or_inline_flags() {
    let dir = TempDir::new().unwrap();
    setup_proxy_vs_interface(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["analyze", "declared-vs-wired"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--profile").or(predicate::str::contains("declared-glob")));
}

#[test]
fn resolves_named_profile_from_config() {
    let dir = TempDir::new().unwrap();
    setup_proxy_vs_interface(dir.path());
    let straymark = dir.path().join(".straymark");
    std::fs::create_dir_all(&straymark).unwrap();
    std::fs::write(
        straymark.join("config.yml"),
        r#"language: en
declared_vs_wired:
  profiles:
    - name: dbus
      declared_glob: "client/**/*.rs"
      declared_pattern: "fn (\\w+)"
      wired_glob: "daemon/**/*.rs"
      wired_pattern: "fn (\\w+)"
"#,
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .args(["analyze", "declared-vs-wired"])
        .arg(dir.path().to_str().unwrap())
        .args(["--profile", "dbus"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("complete_auth_with_tokens"))
        .stdout(predicate::str::contains("dbus"));
}

#[test]
fn bare_analyze_still_runs_complexity() {
    // Backward-compat guard: turning `analyze` into a subcommand group must not
    // break `straymark analyze [path]`.
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("x.rs"), "fn simple() { let a = 1; }\n").unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("analyze")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("StrayMark Analyze"))
        .stdout(predicate::str::contains("Threshold"));
}
