//! `devtrail charter` — Charter lifecycle subcommands.
//!
//! Phase 1 ships `new` (scaffold from template), `list` (enumerate with filters),
//! and `status` (detail view).
//! Phase 2 will add `close` (interactive telemetry) and `drift` (file-vs-commit
//! drift check). Phase 3 will add `audit` (multi-model external audit).

pub mod list;
pub mod new;
pub mod status;
