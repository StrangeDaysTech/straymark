//! Economic telemetry (Baton Phase 2, B4 — revised for #332).
//!
//! Aggregates the per-unit dry-run routing into the §4.2 verdict: does routing
//! save more than it costs to classify? Per granularity, so the §10.4 question —
//! *which* granularity is routable — is answered with data. Costs are
//! illustrative; the saving reported is **relative**.
//!
//! With declared-verb classification (#332) the honest metric is
//! `undeclared_fraction`: units with no `work_verb` are routed up to frontier
//! (no saving) and need an authoring nudge, not a guess. The saving therefore
//! comes entirely from declared (high-confidence) units — surfaced by
//! `low_confidence_savings_fraction` collapsing toward zero.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::classify::{classify, TaskClass};
use crate::intent::Confidence;
use crate::route::route;
use crate::signals::signals_for;
use crate::tiers::{Policy, Tier};
use crate::units::{Granularity, RoutableUnit};

/// One unit's full dry-run routing record (recommendation only).
#[derive(Debug, Clone, Serialize)]
pub struct UnitRouting {
    pub id: String,
    pub granularity: Granularity,
    pub title: String,
    /// `None` = undeclared work_verb (unclassifiable → routed up).
    pub class: Option<TaskClass>,
    pub confidence: Confidence,
    pub tier: Tier,
    pub escalated: bool,
    pub tokens: u64,
    pub cost_frontier: f64,
    pub cost_routed: f64,
    pub rationale: String,
}

impl UnitRouting {
    fn saving(&self) -> f64 {
        self.cost_frontier - self.cost_routed
    }
    fn declared(&self) -> bool {
        self.class.is_some()
    }
}

/// Classify + route one unit (dry-run). Pure given the policy.
pub fn route_unit(unit: &RoutableUnit, policy: &Policy) -> UnitRouting {
    let signals = signals_for(unit);
    let c = classify(&signals);
    let d = route(&signals, c.class, policy);
    let tokens = policy.tokens_for(signals.effort_estimate.as_deref());
    UnitRouting {
        id: unit.id.clone(),
        granularity: unit.granularity,
        title: unit.title.clone(),
        class: c.class,
        confidence: c.confidence,
        tier: d.tier,
        escalated: d.escalated,
        tokens,
        cost_frontier: policy.cost(tokens, Tier::Frontier),
        cost_routed: policy.cost(tokens, d.tier),
        rationale: c.rationale,
    }
}

/// Sensitivity of the routable verdict to the (illustrative) overhead term.
#[derive(Debug, Clone, Serialize)]
pub struct Sensitivity {
    pub breakeven_overhead_per_unit: f64,
    pub robust_at_2x_overhead: bool,
}

/// The §4.2 economic verdict for a set of units (one granularity, or all).
#[derive(Debug, Clone, Serialize)]
pub struct EconomicTelemetry {
    /// `None` = all granularities combined.
    pub granularity: Option<Granularity>,
    pub units_total: usize,
    pub tier_counts: BTreeMap<String, usize>,
    pub cost_all_frontier: f64,
    pub cost_routed: f64,
    pub gross_savings: f64,
    pub classification_overhead: f64,
    pub net_savings: f64,
    pub routable: bool,
    /// Fraction of units with no declared `work_verb` (routed up; need a nudge).
    pub undeclared_fraction: f64,
    pub low_confidence_fraction: f64,
    /// Fraction of the gross saving that rests on Low-confidence routing
    /// (collapses toward zero once classification is declared-verb).
    pub low_confidence_savings_fraction: f64,
    pub sensitivity: Sensitivity,
}

