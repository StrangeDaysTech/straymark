//! The intent overlay — the third plane Loom can consume: *intention*
//! (SpecKit memory) laid over the *emergent* architecture model + code.
//!
//! For every architecture component (`model.yml`) and every intended component
//! (`.specify/memory`) it computes one of three states:
//! - `intended-and-implemented` — designed **and** has implementing code,
//! - `intended-not-implemented` — designed but no implementing code (PolicyEngine),
//! - `implemented-not-intended` — code exists the design never mentioned.
//!
//! File→component matching reuses **`straymark_core::drift::glob_match`** and the
//! source inventory comes from **`straymark_core::architecture::collect_source_files`**
//! — the same matcher/scanner `charter drift` and the Loom projection use, so the
//! overlay can never disagree with them (NFR2; no second matcher).

use std::path::{Path, PathBuf};

use serde::Serialize;
use straymark_core::architecture::{collect_source_files, parse_model, ArchModel, Component};
use straymark_core::drift::glob_match;

use crate::intent::IntentModel;
use crate::speckit::IntendedComponent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentState {
    IntendedAndImplemented,
    IntendedNotImplemented,
    ImplementedNotIntended,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentIntent {
    /// Architecture component id, or the intent slug when not modeled.
    pub component: String,
    pub label: String,
    pub layer: Option<String>,
    pub state: IntentState,
    /// The intended-component slug this maps to, if any.
    pub matched_intent: Option<String>,
    /// Whether the component appears in `model.yml`.
    pub modeled: bool,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct OverlayReport {
    pub model_found: bool,
    pub components: Vec<ComponentIntent>,
}

impl OverlayReport {
    /// Build the overlay for the project at `root` (read-only).
    pub fn build(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        let model = find_model(root);
        let intended = IntentModel::build(root).intended_components;
        let inventory: Vec<String> = collect_source_files(root)
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        let components = compute(model.as_ref(), &intended, &inventory);
        OverlayReport {
            model_found: model.is_some(),
            components,
        }
    }
}

/// Files an architecture component owns — `glob_match` over its globs. The one
/// matcher (NFR2); B4's consistency test pins this to `core`.
pub fn owned_files<'a>(component: &Component, inventory: &'a [String]) -> Vec<&'a str> {
    inventory
        .iter()
        .filter(|f| component.globs.iter().any(|g| glob_match(g, f)))
        .map(|s| s.as_str())
        .collect()
}

/// Pure overlay computation.
pub fn compute(
    model: Option<&ArchModel>,
    intended: &[IntendedComponent],
    inventory: &[String],
) -> Vec<ComponentIntent> {
    let mut out = Vec::new();
    let mut matched_intents: Vec<String> = Vec::new();

    if let Some(m) = model {
        for c in &m.components {
            let implemented = !owned_files(c, inventory).is_empty();
            let matched = intended.iter().find(|i| matches_component(i, c));
            if let Some(i) = matched {
                matched_intents.push(i.id.clone());
            }
            let state = match (matched.is_some(), implemented) {
                (true, true) => IntentState::IntendedAndImplemented,
                (true, false) => IntentState::IntendedNotImplemented,
                (false, true) => IntentState::ImplementedNotIntended,
                // modeled but neither designed nor implemented — not surfaced.
                (false, false) => continue,
            };
            out.push(ComponentIntent {
                component: c.id.clone(),
                label: c.label.clone(),
                layer: Some(c.layer.clone()),
                state,
                matched_intent: matched.map(|i| i.id.clone()),
                modeled: true,
            });
        }
    }

    // Intended components not matched to any modeled component.
    for i in intended {
        if matched_intents.contains(&i.id) {
            continue;
        }
        let implemented = inventory.iter().any(|f| f.to_lowercase().contains(&i.id));
        out.push(ComponentIntent {
            component: i.id.clone(),
            label: i.label.clone(),
            layer: None,
            state: if implemented {
                IntentState::IntendedAndImplemented
            } else {
                IntentState::IntendedNotImplemented
            },
            matched_intent: Some(i.id.clone()),
            modeled: false,
        });
    }

    out.sort_by(|a, b| a.component.cmp(&b.component));
    out
}

/// Conservative intent↔model match: id equality, containment, or the intent
/// slug appearing in one of the component's globs.
fn matches_component(intent: &IntendedComponent, c: &Component) -> bool {
    let cid = c.id.to_lowercase();
    cid == intent.id
        || cid.contains(&intent.id)
        || intent.id.contains(&cid)
        || c.globs.iter().any(|g| g.to_lowercase().contains(&intent.id))
}

/// Locate `model.yml` in the usual places.
fn find_model(root: &Path) -> Option<ArchModel> {
    const CANDIDATES: &[&str] = &[
        ".straymark/architecture/model.yml",
        "architecture/model.yml",
        "experiment-loom/architecture/model.yml",
    ];
    for rel in CANDIDATES {
        let path: PathBuf = root.join(rel);
        if path.is_file() {
            if let Ok(model) = parse_model(&path) {
                return Some(model);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use straymark_core::architecture::parse_model_str;

    fn model() -> ArchModel {
        parse_model_str(
            "version: 0\n\
             layers:\n  - {id: backend, label: Backend, order: 0}\n\
             components:\n  - {id: statuscenter, label: StatusCenter, layer: backend, globs: [\"internal/statuscenter/**\"]}\n",
        )
        .unwrap()
    }

    #[test]
    fn owned_files_uses_core_glob_match() {
        // NFR2: Baton's component ownership must equal a direct core::drift fold.
        let m = model();
        let inv = vec![
            "internal/statuscenter/handler.go".to_string(),
            "web/src/api/types.ts".to_string(),
        ];
        let c = &m.components[0];
        let baton: Vec<&str> = owned_files(c, &inv);
        let direct: Vec<&str> = inv
            .iter()
            .filter(|f| c.globs.iter().any(|g| glob_match(g, f)))
            .map(|s| s.as_str())
            .collect();
        assert_eq!(baton, direct);
        assert_eq!(baton, vec!["internal/statuscenter/handler.go"]);
    }
}
