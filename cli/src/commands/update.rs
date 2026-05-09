use anyhow::Result;
use colored::Colorize;

use crate::utils;

pub fn run(method: &str) -> Result<()> {
    let target = std::env::current_dir()?;

    // Update framework (skip if not initialized)
    if target.join(".straymark").exists() {
        println!("{}", "── Framework ──".bold());
        if let Err(e) = super::update_framework::run() {
            utils::warn(&format!("Framework update failed: {}", e));
        }
    } else {
        utils::warn("StrayMark framework not initialized — skipping framework update.");
    }

    // Update CLI
    println!();
    println!("{}", "── CLI ──".bold());
    if let Err(e) = super::update_cli::run(method) {
        utils::warn(&format!("CLI update failed: {}", e));
    }

    Ok(())
}
