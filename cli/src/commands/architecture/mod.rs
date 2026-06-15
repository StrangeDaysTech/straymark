//! `straymark architecture` — generate / sync / validate the architecture model
//! (Loom Spec 002, the "you are here" map). EXPERIMENTAL.
//!
//! A1.2 ships `generate` (the hybrid codebase + ADR seed). `sync` / `validate`
//! are scaffolded as stubs here and filled in A1.3.

pub mod adr_mining;
pub mod common;
pub mod drawio;
pub mod generate;
pub mod sync;
pub mod validate;
pub mod where_view;
