//! straymark-core — shared document model and knowledge graph for StrayMark.
//!
//! Extracted from the CLI in Loom M0 (`CHARTER-01-loom-server`) so that the
//! CLI and the Loom visualization server parse StrayMark documents with
//! exactly the same code: one parser, structurally no drift
//! (`ADR-2026-06-02-001`).

pub mod ailog;
pub mod architecture;
pub mod charter;
pub mod charter_files;
pub mod config;
pub mod document;
pub mod drift;
pub mod entities;
pub mod graph;
pub mod utils;
