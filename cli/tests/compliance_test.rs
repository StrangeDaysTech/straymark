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

/// Helper to create AI-GOVERNANCE-POLICY.md
fn create_governance_policy(dir: &std::path::Path) {
    let path = dir.join(".straymark/00-governance");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("AI-GOVERNANCE-POLICY.md"),
        "# AI Governance Policy\n\nThis is the governance policy.\n",
    )
    .unwrap();
}

#[test]
fn test_compliance_not_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("not installed"));
}

#[test]
fn test_compliance_no_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--all")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("EU AI Act")
                .and(predicate::str::contains("ISO/IEC 42001"))
                .and(predicate::str::contains("NIST AI RMF")),
        );
}

#[test]
fn test_compliance_eu_ai_act_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("eu-ai-act")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("EU AI Act")
                .and(predicate::str::contains("EU-001")),
        );
}

#[test]
fn test_compliance_iso_42001_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("iso-42001")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("ISO/IEC 42001")
                .and(predicate::str::contains("ISO-001")),
        );
}

#[test]
fn test_compliance_nist_ai_rmf_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("nist-ai-rmf")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("NIST AI RMF")
                .and(predicate::str::contains("NIST-MAP-001")),
        );
}

#[test]
fn test_compliance_with_governance_policy() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_governance_policy(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("iso-42001")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("✓").and(predicate::str::contains("ISO-001")));
}

#[test]
fn test_compliance_with_documents() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());
    create_governance_policy(dir.path());

    create_doc(
        dir.path(),
        "07-ai-audit/ethical-reviews",
        "ETH-2026-03-20-001-privacy-review.md",
        "id: ETH-2026-03-20-001\ntitle: Privacy Review\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: true\nrisk_level: high\neu_ai_act_risk: high\nnist_genai_risks:\n  - privacy\n  - bias\ntags: []\nrelated: []",
    );

    create_doc(
        dir.path(),
        "07-ai-audit/agent-logs",
        "AILOG-2026-03-20-001-init.md",
        "id: AILOG-2026-03-20-001\ntitle: Init\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    create_doc(
        dir.path(),
        "07-ai-audit/decisions",
        "AIDEC-2026-03-20-001-choice.md",
        "id: AIDEC-2026-03-20-001\ntitle: Choice\nstatus: accepted\ncreated: 2026-03-20\nagent: claude-code\nconfidence: high\nreview_required: false\nrisk_level: low\ntags: []\nrelated: []",
    );

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--all")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("✓")
                .and(predicate::str::contains("Overall compliance")),
        );
}

#[test]
fn test_compliance_output_json() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    let output = cmd
        .arg("compliance")
        .arg("--all")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON (array of reports)
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output is not valid JSON: {}", stdout);
}

#[test]
fn test_compliance_output_markdown() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--all")
        .arg("--output")
        .arg("markdown")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("# StrayMark Compliance Report")
                .and(predicate::str::contains("| Check |")),
        );
}

// ---------------------------------------------------------------------------
// China regulatory frameworks (regional_scope: china)
// ---------------------------------------------------------------------------

/// Setup helper that writes a regional_scope including china to config.yml.
fn setup_straymark_with_china(dir: &std::path::Path) {
    setup_straymark(dir);
    let config_path = dir.join(".straymark/config.yml");
    std::fs::write(
        &config_path,
        "language: en\nregional_scope:\n  - global\n  - china\n",
    )
    .unwrap();
}

#[test]
fn test_compliance_china_tc260_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("china-tc260")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("China TC260")
                .and(predicate::str::contains("TC260-001")),
        );
}

#[test]
fn test_compliance_china_pipl_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("china-pipl")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("China PIPL")
                .and(predicate::str::contains("PIPL-001")),
        );
}

#[test]
fn test_compliance_china_gb45438_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("china-gb45438")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("China GB 45438")
                .and(predicate::str::contains("GB45438-001")),
        );
}

#[test]
fn test_compliance_china_cac_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("china-cac")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("China CAC")
                .and(predicate::str::contains("CAC-001")),
        );
}

#[test]
fn test_compliance_china_gb45652_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("china-gb45652")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("China GB/T 45652")
                .and(predicate::str::contains("GB45652-001")),
        );
}

#[test]
fn test_compliance_china_csl_only() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("compliance")
        .arg("--standard")
        .arg("china-csl")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("China CSL 2026")
                .and(predicate::str::contains("CSL-001")),
        );
}

#[test]
fn test_compliance_region_china_runs_six_checkers() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    let output = cmd
        .arg("compliance")
        .arg("--region")
        .arg("china")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let arr = parsed.as_array().expect("array of reports");
    assert_eq!(arr.len(), 6, "expected 6 china reports, got {}", arr.len());
}

#[test]
fn test_compliance_default_excludes_china_when_not_in_scope() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path()); // default scope = [global, eu]

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    let output = cmd
        .arg("compliance")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("China TC260") && !stdout.contains("ChinaPipl"),
        "China standards should not appear without china in regional_scope: {}",
        stdout
    );
}

#[test]
fn test_compliance_all_flag_includes_china() {
    let dir = TempDir::new().unwrap();
    setup_straymark(dir.path()); // default scope (no china)

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    let output = cmd
        .arg("compliance")
        .arg("--all")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(parsed.as_array().unwrap().len(), 9);
}

#[test]
fn test_compliance_china_default_when_in_scope() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_china(dir.path()); // scope = [global, china]

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    let output = cmd
        .arg("compliance")
        .arg("--output")
        .arg("json")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let labels: Vec<String> = parsed
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["standard_label"].as_str().unwrap().to_string())
        .collect();
    // global + china = NIST + ISO + 6 china = 8. EU not in scope.
    assert!(labels.iter().any(|l| l.contains("China TC260")));
    assert!(labels.iter().any(|l| l.contains("NIST")));
    assert!(!labels.iter().any(|l| l.contains("EU AI Act")));
}
