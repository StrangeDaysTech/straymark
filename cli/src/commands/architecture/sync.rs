//! `straymark architecture sync` (Loom A1.3, Spec 002 §5) — keep the model
//! current as code/ADRs evolve, **append-only**.
//!
//! Detects top-level source dirs not yet covered by any component's globs (and
//! enriches them from ADRs), then proposes them. Default is a dry-run report;
//! `--apply` appends — never clobbering human edits (NFR1): new components are
//! text-appended to `model.yml` and new cells text-inserted into `plan.drawio`
//! below the existing geometry. Existing bytes are preserved verbatim.

use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use straymark_core::architecture::{parse_model, parse_model_str, ArchModel, Component};
use straymark_core::drift::glob_match;

use super::{common, drawio};
use crate::utils;

pub fn run(path: &str, out: Option<&str>, apply: bool) -> Result<()> {
    let root = common::resolve_root(path);
    let (_out_dir, model_path, drawio_path) = common::artifact_paths(&root, out);

    if !model_path.exists() {
        bail!(
            "no architecture model at {} — run {} first",
            model_path.display(),
            "straymark architecture generate".cyan()
        );
    }
    let model = parse_model(&model_path)
        .with_context(|| format!("parsing {}", model_path.display()))?;

    // New top-level source dirs not covered by any existing component.
    let scanned = common::top_level_source_dirs(&root);
    let on_disk: Vec<String> = common::collect_source_files(&root)
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    let existing_ids: HashSet<&str> = model.components.iter().map(|c| c.id.as_str()).collect();

    let new_dirs: Vec<String> = scanned
        .into_iter()
        .filter(|dir| !existing_ids.contains(common::kebab(dir).as_str()))
        .filter(|dir| !dir_covered(&model, &on_disk, dir))
        .collect();

    if new_dirs.is_empty() {
        utils::success("Architecture model is up to date — no new source dirs to add.");
        return Ok(());
    }

    // Enrich the new components from ADRs (run over existing+new so links to
    // existing components resolve), then take just the appended ones.
    let mut combined = model.clone();
    let base = combined.components.len();
    combined
        .components
        .extend(new_dirs.iter().map(|d| common::component_from_dir(d)));
    let enrich = common::enrich_from_adrs(&root, &mut combined);
    let appended: Vec<Component> = combined.components[base..].to_vec();

    if !apply {
        utils::info(&format!(
            "{} new component{} would be added (dry-run — pass {} to write):",
            appended.len(),
            common::plural(appended.len()),
            "--apply".cyan()
        ));
        for c in &appended {
            print_proposal(c);
        }
        common::report_enrichment(&enrich);
        utils::info("No files changed.");
        return Ok(());
    }

    apply_changes(&model_path, &drawio_path, &appended)?;
    utils::success(&format!(
        "Appended {} component{} to {}",
        appended.len(),
        common::plural(appended.len()),
        model_path.display()
    ));
    for c in &appended {
        print_proposal(c);
    }
    common::report_enrichment(&enrich);
    Ok(())
}

/// True when an existing component's globs already match a file under `dir/`.
fn dir_covered(model: &ArchModel, on_disk: &[String], dir: &str) -> bool {
    let prefix = format!("{dir}/");
    model.components.iter().any(|c| {
        c.globs.iter().any(|g| {
            on_disk
                .iter()
                .any(|f| f.starts_with(&prefix) && glob_match(g, f))
        })
    })
}

fn print_proposal(c: &Component) {
    let links = if c.links.is_empty() {
        String::new()
    } else {
        format!(" → links: {}", c.links.join(", "))
    };
    println!(
        "  {} {} (globs: {}){}",
        "+".green(),
        c.id.bold(),
        c.globs.join(", "),
        links
    );
}

/// Append the new components to `model.yml` (text) and `plan.drawio` (cells),
/// preserving every existing byte. Validates the resulting model before writing.
fn apply_changes(
    model_path: &std::path::Path,
    drawio_path: &std::path::Path,
    appended: &[Component],
) -> Result<()> {
    let mut yaml = std::fs::read_to_string(model_path)
        .with_context(|| format!("reading {}", model_path.display()))?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    yaml.push_str("# Added by `straymark architecture sync`:\n");
    for c in appended {
        yaml.push_str(&common::render_component_block(c));
    }
    // Safety: the appended file must still parse + validate.
    parse_model_str(&yaml).context(
        "internal error: model.yml would be invalid after append (is `components:` the last key?)",
    )?;

    std::fs::write(model_path, &yaml)
        .with_context(|| format!("writing {}", model_path.display()))?;

    if drawio_path.exists() {
        let xml = std::fs::read_to_string(drawio_path)
            .with_context(|| format!("reading {}", drawio_path.display()))?;
        match drawio::append_cells(&xml, appended) {
            Some(updated) => {
                std::fs::write(drawio_path, &updated)
                    .with_context(|| format!("writing {}", drawio_path.display()))?;
                utils::success(&format!("Appended cells to {}", drawio_path.display()));
            }
            None => utils::warn(&format!(
                "{} is not a recognized DrawIO document — skipped (model.yml updated)",
                drawio_path.display()
            )),
        }
    } else {
        utils::warn("no plan.drawio — model.yml updated only (run `architecture generate`).");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use straymark_core::architecture::Layer;

    fn model_with(globs: &[&str]) -> ArchModel {
        ArchModel {
            version: 0,
            layers: vec![Layer { id: "core".into(), label: "Core".into(), order: 0 }],
            components: vec![Component {
                id: "cli".into(),
                label: "CLI".into(),
                layer: "core".into(),
                globs: globs.iter().map(|g| g.to_string()).collect(),
                links: vec![],
                docs: vec![],
                external: false,
            }],
        }
    }

    #[test]
    fn covered_dir_not_proposed() {
        let model = model_with(&["cli/**"]);
        let on_disk = vec!["cli/src/main.rs".to_string(), "core/src/lib.rs".to_string()];
        // `cli` is owned by an existing glob → covered.
        assert!(dir_covered(&model, &on_disk, "cli"));
        // `core` has files but no component glob covers it → new.
        assert!(!dir_covered(&model, &on_disk, "core"));
    }

    #[test]
    fn renamed_glob_still_covers() {
        // A human narrowed the glob to a subdir; the dir is still covered.
        let model = model_with(&["cli/src/**"]);
        let on_disk = vec!["cli/src/main.rs".to_string()];
        assert!(dir_covered(&model, &on_disk, "cli"));
    }
}
