//! The Architecture Plan API (Loom A2.1, Spec 002 §7): read-only JSON over the
//! `architecture/model.yml` model + the pure `core::architecture::project`
//! status. The Loom server and the CLI's `status --where` build the same
//! `GovernanceState` and call the same projection, so the visual "you are here"
//! and the textual one cannot disagree (NFR3).
//!
//! These functions are **pure of axum** (they take a project root, return
//! serializable data) so they unit-test against fixtures without a server. The
//! thin axum handlers live in [`crate::server`]. NFR4: read-only — nothing here
//! writes; `generate`/`sync` are the CLI's job.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use straymark_baton::overlay::{self as baton_overlay, ComponentIntent};
use straymark_baton::speckit;
use straymark_core::ailog;
use straymark_core::architecture::{
    build_governance_state, collect_source_files, parse_model, project, ArchModel, Projection,
};
use straymark_core::charter::{self, CharterStatus};
use straymark_core::charter_files::parse_files_to_modify;
use straymark_core::drift::glob_match;

/// Default `architecture/` directory: the adopter `.straymark/architecture/`,
/// else a top-level `architecture/`. The server's `--arch-dir` flag overrides
/// this (e.g. the dogfood model under `experiment-loom/architecture/` while
/// governance + globs resolve against the repo root).
pub fn default_arch_dir(project_root: &Path) -> PathBuf {
    let installed = project_root.join(".straymark").join("architecture");
    if installed.is_dir() {
        installed
    } else {
        project_root.join("architecture")
    }
}

/// Load + parse the model from `arch_dir/model.yml`, or `None` when
/// absent/invalid (the API degrades to `model_present: false` rather than
/// erroring — it is a dashboard, not a gate).
fn load_model(arch_dir: &Path) -> Option<ArchModel> {
    let path = arch_dir.join("model.yml");
    if !path.exists() {
        return None;
    }
    parse_model(&path).ok()
}

// ── GET /api/architecture ────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ArchResponse {
    /// False when no `model.yml` exists — the SPA shows a "run generate" hint.
    pub model_present: bool,
    /// True when the project has a SpecKit layout (`.specify/` or `specs/`).
    /// The SPA hides the Intent plane toggle otherwise (gate #5).
    pub speckit_present: bool,
    /// Baton's intent overlay — intended-vs-implemented per component. `None`
    /// when there are no intended components (nothing to lay over the model).
    pub intent: Option<Vec<ComponentIntent>>,
    pub layers: Vec<ArchLayer>,
    pub components: Vec<ArchComponent>,
    pub edges: Vec<ArchEdge>,
}

#[derive(Serialize)]
pub struct ArchLayer {
    pub id: String,
    pub label: String,
    pub order: u32,
    /// Per-state component counts in this layer (the §4 rollup).
    pub counts: BTreeMap<String, usize>,
}

#[derive(Serialize)]
pub struct ArchComponent {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub globs: Vec<String>,
    pub links: Vec<String>,
    pub docs: Vec<String>,
    pub external: bool,
    /// Projected states (`active`/`in-progress`/`implemented`/…), in §4 order.
    pub states: Vec<String>,
}

/// A directed dependency edge (`component.links` → target component id).
#[derive(Serialize)]
pub struct ArchEdge {
    pub source: String,
    pub target: String,
}

/// Build the `/api/architecture` payload: the model (from `arch_dir`) enriched
/// with its projected per-component / per-layer status (governance scanned from
/// `project_root`), plus Baton's intent overlay when the project declares
/// SpecKit intent (gate #5 — the "third plane" Loom consumes).
pub fn build_architecture(project_root: &Path, arch_dir: &Path) -> ArchResponse {
    let speckit_present = !speckit::SpecKitSource::discover(project_root).is_empty();
    let Some(model) = load_model(arch_dir) else {
        return ArchResponse {
            model_present: false,
            speckit_present,
            intent: None,
            layers: Vec::new(),
            components: Vec::new(),
            edges: Vec::new(),
        };
    };
    let projection = projected(project_root, &model);

    // Intent plane: SpecKit's intended components (memory mining, no codescan)
    // laid over THIS model via Baton's pure computation — same `core` matcher
    // and inventory semantics as the projection, so the planes can't disagree.
    let artifacts = speckit::load(project_root);
    let intent = if artifacts.intended_components.is_empty() {
        None
    } else {
        let inventory: Vec<String> = collect_source_files(project_root)
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        Some(baton_overlay::compute(
            Some(&model),
            &artifacts.intended_components,
            &inventory,
        ))
    };

    let layers = model
        .layers
        .iter()
        .map(|l| {
            let counts = projection
                .layers
                .iter()
                .find(|r| r.layer_id == l.id)
                .map(|r| {
                    r.counts
                        .iter()
                        .map(|(s, n)| (s.as_str().to_string(), *n))
                        .collect()
                })
                .unwrap_or_default();
            ArchLayer { id: l.id.clone(), label: l.label.clone(), order: l.order, counts }
        })
        .collect();

    let components = model
        .components
        .iter()
        .map(|c| ArchComponent {
            id: c.id.clone(),
            label: c.label.clone(),
            layer: c.layer.clone(),
            globs: c.globs.clone(),
            links: c.links.clone(),
            docs: c.docs.clone(),
            external: c.external,
            states: states_of(&projection, &c.id),
        })
        .collect();

    let edges = model
        .components
        .iter()
        .flat_map(|c| {
            c.links
                .iter()
                .map(move |t| ArchEdge { source: c.id.clone(), target: t.clone() })
        })
        .collect();

    ArchResponse { model_present: true, speckit_present, intent, layers, components, edges }
}

