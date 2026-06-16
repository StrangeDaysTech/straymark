//! The architecture model and its pure "you are here" status projection.
//!
//! Spec 002 (`experimento/specs/002-architecture-plan/`). Two halves:
//! - [`model`] — typed `architecture/model.yml` (layers, components, globs).
//! - [`projection`] — the pure `(model + governance state) -> Projection`
//!   function (Spec 002 §4) plus the model↔layout integrity signals (§3.3).
//!
//! Both live in `straymark-core` so the CLI (`status --where`, A1.4) and the
//! Loom server (A2) reuse exactly one extractor — the discipline that moved
//! charter/drift into `core` in A1.0.

pub mod gather;
pub mod model;
pub mod projection;

pub use gather::{build_governance_state, collect_source_files};
pub use model::{parse_model, parse_model_str, validate_structure, ArchModel, Component, Layer, ModelIssue};
pub use projection::{
    project, validate_model, ComponentProjection, ComponentState, GovernanceState,
    IntegritySignal, LayerRollup, Projection,
};