/// Compute the telemetry for a slice of routings under a policy.
pub fn telemetry(
    routings: &[UnitRouting],
    policy: &Policy,
    granularity: Option<Granularity>,
) -> EconomicTelemetry {
    let n = routings.len();
    let mut tier_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut cost_all_frontier = 0.0;
    let mut cost_routed = 0.0;
    let mut undeclared = 0usize;
    let mut low_conf = 0usize;
    let mut low_conf_saving = 0.0;
    for r in routings {
        *tier_counts.entry(r.tier.as_str().into()).or_default() += 1;
        cost_all_frontier += r.cost_frontier;
        cost_routed += r.cost_routed;
        if !r.declared() {
            undeclared += 1;
        }
        if r.confidence == Confidence::Low {
            low_conf += 1;
            low_conf_saving += r.saving();
        }
    }
    let gross_savings = cost_all_frontier - cost_routed;
    let classification_overhead = n as f64 * policy.overhead_per_unit;
    let net_savings = gross_savings - classification_overhead;
    let frac = |x: usize| if n == 0 { 0.0 } else { x as f64 / n as f64 };
    EconomicTelemetry {
        granularity,
        units_total: n,
        tier_counts,
        cost_all_frontier,
        cost_routed,
        gross_savings,
        classification_overhead,
        net_savings,
        routable: net_savings > 0.0,
        undeclared_fraction: frac(undeclared),
        low_confidence_fraction: frac(low_conf),
        low_confidence_savings_fraction: if gross_savings > 0.0 {
            low_conf_saving / gross_savings
        } else {
            0.0
        },
        sensitivity: Sensitivity {
            breakeven_overhead_per_unit: if n == 0 { 0.0 } else { gross_savings / n as f64 },
            robust_at_2x_overhead: gross_savings - 2.0 * classification_overhead > 0.0,
        },
    }
}

/// Build the full report: per-unit routings + one telemetry per granularity
/// present, plus a combined (`granularity = None`) block.
pub fn build_report(
    units: &[RoutableUnit],
    policy: &Policy,
) -> (Vec<UnitRouting>, Vec<EconomicTelemetry>) {
    let routings: Vec<UnitRouting> = units.iter().map(|u| route_unit(u, policy)).collect();
    let mut reports = Vec::new();
    for g in Granularity::ALL {
        let slice: Vec<UnitRouting> = routings
            .iter()
            .filter(|r| r.granularity == g)
            .cloned()
            .collect();
        if !slice.is_empty() {
            reports.push(telemetry(&slice, policy, Some(g)));
        }
    }
    reports.push(telemetry(&routings, policy, None));
    (routings, reports)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::SourceRef;

    fn unit(id: &str, verb: Option<&str>, effort: Option<&str>) -> RoutableUnit {
        RoutableUnit {
            id: id.into(),
            granularity: Granularity::Task,
            source: SourceRef { file: "f".into(), symbol: None },
            title: "t".into(),
            effort_estimate: effort.map(str::to_string),
            followup_bucket: None,
            followup_severity: None,
            work_verb: verb.map(str::to_string),
            design_provenance: None,
            scope_globs: Vec::new(),
        }
    }

    #[test]
    fn declared_cheap_verbs_yield_positive_net_savings() {
        let p = Policy::default();
        let units = [
            unit("a", Some("operate"), Some("XS")), // → local (free)
            unit("b", Some("audit"), Some("S")),    // → economic
        ];
        let (_r, reports) = build_report(&units, &p);
        let all = reports.iter().find(|t| t.granularity.is_none()).unwrap();
        assert!(all.gross_savings > 0.0);
        assert!(all.routable);
        assert_eq!(all.undeclared_fraction, 0.0);
    }

    #[test]
    fn undeclared_units_route_to_frontier_and_are_counted() {
        let p = Policy::default();
        let units = [unit("a", None, Some("M"))]; // no verb → frontier, no saving
        let (routings, reports) = build_report(&units, &p);
        assert_eq!(routings[0].tier, Tier::Frontier);
        assert_eq!(routings[0].class, None);
        let all = reports.iter().find(|t| t.granularity.is_none()).unwrap();
        assert_eq!(all.undeclared_fraction, 1.0);
        assert_eq!(all.gross_savings, 0.0);
        assert!(!all.routable, "no saving, positive overhead → not routable");
    }

    #[test]
    fn savings_come_from_declared_units_not_low_confidence() {
        let p = Policy::default();
        let units = [
            unit("a", Some("operate"), Some("XS")), // declared, saves
            unit("b", None, Some("XS")),            // undeclared, frontier, no saving
        ];
        let (_r, reports) = build_report(&units, &p);
        let all = reports.iter().find(|t| t.granularity.is_none()).unwrap();
        // The undeclared unit is Low-confidence but contributes 0 saving.
        assert!(all.low_confidence_fraction > 0.0);
        assert_eq!(
            all.low_confidence_savings_fraction, 0.0,
            "saving rests on declared (high-confidence) routing, not guesses"
        );
    }
}
