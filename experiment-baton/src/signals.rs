//! Cheap signal aggregation (Baton Phase 2, B2).
//!
//! Turns a `RoutableUnit` (B1) into the typed `UnitSignals` the classifier (B3)
//! reads. Cheap-first by design (concept §4.2 corollary): the signals here are
//! the **universally available, near-zero-cost** ones — textual cues (every unit
//! has a title), the carry-forward harvest from B1 (effort, follow-up
//! bucket/severity), and cheap derived values (declared-surface size, risk from
//! severity). `signals_for` is a **pure** function over a `RoutableUnit` — no
//! I/O, no model, no network (NFR2/NFR5).
//!
//! Heavier signals (per-function complexity via `analyze`, architecture state via
//! the Loom projection, Phase-1 coherence findings) are deliberately **not** wired
//! here: they cost more and we only add a signal once calibration shows the cheap
//! ones misclassify (the charter's empirical, cost-aware stance). See the AILOG's
//! deferral note.
//!
//! Cue boundary rule: a keyword matches only at a **word start** (the preceding
//! char is non-alphanumeric; the word may continue, so Spanish/English stems like
//! `rediseñ`→`rediseño` and `implement`→`implements` still fire). This rejects the
//! dangerous *mid-word* substring accident (`latest` ⊅ test, `information` ⊅
//! format) that could route real work *down* — the conservative bias (R1). A
//! word-*start* overlap that survives (e.g. `fixture` → Fix) is R1-safe: Fix is a
//! mid-tier cue, so it only ever routes *up*.

use serde::Serialize;

use crate::units::RoutableUnit;

/// A coarse, language-agnostic textual cue about the *kind* of work a unit is.
/// The classifier (B3) maps cue sets to a `TaskClass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cue {
    /// Architecture / design / decomposition — points at Planner.
    Architecture,
    /// Independent contrast / review / verification — points at Auditor.
    Audit,
    /// Implementation / wiring / feature work — points at Implementer.
    Implement,
    /// Bug fix / repair — points at Implementer.
    Fix,
    /// Commits, docs, cleanup, formatting — points at Operator.
    Operate,
    /// Test / fixture / coverage work — points at Operator.
    Test,
}

/// Normalised risk level (derived; the registry's severity vocabulary collapses
/// into this).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// The cheap signals the classifier consumes for one unit.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UnitSignals {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_estimate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_severity: Option<String>,
    /// Count of declared scope paths/globs (a cheap complexity proxy; charters).
    pub surface_size: usize,
    pub cues: Vec<Cue>,
}

