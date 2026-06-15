//! `straymark charter list` — enumerate Charters with optional status / origin filter.
//!
//! Output is a plain-text table to stdout. Files that fail to parse are
//! reported as a warning to stderr but do not fail the command (Unix-style:
//! list what you can, surface what you can't).

use anyhow::{anyhow, Result};
use colored::Colorize;

use straymark_core::charter::{
    self, display_origin, display_title, origin_kind, Charter, CharterStatus,
};
use crate::utils;

pub fn run(path: &str, status_filter: &str, origin_filter: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let (charters, errors) = charter::discover_and_parse(project_root);

    // Surface parse errors as warnings (non-fatal).
    for (path, err) in &errors {
        utils::warn(&format!(
            "Skipping {}: {}",
            path.strip_prefix(project_root).unwrap_or(path).display(),
            err
        ));
    }

    let filtered = filter(&charters, status_filter, origin_filter);

    if filtered.is_empty() {
        if charters.is_empty() && errors.is_empty() {
            println!("No Charters in this project. Run `straymark charter new` to create one.");
        } else {
            println!("No Charters match the given filter.");
        }
        return Ok(());
    }

    print_table(&filtered);
    Ok(())
}

/// Apply the status and origin filters to a slice of Charters. Returns
/// borrowed references in the original order.
fn filter<'a>(
    charters: &'a [Charter],
    status: &str,
    origin: Option<&str>,
) -> Vec<&'a Charter> {
    charters
        .iter()
        .filter(|c| match status {
            "all" => true,
            "declared" => c.frontmatter.status == CharterStatus::Declared,
            "in-progress" => c.frontmatter.status == CharterStatus::InProgress,
            "closed" => c.frontmatter.status == CharterStatus::Closed,
            _ => true,
        })
        .filter(|c| match origin {
            None | Some("any") => true,
            Some("ailog") => origin_kind(&c.frontmatter) == "ailog",
            Some("spec") => origin_kind(&c.frontmatter) == "spec",
            Some(_) => true,
        })
        .collect()
}

/// Print the table to stdout. Columns: NN, STATUS, EFFORT, ORIGIN, TITLE.
/// Widths are computed from the data so the layout stays tight on small
/// projects and adapts to longer fields when present.
fn print_table(charters: &[&Charter]) {
    // Compute column widths.
    let nn_w = charters
        .iter()
        .map(|c| nn_display(c).len())
        .max()
        .unwrap_or(2)
        .max(2);
    let status_w = charters
        .iter()
        .map(|c| c.frontmatter.status.as_str().len())
        .max()
        .unwrap_or(0)
        .max("STATUS".len());
    let effort_w = "EFFORT".len();
    // Origin column is capped to keep the title readable; longer origins are
    // truncated with an ellipsis. 32 columns is enough for AILOG IDs and
    // typical specs/<n>-feature/spec.md paths.
    const ORIGIN_MAX: usize = 32;
    let origin_w = charters
        .iter()
        .map(|c| utils::visual_width(&display_origin(&c.frontmatter)).min(ORIGIN_MAX))
        .max()
        .unwrap_or(0)
        .max("ORIGIN".len());

    // Header.
    println!(
        "  {}  {}  {}  {}  {}",
        utils::pad_right_visual("NN", nn_w).bold(),
        utils::pad_right_visual("STATUS", status_w).bold(),
        utils::pad_right_visual("EFFORT", effort_w).bold(),
        utils::pad_right_visual("ORIGIN", origin_w).bold(),
        "TITLE".bold(),
    );

    for c in charters {
        let nn = nn_display(c);
        let status_text = c.frontmatter.status.as_str();
        let status_colored = colorize_status(c.frontmatter.status, status_text);
        let origin = utils::truncate_visual(&display_origin(&c.frontmatter), ORIGIN_MAX);
        let title = display_title(c);

        // We pad the raw status string for column alignment, then color it.
        // Padding before coloring is critical — color escape codes don't
        // contribute to visual width, so padding-after-coloring would
        // miscalculate.
        let padded_status = utils::pad_right_visual(status_text, status_w);
        let _ = status_colored; // keep the helper available; current style
                                // colors only for closed/in-progress states
                                // when terminals support it.

        println!(
            "  {}  {}  {}  {}  {}",
            utils::pad_right_visual(&nn, nn_w),
            colorize_status(c.frontmatter.status, &padded_status),
            utils::pad_right_visual(c.frontmatter.effort_estimate.as_str(), effort_w),
            utils::pad_right_visual(&origin, origin_w),
            title,
        );
    }
}

