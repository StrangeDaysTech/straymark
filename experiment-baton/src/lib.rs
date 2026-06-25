//! straymark-baton — the Coherence Bridge (Baton Phase 1).
//!
//! Read-only. Reconciles the *intended* architecture recorded in SpecKit
//! (`specs/**` + `.specify/memory/**`) against the *emergent* architecture
//! StrayMark/Loom derive from governance and code.
//!
//! - `speckit` (B1): tolerant SpecKit adapter — the intent inputs.
//! - `codescan` (B2): heuristic contract-shape extraction from Go/TS code.
//! - `intent` / `provenance` (B2): the typed `IntentModel` and cross-spec
//!   contract provenance edges (the core of issue #304).
//!
//! Spec: `experiment-baton/specs/001-coherence-bridge/spec.md`.
//! Scope rule: touches no models (no routing/tier/budget/token/cost).

pub mod codescan;
pub mod coherence;
pub mod intent;
pub mod overlay;
pub mod provenance;
mod scan;
pub mod speckit;
