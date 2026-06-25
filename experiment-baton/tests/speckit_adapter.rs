//! Integration tests for the B1 SpecKit adapter against a sanitized,
//! Sentinel-shaped fixture project (the #304 oracle, intent side).

use std::path::PathBuf;

use straymark_baton::speckit;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-project")
}

#[test]
fn detects_speckit_version() {
    let a = speckit::load(fixture_root());
    assert_eq!(a.speckit_version.as_deref(), Some("0.11.3"));
    assert!(a.version_supported, "0.11.x must be within the tested range");
}

#[test]
fn parses_both_specs_sorted() {
    let a = speckit::load(fixture_root());
    let ids: Vec<&str> = a.specs.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["001-backend", "005-frontend"]);
}

#[test]
fn backend_backlog_yields_pm002_with_ailog_ref() {
    let a = speckit::load(fixture_root());
    let backend = a.specs.iter().find(|s| s.id == "001-backend").unwrap();
    let pm = backend
        .decisions
        .iter()
        .find(|d| d.id == "PM-002")
        .expect("PM-002 must be mined from the post-MVP backlog");
    assert!(pm.references.contains(&"AILOG-2026-04-24-006".to_string()));
    assert!(pm.status.as_deref().unwrap().contains("CERRADO"));
}

#[test]
fn frontend_has_fr010_and_health_consume_hint() {
    let a = speckit::load(fixture_root());
    let front = a.specs.iter().find(|s| s.id == "005-frontend").unwrap();
    assert!(front.requirements.iter().any(|r| r.id == "FR-010"));
    assert!(
        front
            .consumes
            .iter()
            .any(|c| c.endpoint.contains("/api/v1/services") && c.endpoint.contains("health")),
        "frontend should record the health endpoint as a consume hint"
    );
}

#[test]
fn mines_intended_components_and_ignores_index() {
    let a = speckit::load(fixture_root());
    let ids: Vec<&str> = a.intended_components.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, vec!["policyengine", "statuscenter"], "INDEX.md must be ignored");

    let policy = a
        .intended_components
        .iter()
        .find(|c| c.id == "policyengine")
        .unwrap();
    assert_eq!(policy.kind, speckit::MemoryKind::Architecture);

    let status = a
        .intended_components
        .iter()
        .find(|c| c.id == "statuscenter")
        .unwrap();
    // declared by both Arquitectura - … and Requisitos - …
    assert_eq!(status.kind, speckit::MemoryKind::Both);
}

#[test]
fn non_speckit_dir_is_empty() {
    let a = speckit::load(std::env::temp_dir().join("definitely-not-a-speckit-project-xyz"));
    assert!(a.specs.is_empty());
    assert!(a.intended_components.is_empty());
}
