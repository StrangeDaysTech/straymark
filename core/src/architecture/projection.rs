//! The pure "you are here" status projection (Spec 002 §4) and the model↔layout
//! integrity signals (§3.3).
//!
//! The projection is a **pure function** of `(model + parsed governance state)`
//! with zero I/O — exactly the contract Spec 002 §4 requires so the CLI
//! (`status --where`, A1.4) and the Loom server (A2) compute the same answer.
//! The governance state is delivered as a plain [`GovernanceState`] data struct;
//! the CLI/server build it from the `core` extractors (charters, drift, AILOG
//! `## Modified Files`, TDE docs, declared-vs-wired, on-disk inventory).
//!
//! Glob matching reuses [`crate::drift::glob_match`] — one matcher project-wide,
//! so the components flagged `active`/`in-progress` here line up exactly with
//! `straymark charter drift` (NFR3). Note its semantics: `*` spans `/`, so
//! `dir/**` and `dir/*` both mean "anything under `dir/`".

use crate::architecture::model::ArchModel;
use crate::drift::glob_match;
use std::collections::BTreeMap;

/// Governance-derived file sets, the pure inputs to [`project`] (Spec 002 §4,
/// task T1.2). Every field is a list of repo-relative paths; the projection
/// only matches them against component globs, never touches disk or git.
///
/// The CLI/Loom populate these from the `core` extractors:
/// - `active_charter_files` / `closed_charter_files`:
///   [`crate::charter_files::parse_files_to_modify`] over in-progress / closed
///   Charters (closed also folds in [`crate::ailog::parse_modified_files`]).
/// - `in_progress_files`: declared ∩ git-modified via [`crate::drift`].
/// - `tde_files`: paths/docs of open `TDE` documents.
/// - `wiring_gap_files`: `analyze declared-vs-wired` findings.
/// - `on_disk_files`: the source-file inventory.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GovernanceState {
    /// Declared files of `status: in-progress` Charters.
    pub active_charter_files: Vec<String>,
    /// Within active components, files already touched (declared ∩ git-modified).
    pub in_progress_files: Vec<String>,
    /// Files of `status: closed` Charters and their AILOGs.
    pub closed_charter_files: Vec<String>,
    /// Files/docs related to an open `TDE` (technical-debt) document.
    pub tde_files: Vec<String>,
    /// Files flagged by `analyze declared-vs-wired`.
    pub wiring_gap_files: Vec<String>,
    /// The on-disk source-file inventory.
    pub on_disk_files: Vec<String>,
}

/// The state of a single component, derived per the Spec 002 §4 table. A
/// component may hold several states at once (e.g. `active` + `has-debt`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentState {
    /// "You are here" — an in-progress Charter's declared files match the globs.
    Active,
    /// Within an active component, files already modified (declared ∩ git).
    InProgress,
    /// A closed Charter / its AILOGs touched files matching the globs.
    Implemented,
    /// An open TDE relates to the component's files.
    HasDebt,
    /// `declared-vs-wired` findings fall in the component.
    WiringGap,
    /// Globs match files on disk but no document/charter references them.
    Uncharted,
}

impl ComponentState {
    /// Stable kebab-case name (for JSON / textual output).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::InProgress => "in-progress",
            Self::Implemented => "implemented",
            Self::HasDebt => "has-debt",
            Self::WiringGap => "wiring-gap",
            Self::Uncharted => "uncharted",
        }
    }
}

/// The projected state of one component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentProjection {
    pub component_id: String,
    pub layer: String,
    /// The states this component holds, in [`ComponentState`] order.
    pub states: Vec<ComponentState>,
}

/// Per-layer rollup: component counts by state (Spec 002 §4 last ¶).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerRollup {
    pub layer_id: String,
    pub label: String,
    pub order: u32,
    /// How many components in this layer hold each state (a component with N
    /// states contributes to N entries).
    pub counts: BTreeMap<ComponentState, usize>,
}

/// The whole projection: per-component states + per-layer rollups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projection {
    pub components: Vec<ComponentProjection>,
    pub layers: Vec<LayerRollup>,
}

/// True when any of `component_globs` matches any path in `files`.
fn any_glob_matches(globs: &[String], files: &[String]) -> bool {
    globs
        .iter()
        .any(|g| files.iter().any(|f| glob_match(g, f)))
}

