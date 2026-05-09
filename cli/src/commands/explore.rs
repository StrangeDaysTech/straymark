use anyhow::{bail, Result};

use crate::config::StrayMarkConfig;
use crate::utils;

pub fn run(path: &str, lang_override: Option<&str>) -> Result<()> {
    let resolved = match utils::resolve_project_root(path) {
        Some(r) => r,
        None => {
            utils::warn("StrayMark is not initialized in this directory or repo root.");
            utils::info("Run 'straymark init' to initialize StrayMark.");
            bail!("No StrayMark installation found");
        }
    };

    // Effective language: --lang flag > config.language (when config
    // exists) > $LC_ALL / $LANG > "en".
    let language = match lang_override {
        Some(l) => l.to_string(),
        None => StrayMarkConfig::resolve_language(&resolved.path),
    };

    crate::tui::run(&resolved.path, resolved.is_fallback, &language)
}
