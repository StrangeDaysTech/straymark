use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a minimal StrayMark installation
fn setup_straymark(dir: &std::path::Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("00-governance")).unwrap();
    std::fs::create_dir_all(straymark.join("07-ai-audit/agent-logs")).unwrap();
    std::fs::create_dir_all(straymark.join("07-ai-audit/decisions")).unwrap();
    std::fs::create_dir_all(straymark.join("07-ai-audit/ethical-reviews")).unwrap();
    std::fs::create_dir_all(straymark.join("08-security")).unwrap();
    std::fs::create_dir_all(straymark.join("09-ai-models")).unwrap();
    std::fs::create_dir_all(straymark.join("05-operations/incidents")).unwrap();
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(
        straymark.join("dist-manifest.yml"),
        "version: \"3.0.0\"\ndescription: test\n",
    )
    .unwrap();
}

/// Helper to create a document file with frontmatter
fn create_doc(dir: &std::path::Path, subpath: &str, filename: &str, frontmatter: &str) {
    let path = dir.join(".straymark").join(subpath);
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join(filename),
        format!("---\n{}\n---\n\n# Document\n", frontmatter),
    )
    .unwrap();
}

#[test]
fn test_metrics_not_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("metrics")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("not installed"));
}

#[test]
fn test_metrics_no_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("metrics")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("StrayMark Metrics")
                .and(predicate::str::contains("Total documents")),
        );
}

#[test]
fn test_metrics_with_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    // Use today's date to ensure docs fall in "last-30-days"
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let seq = "001";

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        &format!("AILOG-{}-{}-test.md", today, seq),
        &format!(
            "id: AILOG-{}-{}\ntitle: Test\nstatus: accepted\ncreated: {}\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
            today, seq, today
        ),
    );

    create_doc(
        dir.path(),
        "07-ai-audit/ethical-reviews",
        &format!("ETH-{}-002-review.md", today),
        &format!(
            "id: ETH-{}-002\ntitle: Review\nstatus: draft\ncreated: {}\nagent: claude-code\nconfidence: medium\nreview_required: true\nrisk_level: high\ntags: []\nrelated: []",
            today, today
        ),
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("metrics")
        .arg("--period")
        .arg("last-30-days")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("AILOG")
                .and(predicate::str::contains("ETH"))
                .and(predicate::str::contains("Total documents"))
                .and(predicate::str::contains("Review compliance")),
        );
}

#[test]
fn test_metrics_period_all() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2020-01-01-001-old.md",
        "id: AILOG-2020-01-01-001\ntitle: Old\nstatus: accepted\ncreated: 2020-01-01\nagent: test-agent\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("metrics")
        .arg("--period")
        .arg("all")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("All time")
                .and(predicate::str::contains("AILOG")),
        );
}

#[test]
fn test_metrics_output_json() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    let output = cmd
        .arg("metrics")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output is not valid JSON: {}", stdout);
}

#[test]
fn test_metrics_output_markdown() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("metrics")
        .arg("--output")
        .arg("markdown")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("# StrayMark Metrics Report"));
}

#[test]
fn test_metrics_agent_activity() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        &format!("AILOG-{}-001-a.md", today),
        &format!(
            "id: AILOG-{}-001\ntitle: A\nstatus: accepted\ncreated: {}\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
            today, today
        ),
    );

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        &format!("AILOG-{}-002-b.md", today),
        &format!(
            "id: AILOG-{}-002\ntitle: B\nstatus: accepted\ncreated: {}\nagent: gemini-cli\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
            today, today
        ),
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("metrics")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("claude-code")
                .and(predicate::str::contains("gemini-cli"))
                .and(predicate::str::contains("Agent Activity")),
        );
}
