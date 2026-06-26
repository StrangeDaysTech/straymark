//! #313 — per-type → endpoint keying for generated type files.
//!
//! A `types.gen.ts` holds all API types in one anchorless file; under the old
//! nearest-anchor keying its declarations collapse onto one coarse contract (or
//! are dropped entirely). With call-site binding the field/enum mismatch (C2/C3)
//! fires for the *correct* contract, and a clean sibling contract stays silent.

use std::path::PathBuf;

use straymark_baton::coherence::{CoherenceReport, FindingClass, Severity};

fn report() -> CoherenceReport {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/generated-types-project");
    CoherenceReport::build(root)
}

#[test]
fn binding_isolates_the_drifting_contract_not_a_coarse_blob() {
    let r = report();

    // C2: the drifting consumer fields land on `services.health`, not a `services` blob.
    let c2 = r
        .findings
        .iter()
        .find(|f| f.class == FindingClass::ConsumerFieldWithoutProducer)
        .expect("C2 expected on the health contract");
    assert_eq!(c2.severity, Severity::Blocking);
    assert_eq!(c2.contract.as_deref(), Some("services.health"));
    for orphan in ["status", "cpu"] {
        assert!(c2.message.contains(orphan), "C2 should list orphan {orphan}");
    }

    // C3: the enum mismatch is attributed to the same correct contract.
    let c3 = r
        .findings
        .iter()
        .find(|f| f.class == FindingClass::ContractShapeMismatch)
        .expect("C3 expected on the health contract");
    assert_eq!(c3.contract.as_deref(), Some("services.health"));
    assert!(c3.message.contains("OPERATIONAL"));
    assert!(c3.message.contains("GREEN"));
}

#[test]
fn the_clean_sibling_contract_stays_silent() {
    let r = report();
    // `ServiceRow` (→ `services`) matches its producer exactly: no finding there.
    assert!(
        !r.findings
            .iter()
            .any(|f| f.contract.as_deref() == Some("services")),
        "the matching `services` contract must not produce findings"
    );
}

#[test]
fn exactly_the_health_pair_is_blocking() {
    let r = report();
    assert_eq!(
        r.blocking_count(),
        2,
        "only C2 + C3 on services.health should block"
    );
}