/// Build the cheap signals for a unit. Pure: title + carry-forward only.
pub fn signals_for(unit: &RoutableUnit) -> UnitSignals {
    UnitSignals {
        effort_estimate: unit.effort_estimate.clone(),
        risk_level: unit
            .followup_severity
            .as_deref()
            .and_then(risk_from_severity),
        followup_bucket: unit.followup_bucket.clone(),
        followup_severity: unit.followup_severity.clone(),
        surface_size: unit.scope_globs.len(),
        cues: scan_cues(&unit.title),
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

/// Cue keyword tables (lowercased), as word-start **prefixes**. Bilingual
/// (EN + ES) — Sentinel titles mix both. Order of the outer list is the emission
/// order (deterministic).
const CUE_TABLE: &[(Cue, &[&str])] = &[
    (
        Cue::Architecture,
        &["architect", "arquitect", "redesign", "rediseñ", "design", "diseñ", "trade-off", "decompos", "rfc"],
    ),
    (
        Cue::Audit,
        &["audit", "auditor", "review", "revis", "verify", "verifica", "validat", "valida"],
    ),
    (Cue::Implement, &["implement", "implementa", "wire", "feature", "support", "soporta"]),
    (Cue::Fix, &["fix", "arregl", "bug", "repair", "corrig", "hotfix"]),
    (
        Cue::Operate,
        &["commit", "docs", "readme", "cleanup", "limpia", "rename", "renombr", "bump", "gofmt", "chore", "lint"],
    ),
    (Cue::Test, &["test", "prueba", "fixture", "coverage", "cobertura"]),
];

/// Scan a title for cues. Deduplicated, in `CUE_TABLE` order (deterministic).
pub fn scan_cues(title: &str) -> Vec<Cue> {
    let hay = title.to_lowercase();
    CUE_TABLE
        .iter()
        .filter(|(_, kws)| kws.iter().any(|kw| matches_at_word_start(&hay, kw)))
        .map(|(cue, _)| *cue)
        .collect()
}

/// True when `needle` occurs in `hay` at a **word start** — the char before the
/// match is non-alphanumeric (or the string start). The word may continue after,
/// so `implement` matches `implements` and `rediseñ` matches `rediseño`, while a
/// mid-word substring (`test` in `latest`) is rejected. No regex;
/// char-boundary-safe over accented prose.
fn matches_at_word_start(hay: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = hay[from..].find(needle) {
        let start = from + rel;
        let at_word_start = hay[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        if at_word_start {
            return true;
        }
        from = start + 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::SourceRef;
    use crate::units::Granularity;

    fn unit(title: &str) -> RoutableUnit {
        RoutableUnit {
            id: "X".into(),
            granularity: Granularity::Task,
            source: SourceRef { file: "f".into(), symbol: None },
            title: title.into(),
            effort_estimate: None,
            followup_bucket: None,
            followup_severity: None,
            scope_globs: Vec::new(),
        }
    }

    #[test]
    fn cues_classify_kind_bilingually() {
        assert_eq!(scan_cues("Redesign the architecture of the router"), vec![Cue::Architecture]);
        assert_eq!(scan_cues("Rediseño y trade-off del plano"), vec![Cue::Architecture]);
        assert_eq!(scan_cues("Audit the contract boundary"), vec![Cue::Audit]);
        assert_eq!(scan_cues("Implement and wire the handler"), vec![Cue::Implement]);
        assert_eq!(scan_cues("Fix the redelivery bug"), vec![Cue::Fix]);
        assert_eq!(scan_cues("gofmt cleanup + bump deps"), vec![Cue::Operate]);
        assert_eq!(scan_cues("Write the coverage tests"), vec![Cue::Test]);
    }

    #[test]
    fn mid_word_substrings_do_not_route_down() {
        // The dangerous direction: a mid-word substring must NOT add a cue.
        assert!(scan_cues("Ship the latest greatest contest").is_empty(), "`latest`/`contest` ⊅ test");
        assert!(scan_cues("Surface the information panel").is_empty(), "`information` ⊅ format/wire");
        // But a real word (possibly inflected) still fires.
        assert!(scan_cues("Write the unit test").contains(&Cue::Test));
        assert!(scan_cues("Implements the parser").contains(&Cue::Implement));
    }

    #[test]
    fn word_start_overlap_is_r1_safe() {
        // `fixture` starts with `fix`, so it surfaces both cues. That is safe:
        // Fix is a mid-tier cue, so the overlap only ever routes *up*.
        let cues = scan_cues("Add the test fixture");
        assert!(cues.contains(&Cue::Test) && cues.contains(&Cue::Fix));
    }

    #[test]
    fn ambiguous_title_yields_no_cue() {
        // No cue → B3 will route up. Conservative.
        assert!(scan_cues("Handle the thing for the service").is_empty());
    }

    #[test]
    fn carry_forward_and_derived_signals() {
        let mut u = unit("Implement the dashboard");
        u.effort_estimate = Some("M".into());
        u.followup_bucket = Some("ready".into());
        u.followup_severity = Some("high".into());
        u.scope_globs = vec!["a.go".into(), "b.ts".into()];
        let s = signals_for(&u);
        assert_eq!(s.effort_estimate.as_deref(), Some("M"));
        assert_eq!(s.followup_bucket.as_deref(), Some("ready"));
        assert_eq!(s.risk_level, Some(RiskLevel::High));
        assert_eq!(s.surface_size, 2);
        assert_eq!(s.cues, vec![Cue::Implement]);
    }

    #[test]
    fn risk_derives_from_severity_vocabulary() {
        assert_eq!(risk_from_severity("PROD-BLOCKER"), Some(RiskLevel::High));
        assert_eq!(risk_from_severity("medium"), Some(RiskLevel::Medium));
        assert_eq!(risk_from_severity("minor"), Some(RiskLevel::Low));
        assert_eq!(risk_from_severity("whatever"), None);
    }
}
