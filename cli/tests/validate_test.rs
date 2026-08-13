use assert_cmd::cargo_bin_cmd;
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure() // REF-001 is an Error since #419 — unresolvable related: blocks
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

/// Regression test for issue #149: TDE documents can move to `status: resolved`
/// when the debt described has been addressed. The document is kept on disk as
/// audit history. `accepted` / `superseded` / `deprecated` do not capture this
/// semantics correctly.
#[test]
fn test_validate_tde_resolved_terminal_state() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    create_doc(
        dir.path(),
        "06-evolution/technical-debt",
        "TDE-2026-05-11-001-architectural-refactor.md",
        "id: TDE-2026-05-11-001\ntitle: Architectural Refactor Debt\nstatus: resolved\ncreated: 2026-05-11\nagent: claude-code-v1.0\nconfidence: high\nreview_required: false\nrisk_level: medium\ntype: architecture\nimpact: high\neffort: medium\ntags:\n  - architecture\nrelated: []\npriority: null\nassigned_to: null",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

/// Regression test for issue #149: status values reported by Sentinel as invented
/// local terminals (`final`, `closed`, `completed`) MUST continue to fail
/// validation. They are not canonical AILOG terminals — the canonical AILOG
/// terminal is `accepted` per TEMPLATE-AILOG.md and DOCUMENTATION-POLICY.md §6.
/// Documented in CHANGELOG fw-4.14.2 / cli-3.13.1 as "adopters using `final` /
/// `closed` / `completed` should migrate to `accepted`".
#[test]
fn test_validate_rejects_non_canonical_ailog_terminals() {
    for status in ["final", "closed", "completed"] {
        let dir = TempDir::new().unwrap();
        setup_straymark(dir.path());

        create_doc(
            dir.path(),
            "07-ai-audit/agent-logs",
            "AILOG-2026-05-13-001-test.md",
            &format!(
                "id: AILOG-2026-05-13-001\ntitle: Test\nstatus: {}\ncreated: 2026-05-13\nagent: claude-code-v1.0\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
                status
            ),
        );

        let mut cmd = cargo_bin_cmd!("straymark");
        cmd.arg("validate")
            .arg(dir.path().to_str().unwrap())
            .assert()
            .failure()
            .stdout(predicate::str::contains("META-003"));
    }
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    let mut cmd = cargo_bin_cmd!("straymark");
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
    // document.rs lives in the straymark-core crate since Loom M0
    let source_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/src/document.rs");

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

    let mut cmd = cargo_bin_cmd!("straymark");
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

    cargo_bin_cmd!("straymark")
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

    cargo_bin_cmd!("straymark")
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
    cargo_bin_cmd!("straymark")
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
    cargo_bin_cmd!("straymark")
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

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg("--staged")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains("No staged documentation"));
}

// --- CHARTER-FILES-EXIST (finding #210) ---------------------------------

/// Write a Charter with the given `## Files to modify` body block. `heading`
/// lets a test exercise the locale variants; `rows` is the raw table body.
fn create_charter(dir: &std::path::Path, heading: &str, rows: &str) {
    let charters = dir.join(".straymark").join("charters");
    std::fs::create_dir_all(&charters).unwrap();
    let content = format!(
        "---\ncharter_id: CHARTER-01\nstatus: declared\neffort_estimate: M\ntrigger: \"test trigger\"\n---\n\n# Charter: Files Exist Test\n\n## {}\n\n| File | Change |\n|---|---|\n{}\n## Tasks\n\n1. Run.\n",
        heading, rows
    );
    std::fs::write(charters.join("01-files-exist.md"), content).unwrap();
}

#[test]
fn test_charter_files_exist_flags_missing_path() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Files to modify", "| `src/does-not-exist.rs` | edit |\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success() // warning-only: must NOT fail the exit code
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST"))
        .stdout(predicate::str::contains("src/does-not-exist.rs"));
}

#[test]
fn test_charter_files_exist_skips_new_tagged_row() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(
        dir.path(),
        "Files to modify",
        "| `src/brand-new.rs` | New, `risk_level: low` |\n",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST").not());
}

#[test]
fn test_charter_files_exist_passes_when_path_present() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/real.rs"), "// real\n").unwrap();
    create_charter(dir.path(), "Files to modify", "| `src/real.rs` | edit |\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST").not());
}

#[test]
fn test_charter_files_exist_skips_wildcards() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(
        dir.path(),
        "Files to modify",
        "| `.straymark/07-ai-audit/agent-logs/AILOG-*.md` | logs |\n",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST").not());
}

