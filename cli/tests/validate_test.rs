use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a minimal StrayMark installation
fn setup_straymark(dir: &std::path::Path) {
    let straymark = dir.join(".straymark");
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
fn test_validate_not_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("not installed"));
}

#[test]
fn test_validate_no_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No documents found"));
}

#[test]
fn test_validate_valid_document() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-implement-auth.md",
        "id: AILOG-2025-01-27-001\ntitle: Implement auth\nstatus: draft\ncreated: 2025-01-27\nagent: claude-code-v1.0\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

#[test]
fn test_validate_missing_frontmatter_fields() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("META-001"));
}

#[test]
fn test_validate_invalid_status() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test\nstatus: invalid_status\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("META-003"));
}

#[test]
fn test_validate_cross_001_high_risk_no_review() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: high",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CROSS-001"));
}

#[test]
fn test_validate_sensitive_info_detected() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let path = dir.path().join(".straymark/07-ai-audit/agent-logs");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("AILOG-2025-01-27-001-secrets.md"),
        "---\nid: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low\n---\n\nThe api_key: sk-12345 was leaked\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("SEC-001"));
}

#[test]
fn test_validate_related_not_found() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low\nrelated:\n  - AIDEC-2025-01-27-001",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success() // Warnings don't cause failure
        .stdout(predicate::str::contains("REF-001"));
}

#[test]
fn test_validate_sec_requires_review() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "08-security",
        "SEC-2025-01-27-001-api-review.md",
        "id: SEC-2025-01-27-001\ntitle: API Review\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: medium\nreview_required: false\nrisk_level: high",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CROSS-003"));
}

#[test]
fn test_validate_fix_review_required() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let doc_path = dir
        .path()
        .join(".straymark/08-security/SEC-2025-01-27-001-fix-test.md");
    std::fs::write(
        &doc_path,
        "---\nid: SEC-2025-01-27-001\ntitle: Fix Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: medium\nreview_required: false\nrisk_level: high\n---\n\n# Test\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg("--fix")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .stdout(predicate::str::contains("Fixed"));

    // Verify the file was modified
    let content = std::fs::read_to_string(&doc_path).unwrap();
    assert!(content.contains("review_required: true"));
}

#[test]
fn test_validate_obs_001_tag_without_content() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-obs.md",
        "id: AILOG-2025-01-27-001\ntitle: Obs Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low\ntags:\n  - observability",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success() // Warnings don't cause failure
        .stdout(predicate::str::contains("OBS-001"));
}

#[test]
fn test_validate_inc_needs_severity() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "05-operations/incidents",
        "INC-2025-01-27-001-outage.md",
        "id: INC-2025-01-27-001\ntitle: Outage\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: true\nrisk_level: high",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("TYPE-001"));
}

// =============================================================================
// F2.QA.02 — Verification of new templates
// =============================================================================

/// F2.QA.02.01 — Create a test document for each new type (SEC, MCARD, SBOM, DPIA)
/// and validate with `straymark validate`
#[test]
fn test_validate_sec_document_valid() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "08-security",
        "SEC-2026-03-24-001-api-assessment.md",
        "id: SEC-2026-03-24-001\ntitle: API Security Assessment\nstatus: draft\ncreated: 2026-03-24\nagent: claude-code-v1.0\nconfidence: medium\nreview_required: true\nrisk_level: high\nthreat_model_methodology: STRIDE\nowasp_asvs_level: 1\ntags:\n  - security\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

#[test]
fn test_validate_mcard_document_valid() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "09-ai-models",
        "MCARD-2026-03-24-001-gpt4-turbo.md",
        "id: MCARD-2026-03-24-001\ntitle: GPT-4 Turbo Card\nstatus: draft\ncreated: 2026-03-24\nagent: claude-code-v1.0\nconfidence: medium\nreview_required: true\nrisk_level: medium\nmodel_name: gpt-4-turbo\nmodel_type: LLM\ntags:\n  - ai-model\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

