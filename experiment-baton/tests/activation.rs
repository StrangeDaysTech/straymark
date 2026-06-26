//! CHARTER-02 tests: the SpecKit extension manifest is well-formed and wires the
//! `before_implement` hook, and the engine scopes to the active feature.

use std::path::PathBuf;

use straymark_baton::coherence::{CoherenceReport, FindingClass};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_root() -> PathBuf {
    crate_dir().join("tests/fixtures/sample-project")
}

#[test]
fn extension_manifest_wires_before_implement_hook() {
    let raw = std::fs::read_to_string(crate_dir().join("extension/straymark/extension.yml")).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&raw).expect("extension.yml must be valid YAML");

    assert_eq!(v["extension"]["id"].as_str(), Some("straymark"));
    assert_eq!(
        v["provides"]["commands"][0]["name"].as_str(),
        Some("speckit.straymark.coherence-check")
    );
    assert_eq!(
        v["hooks"]["before_implement"]["command"].as_str(),
        Some("speckit.straymark.coherence-check"),
        "the before_implement hook must invoke the coherence-check command"
    );
}

#[test]
fn extension_files_are_present() {
    let dir = crate_dir().join("extension/straymark");
    for f in [
        "commands/speckit.straymark.coherence-check.md",
        "scripts/bash/coherence-check.sh",
        "config-template.yml",
        "README.md",
    ] {
        assert!(dir.join(f).is_file(), "missing extension file: {f}");
    }
}

#[test]
fn spec_scoping_keeps_only_feature_contracts() {
    let scoped = CoherenceReport::build_scoped(fixture_root(), Some("005-frontend"));
    assert!(!scoped.findings.is_empty());
    assert!(
        scoped
            .findings
            .iter()
            .all(|f| f.contract.as_deref() == Some("services.health")),
        "scoped run must only contain findings about contracts 005-frontend consumes"
    );
    assert!(
        !scoped
            .findings
            .iter()
            .any(|f| f.class == FindingClass::IntendedNotImplemented),
        "repo-wide C1 must be dropped when scoped to a feature"
    );
}

#[test]
fn unknown_spec_yields_no_findings() {
    let scoped = CoherenceReport::build_scoped(fixture_root(), Some("nonexistent"));
    assert!(scoped.findings.is_empty());
}
