//! Tier model + illustrative cost policy (Baton Phase 2, B4).
//!
//! Config-driven (the `architecture:` precedent, #279): a `baton:` block in
//! `.straymark/config.yml` declares the tiers, their **illustrative** cost, the
//! work-size proxy, the class→tier routing, and the §4.2 classification-overhead
//! ceiling term. Absent or unparseable → built-in illustrative defaults with a
//! visible notice (`using_defaults`). Costs are labelled illustrative everywhere:
//! the dry-run measures *relative* saving, not a real bill (real provider pricing
//! is deferred to Phase 3, §10.8).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::classify::TaskClass;

/// A model tier, ordered cheap → expensive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tier {
    Local,
    Economic,
    Frontier,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Local => "local",
            Tier::Economic => "economic",
            Tier::Frontier => "frontier",
        }
    }

    pub fn parse(s: &str) -> Option<Tier> {
        match s.trim().to_lowercase().as_str() {
            "local" => Some(Tier::Local),
            "economic" => Some(Tier::Economic),
            "frontier" => Some(Tier::Frontier),
            _ => None,
        }
    }
}

/// The resolved policy (defaults merged with any `baton:` overrides).
#[derive(Debug, Clone)]
pub struct Policy {
    pub cost_frontier: f64,
    pub cost_economic: f64,
    pub cost_local: f64,
    work_size: BTreeMap<String, u64>,
    default_tokens: u64,
    pub route_planner: Tier,
    pub route_implementer: Tier,
    pub route_auditor: Tier,
    pub route_operator: Tier,
    pub escalate_high_risk: bool,
    pub overhead_per_unit: f64,
    /// True when no `baton:` block was found and built-in defaults are in use.
    pub using_defaults: bool,
}

impl Default for Policy {
    fn default() -> Self {
        // All costs illustrative (USD per Mtok, relative shape only).
        let work_size = BTreeMap::from([
            ("XS".into(), 20_000),
            ("S".into(), 60_000),
            ("M".into(), 200_000),
            ("L".into(), 600_000),
        ]);
        Policy {
            cost_frontier: 15.0,
            cost_economic: 1.0,
            cost_local: 0.0,
            work_size,
            default_tokens: 100_000,
            route_planner: Tier::Frontier,
            route_implementer: Tier::Economic,
            route_auditor: Tier::Economic,
            route_operator: Tier::Local,
            escalate_high_risk: true,
            overhead_per_unit: 0.02,
            using_defaults: true,
        }
    }
}

impl Policy {
    /// Illustrative cost per Mtok for a tier.
    pub fn cost_per_mtok(&self, t: Tier) -> f64 {
        match t {
            Tier::Frontier => self.cost_frontier,
            Tier::Economic => self.cost_economic,
            Tier::Local => self.cost_local,
        }
    }

    /// Illustrative token volume for a unit, from its effort estimate (the
    /// work-size proxy, Q1) or the default when no effort is recorded.
    pub fn tokens_for(&self, effort: Option<&str>) -> u64 {
        effort
            .and_then(|e| self.work_size.get(e).copied())
            .unwrap_or(self.default_tokens)
    }

    /// Base tier for a class (before escalation).
    pub fn route(&self, class: TaskClass) -> Tier {
        match class {
            TaskClass::Planner => self.route_planner,
            TaskClass::Implementer => self.route_implementer,
            TaskClass::Auditor => self.route_auditor,
            TaskClass::Operator => self.route_operator,
        }
    }

    /// Cost of one unit at a tier (illustrative).
    pub fn cost(&self, tokens: u64, tier: Tier) -> f64 {
        tokens as f64 / 1_000_000.0 * self.cost_per_mtok(tier)
    }

    /// Load the policy for a project. `explicit` overrides the default
    /// `<root>/.straymark/config.yml`. Any read/parse failure → defaults.
    pub fn load(root: &Path, explicit: Option<&Path>) -> Policy {
        let path = explicit
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.join(".straymark").join("config.yml"));
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Policy::default();
        };
        let raw: RawRoot = serde_yaml::from_str(&content).unwrap_or_default();
        match raw.baton {
            Some(b) => b.into_policy(),
            None => Policy::default(),
        }
    }
}

// ---- config deserialization (all optional, tolerant) ----------------------

#[derive(Deserialize, Default)]
struct RawRoot {
    baton: Option<RawBaton>,
}

#[derive(Deserialize, Default)]
struct RawBaton {
    cost_per_mtok: Option<BTreeMap<String, f64>>,
    work_size: Option<BTreeMap<String, u64>>,
    routing: Option<BTreeMap<String, String>>,
    escalate_high_risk: Option<bool>,
    overhead_per_unit: Option<f64>,
}

impl RawBaton {
    fn into_policy(self) -> Policy {
        let mut p = Policy {
            using_defaults: false,
            ..Policy::default()
        };
        if let Some(c) = self.cost_per_mtok {
            if let Some(v) = c.get("frontier") {
                p.cost_frontier = *v;
            }
            if let Some(v) = c.get("economic") {
                p.cost_economic = *v;
            }
            if let Some(v) = c.get("local") {
                p.cost_local = *v;
            }
        }
        if let Some(ws) = self.work_size {
            if let Some(d) = ws.get("default") {
                p.default_tokens = *d;
            }
            for k in ["XS", "S", "M", "L"] {
                if let Some(v) = ws.get(k) {
                    p.work_size.insert(k.into(), *v);
                }
            }
        }
        if let Some(r) = self.routing {
            let pick = |key: &str, fallback: Tier| {
                r.get(key).and_then(|s| Tier::parse(s)).unwrap_or(fallback)
            };
            p.route_planner = pick("planner", p.route_planner);
            p.route_implementer = pick("implementer", p.route_implementer);
            p.route_auditor = pick("auditor", p.route_auditor);
            p.route_operator = pick("operator", p.route_operator);
        }
        if let Some(e) = self.escalate_high_risk {
            p.escalate_high_risk = e;
        }
        if let Some(o) = self.overhead_per_unit {
            p.overhead_per_unit = o;
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_illustrative_and_marked() {
        let p = Policy::default();
        assert!(p.using_defaults);
        assert_eq!(p.cost_per_mtok(Tier::Frontier), 15.0);
        assert_eq!(p.route(TaskClass::Operator), Tier::Local);
        assert_eq!(p.tokens_for(Some("L")), 600_000);
        assert_eq!(p.tokens_for(None), 100_000);
        assert_eq!(p.tokens_for(Some("???")), 100_000);
    }

    #[test]
    fn baton_block_overrides_merge_into_defaults() {
        let yaml = "\
language: en
baton:
  cost_per_mtok: { frontier: 30.0 }
  routing: { implementer: frontier }
  overhead_per_unit: 0.5
";
        let raw: RawRoot = serde_yaml::from_str(yaml).unwrap();
        let p = raw.baton.unwrap().into_policy();
        assert!(!p.using_defaults);
        assert_eq!(p.cost_frontier, 30.0);
        assert_eq!(p.cost_economic, 1.0); // untouched default
        assert_eq!(p.route(TaskClass::Implementer), Tier::Frontier);
        assert_eq!(p.route(TaskClass::Operator), Tier::Local); // untouched
        assert_eq!(p.overhead_per_unit, 0.5);
    }

    #[test]
    fn missing_baton_block_yields_defaults() {
        let raw: RawRoot = serde_yaml::from_str("language: en\n").unwrap();
        assert!(raw.baton.is_none());
    }
}
