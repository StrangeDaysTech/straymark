//! Dry-run routing (Baton Phase 2, B4).
//!
//! Maps a classified unit to a **tier recommendation** — it never executes
//! anything (NFR2). Base tier from the policy's class→tier map, with one
//! escalation: an Implementer unit carrying a high-risk signal is lifted to
//! frontier (the conservative bias, R1 — quality over saving).

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

/// Recommend a tier for a classified unit (dry-run).
pub fn route(signals: &UnitSignals, class: TaskClass, policy: &Policy) -> TierDecision {
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
    fn implementer_high_risk_escalates_to_frontier() {
        let p = Policy::default();
        let d = route(&high_risk(), TaskClass::Implementer, &p);
        assert_eq!(d.tier, Tier::Frontier);
        assert!(d.escalated);
    }

    #[test]
    fn implementer_normal_risk_stays_economic() {
        let p = Policy::default();
        let d = route(&UnitSignals::default(), TaskClass::Implementer, &p);
        assert_eq!(d.tier, Tier::Economic);
        assert!(!d.escalated);
    }

    #[test]
    fn operator_is_not_escalated_by_risk() {
        // Escalation only lifts Implementer; Operator stays local even at high risk
        // (a high-risk Operator unit is unusual; the classifier would have ranked
        // it up first if it were really risky design).
        let p = Policy::default();
        let d = route(&high_risk(), TaskClass::Operator, &p);
        assert_eq!(d.tier, Tier::Local);
    }

    #[test]
    fn escalation_can_be_disabled_by_policy() {
        let mut p = Policy::default();
        p.escalate_high_risk = false;
        let d = route(&high_risk(), TaskClass::Implementer, &p);
        assert_eq!(d.tier, Tier::Economic);
        assert!(!d.escalated);
    }
}
