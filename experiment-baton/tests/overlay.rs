//! B4 integration tests: the intent overlay (intended vs implemented) over the
//! fixture's architecture model.yml + .specify/memory. Covers all three states.

use std::path::PathBuf;

use straymark_baton::overlay::{IntentState, OverlayReport};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-project")
}

fn state_of<'a>(r: &'a OverlayReport, component: &str) -> Option<&'a IntentState> {
    r.components.iter().find(|c| c.component == component).map(|c| &c.state)
}

#[test]
fn model_is_found() {
    assert!(OverlayReport::build(fixture_root()).model_found);
}

#[test]
fn statuscenter_is_intended_and_implemented() {
    let r = OverlayReport::build(fixture_root());
    assert_eq!(state_of(&r, "statuscenter"), Some(&IntentState::IntendedAndImplemented));
}

#[test]
fn policyengine_is_intended_not_implemented_and_unmodeled() {
    let r = OverlayReport::build(fixture_root());
    let pe = r
        .components
        .iter()
        .find(|c| c.component == "policyengine")
        .expect("PolicyEngine in overlay");
    assert_eq!(pe.state, IntentState::IntendedNotImplemented);
    assert!(!pe.modeled, "PolicyEngine is in memory but not in model.yml");
}

#[test]
fn web_api_is_implemented_not_intended() {
    let r = OverlayReport::build(fixture_root());
    assert_eq!(state_of(&r, "web-api"), Some(&IntentState::ImplementedNotIntended));
}
