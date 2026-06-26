//! Cheap task classifier (Baton Phase 2, B3).
//!
//! Pure `classify(&UnitSignals) -> Classification` — deterministic rules over the
//! cheap signals from B2, no LLM, no I/O (concept §4.2 corollary; NFR2/NFR3). Maps
//! a unit to a `TaskClass` (Planner / Implementer / Auditor / Operator — the
//! routing targets of §4.2) that B4 turns into a tier recommendation.
//!
//! Conservative bias (R1): when cues conflict, **route up** to the highest-tier
//! class present; when nothing is known, default to the *escalatable middle*
//! (Implementer) rather than claiming a unit is cheap (Operator). A false "this is
//! mechanical" routes real work *down* and costs quality — the one error class we
//! refuse. The `rationale` records the driving signals so a recommendation is
//! auditable and calibration deltas are legible.

use serde::Serialize;

use crate::intent::Confidence;
use crate::signals::{Cue, RiskLevel, UnitSignals};

/// The routing target — what kind of model tier this work wants (§4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskClass {
    Planner,
    Implementer,
    Auditor,
    Operator,
}

impl TaskClass {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskClass::Planner => "planner",
            TaskClass::Implementer => "implementer",
            TaskClass::Auditor => "auditor",
            TaskClass::Operator => "operator",
        }
    }

    /// Route-up rank: higher wins a conflict (Planner is the most capable tier;
    /// Implementer outranks Auditor because it can escalate to frontier).
    fn rank(self) -> u8 {
        match self {
            TaskClass::Planner => 4,
            TaskClass::Implementer => 3,
            TaskClass::Auditor => 2,
            TaskClass::Operator => 1,
        }
    }
}

/// A classification with its confidence and the signals that drove it.
#[derive(Debug, Clone, Serialize)]
pub struct Classification {
    pub class: TaskClass,
    pub confidence: Confidence,
    /// True when the unit's signals pointed at more than one class and we routed
    /// up — a within-unit heterogeneity marker the telemetry aggregates (§10.4).
    pub conflict: bool,
    pub rationale: String,
}

/// The class a single cue points at.
fn cue_class(cue: Cue) -> TaskClass {
    match cue {
        Cue::Architecture => TaskClass::Planner,
        Cue::Audit => TaskClass::Auditor,
        Cue::Implement | Cue::Fix => TaskClass::Implementer,
        Cue::Operate | Cue::Test => TaskClass::Operator,
    }
}

/// Classify a unit from its cheap signals.
pub fn classify(s: &UnitSignals) -> Classification {
    let mut candidates: Vec<(TaskClass, String)> = s
        .cues
        .iter()
        .map(|&c| (cue_class(c), format!("{:?} cue", c)))
        .collect();

    // High risk on an unbounded surface looks like open-ended design (§5).
    if s.risk_level == Some(RiskLevel::High) && s.surface_size == 0 {
        candidates.push((TaskClass::Planner, "high risk, unbounded surface".into()));
    }

    if candidates.is_empty() {
        // No cue, no risk signal. Don't claim it's cheap — default to the
        // escalatable middle (Implementer), which B4 still lifts to frontier on
        // an escalation signal. Low confidence.
        return Classification {
            class: TaskClass::Implementer,
            confidence: Confidence::Low,
            conflict: false,
            rationale: "no cue or risk signal — default Implementer (route up from Operator)".into(),
        };
    }

    // Route up: the highest-rank class present wins any conflict.
    let class = candidates
        .iter()
        .map(|(c, _)| *c)
        .max_by_key(|c| c.rank())
        .unwrap();
    let conflicting = candidates.iter().any(|(c, _)| c.rank() != class.rank());

    let confidence = if conflicting {
        Confidence::Low
    } else if corroborated(class, s) {
        Confidence::High
    } else {
        Confidence::Medium
    };

    let mut reasons: Vec<String> = candidates
        .iter()
        .filter(|(c, _)| *c == class)
        .map(|(_, r)| r.clone())
        .collect();
    if conflicting {
        let dropped: Vec<&str> = candidates
            .iter()
            .filter(|(c, _)| *c != class)
            .map(|(c, _)| c.as_str())
            .collect();
        reasons.push(format!("routed up over {}", dropped.join(", ")));
    }

    Classification {
        class,
        confidence,
        conflict: conflicting,
        rationale: reasons.join("; "),
    }
}

/// A non-cue signal that corroborates the class → High confidence.
fn corroborated(class: TaskClass, s: &UnitSignals) -> bool {
    let effort = s.effort_estimate.as_deref();
    match class {
        TaskClass::Operator => effort == Some("XS"),
        TaskClass::Implementer => matches!(effort, Some("XS" | "S" | "M")),
        TaskClass::Planner => effort == Some("L") || s.risk_level == Some(RiskLevel::High),
        // An audit cue alone is enough to act on, but nothing else corroborates it.
        TaskClass::Auditor => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(cues: &[Cue]) -> UnitSignals {
        UnitSignals {
            cues: cues.to_vec(),
            ..Default::default()
        }
    }

    #[test]
    fn each_cue_maps_to_its_class() {
        assert_eq!(classify(&sig(&[Cue::Architecture])).class, TaskClass::Planner);
        assert_eq!(classify(&sig(&[Cue::Audit])).class, TaskClass::Auditor);
        assert_eq!(classify(&sig(&[Cue::Implement])).class, TaskClass::Implementer);
        assert_eq!(classify(&sig(&[Cue::Fix])).class, TaskClass::Implementer);
        assert_eq!(classify(&sig(&[Cue::Operate])).class, TaskClass::Operator);
        assert_eq!(classify(&sig(&[Cue::Test])).class, TaskClass::Operator);
    }

    #[test]
    fn conflict_routes_up_with_low_confidence() {
        // A title that is both architecture and cleanup → Planner (route up).
        let c = classify(&sig(&[Cue::Architecture, Cue::Operate]));
        assert_eq!(c.class, TaskClass::Planner);
        assert_eq!(c.confidence, Confidence::Low);
        assert!(c.rationale.contains("routed up over"));
    }

    #[test]
    fn no_signal_defaults_to_escalatable_middle() {
        let c = classify(&UnitSignals::default());
        assert_eq!(c.class, TaskClass::Implementer);
        assert_eq!(c.confidence, Confidence::Low);
        assert!(c.rationale.contains("no cue"));
    }

    #[test]
    fn high_risk_unbounded_surface_is_planner() {
        let s = UnitSignals {
            risk_level: Some(RiskLevel::High),
            surface_size: 0,
            ..Default::default()
        };
        assert_eq!(classify(&s).class, TaskClass::Planner);
    }

    #[test]
    fn corroborating_effort_lifts_confidence_to_high() {
        let mut s = sig(&[Cue::Operate]);
        s.effort_estimate = Some("XS".into());
        assert_eq!(classify(&s).confidence, Confidence::High);

        let mut s = sig(&[Cue::Implement]);
        s.effort_estimate = Some("S".into());
        assert_eq!(classify(&s).confidence, Confidence::High);

        // Cue alone, no corroboration → Medium.
        assert_eq!(classify(&sig(&[Cue::Implement])).confidence, Confidence::Medium);
    }
}
