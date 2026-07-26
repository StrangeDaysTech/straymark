//! `straymark followups note FU-NNN "<text>"` — append a dated annotation to
//! an entry's `Notes`.
//!
//! Before this verb (Weft field report #355), recording that an entry received
//! a partial mitigation — without changing its status — meant hand-editing a
//! CLI-parsed markdown file. Two things go wrong there: the edit can malform
//! the entry (wrong bullet shape, wrong field name) and break `list`/`status`/
//! `drift`; and nothing enforces *when* the annotation was made or *what*
//! motivated it, so an entry accumulates undated prose whose provenance is
//! gone by the next triage.
//!
//! This verb writes through the same surgical field editor every other write
//! command uses, stamps the date, and records `--source` (the Charter or AILOG
//! that motivated the note) when given.

use anyhow::{anyhow, bail, Result};
use chrono::Local;

use crate::followups;
use crate::utils;

pub fn run(path: &str, fu_id: &str, text: &str, source: Option<&str>) -> Result<()> {
    if text.trim().is_empty() {
        bail!("Note text is empty — pass the annotation as the second argument.");
    }

    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: see STRAYMARK.md §16 for the adoption walkthrough.",
            registry_path.display()
        );
    }
    let registry = followups::parse_registry(&registry_path)?;

    // A malformed registry aborts BEFORE any write: editing one entry inside a
    // file whose structure the parser already distrusts is how a governance
    // artifact gets half-corrupted.
    guard_parse_warnings(&registry)?;

    let entry = followups::find_entry(&registry, fu_id)
        .ok_or_else(|| {
            anyhow!(
                "Entry {} not found in the registry.\n  hint: run `straymark followups list` to see entries.",
                fu_id
            )
        })?
        .clone();

    let date = Local::now().format("%Y-%m-%d").to_string();
    let notes = followups::append_note(entry.notes.as_deref(), text, &date, source);
    let body = followups::set_entry_field(&registry.body, &entry, "Notes", &notes);
    followups::write_recounted(&registry_path, &registry.frontmatter_raw, &body)?;

    utils::success(&format!("{} annotated.", entry.fu_id));
    println!("  Notes: {}", notes);
    Ok(())
}

/// Refuse to mutate a registry the parser could not read cleanly.
pub(crate) fn guard_parse_warnings(registry: &followups::Registry) -> Result<()> {
    if registry.warnings.is_empty() {
        return Ok(());
    }
    for w in &registry.warnings {
        utils::warn(w);
    }
    bail!(
        "Refusing to write: the registry has {} parse warning(s) above. Fix the malformed \
         entr{} first — a surgical edit against a structure the parser mis-read can corrupt \
         neighbouring entries.",
        registry.warnings.len(),
        if registry.warnings.len() == 1 { "y" } else { "ies" }
    );
}
