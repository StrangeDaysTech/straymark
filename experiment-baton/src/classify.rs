//! Task classifier (Baton Phase 2, B3 — revised for #332).
//!
//! Pure `classify(&UnitSignals) -> Classification`. The class comes from the
//! **declared `work_verb`** (controlled vocabulary), with one refinement: an
//! `implement` unit whose `design_provenance` is `upstream` only instruments
//! prior design → it is mechanical → `operator` (the residual-cognitive-load
//! principle from the Sentinel calibration, #331/#332).
//!
//! A unit with **no declared verb is unclassifiable** (`class = None`): the
//! router sends it up conservatively (frontier) and the telemetry nudges the
//! author to declare the verb. We never guess a class from the title.

use serde::Serialize;

use crate::intent::Confidence;
use crate::signals::{DesignProvenance, UnitSignals, WorkVerb};

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
}

/// A classification. `class = None` means the unit declared no work verb and is
/// unclassifiable — route up + nudge, never guess.
#[derive(Debug, Clone, Serialize)]
pub struct Classification {
    pub class: Option<TaskClass>,
    pub confidence: Confidence,
    pub rationale: String,
}

/// Classify a unit from its declared signals.
pub fn classify(s: &UnitSignals) -> Classification {
    let Some(verb) = s.work_verb else {
        return Classification {
            class: None,
            confidence: Confidence::Low,
            rationale: "no work_verb declared — unclassifiable; route up and declare the verb".into(),
        };
    };

    // `implement` that only instruments upstream design is mechanical → operator.
    if verb == WorkVerb::Implement && s.design_provenance == Some(DesignProvenance::Upstream) {
        return Classification {
            class: Some(TaskClass::Operator),
            confidence: Confidence::High,
            rationale: "work_verb=implement + design_provenance=upstream → operator (instruments prior design)".into(),
        };
    }

    let class = match verb {
        WorkVerb::Design => TaskClass::Planner,
        WorkVerb::Implement => TaskClass::Implementer,
        WorkVerb::Audit => TaskClass::Auditor,
        WorkVerb::Operate => TaskClass::Operator,
    };
    Classification {
        // A declared verb is authoritative — the author knew it for free.
        class: Some(class),
        confidence: Confidence::High,
        rationale: format!("work_verb={}", verb_str(verb)),
    }
}

fn verb_str(v: WorkVerb) -> &'static str {
    match v {
        WorkVerb::Design => "design",
        WorkVerb::Implement => "implement",
        WorkVerb::Audit => "audit",
        WorkVerb::Operate => "operate",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(verb: Option<WorkVerb>, prov: Option<DesignProvenance>) -> UnitSignals {
        UnitSignals {
            work_verb: verb,
            design_provenance: prov,
            ..Default::default()
        }
    }

    #[test]
    fn declared_verb_maps_to_class_at_high_confidence() {
        for (v, c) in [
            (WorkVerb::Design, TaskClass::Planner),
            (WorkVerb::Implement, TaskClass::Implementer),
            (WorkVerb::Audit, TaskClass::Auditor),
            (WorkVerb::Operate, TaskClass::Operator),
        ] {
            let r = classify(&sig(Some(v), None));
            assert_eq!(r.class, Some(c));
            assert_eq!(r.confidence, Confidence::High);
        }
    }

    #[test]
    fn implement_upstream_degrades_to_operator() {
        let r = classify(&sig(Some(WorkVerb::Implement), Some(DesignProvenance::Upstream)));
        assert_eq!(r.class, Some(TaskClass::Operator));
        assert!(r.rationale.contains("upstream"));
    }

    #[test]
    fn implement_new_stays_implementer() {
        let r = classify(&sig(Some(WorkVerb::Implement), Some(DesignProvenance::New)));
        assert_eq!(r.class, Some(TaskClass::Implementer));
    }

    #[test]
    fn undeclared_is_unclassifiable() {
        let r = classify(&sig(None, None));
        assert_eq!(r.class, None);
        assert!(r.rationale.contains("no work_verb"));
    }
}