// ── GET /api/architecture/component/:id ──────────────────────────────────────

#[derive(Serialize)]
pub struct ComponentDetail {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub globs: Vec<String>,
    pub links: Vec<String>,
    pub docs: Vec<String>,
    pub external: bool,
    pub states: Vec<String>,
    /// Source files on disk that the component's globs own (S2).
    pub owned_files: Vec<String>,
    /// Charters whose declared files fall in this component (S2 — "which
    /// Charters touch it"). These are KG nodes, so the client can cross-link.
    pub charters: Vec<RelatedCharter>,
}

/// A Charter that touches a component, with the id the KG node uses.
#[derive(Serialize)]
pub struct RelatedCharter {
    pub charter_id: String,
    pub title: String,
    pub status: String,
}

/// Detail for one component, or `None` when no model / unknown id.
pub fn component_detail(project_root: &Path, arch_dir: &Path, id: &str) -> Option<ComponentDetail> {
    let model = load_model(arch_dir)?;
    let comp = model.components.iter().find(|c| c.id == id)?;
    let projection = projected(project_root, &model);

    let on_disk: Vec<String> = collect_source_files(project_root)
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    let mut owned_files: Vec<String> = on_disk
        .into_iter()
        .filter(|f| comp.globs.iter().any(|g| glob_match(g, f)))
        .collect();
    owned_files.sort();

    // Charters whose declared `## Files to modify` fall in this component.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let related: Vec<RelatedCharter> = charters
        .iter()
        .filter(|c| {
            parse_files_to_modify(&c.body)
                .iter()
                .any(|f| comp.globs.iter().any(|g| glob_match(g, &f.path)))
        })
        .map(|c| RelatedCharter {
            charter_id: c.frontmatter.charter_id.clone(),
            title: charter::display_title(c),
            status: c.frontmatter.status.as_str().to_string(),
        })
        .collect();

    Some(ComponentDetail {
        id: comp.id.clone(),
        label: comp.label.clone(),
        layer: comp.layer.clone(),
        globs: comp.globs.clone(),
        links: comp.links.clone(),
        docs: comp.docs.clone(),
        external: comp.external,
        states: states_of(&projection, &comp.id),
        owned_files,
        charters: related,
    })
}

// ── GET /api/where ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct WhereResponse {
    pub active_charters: Vec<ActiveCharter>,
    /// Declared files of the active (in-progress) Charters.
    pub declared_files: usize,
    /// Of those, the ones already touched in the working tree (declared ∩ git).
    pub touched_files: usize,
    pub recent_ailogs: Vec<String>,
    /// Files related to an open TDE — a proxy for outstanding debt.
    pub open_debt_files: usize,
}

#[derive(Serialize)]
pub struct ActiveCharter {
    pub charter_id: String,
    pub title: String,
}

/// The "where are we" summary (Spec 002 §8) — the textual companion's data, as
/// JSON. Independent of `model.yml` (it reads governance directly).
pub fn build_where(project_root: &Path) -> WhereResponse {
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let active_charters = charters
        .iter()
        .filter(|c| c.frontmatter.status == CharterStatus::InProgress)
        .map(|c| ActiveCharter {
            charter_id: c.frontmatter.charter_id.clone(),
            title: charter::display_title(c),
        })
        .collect();

    let state = build_governance_state(project_root);

    WhereResponse {
        active_charters,
        declared_files: state.active_charter_files.len(),
        touched_files: state.in_progress_files.len(),
        recent_ailogs: recent_ailogs(project_root, 5),
        open_debt_files: state.tde_files.len(),
    }
}

// ── GET /api/architecture/plan.drawio ────────────────────────────────────────

/// The raw `plan.drawio` XML, or `None` when absent. Served verbatim so the
/// client (maxGraph, A2.3) keeps the human geometry; status is an overlay the
/// client applies from `/api/architecture`, never a server rewrite (NFR1/NFR4).
pub fn read_plan_drawio(arch_dir: &Path) -> Option<String> {
    std::fs::read_to_string(arch_dir.join("plan.drawio")).ok()
}

