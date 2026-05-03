//! `devtrail charter` — Charter lifecycle subcommands.
//!
//! Phase 1 shipped `new` (scaffold from template), `list` (enumerate with filters),
//! and `status` (detail view).
//! Phase 2 adds `close` (interactive telemetry) and `drift` (file-vs-commit
//! drift check, in PR 3).
//! Phase 3 will add `audit` (multi-model external audit).

pub mod audit;
pub mod close;
pub mod drift;
pub mod list;
pub mod new;
pub mod status;