#[test]
fn test_charter_files_exist_skips_exemption_markers() {
    // #215 Gap 3: cross-repo / removed / relocated paths in a (historical)
    // Charter table must not be flagged when explicitly marked.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(
        dir.path(),
        "Files to modify",
        "| `dist/.straymark/templates/charter-template.md` (external) | cross-repo |\n\
         | `interfaces/module.go` (removed) | never materialized |\n\
         | `src/old.rs` (relocated: src/new.rs) | renamed |\n",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST").not());
}

#[test]
fn test_charter_files_exist_still_flags_unmarked_missing_path() {
    // A missing path WITHOUT an exemption marker (e.g. a never-substituted
    // placeholder) must still be flagged — the markers don't blanket-silence.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(
        dir.path(),
        "Files to modify",
        "| `src/does-not-exist.rs` | edit |\n\
         | `src/external-ok.rs` (external) | cross-repo |\n",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST"))
        .stdout(predicate::str::contains("src/does-not-exist.rs"))
        .stdout(predicate::str::contains("src/external-ok.rs").not());
}

#[test]
fn test_charter_files_exist_detects_spanish_heading() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter(dir.path(), "Archivos a modificar", "| `src/no-existe.rs` | editar |\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-FILES-EXIST"))
        .stdout(predicate::str::contains("src/no-existe.rs"));
}

// --- CHARTER-WORK-VERB / CHARTER-DESIGN-PROVENANCE (Baton #332 graduation) ---

/// Write a minimal Charter whose frontmatter carries the given extra YAML lines
/// (e.g. `work_verb: implement`). Used to exercise the declared-classification
/// advisory.
fn create_charter_with_frontmatter(dir: &std::path::Path, extra_frontmatter: &str) {
    let charters = dir.join(".straymark").join("charters");
    std::fs::create_dir_all(&charters).unwrap();
    let content = format!(
        "---\ncharter_id: CHARTER-01\nstatus: declared\neffort_estimate: M\ntrigger: \"test trigger\"\n{}---\n\n# Charter: Work Verb Test\n\n## Tasks\n\n1. Run.\n",
        extra_frontmatter
    );
    std::fs::write(charters.join("01-work-verb.md"), content).unwrap();
}

#[test]
fn test_charter_work_verb_invalid_value_warns() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter_with_frontmatter(dir.path(), "work_verb: refactor\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success() // advisory: must NOT fail the exit code
        .stdout(predicate::str::contains("CHARTER-WORK-VERB"));
}

#[test]
fn test_charter_work_verb_valid_value_is_quiet() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter_with_frontmatter(dir.path(), "work_verb: implement\ndesign_provenance: upstream\n");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-WORK-VERB").not())
        .stdout(predicate::str::contains("CHARTER-DESIGN-PROVENANCE").not());
}

#[test]
fn test_charter_work_verb_absent_is_quiet() {
    // Anti-noise: an undeclared field must emit nothing (the legacy corpus is
    // 100% undeclared and must stay quiet).
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter_with_frontmatter(dir.path(), "");

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-WORK-VERB").not());
}

#[test]
fn test_charter_design_provenance_invalid_value_warns() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_charter_with_frontmatter(
        dir.path(),
        "work_verb: implement\ndesign_provenance: inherited\n",
    );

    let mut cmd = cargo_bin_cmd!("straymark");
    cmd.arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-DESIGN-PROVENANCE"));
}

// ── #377: validate covers .telemetry.yaml ────────────────────────────────

fn setup_telemetry_schema(dir: &std::path::Path) {
    let schemas = dir.join(".straymark/schemas");
    std::fs::create_dir_all(&schemas).unwrap();
    std::fs::write(
        schemas.join("charter-telemetry.schema.v0.json"),
        r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["charter_telemetry"],
  "properties": {
    "charter_telemetry": {
      "type": "object",
      "required": ["charter_id", "charter_title", "closed_at", "effort", "outcome"],
      "properties": {
        "charter_id": { "type": "string", "pattern": "^CHARTER-[0-9]{2,}(-[a-z0-9-]+)?$" },
        "charter_title": { "type": "string" },
        "closed_at": { "type": "string" },
        "effort": {
          "type": "object",
          "required": ["estimated_effort"],
          "properties": {
            "estimated_effort": { "type": "string", "pattern": "^(XS|S|M|L)( \\(~.+\\))?$" }
          }
        },
        "outcome": {
          "type": "object",
          "required": ["completed_as_planned", "scope_changes"],
          "properties": {
            "completed_as_planned": { "type": "boolean" },
            "scope_changes": { "type": "string", "enum": ["ninguno", "menor", "mayor"] }
          }
        },
        "trigger": {
          "type": "object",
          "properties": {
            "declared_kind": { "type": "string", "enum": ["event_trigger", "date", "metric_threshold", "infrastructure_milestone"] }
          }
        }
      }
    }
  }
}"#,
    )
    .unwrap();
}

