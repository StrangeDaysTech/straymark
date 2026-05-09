//! `straymark charter status [CHARTER-ID]` — show detail for a Charter.
//!
//! With an ID: prints frontmatter, file location, body section list, and
//! placeholders for telemetry / drift-check (Phase 2 features).
//! Without an ID: prints the last 5 Charters by NN descending.

use anyhow::{anyhow, Result};
use colored::Colorize;

use crate::charter::{
    self, display_origin, display_title, Charter, CharterStatus,
};
use crate::utils;

/// Number of Charters shown when `charter status` is called without an ID.
const RECENT_LIMIT: usize = 5;

pub fn run(path: &str, charter_id: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let (charters, errors) = charter::discover_and_parse(project_root);

    for (path, err) in &errors {
        utils::warn(&format!(
            "Skipping {}: {}",
            path.strip_prefix(project_root).unwrap_or(path).display(),
            err
        ));
    }

    match charter_id {
        Some(id) => {
            let c = charter::find_by_id(&charters, id).ok_or_else(|| {
                anyhow!(
                    "Charter not found: '{}'. Run `straymark charter list` to see available Charters.",
                    id
                )
            })?;
            print_detail(c, project_root);
        }
        None => {
            if charters.is_empty() && errors.is_empty() {
                println!("No Charters in this project. Run `straymark charter new` to create one.");
                return Ok(());
            }
            print_recent(&charters);
        }
    }

    Ok(())
}

fn print_detail(c: &Charter, project_root: &std::path::Path) {
    let rel_path = c
        .path
        .strip_prefix(project_root)
        .unwrap_or(&c.path)
        .display();

    println!();
    println!("{} {}", "Charter:".bold(), c.frontmatter.charter_id);
    println!();
    println!(
        "  {}     {}",
        "Status:".bold(),
        colorize_status(c.frontmatter.status, c.frontmatter.status.as_str()),
    );
    println!(
        "  {}     {}",
        "Effort:".bold(),
        c.frontmatter.effort_estimate.as_str(),
    );
    println!("  {}    {}", "Trigger:".bold(), c.frontmatter.trigger);
    println!("  {}     {}", "Origin:".bold(), display_origin(&c.frontmatter));
    println!("  {}      {}", "Title:".bold(), display_title(c));
    println!("  {}       {}", "File:".bold(), rel_path);

    let sections = body_section_headings(&c.body);
    if sections.is_empty() {
        println!("  {}   (no `## ` headings detected)", "Sections:".bold());
    } else {
        println!("  {}   {}", "Sections:".bold(), sections.join(", "));
    }

    println!();
    println!("  {}", "Phase 2 features (not yet available):".dimmed());
    println!("    {}", "telemetry — straymark charter close (planned cli-3.7.0)".dimmed());
    println!("    {}", "drift-check — straymark charter drift (planned cli-3.7.0)".dimmed());
    println!();
}

fn print_recent(charters: &[Charter]) {
    // Sort by NN descending (most recent first).
    let mut indexed: Vec<&Charter> = charters.iter().collect();
    indexed.sort_by(|a, b| {
        nn_of(b)
            .cmp(&nn_of(a))
            .then_with(|| a.frontmatter.charter_id.cmp(&b.frontmatter.charter_id))
    });
    let shown = indexed.iter().take(RECENT_LIMIT).copied().collect::<Vec<_>>();

    println!();
    println!(
        "{} ({})",
        "Most recent Charters".bold(),
        if charters.len() <= RECENT_LIMIT {
            format!("{}", charters.len())
        } else {
            format!("last {} of {}", shown.len(), charters.len())
        }
    );
    println!();

    for c in &shown {
        let nn_str = format!("{:0>2}", nn_of(c));
        let status_text = c.frontmatter.status.as_str();
        let status_padded = utils::pad_right_visual(status_text, "in-progress".len());
        println!(
            "  {}  {}  {}  {}",
            nn_str,
            colorize_status(c.frontmatter.status, &status_padded),
            utils::pad_right_visual(c.frontmatter.effort_estimate.as_str(), 2),
            display_title(c),
        );
    }

    println!();
    println!(
        "  Run {} for detail, or {} to see all.",
        "straymark charter status CHARTER-NN".bold(),
        "straymark charter list".bold()
    );
    println!();
}

fn body_section_headings(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            // Only top-level section headings (## ...). Skip ### and below
            // (those are sub-sections of Verification etc., noise here).
            if let Some(rest) = trimmed.strip_prefix("## ") {
                Some(rest.trim().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Numeric NN extracted from a Charter's ID. Returns 0 for malformed IDs so
/// they sort before everything else (visually flagging the issue).
fn nn_of(c: &Charter) -> u32 {
    let id = &c.frontmatter.charter_id;
    let after_prefix = id.strip_prefix("CHARTER-").unwrap_or(id);
    let digits: String = after_prefix
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse::<u32>().unwrap_or(0)
}

fn colorize_status(status: CharterStatus, text: &str) -> colored::ColoredString {
    match status {
        CharterStatus::Declared => text.normal(),
        CharterStatus::InProgress => text.yellow(),
        CharterStatus::Closed => text.green(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charter::{CharterFrontmatter, EffortEstimate};
    use std::path::PathBuf;

    fn make(id: &str, body: &str) -> Charter {
        Charter {
            path: PathBuf::from(format!("docs/charters/{}.md", id.to_lowercase())),
            frontmatter: CharterFrontmatter {
                charter_id: id.to_string(),
                status: CharterStatus::Declared,
                effort_estimate: EffortEstimate::M,
                trigger: "x".to_string(),
                originating_ailogs: None,
                originating_spec: None,
            },
            body: body.to_string(),
        }
    }

    #[test]
    fn body_section_headings_extracts_top_level_only() {
        let body = "# Charter: Title\n\n## Context\n\nfoo\n\n### Local checks\n\n## Scope\n\nbar\n";
        let sections = body_section_headings(body);
        assert_eq!(sections, vec!["Context", "Scope"]);
    }

    #[test]
    fn body_section_headings_empty_body() {
        assert!(body_section_headings("").is_empty());
        assert!(body_section_headings("just text\n").is_empty());
    }

    #[test]
    fn nn_of_extracts_correctly() {
        let c1 = make("CHARTER-01-foo", "");
        assert_eq!(nn_of(&c1), 1);
        let c10 = make("CHARTER-10-bar", "");
        assert_eq!(nn_of(&c10), 10);
        let c100 = make("CHARTER-100-baz", "");
        assert_eq!(nn_of(&c100), 100);
        let bad = make("CHARTER-abc", "");
        assert_eq!(nn_of(&bad), 0);
    }
}
