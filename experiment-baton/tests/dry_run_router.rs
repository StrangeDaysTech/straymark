//! B5 acceptance — the dry-run router runs end-to-end over a governance corpus,
//! produces per-granularity economic telemetry, and is verifiably read-only +
//! recommend-only (NFR1/NFR2).

use std::path::{Path, PathBuf};

use straymark_baton::classify::TaskClass;
use straymark_baton::tiers::{Policy, Tier};
use straymark_baton::units::{inventory, Granularity};
use straymark_baton::telemetry::build_report;

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/governance-corpus")
}

#[test]
fn report_covers_every_granularity_plus_a_combined_block() {
    let units = inventory(&corpus(), None);
    let (routings, reports) = build_report(&units, &Policy::default());

    assert_eq!(routings.len(), units.len());
    // One combined block (granularity = None).
    let combined: Vec<_> = reports.iter().filter(|t| t.granularity.is_none()).collect();
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].units_total, units.len());

    // Every granularity present in the corpus has its own block.
    for g in Granularity::ALL {
        let present = !inventory(&corpus(), Some(g)).is_empty();
        let reported = reports.iter().any(|t| t.granularity == Some(g));
        assert_eq!(present, reported, "granularity {} coverage mismatch", g.as_str());
    }
}

#[test]
fn declared_verbs_route_by_class_and_undeclared_go_to_frontier() {
    let units = inventory(&corpus(), None);
    let (routings, reports) = build_report(&units, &Policy::default());
    let by_id = |id: &str| routings.iter().find(|r| r.id == id).unwrap().clone();

    // Declared charter (implement/new) → implementer → economic.
    let charter = by_id("CHARTER-01-example");
    assert_eq!(charter.class, Some(TaskClass::Implementer));
    assert_eq!(charter.tier, Tier::Economic);

    // implement + design_provenance=upstream → operator (mechanical) → local.
    let fu = by_id("FU-201");
    assert_eq!(fu.class, Some(TaskClass::Operator));
    assert_eq!(fu.tier, Tier::Local);

    // Undeclared task → unclassifiable → frontier (conservative), counted.
    let task = routings
        .iter()
        .find(|r| r.granularity == Granularity::Task)
        .unwrap();
    assert_eq!(task.class, None);
    assert_eq!(task.tier, Tier::Frontier);

    let all = reports.iter().find(|t| t.granularity.is_none()).unwrap();
    assert!(all.undeclared_fraction > 0.0 && all.undeclared_fraction < 1.0);
}

#[test]
fn telemetry_is_internally_consistent() {
    let units = inventory(&corpus(), None);
    let (_r, reports) = build_report(&units, &Policy::default());
    for t in &reports {
        // net = gross - overhead; routable iff net > 0.
        let net = t.gross_savings - t.classification_overhead;
        assert!((net - t.net_savings).abs() < 1e-9);
        assert_eq!(t.routable, t.net_savings > 0.0);
        // tier counts sum to the unit total.
        let counted: usize = t.tier_counts.values().sum();
        assert_eq!(counted, t.units_total);
    }
}

#[test]
fn punishing_overhead_reports_not_routable_rather_than_forcing_it() {
    let units = inventory(&corpus(), None);
    let mut policy = Policy::default();
    policy.overhead_per_unit = 1_000_000.0; // dwarfs any illustrative saving
    let (_r, reports) = build_report(&units, &policy);
    let combined = reports.iter().find(|t| t.granularity.is_none()).unwrap();
    assert!(!combined.routable);
    assert!(!combined.sensitivity.robust_at_2x_overhead);
}

#[test]
fn analysis_is_read_only() {
    let before = snapshot(&corpus());
    let units = inventory(&corpus(), None);
    let _ = build_report(&units, &Policy::default());
    let after = snapshot(&corpus());
    assert_eq!(before, after, "the dry-run router must not write to the project");
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