#[test]
fn test_validate_sbom_document_valid() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit",
        "SBOM-2026-03-24-001-platform-deps.md",
        "id: SBOM-2026-03-24-001\ntitle: Platform AI SBOM\nstatus: accepted\ncreated: 2026-03-24\nagent: claude-code-v1.0\nconfidence: high\nreview_required: false\nrisk_level: low\ntags:\n  - sbom\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

#[test]
fn test_validate_dpia_document_valid() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/ethical-reviews",
        "DPIA-2026-03-24-001-user-profiling.md",
        "id: DPIA-2026-03-24-001\ntitle: User Profiling DPIA\nstatus: draft\ncreated: 2026-03-24\nagent: claude-code-v1.0\nconfidence: low\nreview_required: true\nrisk_level: high\ntags:\n  - privacy\n  - gdpr\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

/// Regression test for issue #130: TDE documents ship with `status: identified`
/// per TEMPLATE-TDE.md and DOCUMENTATION-POLICY.md §6 — the validator must accept
/// it as a valid lifecycle entry state.
#[test]
fn test_validate_tde_document_valid() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "06-evolution/technical-debt",
        "TDE-2026-05-11-001-architectural-refactor.md",
        "id: TDE-2026-05-11-001\ntitle: Architectural Refactor Debt\nstatus: identified\ncreated: 2026-05-11\nagent: claude-code-v1.0\nconfidence: high\nreview_required: false\nrisk_level: medium\ntype: architecture\nimpact: high\neffort: medium\ntags:\n  - architecture\nrelated: []\npriority: null\nassigned_to: null",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

/// F2.QA.02.01 — Also verify that MCARD and DPIA fail without review_required: true
#[test]
fn test_validate_mcard_requires_review() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "09-ai-models",
        "MCARD-2026-03-24-001-no-review.md",
        "id: MCARD-2026-03-24-001\ntitle: Test\nstatus: draft\ncreated: 2026-03-24\nagent: test\nconfidence: medium\nreview_required: false\nrisk_level: medium\ntags: []\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CROSS-003"));
}

#[test]
fn test_validate_dpia_requires_review() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/ethical-reviews",
        "DPIA-2026-03-24-001-no-review.md",
        "id: DPIA-2026-03-24-001\ntitle: Test\nstatus: draft\ncreated: 2026-03-24\nagent: test\nconfidence: low\nreview_required: false\nrisk_level: high\ntags: []\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CROSS-003"));
}

/// F2.QA.02.03 — Verify that new templates and directories exist in dist/
#[test]
fn test_new_templates_exist_in_dist() {
    let dist_templates = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("dist/.straymark/templates");

    // EN templates
    assert!(dist_templates.join("TEMPLATE-SEC.md").exists(), "TEMPLATE-SEC.md (EN) missing");
    assert!(dist_templates.join("TEMPLATE-MCARD.md").exists(), "TEMPLATE-MCARD.md (EN) missing");
    assert!(dist_templates.join("TEMPLATE-SBOM.md").exists(), "TEMPLATE-SBOM.md (EN) missing");
    assert!(dist_templates.join("TEMPLATE-DPIA.md").exists(), "TEMPLATE-DPIA.md (EN) missing");

    // ES templates
    let es = dist_templates.join("i18n/es");
    assert!(es.join("TEMPLATE-SEC.md").exists(), "TEMPLATE-SEC.md (ES) missing");
    assert!(es.join("TEMPLATE-MCARD.md").exists(), "TEMPLATE-MCARD.md (ES) missing");
    assert!(es.join("TEMPLATE-SBOM.md").exists(), "TEMPLATE-SBOM.md (ES) missing");
    assert!(es.join("TEMPLATE-DPIA.md").exists(), "TEMPLATE-DPIA.md (ES) missing");

    // New directories
    let straymark = dist_templates.parent().unwrap();
    assert!(straymark.join("08-security").exists(), "08-security/ directory missing");
    assert!(straymark.join("09-ai-models").exists(), "09-ai-models/ directory missing");
}

