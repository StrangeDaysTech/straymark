//! Sanity tests for the §12 Audit Checkpoint guidance added to
//! AGENT-RULES.md across the 3 languages. These verify the section is
//! present, has the load-bearing structural elements, and stays in
//! parity across translations.

use std::path::PathBuf;

fn agent_rules_path(lang: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let base = PathBuf::from(manifest_dir)
        .join("..")
        .join("dist")
        .join(".devtrail")
        .join("00-governance");
    match lang {
        "en" => base.join("AGENT-RULES.md"),
        other => base.join("i18n").join(other).join("AGENT-RULES.md"),
    }
}

fn read(lang: &str) -> String {
    let path = agent_rules_path(lang);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("expected file to exist: {}", path.display()))
}

#[test]
fn agent_rules_en_has_audit_checkpoint_section() {
    let body = read("en");
    assert!(
        body.contains("\n## 12. Audit Checkpoint"),
        "EN AGENT-RULES.md must declare §12 Audit Checkpoint"
    );
    // Load-bearing structural elements
    assert!(
        body.contains("When to emit the checkpoint"),
        "must describe trigger conditions"
    );
    assert!(
        body.contains("/devtrail-audit-prompt"),
        "must reference the audit-prompt skill"
    );
    assert!(
        body.contains("/devtrail-audit-review"),
        "must reference the audit-review skill"
    );
    assert!(
        body.contains("permanent v0+v1 design decision"),
        "must surface the permanent no-enforcement commitment"
    );
    assert!(
        body.contains("2× the configured threshold"),
        "must include the arborist heuristic"
    );
    assert!(
        body.contains("graceful-degradation") || body.contains("Graceful-degradation"),
        "must mention graceful degradation when analyze feature is absent"
    );
}

#[test]
fn agent_rules_es_has_audit_checkpoint_section() {
    let body = read("es");
    assert!(
        body.contains("\n## 12. Checkpoint de Auditoría"),
        "ES AGENT-RULES.md must declare §12 Checkpoint de Auditoría"
    );
    assert!(body.contains("/devtrail-audit-prompt"));
    assert!(body.contains("/devtrail-audit-review"));
    assert!(
        body.contains("decisión de diseño v0+v1 permanente"),
        "must surface the permanent no-enforcement commitment in ES"
    );
}

#[test]
fn agent_rules_zh_has_audit_checkpoint_section() {
    let body = read("zh-CN");
    assert!(
        body.contains("\n## 12. 审计检查点"),
        "zh-CN AGENT-RULES.md must declare §12 Audit Checkpoint"
    );
    assert!(body.contains("/devtrail-audit-prompt"));
    assert!(body.contains("/devtrail-audit-review"));
    assert!(
        body.contains("v0+v1 永久设计决策"),
        "must surface the permanent no-enforcement commitment in zh-CN"
    );
}

#[test]
fn audit_checkpoint_section_three_langs_share_load_bearing_anchors() {
    // Anchors that must appear identically across translations because
    // they reference language-agnostic identifiers (skill names, file
    // paths, propuesta sections).
    let anchors = [
        "/devtrail-audit-prompt",
        "/devtrail-audit-review",
        "complexity.threshold",
        "Propuesta/devtrail-audit-skills.md",
    ];
    for lang in ["en", "es", "zh-CN"] {
        let body = read(lang);
        for anchor in anchors {
            assert!(
                body.contains(anchor),
                "anchor {anchor:?} missing in {lang} AGENT-RULES.md"
            );
        }
    }
}