/// Two-digit (or longer if needed) zero-padded NN extracted from the charter_id.
/// Returns "??" when the charter_id doesn't contain a parseable NN, which
/// indicates a malformed Charter — surface as visual cue.
fn nn_display(c: &Charter) -> String {
    let id = &c.frontmatter.charter_id;
    let after_prefix = id.strip_prefix("CHARTER-").unwrap_or(id);
    let digits: String = after_prefix
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        "??".to_string()
    } else {
        // Preserve at least 2 chars width.
        format!("{:0>2}", digits)
    }
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
    use straymark_core::charter::{CharterFrontmatter, EffortEstimate};
    use std::path::PathBuf;

    fn make(id: &str, status: CharterStatus, ailog: Option<Vec<String>>, spec: Option<String>) -> Charter {
        Charter {
            path: PathBuf::from(format!(".straymark/charters/{}.md", id.to_lowercase())),
            frontmatter: CharterFrontmatter {
                charter_id: id.to_string(),
                status,
                effort_estimate: EffortEstimate::M,
                trigger: "x".to_string(),
                originating_ailogs: ailog,
                originating_spec: spec,
            },
            body: String::new(),
        }
    }

    #[test]
    fn filter_status_all_returns_everything() {
        let charters = vec![
            make("CHARTER-01-a", CharterStatus::Declared, None, None),
            make("CHARTER-02-b", CharterStatus::InProgress, None, None),
            make("CHARTER-03-c", CharterStatus::Closed, None, None),
        ];
        let filtered = filter(&charters, "all", None);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn filter_status_declared_only() {
        let charters = vec![
            make("CHARTER-01-a", CharterStatus::Declared, None, None),
            make("CHARTER-02-b", CharterStatus::InProgress, None, None),
            make("CHARTER-03-c", CharterStatus::Declared, None, None),
        ];
        let filtered = filter(&charters, "declared", None);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|c| c.frontmatter.status == CharterStatus::Declared));
    }

    #[test]
    fn filter_origin_ailog_only() {
        let charters = vec![
            make(
                "CHARTER-01-a",
                CharterStatus::Declared,
                Some(vec!["AILOG-2026-04-28-021".into()]),
                None,
            ),
            make(
                "CHARTER-02-b",
                CharterStatus::Declared,
                None,
                Some("specs/001/spec.md".into()),
            ),
            make("CHARTER-03-c", CharterStatus::Declared, None, None),
        ];
        let filtered = filter(&charters, "all", Some("ailog"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].frontmatter.charter_id, "CHARTER-01-a");
    }

    #[test]
    fn filter_origin_spec_only() {
        let charters = vec![
            make(
                "CHARTER-01-a",
                CharterStatus::Declared,
                Some(vec!["AILOG-2026-04-28-021".into()]),
                None,
            ),
            make(
                "CHARTER-02-b",
                CharterStatus::Declared,
                None,
                Some("specs/001/spec.md".into()),
            ),
        ];
        let filtered = filter(&charters, "all", Some("spec"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].frontmatter.charter_id, "CHARTER-02-b");
    }

    #[test]
    fn filter_origin_any_matches_no_origin_too() {
        let charters = vec![
            make("CHARTER-01-a", CharterStatus::Declared, None, None),
            make(
                "CHARTER-02-b",
                CharterStatus::Declared,
                Some(vec!["AILOG-2026-04-28-021".into()]),
                None,
            ),
        ];
        let filtered = filter(&charters, "all", Some("any"));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn filter_combines_status_and_origin() {
        let charters = vec![
            make(
                "CHARTER-01-a",
                CharterStatus::Closed,
                Some(vec!["AILOG-2026-04-28-021".into()]),
                None,
            ),
            make(
                "CHARTER-02-b",
                CharterStatus::Declared,
                Some(vec!["AILOG-2026-04-28-022".into()]),
                None,
            ),
            make(
                "CHARTER-03-c",
                CharterStatus::Closed,
                None,
                Some("specs/001/spec.md".into()),
            ),
        ];
        let filtered = filter(&charters, "closed", Some("ailog"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].frontmatter.charter_id, "CHARTER-01-a");
    }

    #[test]
    fn nn_display_zero_pads_single_digits() {
        let c = make("CHARTER-01-a", CharterStatus::Declared, None, None);
        assert_eq!(nn_display(&c), "01");
    }

    #[test]
    fn nn_display_preserves_three_or_more_digits() {
        let c = make("CHARTER-100-x", CharterStatus::Declared, None, None);
        assert_eq!(nn_display(&c), "100");
    }

    #[test]
    fn nn_display_returns_question_marks_when_id_is_malformed() {
        let c = make("CHARTER-foo", CharterStatus::Declared, None, None);
        assert_eq!(nn_display(&c), "??");
    }
}