/// Project each component onto its set of states (Spec 002 §4) — a pure
/// function of `(model, state)`. Deterministic: equal inputs yield an equal
/// [`Projection`].
pub fn project(model: &ArchModel, state: &GovernanceState) -> Projection {
    let mut components = Vec::with_capacity(model.components.len());

    for c in &model.components {
        let mut states = Vec::new();
        let matches = |files: &[String]| any_glob_matches(&c.globs, files);

        let is_active = matches(&state.active_charter_files);
        if is_active {
            states.push(ComponentState::Active);
            // in-progress is a refinement of active: declared files already
            // touched. Only meaningful inside an active component.
            if matches(&state.in_progress_files) {
                states.push(ComponentState::InProgress);
            }
        }
        if matches(&state.closed_charter_files) {
            states.push(ComponentState::Implemented);
        }
        if matches(&state.tde_files) {
            states.push(ComponentState::HasDebt);
        }
        if matches(&state.wiring_gap_files) {
            states.push(ComponentState::WiringGap);
        }
        // uncharted: globs match real files but no governance set references
        // them. Mutually exclusive with every documented state above.
        if states.is_empty() && matches(&state.on_disk_files) {
            states.push(ComponentState::Uncharted);
        }

        components.push(ComponentProjection {
            component_id: c.id.clone(),
            layer: c.layer.clone(),
            states,
        });
    }

    let layers = rollup_by_layer(model, &components);
    Projection { components, layers }
}

/// Aggregate component states into per-layer counts, preserving the model's
/// layer order.
fn rollup_by_layer(model: &ArchModel, components: &[ComponentProjection]) -> Vec<LayerRollup> {
    model
        .layers
        .iter()
        .map(|layer| {
            let mut counts: BTreeMap<ComponentState, usize> = BTreeMap::new();
            for cp in components.iter().filter(|c| c.layer == layer.id) {
                for st in &cp.states {
                    *counts.entry(*st).or_insert(0) += 1;
                }
            }
            LayerRollup {
                layer_id: layer.id.clone(),
                label: layer.label.clone(),
                order: layer.order,
                counts,
            }
        })
        .collect()
}

/// A model↔layout integrity signal (Spec 002 §3.3) — analogous to the knowledge
/// graph's orphan/dangling detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegritySignal {
    /// Component in `model.yml` but no cell in `plan.drawio`.
    Undrawn(String),
    /// A `straymark_component_id` in the DrawIO not present in `model.yml`.
    Unmodeled(String),
    /// Globs match no files on disk.
    Empty(String),
}

