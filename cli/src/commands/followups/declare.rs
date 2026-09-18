//! `straymark followups declare FU-NNN --work-verb … [--design-provenance …]`
//! — write an existing entry's declared work classification (Baton #332).
//!
//! The registry is CLI-owned ("never hand-edit an entry"), and Track C asks
//! follow-ups to declare `Work verb` / `Design provenance` — yet no verb could
//! write them: `new` had no flags, `note` only appends to `Notes` (not the
//! slot Baton reads), and `drift --apply` extracts without them. An adopter
//! honouring both rules was left with undeclared entries (Estoa, #432).
//!
//! The declaration is written as a unit: both bullets are replaced together,
//! so omitting `--design-provenance` removes a previous one instead of letting
//! a stale provenance pair with the new verb. The vocabulary is validated here
//! (the strict writer); `validate` and Baton stay lenient readers.

use anyhow::{anyhow, bail, Result};
use colored::Colorize;

use crate::commands::followups::note::guard_parse_warnings;
use crate::followups::{self, Declaration};

pub fn run(path: &str, fu_id: &str, work_verb: &str, design_provenance: Option<&str>) -> Result<()> {
    let declaration = Declaration::new(work_verb, design_provenance)?;

    let resolved = crate::utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let registry_path = followups::registry_path(&resolved.path);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: see STRAYMARK.md §16 for the adoption walkthrough.",
            registry_path.display()
        );
    }
    let registry = followups::parse_registry(&registry_path)?;
    guard_parse_warnings(&registry)?;

    let entry = followups::find_entry_unique(&registry, fu_id)?.clone();
    let describe = |verb: Option<&str>, provenance: Option<&str>| match (verb, provenance) {
        (None, _) => "undeclared".to_string(),
        (Some(v), None) => v.to_string(),
        (Some(v), Some(p)) => format!("{v} / {p}"),
    };
    let previous = describe(entry.work_verb.as_deref(), entry.design_provenance.as_deref());
    let next = describe(
        Some(&declaration.work_verb),
        declaration.design_provenance.as_deref(),
    );

    if declaration.matches(&entry) {
        println!(
            "  {} {} is already declared `{}` — nothing to change.",
            "OK".green().bold(),
            entry.fu_id,
            next
        );
        return Ok(());
    }

    let body = followups::set_entry_declaration(&registry.body, &entry, &declaration);
    followups::write_recounted(&registry_path, &registry.frontmatter_raw, &body)?;

    crate::utils::success(&format!(
        "{}: {} → {}",
        entry.fu_id,
        previous.dimmed(),
        next.bold()
    ));
    Ok(())
}
