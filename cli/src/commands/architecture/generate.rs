//! `straymark architecture generate` (Loom A1.2, Spec 002 §5) — write a
//! first-draft `architecture/model.yml` + `plan.drawio`.
//!
//! Hybrid seed: top-level source dirs become components (globs `dir/**`),
//! enriched by mining the C4 diagrams and "Affected Components" tables of
//! existing ADRs (better labels + `links`). The result is intentionally rough —
//! a starting point the human refines (A1.5 hand-refines the dogfood). The Loom
//! server stays read-only; writing files is the CLI's job (NFR4). The scanning,
//! mining, and rendering machinery lives in [`super::common`] (shared with
//! `sync`/`validate`).

use anyhow::{bail, Context, Result};
use colored::Colorize;
use straymark_core::architecture::{parse_model_str, ArchModel};

use super::{common, drawio};
use crate::utils;

pub fn run(path: &str, force: bool, out: Option<&str>) -> Result<()> {
    let root = common::resolve_root(path);
    let (out_dir, model_path, drawio_path) = common::artifact_paths(&root, out);

    if !force && (model_path.exists() || drawio_path.exists()) {
        bail!(
            "architecture artifacts already exist in {} — pass {} to overwrite",
            out_dir.display(),
            "--force".cyan()
        );
    }

    utils::info(&format!("Scanning {} for source structure…", root.display()));
    let dirs = common::source_component_dirs(&root);
    if dirs.is_empty() {
        bail!(
            "no source directories found under {} (nothing to model)",
            root.display()
        );
    }

    let mut model = ArchModel {
        version: 0,
        layers: common::seed_layers(),
        components: dirs.iter().map(|d| common::component_from_dir(d)).collect(),
    };

    let enrich = common::enrich_from_adrs(&root, &mut model);

    // Never emit an invalid model: render → parse back through core.
    let yaml = common::render_model_yaml(&model);
    parse_model_str(&yaml)
        .context("internal error: generated model.yml failed core validation")?;
    let drawio_xml = drawio::render_drawio(&model);

    utils::ensure_dir(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    std::fs::write(&model_path, &yaml)
        .with_context(|| format!("writing {}", model_path.display()))?;
    std::fs::write(&drawio_path, &drawio_xml)
        .with_context(|| format!("writing {}", drawio_path.display()))?;

    utils::success(&format!(
        "Wrote {} ({} component{}, {} layer{})",
        model_path.display(),
        model.components.len(),
        common::plural(model.components.len()),
        model.layers.len(),
        common::plural(model.layers.len()),
    ));
    utils::success(&format!("Wrote {}", drawio_path.display()));
    common::report_enrichment(&enrich);
    utils::info(&format!(
        "Refine by hand: reassign components from `{}` to real layers, then open the plan in DrawIO.",
        common::UNASSIGNED
    ));
    Ok(())
}
