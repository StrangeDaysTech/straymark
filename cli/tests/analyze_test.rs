use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a Rust source file with a simple function
fn create_rust_file(dir: &std::path::Path, name: &str, code: &str) {
    std::fs::write(dir.join(name), code).unwrap();
}

/// Helper to create a minimal StrayMark installation with complexity config
fn setup_straymark_with_config(dir: &std::path::Path, threshold: u32) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(&straymark).unwrap();
    std::fs::write(
        straymark.join("config.yml"),
        format!("language: en\ncomplexity:\n  threshold: {}\n", threshold),
    )
    .unwrap();
}

#[test]
fn test_analyze_empty_directory() {
    let dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("StrayMark Analyze")
                .and(predicate::str::contains("Files analyzed: 0")),
        );
}

#[test]
fn test_analyze_with_rust_file() {
    let dir = TempDir::new().unwrap();
    create_rust_file(
        dir.path(),
        "lib.rs",
        r#"
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}
"#,
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Files analyzed: 1")
                .and(predicate::str::contains("Total functions: 1")),
        );
}

#[test]
fn test_analyze_threshold_flag() {
    let dir = TempDir::new().unwrap();
    // Function with nested ifs to generate cognitive complexity
    create_rust_file(
        dir.path(),
        "complex.rs",
        r#"
fn nested(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x * 2;
            }
            return x + 1;
        }
        return x;
    }
    0
}
"#,
    );

    // With threshold 1, the function should exceed it
    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg("--threshold")
        .arg("1")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Functions exceeding threshold"));
}

#[test]
fn test_analyze_top_flag() {
    let dir = TempDir::new().unwrap();
    create_rust_file(
        dir.path(),
        "multi.rs",
        r#"
fn a(x: i32) -> i32 {
    if x > 0 { if x > 1 { x } else { 0 } } else { -1 }
}
fn b(x: i32) -> i32 {
    if x > 0 { x } else { 0 }
}
fn c() -> i32 { 42 }
"#,
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg("--top")
        .arg("1")
        .arg("--threshold")
        .arg("0")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // With --top 1 and json output, only 1 function in the array
    let output = cargo_bin_cmd!("straymark")
        .arg("analyze")
        .arg("--top")
        .arg("1")
        .arg(dir.path().to_str().unwrap())
        .arg("--output")
        .arg("json")
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let functions = json["functions"].as_array().unwrap();
    assert_eq!(functions.len(), 1);
}

#[test]
fn test_analyze_output_json() {
    let dir = TempDir::new().unwrap();
    create_rust_file(dir.path(), "simple.rs", "fn add(a: i32, b: i32) -> i32 { a + b }\n");

    let output = cargo_bin_cmd!("straymark")
        .arg("analyze")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");
    assert!(json["summary"]["files_analyzed"].is_number());
    assert!(json["functions"].is_array());
}

#[test]
fn test_analyze_output_markdown() {
    let dir = TempDir::new().unwrap();
    create_rust_file(dir.path(), "lib.rs", "fn foo() -> i32 { 1 }\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg("--output")
        .arg("markdown")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("# StrayMark Analyze Report")
                .and(predicate::str::contains("## Summary")),
        );
}

#[test]
fn test_analyze_no_straymark_required() {
    // Works in a dir without .straymark/ initialized
    let dir = TempDir::new().unwrap();
    create_rust_file(dir.path(), "main.rs", "fn main() {}\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Files analyzed: 1"));
}

#[test]
fn test_analyze_reads_config_threshold() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_config(dir.path(), 1);

    // Function with cognitive complexity > 1
    create_rust_file(
        dir.path(),
        "check.rs",
        r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            return x;
        }
        return 0;
    }
    -1
}
"#,
    );

    // With config threshold=1, this function should exceed it
    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Threshold:").and(predicate::str::contains("> 1")));
}

#[test]
fn test_analyze_skips_excluded_dirs() {
    let dir = TempDir::new().unwrap();

    // Source file in root — should be analyzed
    create_rust_file(dir.path(), "main.rs", "fn root_fn() -> i32 { 1 }\n");

    // Source files in excluded directories — should be skipped
    let node_modules = dir.path().join("node_modules").join("some_pkg");
    std::fs::create_dir_all(&node_modules).unwrap();
    create_rust_file(&node_modules, "lib.rs", "fn hidden() -> i32 { 2 }\n");

    let target = dir.path().join("target").join("debug");
    std::fs::create_dir_all(&target).unwrap();
    create_rust_file(&target, "build.rs", "fn build_fn() -> i32 { 3 }\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let output = cargo_bin_cmd!("straymark")
        .arg("analyze")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(json["summary"]["files_analyzed"].as_u64().unwrap(), 1);
    assert_eq!(json["summary"]["total_functions"].as_u64().unwrap(), 1);
}

#[test]
fn test_analyze_rust_file_below_threshold() {
    let dir = TempDir::new().unwrap();

    // Function with low cognitive complexity (single if = ~1)
    create_rust_file(
        dir.path(),
        "simple.rs",
        r#"
fn low_complexity(x: i32) -> &'static str {
    if x > 0 {
        return "positive";
    }
    "non-positive"
}
"#,
    );

    // Default threshold is 8 — function should NOT exceed it
    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("analyze")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Total functions: 1")
                .and(predicate::str::contains("Above threshold: 0")),
        );
}
