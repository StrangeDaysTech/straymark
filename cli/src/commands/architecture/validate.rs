//! `straymark architecture validate` — stub (A1.3).
//!
//! Will parse `model.yml` + `plan.drawio` cell ids and report the Spec 002 §3.3
//! integrity signals (undrawn / unmodeled / empty) via
//! `straymark_core::architecture::validate_model`.

use anyhow::Result;

use crate::utils;

pub fn run(_path: &str) -> Result<()> {
    utils::warn("`architecture validate` is not yet implemented (coming in Loom A1.3).");
    Ok(())
}
