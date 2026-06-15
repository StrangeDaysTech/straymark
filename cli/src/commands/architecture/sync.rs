//! `straymark architecture sync` — stub (A1.3).
//!
//! Will detect new code dirs / new ADR components since generation and append
//! suggestions to `model.yml` without clobbering human edits (Spec 002 §5).

use anyhow::Result;

use crate::utils;

pub fn run(_path: &str) -> Result<()> {
    utils::warn("`architecture sync` is not yet implemented (coming in Loom A1.3).");
    utils::info("For now, re-run `straymark architecture generate --force` to regenerate the seed.");
    Ok(())
}
