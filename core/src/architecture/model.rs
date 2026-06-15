//! The semantic architecture model — typed view of `architecture/model.yml`.
//!
//! Spec 002 §3.1: one **model**, expressed as two linked artifacts (the BIM
//! "model vs. drawing" split). This module owns the *semantic* half — layers,
//! components, globs, links — the source of truth for *meaning*. The *layout*
//! half (`plan.drawio`) is owned by the CLI/Loom render path (A1.3 / A2).
//!
//! The model is the join key to two things: governance state (via component
//! globs → [`crate::architecture::projection`]) and the DrawIO cells (via
//! `component.id` ↔ `straymark_component_id`, checked by `validate_model`).

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// A parsed `architecture/model.yml` (Spec 002 §3.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchModel {
    /// Schema version of the model document. `0` is the experimental format.
    pub version: u32,
    /// The "floors": ordered groupings components are placed in.
    pub layers: Vec<Layer>,
    /// Named groups of file globs, each placed in a layer.
    pub components: Vec<Component>,
}

/// A layer — the "floor" a component sits on. Seeded from `.straymark/` stages
/// 00–09 by `architecture generate` (A1.2), but freely user-editable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Stable id; the join key from `component.layer`.
    pub id: String,
    /// Human-readable label shown in the rollup / plan.
    pub label: String,
    /// Render order (lowest first). Ties broken by declaration order.
    pub order: u32,
}

/// A component — a named group of file globs placed in a layer, with optional
/// architectural links to other components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    /// Stable id — the join key to the DrawIO cell and to status projection.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// The `Layer::id` this component belongs to.
    pub layer: String,
    /// File globs this component owns; the join to governance state (§4).
    /// Matched with [`crate::drift::glob_match`] (one matcher project-wide).
    pub globs: Vec<String>,
    /// Architectural edges to other component ids.
    #[serde(default)]
    pub links: Vec<String>,
    /// Optional explicit doc ids; normally docs are inferred via globs.
    #[serde(default)]
    pub docs: Vec<String>,
    /// True for components outside the codebase (third-party / external systems).
    #[serde(default)]
    pub external: bool,
}

/// A structural problem found while validating the *shape* of a model (distinct
/// from the model-vs-layout integrity signals in
/// [`crate::architecture::projection::IntegritySignal`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelIssue {
    /// A `component.layer` does not name any declared `Layer::id` (hard error).
    UnknownLayer { component: String, layer: String },
    /// Two layers or two components share an id (hard error).
    DuplicateId { id: String },
    /// A component declares no globs — it can never match files (warning).
    EmptyGlobs { component: String },
}

impl ModelIssue {
    /// True for issues severe enough to reject the model (`parse_model` errors).
    /// Empty-globs is a warning a caller may surface but the model still loads.
    pub fn is_error(&self) -> bool {
        !matches!(self, ModelIssue::EmptyGlobs { .. })
    }
}

/// Load and validate `architecture/model.yml` from disk. Errors on unreadable
/// file, malformed YAML, or any hard structural issue (unknown layer ref,
/// duplicate id). Empty-globs warnings do not block the load; use
/// [`validate_structure`] directly to inspect them.
pub fn parse_model(path: &Path) -> Result<ArchModel> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading architecture model {}", path.display()))?;
    parse_model_str(&content)
}

/// Parse + validate a model from an in-memory string (the I/O-free core of
/// [`parse_model`]; used directly by tests). Mirrors the path-vs-str split of
/// [`crate::charter::parse_charter_str`].
pub fn parse_model_str(content: &str) -> Result<ArchModel> {
    let model: ArchModel =
        serde_yaml::from_str(content).context("parsing architecture model YAML")?;
    if let Some(issue) = validate_structure(&model).iter().find(|i| i.is_error()) {
        return Err(anyhow!(model_issue_message(issue)));
    }
    Ok(model)
}

