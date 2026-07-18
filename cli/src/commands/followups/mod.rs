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

pub mod drift;
pub mod list;
pub mod promote;
pub mod recount;
pub mod status;
pub mod verify;
