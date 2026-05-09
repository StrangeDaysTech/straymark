use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Set up a minimal StrayMark installation with a template for the given type
fn setup_straymark_with_template(dir: &std::path::Path, doc_type: &str) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::write(
        straymark.join("config.yml"),
        "language: en\n",
    )
    .unwrap();

    let template_name = format!("TEMPLATE-{}.md", doc_type.to_uppercase());
    let template_content = format!(
        "---\nid: {}-YYYY-MM-DD-NNN\ntitle: \"[Title]\"\nstatus: draft\ncreated: YYYY-MM-DD\nagent: \"[agent-name-v1.0]\"\nconfidence: medium\nreview_required: false\nrisk_level: low\n---\n\n# [Title]\n",
        doc_type.to_uppercase()
    );
    std::fs::write(
        straymark.join("templates").join(&template_name),
        template_content,
    )
    .unwrap();
}

#[test]
fn test_new_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("new")
        .arg("--doc-type")
        .arg("ailog")
        .arg("--title")
        .arg("Test")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn test_new_with_type_and_title_args() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_template(dir.path(), "AILOG");

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("new")
        .arg("--doc-type")
        .arg("ailog")
        .arg("--title")
        .arg("Test Document")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created:"));

    // Verify file was created in the correct directory
    let agent_logs_dir = dir.path().join(".straymark/07-ai-audit/agent-logs");
    assert!(agent_logs_dir.exists(), "agent-logs directory should exist");

    let entries: Vec<_> = std::fs::read_dir(&agent_logs_dir)
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(entries.len(), 1, "should have exactly one file");

    let filename = entries[0].file_name();
    let filename = filename.to_str().unwrap();
    assert!(filename.starts_with("AILOG-"), "filename should start with AILOG-");
    assert!(filename.ends_with("-test-document.md"), "filename should end with slug");
    assert!(filename.contains("-001-"), "filename should contain sequence 001");
}

#[test]
fn test_new_sequence_increments() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_template(dir.path(), "AILOG");

    // Create first document
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("new")
        .arg("--doc-type")
        .arg("ailog")
        .arg("--title")
        .arg("First")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // Create second document
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("new")
        .arg("--doc-type")
        .arg("ailog")
        .arg("--title")
        .arg("Second")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let agent_logs_dir = dir.path().join(".straymark/07-ai-audit/agent-logs");
    let entries: Vec<String> = std::fs::read_dir(&agent_logs_dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_str().unwrap().to_string())
        .collect();

    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|f| f.contains("-001-")), "should have seq 001");
    assert!(entries.iter().any(|f| f.contains("-002-")), "should have seq 002");
}

#[test]
fn test_new_unknown_type() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_template(dir.path(), "AILOG");

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("new")
        .arg("--doc-type")
        .arg("invalid")
        .arg("--title")
        .arg("Test")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown document type"));
}