/// Validate a model's internal shape, returning every issue (errors + the
/// empty-globs warnings). `parse_model_str` rejects the model on the first
/// error; callers wanting the warnings call this directly.
pub fn validate_structure(model: &ArchModel) -> Vec<ModelIssue> {
    let mut issues = Vec::new();

    // Duplicate ids — across layers and across components (ids are the join
    // keys; collisions make the joins ambiguous).
    let mut seen: HashSet<&str> = HashSet::new();
    for id in model
        .layers
        .iter()
        .map(|l| l.id.as_str())
        .chain(model.components.iter().map(|c| c.id.as_str()))
    {
        if !seen.insert(id) {
            issues.push(ModelIssue::DuplicateId { id: id.to_string() });
        }
    }

    let layer_ids: HashSet<&str> = model.layers.iter().map(|l| l.id.as_str()).collect();
    for c in &model.components {
        if !layer_ids.contains(c.layer.as_str()) {
            issues.push(ModelIssue::UnknownLayer {
                component: c.id.clone(),
                layer: c.layer.clone(),
            });
        }
        if c.globs.is_empty() {
            issues.push(ModelIssue::EmptyGlobs {
                component: c.id.clone(),
            });
        }
    }

    issues
}

fn model_issue_message(issue: &ModelIssue) -> String {
    match issue {
        ModelIssue::UnknownLayer { component, layer } => format!(
            "component `{component}` references unknown layer `{layer}`"
        ),
        ModelIssue::DuplicateId { id } => format!("duplicate id `{id}` in model"),
        ModelIssue::EmptyGlobs { component } => {
            format!("component `{component}` declares no globs")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical example from Spec 002 §3.1.
    const CANONICAL: &str = r#"
version: 0
layers:
  - { id: frontend, label: Frontend, order: 0 }
  - { id: core, label: Core Layer, order: 1 }
components:
  - id: identity-module
    label: Identity Module
    layer: core
    globs: ["internal/identity/**", "cli/src/identity/**"]
    links: [policy-engine]
    docs: []
    external: false
"#;

    #[test]
    fn parses_canonical_model() {
        let m = parse_model_str(CANONICAL).unwrap();
        assert_eq!(m.version, 0);
        assert_eq!(m.layers.len(), 2);
        assert_eq!(m.components.len(), 1);
        let c = &m.components[0];
        assert_eq!(c.id, "identity-module");
        assert_eq!(c.layer, "core");
        assert_eq!(c.globs, vec!["internal/identity/**", "cli/src/identity/**"]);
        assert_eq!(c.links, vec!["policy-engine"]);
        assert!(!c.external);
    }

    #[test]
    fn optional_fields_default() {
        // links/docs/external omitted — serde defaults apply.
        let yaml = r#"
version: 0
layers:
  - { id: core, label: Core, order: 0 }
components:
  - { id: a, label: A, layer: core, globs: ["src/**"] }
"#;
        let m = parse_model_str(yaml).unwrap();
        let c = &m.components[0];
        assert!(c.links.is_empty());
        assert!(c.docs.is_empty());
        assert!(!c.external);
    }

    #[test]
    fn rejects_unknown_layer_reference() {
        let yaml = r#"
version: 0
layers:
  - { id: core, label: Core, order: 0 }
components:
  - { id: a, label: A, layer: nonexistent, globs: ["src/**"] }
"#;
        let err = parse_model_str(yaml).unwrap_err().to_string();
        assert!(err.contains("unknown layer"), "got: {err}");
        assert!(err.contains("nonexistent"), "got: {err}");
    }

    #[test]
    fn rejects_duplicate_ids() {
        let yaml = r#"
version: 0
layers:
  - { id: core, label: Core, order: 0 }
components:
  - { id: core, label: A, layer: core, globs: ["src/**"] }
"#;
        // Component id collides with a layer id.
        let err = parse_model_str(yaml).unwrap_err().to_string();
        assert!(err.contains("duplicate id"), "got: {err}");
        assert!(err.contains("core"), "got: {err}");
    }

    #[test]
    fn empty_globs_is_warning_not_error() {
        let yaml = r#"
version: 0
layers:
  - { id: core, label: Core, order: 0 }
components:
  - { id: a, label: A, layer: core, globs: [] }
"#;
        // Model still loads (empty-globs is a warning)...
        let m = parse_model_str(yaml).unwrap();
        // ...but validate_structure surfaces the warning.
        let issues = validate_structure(&m);
        assert!(issues
            .iter()
            .any(|i| matches!(i, ModelIssue::EmptyGlobs { component } if component == "a")));
        assert!(issues.iter().all(|i| !i.is_error()));
    }
}
