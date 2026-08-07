//! `straymark followups` — first-class CLI surface for the follow-ups
//! backlog registry (`.straymark/follow-ups-backlog.md`).
//!
//! Subcommands mirror the `charter` namespace: `list` / `status` enumerate
//! and inspect, `drift` keeps the registry in sync with AILOGs (native
//! replacement for the deprecated adopter-side bash script), `recount`
//! reconciles the CLI-owned counters after a manual-triage session,
//! `promote` automates the FU → TDE elevation, and `verify` re-checks an
//! entry's premise at execution time (AIDEC-2026-07-18-001). See
//! `FOLLOW-UPS-BACKLOG-PATTERN.md` and `STRAYMARK.md §16`.
//!
//! `note`, `set-status` and `new` (CHARTER-01, from Weft reports #355/#360)
//! close the last paths that required hand-editing a CLI-parsed file: they
//! annotate, re-state and create entries through the same surgical helpers the
//! rest of the namespace writes with, recomputing the CLI-owned counters in the
//! same step so the edit-then-`recount` desync window does not exist.

pub mod drift;
pub mod install_merge_driver;
pub mod list;
pub mod merge_driver;
pub mod new;
pub mod note;
pub mod promote;
pub mod recount;
pub mod set_status;
pub mod status;
pub mod verify;