// ── watcher support (A2.2) ───────────────────────────────────────────────────

/// A stable JSON signature of the current architecture payload, for the
/// watcher's no-op suppression: it broadcasts an `architecture` event only when
/// this changes (a governance edit that actually moves a component's state, or a
/// `model.yml`/`plan.drawio` change), never on a bare mtime touch.
pub fn projection_signature(project_root: &Path, arch_dir: &Path) -> String {
    serde_json::to_string(&build_architecture(project_root, arch_dir)).unwrap_or_default()
}

/// True when a changed path can affect the architecture overlay: governance
/// markdown (`.md` → charters/AILOGs/TDEs), or the model / plan files.
pub fn is_architecture_relevant(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str());
    matches!(ext, Some("md"))
        || path.file_name().and_then(|n| n.to_str()) == Some("model.yml")
        || ext == Some("drawio")
}

// ── shared helpers ───────────────────────────────────────────────────────────

fn projected(project_root: &Path, model: &ArchModel) -> Projection {
    let state = build_governance_state(project_root);
    project(model, &state)
}

/// The kebab-case state names for a component id, in projection order.
fn states_of(projection: &Projection, id: &str) -> Vec<String> {
    projection
        .components
        .iter()
        .find(|c| c.component_id == id)
        .map(|c| c.states.iter().map(|s| s.as_str().to_string()).collect())
        .unwrap_or_default()
}

