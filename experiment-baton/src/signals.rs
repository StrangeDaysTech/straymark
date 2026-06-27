//! Signal aggregation (Baton Phase 2, B2 — revised for #332).
//!
//! Turns a `RoutableUnit` (B1) into the typed `UnitSignals` the classifier (B3)
//! reads. The authoritative classification signal is the **declared `work_verb`**
//! (+ `design_provenance`), captured at authoring — not inferred from the title.
//!
//! Title-scan was discontinued (#332): inferring the work type from an
//! unconstrained, schema-less, often-bilingual title by substring is unviable
//! (object-vs-verb collisions like a module named `Audit`, incidental tokens,
//! and a per-adopter keyword treadmill — the #321 anti-pattern). A unit with no
//! declared verb is *unclassifiable* — routed up conservatively with a nudge,
//! never guessed.
//!
//! `signals_for` is a **pure** function over a `RoutableUnit` — no I/O, no model,
//! no network (NFR2/NFR5).

use serde::Serialize;

use crate::units::RoutableUnit;

/// The authored work verb — the controlled-vocabulary classification signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkVerb {
    /// Open-ended architecture/pattern/scalability decisions, spec authoring.
    Design,
    /// Service logic, diagnosed fixes, complex/optimised queries, new tooling,
    /// defining bounded foundational contracts.
    Implement,
    /// Independent review / verification / contrast against reality.
    Audit,
    /// Mechanical tests/test-infra/migrations, scaffolding, docs, cleanup, bulk.
    Operate,
}

impl WorkVerb {
    pub fn parse(s: &str) -> Option<WorkVerb> {
        match s.trim().to_lowercase().as_str() {
            "design" => Some(WorkVerb::Design),
            "implement" => Some(WorkVerb::Implement),
            "audit" => Some(WorkVerb::Audit),
            "operate" => Some(WorkVerb::Operate),
            _ => None,
        }
    }
}

/// The residual-cognitive-load dimension: was the hard thinking already spent
/// upstream? An `implement` unit that only instruments prior design is mechanical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesignProvenance {
    New,
    Upstream,
}

impl DesignProvenance {
    pub fn parse(s: &str) -> Option<DesignProvenance> {
        match s.trim().to_lowercase().as_str() {
            "new" => Some(DesignProvenance::New),
            "upstream" => Some(DesignProvenance::Upstream),
            _ => None,
        }
    }
}

/// Normalised risk level (derived from the registry's severity vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// The signals the classifier consumes for one unit.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UnitSignals {
    /// The authoritative classification signal (#332). `None` = undeclared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_verb: Option<WorkVerb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design_provenance: Option<DesignProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_estimate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_severity: Option<String>,
    /// Count of declared scope paths/globs (charters).
    pub surface_size: usize,
}

/// Build the signals for a unit. Pure: declared fields only.
pub fn signals_for(unit: &RoutableUnit) -> UnitSignals {
    UnitSignals {
        work_verb: unit.work_verb.as_deref().and_then(WorkVerb::parse),
        design_provenance: unit.design_provenance.as_deref().and_then(DesignProvenance::parse),
        effort_estimate: unit.effort_estimate.clone(),
        risk_level: unit
            .followup_severity
            .as_deref()
            .and_then(risk_from_severity),
        followup_bucket: unit.followup_bucket.clone(),
        followup_severity: unit.followup_severity.clone(),
        surface_size: unit.scope_globs.len(),
    }
}

/// Collapse the registry's free-form severity vocabulary into a `RiskLevel`.
fn risk_from_severity(sev: &str) -> Option<RiskLevel> {
    match sev.trim().to_lowercase().as_str() {
        "critical" | "blocker" | "prod-blocker" | "high" => Some(RiskLevel::High),
        "medium" | "moderate" => Some(RiskLevel::Medium),
        "low" | "minor" => Some(RiskLevel::Low),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::SourceRef;
    use crate::units::Granularity;

    fn unit() -> RoutableUnit {
        RoutableUnit {
            id: "X".into(),
            granularity: Granularity::Charter,
            source: SourceRef { file: "f".into(), symbol: None },
            title: "whatever".into(),
            effort_estimate: None,
            followup_bucket: None,
            followup_severity: None,
            work_verb: None,
            design_provenance: None,
            scope_globs: Vec::new(),
        }
    }

    #[test]
    fn declared_verb_and_provenance_are_parsed() {
        let mut u = unit();
        u.work_verb = Some("Implement".into());
        u.design_provenance = Some("upstream".into());
        let s = signals_for(&u);
        assert_eq!(s.work_verb, Some(WorkVerb::Implement));
        assert_eq!(s.design_provenance, Some(DesignProvenance::Upstream));
    }

    #[test]
    fn undeclared_verb_stays_none() {
        let s = signals_for(&unit());
        assert_eq!(s.work_verb, None);
        assert_eq!(s.design_provenance, None);
    }

    #[test]
    fn unknown_verb_value_is_rejected() {
        let mut u = unit();
        u.work_verb = Some("refactor".into()); // not in the controlled vocabulary
        assert_eq!(signals_for(&u).work_verb, None);
    }

    #[test]
    fn carry_forward_and_derived_signals() {
        let mut u = unit();
        u.effort_estimate = Some("M".into());
        u.followup_bucket = Some("ready".into());
        u.followup_severity = Some("high".into());
        u.scope_globs = vec!["a.go".into(), "b.ts".into()];
        let s = signals_for(&u);
        assert_eq!(s.effort_estimate.as_deref(), Some("M"));
        assert_eq!(s.followup_bucket.as_deref(), Some("ready"));
        assert_eq!(s.risk_level, Some(RiskLevel::High));
        assert_eq!(s.surface_size, 2);
    }

    #[test]
    fn risk_derives_from_severity_vocabulary() {
        assert_eq!(risk_from_severity("PROD-BLOCKER"), Some(RiskLevel::High));
        assert_eq!(risk_from_severity("medium"), Some(RiskLevel::Medium));
        assert_eq!(risk_from_severity("minor"), Some(RiskLevel::Low));
        assert_eq!(risk_from_severity("whatever"), None);
    }
}