/// F2.QA.02.02 — Verify straymark new supports all 12 document types via DocType::ALL
#[test]
fn test_new_supports_all_doc_types() {
    let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/document.rs");

    let content = std::fs::read_to_string(&source_path).expect("Cannot read document.rs");

    for doc_type in &["Ailog", "Aidec", "Adr", "Eth", "Req", "Tes", "Inc", "Tde", "Sec", "Mcard", "Sbom", "Dpia"] {
        assert!(
            content.contains(&format!("DocType::{}", doc_type)),
            "document.rs missing DocType::{}", doc_type
        );
    }
}

#[test]
fn test_validate_staged_no_git_repo() {
    let dir = tempfile::TempDir::new().unwrap();

    // Create minimal .straymark/
    let straymark = dir.path().join(".straymark");
    std::fs::create_dir_all(&straymark).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg("--staged")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicates::str::contains("git"));
}

// ── --check-pending-reviews (Phase 2 / fw-4.6.0) ─────────────────────────

/// Today minus N days as a YYYY-MM-DD string.
fn days_ago(n: i64) -> String {
    use chrono::{Duration, Local};
    (Local::now().date_naive() - Duration::days(n))
        .format("%Y-%m-%d")
        .to_string()
}

#[test]
fn test_check_pending_reviews_flags_old_pending_aidec() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let created = days_ago(30);
    create_doc(
        dir.path(),
        "07-ai-audit/decisions",
        "AIDEC-2026-04-01-001-old.md",
        &format!(
            r#"id: AIDEC-2026-04-01-001
title: Old decision
status: accepted
created: {created}
agent: test
confidence: high
review_required: true
risk_level: medium"#,
            created = created
        ),
    );

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "validate",
            "--check-pending-reviews",
            "--max-pending-days",
            "14",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        // Warn-only: success exit code even with the warning.
        .success()
        .stdout(predicate::str::contains("REVIEW-PENDING"))
        .stdout(predicate::str::contains("AIDEC-2026-04-01-001"));
}

#[test]
fn test_check_pending_reviews_silent_when_outcome_set() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let created = days_ago(30);
    create_doc(
        dir.path(),
        "07-ai-audit/decisions",
        "AIDEC-2026-04-01-002-approved.md",
        &format!(
            r#"id: AIDEC-2026-04-01-002
title: Approved decision
status: accepted
created: {created}
agent: test
confidence: high
review_required: true
reviewed_by: pepe@example.com
reviewed_at: {created}
review_outcome: approved
risk_level: medium"#,
            created = created
        ),
    );

    Command::cargo_bin("straymark")
        .unwrap()
        .args([
            "validate",
            "--check-pending-reviews",
            "--max-pending-days",
            "14",
        ])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("REVIEW-PENDING").not());
}

#[test]
fn test_check_pending_reviews_threshold_respected() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let recent = days_ago(5); // newer than the default 14-day threshold
    create_doc(
        dir.path(),
        "07-ai-audit/decisions",
        "AIDEC-2026-05-01-001-recent.md",
        &format!(
            r#"id: AIDEC-2026-05-01-001
title: Recent decision
status: accepted
created: {created}
agent: test
confidence: high
review_required: true
risk_level: medium"#,
            created = recent
        ),
    );

    // Default max-pending-days = 14, doc is 5 days old → no warning.
    Command::cargo_bin("straymark")
        .unwrap()
        .args(["validate", "--check-pending-reviews"])
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("REVIEW-PENDING").not());
}

#[test]
fn test_check_pending_reviews_skipped_without_flag() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let created = days_ago(30);
    create_doc(
        dir.path(),
        "07-ai-audit/decisions",
        "AIDEC-2026-04-01-003-stale.md",
        &format!(
            r#"id: AIDEC-2026-04-01-003
title: Stale decision
status: accepted
created: {created}
agent: test
confidence: high
review_required: true
risk_level: medium"#,
            created = created
        ),
    );

    // Without --check-pending-reviews, the warning does not appear.
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("REVIEW-PENDING").not());
}

// ── --staged ─────────────────────────────────────────────────────────────

#[test]
fn test_validate_staged_no_staged_docs() {
    let dir = tempfile::TempDir::new().unwrap();

    // Init a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    // Create .straymark/
    let straymark = dir.path().join(".straymark");
    std::fs::create_dir_all(&straymark).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("validate")
        .arg("--staged")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains("No staged documentation"));
}