/// Filenames (without extension) of the most recent `n` AILOGs (date-prefixed
/// filenames sort lexically).
fn recent_ailogs(project_root: &Path, n: usize) -> Vec<String> {
    let dir = ailog::agent_logs_dir(project_root);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("AILOG-") && s.ends_with(".md"))
                    .unwrap_or(false)
            })
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
            .collect(),
        Err(_) => return Vec::new(),
    };
    names.sort();
    names.into_iter().rev().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixture project: `.straymark/architecture/model.yml` with two
    /// components in one layer, an in-progress Charter on `auth`, a closed one on
    /// `billing`, and matching source files on disk.
    fn fixture() -> tempfile::TempDir {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let arch = root.join(".straymark/architecture");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(
            arch.join("model.yml"),
            r#"version: 0
layers:
  - { id: "core", label: "Core", order: 0 }
components:
  - { id: "auth", label: "Auth", layer: "core", globs: ["src/auth/**"], links: ["billing"], docs: [], external: false }
  - { id: "billing", label: "Billing", layer: "core", globs: ["src/billing/**"], links: [], docs: [], external: false }
"#,
        )
        .unwrap();
        std::fs::write(arch.join("plan.drawio"), "<mxfile>plan</mxfile>\n").unwrap();

        let charters = root.join(".straymark/charters");
        std::fs::create_dir_all(&charters).unwrap();
        std::fs::write(
            charters.join("01-auth.md"),
            "---\ncharter_id: CHARTER-01\nstatus: in-progress\neffort_estimate: M\ntrigger: \"t\"\n---\n\n# Charter: Auth\n\n## Files to modify\n\n| File | Change |\n|---|---|\n| `src/auth/login.rs` | edit |\n\n## Tasks\n\n1. Go.\n",
        )
        .unwrap();
        std::fs::write(
            charters.join("02-billing.md"),
            "---\ncharter_id: CHARTER-02\nstatus: closed\neffort_estimate: S\ntrigger: \"t\"\n---\n\n# Charter: Billing\n\n## Files to modify\n\n| File | Change |\n|---|---|\n| `src/billing/charge.rs` | edit |\n\n## Tasks\n\n1. Done.\n",
        )
        .unwrap();

        std::fs::create_dir_all(root.join("src/auth")).unwrap();
        std::fs::create_dir_all(root.join("src/billing")).unwrap();
        std::fs::write(root.join("src/auth/login.rs"), "// auth\n").unwrap();
        std::fs::write(root.join("src/billing/charge.rs"), "// bill\n").unwrap();
        tmp
    }

    fn arch(root: &Path) -> PathBuf {
        root.join(".straymark/architecture")
    }

    #[test]
    fn architecture_absent_model_degrades() {
        let tmp = tempfile::TempDir::new().unwrap();
        let r = build_architecture(tmp.path(), &arch(tmp.path()));
        assert!(!r.model_present);
        assert!(r.components.is_empty());
    }

    #[test]
    fn architecture_projects_states_and_edges() {
        let tmp = fixture();
        let r = build_architecture(tmp.path(), &arch(tmp.path()));
        assert!(r.model_present);
        assert_eq!(r.components.len(), 2);
        // auth: active (in-progress Charter declares src/auth/**).
        let auth = r.components.iter().find(|c| c.id == "auth").unwrap();
        assert!(auth.states.contains(&"active".to_string()));
        // billing: implemented (closed Charter).
        let billing = r.components.iter().find(|c| c.id == "billing").unwrap();
        assert!(billing.states.contains(&"implemented".to_string()));
        // edge auth → billing from links.
        assert!(r.edges.iter().any(|e| e.source == "auth" && e.target == "billing"));
        // layer rollup counts the active component.
        let core = r.layers.iter().find(|l| l.id == "core").unwrap();
        assert_eq!(core.counts.get("active"), Some(&1));
    }

    #[test]
    fn component_detail_lists_owned_files() {
        let tmp = fixture();
        let d = component_detail(tmp.path(), &arch(tmp.path()), "auth").unwrap();
        assert_eq!(d.label, "Auth");
        assert_eq!(d.owned_files, vec!["src/auth/login.rs"]);
        // CHARTER-01 declares src/auth/login.rs → it touches the auth component.
        assert_eq!(d.charters.len(), 1);
        assert_eq!(d.charters[0].charter_id, "CHARTER-01");
        assert!(component_detail(tmp.path(), &arch(tmp.path()), "ghost").is_none());
    }

    #[test]
    fn where_reports_active_charter_and_progress() {
        let tmp = fixture();
        let w = build_where(tmp.path());
        assert_eq!(w.active_charters.len(), 1);
        assert_eq!(w.active_charters[0].charter_id, "CHARTER-01");
        assert_eq!(w.declared_files, 1); // src/auth/login.rs declared
    }

    #[test]
    fn plan_drawio_served_or_absent() {
        let tmp = fixture();
        assert!(read_plan_drawio(&arch(tmp.path())).unwrap().contains("mxfile"));
        let empty = tempfile::TempDir::new().unwrap();
        assert!(read_plan_drawio(&arch(empty.path())).is_none());
    }

    #[test]
    fn signature_changes_when_governance_changes() {
        let tmp = fixture();
        let a = arch(tmp.path());
        let sig1 = projection_signature(tmp.path(), &a);
        // Flip the auth Charter in-progress → declared: auth is no longer active.
        std::fs::write(
            tmp.path().join(".straymark/charters/01-auth.md"),
            "---\ncharter_id: CHARTER-01\nstatus: declared\neffort_estimate: M\ntrigger: \"t\"\n---\n\n# Charter: Auth\n\n## Files to modify\n\n| File | Change |\n|---|---|\n| `src/auth/login.rs` | edit |\n",
        )
        .unwrap();
        let sig2 = projection_signature(tmp.path(), &a);
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn architecture_relevance_filter() {
        assert!(is_architecture_relevant(Path::new("x/CHARTER-01.md")));
        assert!(is_architecture_relevant(Path::new("a/architecture/model.yml")));
        assert!(is_architecture_relevant(Path::new("a/plan.drawio")));
        assert!(!is_architecture_relevant(Path::new("a/.straymark/config.yml")));
        assert!(!is_architecture_relevant(Path::new("a/notes.txt")));
    }

    #[test]
    fn intent_overlay_folded_for_speckit_project() {
        use straymark_baton::overlay::IntentState;
        let tmp = fixture();
        let memory = tmp.path().join(".specify/memory");
        std::fs::create_dir_all(&memory).unwrap();
        // "Auth" matches the modeled + implemented component.
        std::fs::write(memory.join("Arquitectura - Auth.md"), "# Auth\n").unwrap();
        // "PolicyEngine" is designed but never implemented (and not modeled).
        std::fs::write(memory.join("Arquitectura - PolicyEngine.md"), "# PE\n").unwrap();

        let r = build_architecture(tmp.path(), &arch(tmp.path()));
        assert!(r.speckit_present);
        let intent = r.intent.as_ref().expect("intent overlay present");
        let find = |id: &str| intent.iter().find(|c| c.component == id).unwrap();

        let auth = find("auth");
        assert_eq!(auth.state, IntentState::IntendedAndImplemented);
        assert!(auth.modeled);
        assert_eq!(auth.matched_intent.as_deref(), Some("auth"));

        // billing has code but the design never mentioned it.
        let billing = find("billing");
        assert_eq!(billing.state, IntentState::ImplementedNotIntended);

        let pe = find("policyengine");
        assert_eq!(pe.state, IntentState::IntendedNotImplemented);
        assert!(!pe.modeled);
    }

    #[test]
    fn intent_overlay_absent_without_speckit() {
        // The base fixture has no `.specify/` or `specs/` — the plane stays off.
        let tmp = fixture();
        let r = build_architecture(tmp.path(), &arch(tmp.path()));
        assert!(!r.speckit_present);
        assert!(r.intent.is_none());
    }
}
