//! `straymark followups promote FU-NNN` — automate the FU → TDE elevation.
//!
//! Automates the three-step flow of `FOLLOW-UPS-BACKLOG-PATTERN.md`
//! §Promotion to TDE:
//!
//! 1. Create the TDE document from the framework template (same machinery as
//!    `straymark new --type tde`), pre-filling the title from the FU
//!    description and `promoted_from_followup: FU-NNN` for traceability.
//! 2. In the registry entry: `Status: promoted`, `Destination: <TDE id>`,
//!    `Promoted to: <TDE id>`.
//! 3. Recompute the CLI-owned frontmatter counters (+ v0 → v1 upgrade).
//!
//! Non-interactive by design (agent-friendly): the FU description becomes the
//! TDE title unless `--title` overrides it. The operator fills impact/effort/
//! type in the created document — promotion is the *traceable hand-off*, not
//! the prioritization (that stays human, per AGENT-RULES.md §3).

use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use colored::Colorize;
use std::path::Path;

use crate::followups::{self, FuStatus};
use crate::utils;

pub fn run(path: &str, fu_id: &str, title: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;
    let straymark_dir = project_root.join(".straymark");

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: see STRAYMARK.md §16 for the adoption walkthrough.",
            registry_path.display()
        );
    }
    let registry = followups::parse_registry(&registry_path)?;
    let entry = followups::find_entry(&registry, fu_id)
        .ok_or_else(|| {
            anyhow!(
                "Entry {} not found in the registry.\n  hint: run `straymark followups list` to see entries.",
                fu_id
            )
        })?
        .clone();

    if entry.status == FuStatus::Promoted {
        bail!(
            "{} is already promoted (Promoted to: {}). Nothing to do.",
            entry.fu_id,
            entry.promoted_to.as_deref().unwrap_or("?")
        );
    }
    if matches!(entry.status, FuStatus::Closed | FuStatus::Superseded) {
        bail!(
            "{} is {} — promote applies to open/in-progress entries. Reopen it first if the debt is real.",
            entry.fu_id,
            entry.status.as_str()
        );
    }

    // ── 1. Create the TDE document ──
    let tde_title = title.unwrap_or(&entry.description).to_string();
    let (tde_id, tde_path) = create_tde(project_root, &straymark_dir, &tde_title, &entry)?;

    // ── 2 + 3. Update the registry entry + counters ──
    let mut body = followups::set_entry_field(&registry.body, &entry, "Status", "promoted");
    // Spans moved after the first edit — re-parse before each subsequent one.
    let reparsed = followups::parse_registry_str(
        &registry_path,
        &followups::assemble(&registry.frontmatter_raw, &body),
    )?;
    let entry2 = followups::find_entry(&reparsed, &entry.fu_id).unwrap().clone();
    body = followups::set_entry_field(&body, &entry2, "Destination", &tde_id);
    let reparsed = followups::parse_registry_str(
        &registry_path,
        &followups::assemble(&registry.frontmatter_raw, &body),
    )?;
    let entry3 = followups::find_entry(&reparsed, &entry.fu_id).unwrap().clone();
    body = followups::set_entry_field(&body, &entry3, "Promoted to", &tde_id);

    let final_registry = followups::parse_registry_str(
        &registry_path,
        &followups::assemble(&registry.frontmatter_raw, &body),
    )?;
    let counters = followups::compute_counters(&final_registry);
    let fm = followups::fm_apply_counters_and_v1(&registry.frontmatter_raw, &counters);
    std::fs::write(&registry_path, followups::assemble(&fm, &body))?;

    let tde_rel = tde_path
        .strip_prefix(project_root)
        .unwrap_or(&tde_path)
        .display()
        .to_string();
    utils::success(&format!("{} promoted → {}", entry.fu_id, tde_id));
    println!("  TDE created: {}", tde_rel);
    println!(
        "  Registry updated: Status promoted, Destination/Promoted to → {} (counters recomputed).",
        tde_id
    );
    println!();
    println!("  {}", "Next steps:".bold());
    println!("    1. Fill impact / effort / type and the body sections in the TDE.");
    println!("    2. Prioritization and assignment stay human (AGENT-RULES.md §3).");
    println!(
        "    3. Commit both files together: {}",
        format!("git add {} .straymark/follow-ups-backlog.md", tde_rel).dimmed()
    );
    println!();
    Ok(())
}

