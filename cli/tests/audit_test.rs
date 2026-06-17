use assert_cmd::cargo_bin_cmd;
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
        "version: \"3.2.0\"\ndescription: test\n",
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
fn test_audit_not_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("not installed"));
}

#[test]
fn test_audit_no_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 document(s)"));
}

#[test]
fn test_audit_with_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-20-001-init.md",
        "id: AILOG-2026-03-20-001\ntitle: Init project\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("1 document(s)")
                .and(predicate::str::contains("AILOG"))
                .and(predicate::str::contains("Init project")),
        );
}

#[test]
fn test_audit_date_range_filter() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-01-15-001-old.md",
        "id: AILOG-2026-01-15-001\ntitle: Old entry\nstatus: accepted\ncreated: 2026-01-15\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-15-001-mid.md",
        "id: AILOG-2026-03-15-001\ntitle: March entry\nstatus: accepted\ncreated: 2026-03-15\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-06-01-001-new.md",
        "id: AILOG-2026-06-01-001\ntitle: June entry\nstatus: accepted\ncreated: 2026-06-01\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg("--from")
        .arg("2026-03-01")
        .arg("--to")
        .arg("2026-03-31")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("1 document(s)")
                .and(predicate::str::contains("March entry"))
                .and(predicate::str::contains("Old entry").not())
                .and(predicate::str::contains("June entry").not()),
        );
}

#[test]
fn test_audit_system_filter() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-20-001-auth.md",
        "id: AILOG-2026-03-20-001\ntitle: Auth change\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags:\n  - auth-service\n  - security\nrelated: []",
    );

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-21-001-pay.md",
        "id: AILOG-2026-03-21-001\ntitle: Payment update\nstatus: accepted\ncreated: 2026-03-21\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags:\n  - payment\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg("--system")
        .arg("auth")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("1 document(s)")
                .and(predicate::str::contains("Auth change"))
                .and(predicate::str::contains("Payment update").not()),
        );
}

#[test]
fn test_audit_traceability() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "01-requirements",
        "REQ-2026-03-01-001-login.md",
        "id: REQ-2026-03-01-001\ntitle: Login Requirement\nstatus: accepted\ncreated: 2026-03-01\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: medium\ntags: []\nrelated:\n  - ADR-2026-03-02-001",
    );

    create_doc(
        dir.path(),
        "02-design/decisions",
        "ADR-2026-03-02-001-jwt.md",
        "id: ADR-2026-03-02-001\ntitle: Use JWT\nstatus: accepted\ncreated: 2026-03-02\nagent: claude-code\nconfidence: high\nreview_required: true\nrisk_level: medium\ntags: []\nrelated:\n  - AILOG-2026-03-03-001",
    );

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-03-001-impl.md",
        "id: AILOG-2026-03-03-001\ntitle: Implement JWT\nstatus: accepted\ncreated: 2026-03-03\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Traceability")
                .and(predicate::str::contains("REQ"))
                .and(predicate::str::contains("ADR"))
                .and(predicate::str::contains("AILOG")),
        );
}

#[test]
fn test_audit_output_markdown() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-20-001-test.md",
        "id: AILOG-2026-03-20-001\ntitle: Test\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg("--output")
        .arg("markdown")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("# StrayMark Audit Report")
                .and(predicate::str::contains("| Date |")),
        );
}

#[test]
fn test_audit_output_html() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-20-001-test.md",
        "id: AILOG-2026-03-20-001\ntitle: Test\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("audit")
        .arg("--output")
        .arg("html")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("<html")
                .and(predicate::str::contains("<table>")),
        );
}

#[test]
fn test_audit_output_json() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-20-001-test.md",
        "id: AILOG-2026-03-20-001\ntitle: Test\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    let output = cmd
        .arg("audit")
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