#[test]
fn test_validate_catches_invalid_telemetry() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    setup_telemetry_schema(dir.path());
    let charters = dir.path().join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    std::fs::write(
        charters.join("CHARTER-99.telemetry.yaml"),
        r#"charter_telemetry:
  charter_id: "CHARTER-99"
  charter_title: "Reproducing the validation gap"
  closed_at: "2026-07-28"
  trigger:
    declared_kind: "this_is_not_in_the_enum"
  effort:
    estimated_effort: "L (>= 1 week)"
  outcome:
    completed_as_planned: true
    scope_changes: ninguno
"#,
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .failure()
        .stdout(predicate::str::contains("TELEMETRY-SCHEMA"));
}

#[test]
fn test_validate_passes_valid_telemetry() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    setup_telemetry_schema(dir.path());
    let charters = dir.path().join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    std::fs::write(
        charters.join("CHARTER-01.telemetry.yaml"),
        r#"charter_telemetry:
  charter_id: "CHARTER-01"
  charter_title: "Valid telemetry"
  closed_at: "2026-07-28"
  effort:
    estimated_effort: "M"
  outcome:
    completed_as_planned: true
    scope_changes: ninguno
"#,
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .success()
        .stdout(predicate::str::contains("TELEMETRY-SCHEMA").not());
}

#[test]
fn test_validate_catches_unparseable_telemetry() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    setup_telemetry_schema(dir.path());
    let charters = dir.path().join(".straymark/charters");
    std::fs::create_dir_all(&charters).unwrap();
    std::fs::write(
        charters.join("CHARTER-02.telemetry.yaml"),
        "charter_telemetry:\n  charter_id: [invalid yaml\n",
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--include-charters")
        .assert()
        .failure()
        .stdout(predicate::str::contains("TELEMETRY-PARSE"));
}

const FU_REGISTRY: &str = r#"---
schema_version: v1
last_scan: 2026-08-04
buckets:
  - ready
fully_extracted_ailogs: []
---

## Bucket: ready

### FU-001 — Known follow-up
- **Origin**: AILOG-2026-07-01-001 §Follow-ups
- **Status**: open

### FU-335 — FU-058-022 — rotate the staging credential (R2).
- **Origin**: AILOG-2026-08-01-004 §Follow-ups
- **Status**: open

## Closed in this scan

- **FU-062 — TDE-001 resolution complete** → CLOSED (entry pruned by
  post-merge triage 2026-05-12).
"#;

