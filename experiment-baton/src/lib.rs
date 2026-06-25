//! straymark-baton — the Coherence Bridge (Baton Phase 1).
//!
//! Read-only. Reconciles the *intended* architecture recorded in SpecKit
//! (`specs/**` + `.specify/memory/**`) against the *emergent* architecture
//! StrayMark/Loom derive from governance and code. Phase 1 builds the
//! **read side**: a tolerant SpecKit adapter that mines the intent inputs the
//! coherence engine (B2/B3) will reconcile.
//!
//! Spec: `experiment-baton/specs/001-coherence-bridge/spec.md`.
//! Scope rule: touches no models (no routing/tier/budget/token/cost).

pub mod speckit;
