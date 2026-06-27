//! Dry-run routing (Baton Phase 2, B4 — revised for #332).
//!
//! Maps a classified unit to a **tier recommendation** — it never executes
//! anything (NFR2). An **undeclared** unit (no `work_verb`) is routed up to
//! frontier by default (conservative — quality over saving, R1); the telemetry
//! flags it for the author to declare a verb. A declared unit routes by the
//! policy's class→tier map, with one escalation: a high-risk Implementer is
//! lifted to frontier.

use serde::Serialize;

use crate::classify::TaskClass;
use crate::signals::{RiskLevel, UnitSignals};
use crate::tiers::{Policy, Tier};

/// A tier recommendation for one unit. A recommendation, not an action.
#[derive(Debug, Clone, Serialize)]
pub struct TierDecision {
    pub tier: Tier,
    pub escalated: bool,
    pub reason: String,
}

/// Recommend a tier for a (possibly undeclared) unit (dry-run).
pub fn route(signals: &UnitSignals, class: Option<TaskClass>, policy: &Policy) -> TierDecision {
    let Some(class) = class else {
        return TierDecision {
            tier: Tier::Frontier,
            escalated: false,
            reason: "undeclared work_verb → frontier (conservative default)".into(),
        };
    };
    let base = policy.route(class);
    if policy.escalate_high_risk
        && class == TaskClass::Implementer
        && signals.risk_level == Some(RiskLevel::High)
        && base < Tier::Frontier
    {
        return TierDecision {
            tier: Tier::Frontier,
            escalated: true,
            reason: format!("{} → frontier (high-risk escalation)", base.as_str()),
        };
    }
    TierDecision {
        tier: base,
        escalated: false,
        reason: format!("routing[{}] = {}", class.as_str(), base.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn high_risk() -> UnitSignals {
        UnitSignals {
            risk_level: Some(RiskLevel::High),
            ..Default::default()
        }
    }

    #[test]
    fn undeclared_routes_to_frontier() {
        let d = route(&UnitSignals::default(), None, &Policy::default());
        assert_eq!(d.tier, Tier::Frontier);
        assert!(d.reason.contains("undeclared"));
    }

    #[test]
    fn implementer_high_risk_escalates_to_frontier() {
        let d = route(&high_risk(), Some(TaskClass::Implementer), &Policy::default());
        assert_eq!(d.tier, Tier::Frontier);
        assert!(d.escalated);
    }

    #[test]
    fn implementer_normal_risk_stays_economic() {
        let d = route(&UnitSignals::default(), Some(TaskClass::Implementer), &Policy::default());
        assert_eq!(d.tier, Tier::Economic);
        assert!(!d.escalated);
    }

    #[test]
    fn operator_routes_local() {
        let d = route(&UnitSignals::default(), Some(TaskClass::Operator), &Policy::default());
        assert_eq!(d.tier, Tier::Local);
    }
}