#[test]
fn fu_id_mentioned_outside_followups_section_warns_when_unregistered() {
    // GH #392: an id coined in prose never reaches the registry; validate
    // must surface it. Registered ids and ids inside the document's own
    // `## Follow-ups` section stay quiet.
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    std::fs::write(
        dir.path().join(".straymark/follow-ups-backlog.md"),
        FU_REGISTRY,
    )
    .unwrap();
    std::fs::write(
        dir
            .path()
            .join(".straymark/07-ai-audit/agent-logs/AILOG-2026-07-29-005-scope.md"),
        r#"---
id: AILOG-2026-07-29-005
title: Scope amendment
status: accepted
created: 2026-07-29
agent: test-agent-v1.0
confidence: high
review_required: false
risk_level: low
---

# AILOG: Scope amendment

Deferred unification is **FU-002**: null behavioural change, but it touches
the one path that has never failed. Cross-reference to FU-001 is legitimate.

## Follow-ups

- Close FU-009 once the registry catches up (mentions here are declarations).
"#,
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID"))
        .stdout(predicate::str::contains("FU-002"))
        // Exactly one untracked-id warning: FU-001 is registered, FU-009
        // lives in the document's own Follow-ups section.
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID").count(1));
}

/// Write an AILOG under `agent-logs/` with the boilerplate frontmatter the
/// other follow-up mention tests share. `slug` doubles as the `AILOG-` id.
fn write_fu_ailog(dir: &std::path::Path, slug: &str, body: &str) {
    let (id, _) = slug.split_at(20); // `AILOG-YYYY-MM-DD-NNN`
    std::fs::write(
        dir.join(".straymark/07-ai-audit/agent-logs")
            .join(format!("{slug}.md")),
        format!(
            "---\nid: {id}\ntitle: Follow-up mention fixture\nstatus: accepted\n\
             created: 2026-08-05\nagent: test-agent-v1.0\nconfidence: high\n\
             review_required: false\nrisk_level: low\n---\n\n# AILOG: fixture\n\n{body}"
        ),
    )
    .unwrap();
}

/// Set up a project whose registry is [`FU_REGISTRY`], plus one AILOG.
fn fu_project(slug: &str, body: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    std::fs::write(
        dir.path().join(".straymark/follow-ups-backlog.md"),
        FU_REGISTRY,
    )
    .unwrap();
    write_fu_ailog(dir.path(), slug, body);
    dir
}

#[test]
fn fu_author_id_alias_in_entry_title_stays_quiet() {
    // GH #392 follow-up (adopter field report): two id spaces coexist — the
    // registry id `FU-335` the CLI assigns, and the author id `FU-058-022`
    // that survives inside the entry title. Citing the author id is the more
    // traceable prose (it names the Charter), so it must not warn.
    let dir = fu_project(
        "AILOG-2026-08-05-001-alias",
        "The credential rotation tracked as FU-058-022 landed this batch.\n",
    );

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID").not());
}

#[test]
fn fu_id_only_in_registry_closure_section_stays_quiet() {
    // An entry closed and pruned by triage leaves its record in a closure
    // section, not as a `### FU-NNN` entry. Referring back to it is a normal
    // cross-reference — the registry still remembers the id.
    let dir = fu_project(
        "AILOG-2026-08-05-002-closed",
        "Verified post-merge that FU-062 was already resolved by CHARTER-15.\n",
    );

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID").not());
}

#[test]
fn fu_id_declared_later_in_own_followups_section_stays_quiet() {
    // Prose cites an id the document itself declares further down. The
    // extractor will see that declaration, so the mention is not the defect
    // the rule hunts — even before `drift --apply` has run.
    let dir = fu_project(
        "AILOG-2026-08-05-003-selfref",
        "Summary: the deferred work is captured below as FU-059-001.\n\n\
         ## Follow-ups\n\n\
         - **FU-059-001** — single-source the retry budget.\n",
    );

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID").not());
}

#[test]
fn fu_untracked_two_segment_id_still_warns() {
    // The case that opened #392, verbatim: a follow-up coined in a
    // "scope declared out" section. Nothing in the registry knows the id and
    // the document never declares it where the extractor looks.
    let dir = fu_project(
        "AILOG-2026-08-05-004-outofscope",
        "## Scope declared out\n\n\
         Unifying this is **FU-057-006**: null behavioural change, but it \
         touches the one RLS path that has never failed.\n",
    );

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID").count(1))
        .stdout(predicate::str::contains("FU-057-006"));
}

#[test]
fn fu_mention_check_stays_quiet_without_registry() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    std::fs::write(
        dir
            .path()
            .join(".straymark/07-ai-audit/agent-logs/AILOG-2026-07-29-006-x.md"),
        r#"---
id: AILOG-2026-07-29-006
title: No registry yet
status: accepted
created: 2026-07-29
agent: test-agent-v1.0
confidence: high
review_required: false
risk_level: low
---

# AILOG: No registry yet

This mentions FU-012 but there is no registry to compare against.
"#,
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("FOLLOWUP-UNTRACKED-ID").not());
}

// ── #419: name resolution for the markdown layer ─────────────────────

#[test]
fn test_validate_commit_msg_phantom_fails() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low",
    );
    let msg = dir.path().join("COMMIT_EDITMSG");
    std::fs::write(&msg, "fix: close finding, see AILOG-1999-01-01-001\n").unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--commit-msg")
        .arg(msg.to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("COMMIT-REF-001"))
        .stdout(predicate::str::contains("AILOG-1999-01-01-001"));
}

#[test]
fn test_validate_commit_msg_resolving_passes() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low",
    );
    let msg = dir.path().join("COMMIT_EDITMSG");
    std::fs::write(&msg, "fix: close finding, see AILOG-2025-01-27-001\n").unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--commit-msg")
        .arg(msg.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("resolves"));
}

#[test]
fn test_validate_commit_msg_no_ids_passes() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    let msg = dir.path().join("COMMIT_EDITMSG");
    std::fs::write(&msg, "chore: bump version\n").unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .arg("--commit-msg")
        .arg(msg.to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn test_validate_ref003_body_phantom_warns() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2025-01-27-001-test.md",
        "id: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low",
    );
    // The create_doc body is "# Document"; rewrite with a phantom citation.
    std::fs::write(
        dir.path()
            .join(".straymark/07-ai-audit/agent-logs/AILOG-2025-01-27-001-test.md"),
        "---\nid: AILOG-2025-01-27-001\ntitle: Test\nstatus: draft\ncreated: 2025-01-27\nagent: test\nconfidence: high\nreview_required: false\nrisk_level: low\n---\n\n# Document\n\nBody cites AILOG-1999-01-01-099, which does not exist.\n",
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success() // REF-003 is warn-first
        .stdout(predicate::str::contains("REF-003"))
        .stdout(predicate::str::contains("AILOG-1999-01-01-099"));
}