/// Report the §3.3 integrity signals for a model. `drawio_cell_ids` is the set
/// of `straymark_component_id` values found in `plan.drawio` (a stub input in
/// A1.1; A1.3 populates it by parsing the DrawIO); `on_disk` is the source-file
/// inventory.
pub fn validate_model(
    model: &ArchModel,
    drawio_cell_ids: &[String],
    on_disk: &[String],
) -> Vec<IntegritySignal> {
    let mut signals = Vec::new();
    let drawn: std::collections::HashSet<&str> =
        drawio_cell_ids.iter().map(|s| s.as_str()).collect();
    let modeled: std::collections::HashSet<&str> =
        model.components.iter().map(|c| c.id.as_str()).collect();

    for c in &model.components {
        if !drawn.contains(c.id.as_str()) {
            signals.push(IntegritySignal::Undrawn(c.id.clone()));
        }
        if !any_glob_matches(&c.globs, on_disk) {
            signals.push(IntegritySignal::Empty(c.id.clone()));
        }
    }
    for cell in drawio_cell_ids {
        if !modeled.contains(cell.as_str()) {
            signals.push(IntegritySignal::Unmodeled(cell.clone()));
        }
    }

    signals
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architecture::model::{Component, Layer};

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    fn comp(id: &str, layer: &str, globs: &[&str]) -> Component {
        Component {
            id: id.to_string(),
            label: id.to_string(),
            layer: layer.to_string(),
            globs: s(globs),
            links: Vec::new(),
            docs: Vec::new(),
            external: false,
        }
    }

    /// A two-layer model: `core` (auth, billing) + `frontend` (ui).
    fn model() -> ArchModel {
        ArchModel {
            version: 0,
            layers: vec![
                Layer { id: "core".into(), label: "Core".into(), order: 0 },
                Layer { id: "frontend".into(), label: "Frontend".into(), order: 1 },
            ],
            components: vec![
                comp("auth", "core", &["src/auth/**"]),
                comp("billing", "core", &["src/billing/**"]),
                comp("ui", "frontend", &["web/src/**"]),
            ],
        }
    }

    fn states_of<'a>(p: &'a Projection, id: &str) -> &'a [ComponentState] {
        &p.components.iter().find(|c| c.component_id == id).unwrap().states
    }

    #[test]
    fn active_and_in_progress_within_active() {
        let gs = GovernanceState {
            active_charter_files: s(&["src/auth/login.rs", "src/auth/token.rs"]),
            in_progress_files: s(&["src/auth/login.rs"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        assert_eq!(
            states_of(&p, "auth"),
            &[ComponentState::Active, ComponentState::InProgress]
        );
    }

    #[test]
    fn in_progress_requires_active() {
        // git-modified file inside billing but no active charter there → billing
        // is NOT in-progress (in-progress is a refinement of active).
        let gs = GovernanceState {
            in_progress_files: s(&["src/billing/charge.rs"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        assert!(states_of(&p, "billing").is_empty());
    }

    #[test]
    fn implemented_from_closed_charter() {
        let gs = GovernanceState {
            closed_charter_files: s(&["src/billing/charge.rs"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        assert_eq!(states_of(&p, "billing"), &[ComponentState::Implemented]);
    }

    #[test]
    fn has_debt_and_wiring_gap_stack() {
        let gs = GovernanceState {
            closed_charter_files: s(&["src/auth/login.rs"]),
            tde_files: s(&["src/auth/legacy.rs"]),
            wiring_gap_files: s(&["src/auth/rpc.rs"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        assert_eq!(
            states_of(&p, "auth"),
            &[
                ComponentState::Implemented,
                ComponentState::HasDebt,
                ComponentState::WiringGap
            ]
        );
    }

    #[test]
    fn uncharted_when_only_on_disk() {
        let gs = GovernanceState {
            on_disk_files: s(&["web/src/app.tsx"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        assert_eq!(states_of(&p, "ui"), &[ComponentState::Uncharted]);
    }

    #[test]
    fn uncharted_suppressed_by_any_documented_state() {
        // on-disk match AND a closed-charter match → implemented, not uncharted.
        let gs = GovernanceState {
            closed_charter_files: s(&["web/src/app.tsx"]),
            on_disk_files: s(&["web/src/app.tsx"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        assert_eq!(states_of(&p, "ui"), &[ComponentState::Implemented]);
    }

    #[test]
    fn glob_edge_cases_double_star_star_and_literal() {
        // Under drift::glob_match only `*` creates a wildcard, and it spans `/`,
        // so `dir/**` and `dir/*` both match a deeply-nested file. A bare
        // trailing-slash path (`src/c/`) has NO wildcard → literal equality →
        // it does NOT match files beneath it (model authors must write a glob).
        let m = ArchModel {
            version: 0,
            layers: vec![Layer { id: "core".into(), label: "Core".into(), order: 0 }],
            components: vec![
                comp("a", "core", &["src/a/**"]),
                comp("b", "core", &["src/b/*"]),
                comp("c", "core", &["src/c/"]),
            ],
        };
        let gs = GovernanceState {
            on_disk_files: s(&["src/a/deep/nested.rs", "src/b/x.rs", "src/c/y.rs"]),
            ..Default::default()
        };
        let p = project(&m, &gs);
        assert_eq!(states_of(&p, "a"), &[ComponentState::Uncharted]); // `**` spans dirs
        assert_eq!(states_of(&p, "b"), &[ComponentState::Uncharted]); // `*` spans `/`
        assert!(states_of(&p, "c").is_empty()); // literal `src/c/` matches nothing
    }

    #[test]
    fn layer_rollup_counts_by_state() {
        let gs = GovernanceState {
            active_charter_files: s(&["src/auth/x.rs"]),
            closed_charter_files: s(&["src/billing/y.rs"]),
            ..Default::default()
        };
        let p = project(&model(), &gs);
        let core = p.layers.iter().find(|l| l.layer_id == "core").unwrap();
        assert_eq!(core.counts.get(&ComponentState::Active), Some(&1));
        assert_eq!(core.counts.get(&ComponentState::Implemented), Some(&1));
        // frontend layer has no states.
        let fe = p.layers.iter().find(|l| l.layer_id == "frontend").unwrap();
        assert!(fe.counts.is_empty());
        // layer order preserved.
        assert_eq!(p.layers.iter().map(|l| l.order).collect::<Vec<_>>(), vec![0, 1]);
    }

    #[test]
    fn project_is_deterministic() {
        let gs = GovernanceState {
            active_charter_files: s(&["src/auth/x.rs"]),
            ..Default::default()
        };
        assert_eq!(project(&model(), &gs), project(&model(), &gs));
    }

    #[test]
    fn integrity_signals_undrawn_unmodeled_empty() {
        // auth: drawn + on-disk (clean). billing: undrawn. ui: empty (no disk).
        // "ghost" cell id in drawio: unmodeled.
        let drawio = s(&["auth", "ui", "ghost"]);
        let on_disk = s(&["src/auth/login.rs"]);
        let signals = validate_model(&model(), &drawio, &on_disk);
        assert!(signals.contains(&IntegritySignal::Undrawn("billing".into())));
        assert!(signals.contains(&IntegritySignal::Unmodeled("ghost".into())));
        assert!(signals.contains(&IntegritySignal::Empty("billing".into())));
        assert!(signals.contains(&IntegritySignal::Empty("ui".into())));
        // auth is both drawn and non-empty → no signal for it.
        assert!(!signals.iter().any(|sig| matches!(sig,
            IntegritySignal::Undrawn(id) | IntegritySignal::Empty(id) if id == "auth")));
    }
}