/// Create the TDE document from the (localized) framework template. Returns
/// `(tde_id, path)`. Mirrors the fill logic of `commands::new` for the TDE
/// type, plus the `promoted_from_followup` traceability field.
fn create_tde(
    project_root: &Path,
    straymark_dir: &Path,
    title: &str,
    entry: &followups::Entry,
) -> Result<(String, std::path::PathBuf)> {
    let lang = crate::config::StrayMarkConfig::resolve_language(project_root);
    let templates_dir = straymark_dir.join("templates");
    let template_path = utils::resolve_localized_path(&templates_dir, "TEMPLATE-TDE.md", &lang);
    let template = std::fs::read_to_string(&template_path).with_context(|| {
        format!(
            "TDE template not found at {}. Run `straymark repair` to restore framework files.",
            template_path.display()
        )
    })?;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let tde_dir = straymark_dir.join("06-evolution").join("technical-debt");
    let seq = next_tde_sequence(&tde_dir, &today);
    let tde_id = format!("TDE-{}-{}", today, seq);

    let mut content = template
        .replace("id: TDE-YYYY-MM-DD-NNN", &format!("id: {}", tde_id))
        .replace("YYYY-MM-DD-NNN", &format!("{}-{}", today, seq))
        .replace("YYYY-MM-DD", &today)
        .replace("[Technical debt title]", title)
        .replace("[Título de la deuda técnica]", title)
        .replace("[Technical Debt Title]", title)
        .replace("[agent-name-v1.0]", "straymark-cli-promote")
        .replace("[nombre-agente-v1.0]", "straymark-cli-promote");

    // Traceability: the template ships `promoted_from_followup: null` from
    // fw-4.13.1; replace the value (tolerating the trailing comment).
    if let Some(idx) = content.find("promoted_from_followup:") {
        let line_end = content[idx..]
            .find('\n')
            .map(|i| idx + i)
            .unwrap_or(content.len());
        content.replace_range(
            idx..line_end,
            &format!("promoted_from_followup: {}", entry.fu_id),
        );
    } else if let Some(idx) = content.find("\n---\n") {
        // Older template without the field — insert it before the closing ---.
        content.insert_str(idx, &format!("\npromoted_from_followup: {}", entry.fu_id));
    }

    // Seed the Summary with the FU context when the placeholder is present.
    let origin_note = format!(
        "{} (origin: {})",
        entry.description,
        entry.origin.as_deref().unwrap_or("follow-ups backlog")
    );
    content = content
        .replace(
            "[Brief description of the identified technical debt]",
            &origin_note,
        )
        .replace(
            "[Descripción breve de la deuda técnica identificada]",
            &origin_note,
        );

    let slug = slugify(title);
    let filename = format!("TDE-{}-{}-{}.md", today, seq, slug);
    utils::ensure_dir(&tde_dir)?;
    let filepath = tde_dir.join(filename);
    std::fs::write(&filepath, content)?;
    Ok((tde_id, filepath))
}

fn next_tde_sequence(tde_dir: &Path, today: &str) -> String {
    let prefix = format!("TDE-{}-", today);
    let mut max_seq = 0u32;
    if let Ok(entries) = std::fs::read_dir(tde_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_str().unwrap_or("");
            if let Some(rest) = name.strip_prefix(&prefix) {
                let head: String = rest.chars().take(3).collect();
                if head.chars().count() == 3 {
                    if let Ok(n) = head.parse::<u32>() {
                        max_seq = max_seq.max(n);
                    }
                }
            }
        }
    }
    format!("{:03}", max_seq + 1)
}

fn slugify(title: &str) -> String {
    let lower = title.to_lowercase();
    let parts: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let slug = parts.join("-");
    if slug.chars().count() > 50 {
        let truncated: String = slug.chars().take(50).collect();
        truncated.trim_end_matches('-').to_string()
    } else {
        slug
    }
}
