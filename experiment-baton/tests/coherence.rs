//! B3 integration tests: the coherence engine catches the #304 drift end-to-end
//! (C1 PolicyEngine, C2 phantom fields, C3 enum mismatch, C4 spec ↛ decision),
//! and is verifiably read-only (NFR1).

use std::path::{Path, PathBuf};

use straymark_baton::coherence::{CoherenceReport, FindingClass, Severity};
use straymark_baton::intent::Confidence;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-project")
}

fn report() -> CoherenceReport {
    CoherenceReport::build(fixture_root())
}

fn has_class(r: &CoherenceReport, class: FindingClass) -> bool {
    r.findings.iter().any(|f| f.class == class)
}

#[test]
fn c1_flags_policyengine_as_intended_not_implemented() {
    let r = report();
    let c1: Vec<_> = r
        .findings
        .iter()
        .filter(|f| f.class == FindingClass::IntendedNotImplemented)
        .collect();
    assert_eq!(c1.len(), 1, "only PolicyEngine should be unimplemented");
    assert!(c1[0].message.contains("PolicyEngine"));
    assert_eq!(c1[0].severity, Severity::Warning);
}

#[test]
fn c2_flags_phantom_consumer_fields() {
    let r = report();
    let c2 = r
        .findings
        .iter()
        .find(|f| f.class == FindingClass::ConsumerFieldWithoutProducer)
        .expect("C2 expected");
    assert_eq!(c2.severity, Severity::Blocking);
    assert_eq!(c2.confidence, Confidence::High);
    for f in ["status", "latency_p95_ms", "cpu", "memory"] {
        assert!(c2.message.contains(f), "C2 should list orphan field {f}");
    }
}

#[test]
fn c3_flags_enum_mismatch() {
    let r = report();
    let c3 = r
        .findings
        .iter()
        .find(|f| f.class == FindingClass::ContractShapeMismatch)
        .expect("C3 expected");
    assert_eq!(c3.severity, Severity::Blocking);
    assert!(c3.message.contains("OPERATIONAL"));
    assert!(c3.message.contains("GREEN"));
}

#[test]
fn c4_flags_frontend_not_referencing_pm002() {
    let r = report();
    assert!(
        r.findings.iter().any(|f| f.class == FindingClass::ConsumerVsChangedDecision
            && f.message.contains("005-frontend")
            && f.message.contains("PM-002")),
        "C4 should flag the frontend spec ignoring PM-002"
    );
}

#[test]
fn two_blocking_findings_set_the_ci_gate() {
    let r = report();
    assert_eq!(r.blocking_count(), 2, "C2 + C3 are the blocking pair");
}

#[test]
fn min_confidence_high_keeps_only_the_blocking_pair() {
    let r = report().filtered(Confidence::High);
    // C1 (medium) and C4 (medium) drop; C2 + C3 (high) remain.
    assert!(!has_class(&r, FindingClass::IntendedNotImplemented));
    assert!(!has_class(&r, FindingClass::ConsumerVsChangedDecision));
    assert!(has_class(&r, FindingClass::ConsumerFieldWithoutProducer));
    assert!(has_class(&r, FindingClass::ContractShapeMismatch));
}

#[test]
fn analysis_is_read_only() {
    // NFR1: building the report must not mutate the target tree.
    let before = snapshot(&fixture_root());
    let _ = report();
    let after = snapshot(&fixture_root());
    assert_eq!(before, after, "coherence must not write to the project");
}

/// Sorted `(relative path, byte length)` of every file under `root`.
fn snapshot(root: &Path) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(rel) = p.strip_prefix(root) {
                let len = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                out.push((rel.to_string_lossy().to_string(), len));
            }
        }
    }
    out.sort();
    out
}
