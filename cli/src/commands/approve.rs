//! `straymark approve` — record a formal human approval on a `review_required`
//! document. Implements the closure signal canonized in DOCUMENTATION-POLICY
//! §3.5 (added in fw-4.6.0): writes the `reviewed_by` / `reviewed_at` /
//! `review_outcome` frontmatter fields and appends the canonical `## Approval`
//! body section in one atomic edit.
//!
//! When the outcome is `approved`, the document's `status` field is also
//! transitioned to `accepted` (cli-3.40.0, #378): a document that is approved
//! is, by definition, accepted. This removes the contradictory `status: draft`
//! + `review_outcome: approved` state that previously required a manual edit.
//!
//! Two operating modes:
//! - **Flag-driven** (CI / scripted): `straymark approve <doc-id> --outcome
//!   approved --reviewer <id> [--notes "..."] [--at YYYY-MM-DD]`. Fully
//!   non-interactive when all required flags are present.
//! - **Interactive fallback**: when `--outcome` or `--reviewer` is missing
//!   and stdin is a TTY, prompt for them. Bails on non-TTY missing flags.

use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use colored::Colorize;
use std::path::{Path, PathBuf};

use straymark_core::document;
use crate::prompts;
use crate::utils;

const VALID_OUTCOMES: &[&str] = &["approved", "revisions_requested", "rejected"];

pub fn run(
    path: &str,
    doc_id: &str,
    outcome: Option<&str>,
    reviewer: Option<&str>,
    at: Option<&str>,
    notes: Option<&str>,
    force: bool,
    quiet: bool,
) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;
    let straymark_dir = project_root.join(".straymark");

    // Locate the document.
    let doc_path = find_document_by_id(&straymark_dir, doc_id)?;

    // F4 (cli-3.7.1): detect existing approval before resolving any prompts.
    if let Ok(parsed) = document::parse_document(&doc_path) {
        if let (Some(prev_outcome), Some(prev_reviewer)) = (
            parsed.frontmatter.review_outcome.as_deref(),
            parsed.frontmatter.reviewed_by.as_deref(),
        ) {
            if !force {
                if !quiet {
                    let prev_at = parsed
                        .frontmatter
                        .reviewed_at
                        .as_deref()
                        .unwrap_or("(unknown date)");
                    println!(
                        "{} {} already has approval on file ({} by `{}` on {}).",
                        "✓".green().bold(),
                        doc_id.bold(),
                        prev_outcome.bold(),
                        prev_reviewer,
                        prev_at
                    );
                    println!(
                        "  Pass {} to overwrite or amend (e.g., revisions_requested → approved).",
                        "--force".cyan()
                    );
                }
                return Ok(());
            }
        }
    }

    // Resolve outcome + reviewer (flags or prompts).
    let outcome = resolve_outcome(outcome)?;
    let reviewer = resolve_reviewer(reviewer)?;
    let at = at
        .map(|s| s.to_string())
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

    // Validate outcome is in the allowed enum.
    if !VALID_OUTCOMES.contains(&outcome.as_str()) {
        bail!(
            "invalid --outcome '{}': must be one of {}",
            outcome,
            VALID_OUTCOMES.join(" | ")
        );
    }

    // Read + edit the document.
    let raw = std::fs::read_to_string(&doc_path)
        .with_context(|| format!("Failed to read {}", doc_path.display()))?;

    // F5 (cli-3.8.0): warnings about review_required:false and risk_level.
    // The high/critical risk warning is INTENTIONALLY not silenceable by
    // --quiet — bulk-approving high-risk documents without seeing it is
    // exactly the failure mode --quiet would otherwise enable. The
    // `review_required: false` info-warning IS silenceable since it just
    // documents an unusual-but-allowed retroactive sign-off pattern.
    if let Ok(parsed) = document::parse_document(&doc_path) {
        if !parsed.frontmatter.review_required.unwrap_or(false) && !quiet {
            eprintln!(
                "{} document does not have `review_required: true`; recording approval anyway.",
                "warning:".yellow().bold()
            );
        }
        if let Some(risk) = parsed.frontmatter.risk_level.as_deref() {
            if matches!(risk, "high" | "critical") {
                // Always surface — even under --quiet.
                eprintln!(
                    "{} {} has `risk_level: {}` — ensure thorough human review (per AGENT-RULES.md §4).",
                    "WARNING:".red().bold(),
                    doc_id.bold(),
                    risk.bold()
                );
            }
        }
    }

    let updated = apply_approval(&raw, &reviewer, &at, &outcome, notes)?;
    std::fs::write(&doc_path, &updated)
        .with_context(|| format!("Failed to write {}", doc_path.display()))?;

    if !quiet {
        println!(
            "{} {} marked as {}.",
            "✔".green().bold(),
            doc_id.bold(),
            outcome.bold()
        );
        println!("  Reviewer: {}", reviewer);
        println!("  Date:     {}", at);
        println!("  File:     {}", doc_path.display());
    }
    Ok(())
}

/// Search `.straymark/` for a document whose filename starts with the given ID.
/// The ID may be the bare prefix (`AIDEC-2026-05-02-001`) or include the slug
/// (`AIDEC-2026-05-02-001-foo`); both forms resolve to the same file.
fn find_document_by_id(straymark_dir: &Path, doc_id: &str) -> Result<PathBuf> {
    let docs = document::discover_documents(straymark_dir);
    // Strip optional slug to get canonical prefix (TYPE-YYYY-MM-DD-NNN).
    let prefix = canonical_prefix(doc_id);
    for path in docs {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(&prefix) {
                return Ok(path);
            }
        }
    }
    bail!(
        "document {} not found under .straymark/. Run `straymark validate` to see all documents.",
        doc_id
    )
}

fn canonical_prefix(doc_id: &str) -> String {
    // Take the first 5 hyphen-separated segments: TYPE, YYYY, MM, DD, NNN.
    doc_id
        .splitn(6, '-')
        .take(5)
        .collect::<Vec<_>>()
        .join("-")
}

fn resolve_outcome(flag: Option<&str>) -> Result<String> {
    if let Some(o) = flag {
        return Ok(o.to_string());
    }
    prompts::require_interactive()?;
    prompts::prompt_enum(
        "Review outcome",
        &["approved", "revisions_requested", "rejected"],
        0,
    )
}

fn resolve_reviewer(flag: Option<&str>) -> Result<String> {
    if let Some(r) = flag {
        if r.trim().is_empty() {
            bail!("--reviewer cannot be empty");
        }
        return Ok(r.to_string());
    }
    prompts::require_interactive()?;
    prompts::prompt_string("Reviewer (email | github-handle | DID)", None, false)
}

/// Apply the approval edit to the raw document text. Returns the updated text.
///
/// Mutations:
/// - Frontmatter: insert/update the three approval fields (after
///   `review_required:` or, lacking that line, before the closing `---`).
/// - Frontmatter: when outcome is `approved`, transition `status` to
///   `accepted` (#378).
/// - Body: append a `## Approval` section near the end (before any final
///   HTML comment marker, e.g., `<!-- Template: StrayMark | ... -->`).
fn apply_approval(
    raw: &str,
    reviewer: &str,
    at: &str,
    outcome: &str,
    notes: Option<&str>,
) -> Result<String> {
    let updated_fm = update_frontmatter(raw, reviewer, at, outcome)?;
    let with_section = append_body_section(&updated_fm, reviewer, at, outcome, notes);
    Ok(with_section)
}

/// Replace existing approval frontmatter fields, or insert them if absent.
/// We operate on text rather than re-serializing YAML to preserve comments,
/// field order, and any project-specific extensions the framework doesn't
/// know about.
///
/// When `outcome` is `"approved"`, also transitions `status` to `accepted`
/// (#378): an approved document is accepted by definition.
fn update_frontmatter(raw: &str, reviewer: &str, at: &str, outcome: &str) -> Result<String> {
    let (frontmatter, rest) = split_frontmatter(raw)?;
    let new_fm = upsert_approval_fields(&frontmatter, reviewer, at, outcome);
    let new_fm = if outcome == "approved" {
        transition_status_to_accepted(&new_fm)
    } else {
        new_fm
    };
    Ok(format!("---\n{}---{}", new_fm, rest))
}

/// Returns (frontmatter_body, rest_of_document) from a `---\n…\n---\n…` file.
fn split_frontmatter(raw: &str) -> Result<(String, String)> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    let after_first = trimmed
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow!("document does not start with `---` frontmatter delimiter"))?;
    let end = after_first
        .find("\n---")
        .ok_or_else(|| anyhow!("frontmatter is not terminated by `---`"))?;
    let fm_body = &after_first[..end + 1]; // include trailing newline
    let rest_start = end + 4; // skip "\n---"
    let rest = if rest_start < after_first.len() {
        after_first[rest_start..].to_string()
    } else {
        String::new()
    };
    Ok((fm_body.to_string(), rest))
}

/// Upsert the three approval fields into a frontmatter body. If a field is
/// already present (uncommented), replace its value. Otherwise, insert all
/// three after the `review_required:` line, or at the end of the frontmatter
/// if `review_required:` is absent.
fn upsert_approval_fields(fm: &str, reviewer: &str, at: &str, outcome: &str) -> String {
    let fields = [
        ("reviewed_by", reviewer.to_string()),
        ("reviewed_at", at.to_string()),
        ("review_outcome", outcome.to_string()),
    ];

    let mut lines: Vec<String> = fm.lines().map(|s| s.to_string()).collect();

    // Pass 1: replace any existing uncommented field.
    let mut applied = std::collections::HashMap::new();
    for line in lines.iter_mut() {
        for (key, value) in &fields {
            if applied.contains_key(*key) {
                continue;
            }
            if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
                let _ = rest;
                *line = format!("{}: {}", key, yaml_scalar(value));
                applied.insert(*key, ());
            }
        }
    }

    // Pass 2: insert missing fields. Insertion point: after `review_required:`
    // (commented or not), or at the end of the frontmatter.
    let missing: Vec<(&str, String)> = fields
        .iter()
        .filter(|(k, _)| !applied.contains_key(*k))
        .map(|(k, v)| (*k, v.clone()))
        .collect();

    if !missing.is_empty() {
        let insertion_idx = lines
            .iter()
            .position(|l| l.trim_start().starts_with("review_required:"))
            .map(|i| i + 1);
        let new_lines: Vec<String> = missing
            .into_iter()
            .map(|(k, v)| format!("{}: {}", k, yaml_scalar(&v)))
            .collect();

        match insertion_idx {
            Some(i) => {
                for (offset, l) in new_lines.into_iter().enumerate() {
                    lines.insert(i + offset, l);
                }
            }
            None => {
                lines.extend(new_lines);
            }
        }
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Transition the `status:` field to `accepted` in a frontmatter body.
/// If `status:` is present, replace its value. If absent, insert it after
/// `id:` (or at the top if `id:` is also absent).
fn transition_status_to_accepted(fm: &str) -> String {
    let mut lines: Vec<String> = fm.lines().map(|s| s.to_string()).collect();
    let mut found = false;
    for line in lines.iter_mut() {
        if line.starts_with("status:") {
            *line = "status: accepted".to_string();
            found = true;
            break;
        }
    }
    if !found {
        // Insert after `id:` line, or at the beginning.
        let idx = lines
            .iter()
            .position(|l| l.starts_with("id:"))
            .map(|i| i + 1)
            .unwrap_or(0);
        lines.insert(idx, "status: accepted".to_string());
    }
    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Render a string as a YAML scalar. Quote if the value contains characters
/// that YAML would otherwise misinterpret. Conservative: quote when in doubt.
fn yaml_scalar(value: &str) -> String {
    let needs_quote = value.is_empty()
        || value.contains(':')
        || value.contains('#')
        || value.contains('\n')
        || value.contains('"')
        || value.contains('\'')
        || value.starts_with(' ')
        || value.ends_with(' ');
    if needs_quote {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

/// Append a `## Approval` section to the body. Inserts before any trailing
/// HTML comment marker (e.g., `<!-- Template: StrayMark | ... -->`) so the
/// canonical signature stays at the very bottom.
fn append_body_section(
    raw: &str,
    reviewer: &str,
    at: &str,
    outcome: &str,
    notes: Option<&str>,
) -> String {
    let outcome_label = match outcome {
        "approved" => "Approved",
        "revisions_requested" => "Revisions requested",
        "rejected" => "Rejected",
        other => other,
    };
    let mut block = format!("\n## Approval\n\n**{}**: {} by `{}`.\n", outcome_label, at, reviewer);
    if let Some(n) = notes {
        let trimmed = n.trim();
        if !trimmed.is_empty() {
            block.push('\n');
            block.push_str(trimmed);
            block.push('\n');
        }
    }

    // Find a trailing HTML comment marker (the template signature) and
    // insert the block immediately before it. If absent, append at end.
    let needle = "<!-- Template: StrayMark";
    if let Some(pos) = raw.rfind(needle) {
        // Ensure we land at the start of the line containing the marker.
        let line_start = raw[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let mut out = String::with_capacity(raw.len() + block.len());
        out.push_str(&raw[..line_start]);
        out.push_str(&block);
        out.push('\n');
        out.push_str(&raw[line_start..]);
        return out;
    }

    let mut out = raw.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&block);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_prefix_strips_slug() {
        assert_eq!(canonical_prefix("AIDEC-2026-05-02-001"), "AIDEC-2026-05-02-001");
        assert_eq!(
            canonical_prefix("AIDEC-2026-05-02-001-foo-bar"),
            "AIDEC-2026-05-02-001"
        );
    }

    #[test]
    fn yaml_scalar_quotes_when_needed() {
        assert_eq!(yaml_scalar("plain"), "plain");
        assert_eq!(yaml_scalar("user@example.com"), "user@example.com");
        assert_eq!(yaml_scalar("has: colon"), "\"has: colon\"");
        assert_eq!(yaml_scalar("needs # comment"), "\"needs # comment\"");
        assert_eq!(yaml_scalar(""), "\"\"");
    }

    #[test]
    fn upsert_inserts_after_review_required() {
        let fm = "id: AIDEC-2026-05-02-001\nstatus: accepted\nreview_required: true\nrisk_level: medium\n";
        let updated = upsert_approval_fields(fm, "pepe@example.com", "2026-05-02", "approved");
        let expected = "id: AIDEC-2026-05-02-001\nstatus: accepted\nreview_required: true\nreviewed_by: pepe@example.com\nreviewed_at: 2026-05-02\nreview_outcome: approved\nrisk_level: medium\n";
        assert_eq!(updated, expected);
    }

    #[test]
    fn upsert_replaces_existing_fields() {
        let fm = "review_required: true\nreviewed_by: stale@old.com\nreviewed_at: 2026-04-01\nreview_outcome: revisions_requested\n";
        let updated = upsert_approval_fields(fm, "fresh@new.com", "2026-05-02", "approved");
        assert!(updated.contains("reviewed_by: fresh@new.com"));
        assert!(updated.contains("reviewed_at: 2026-05-02"));
        assert!(updated.contains("review_outcome: approved"));
        assert!(!updated.contains("stale@old.com"));
        assert!(!updated.contains("revisions_requested"));
    }

    #[test]
    fn upsert_appends_when_review_required_absent() {
        let fm = "id: AIDEC-x\nstatus: accepted\n";
        let updated = upsert_approval_fields(fm, "pepe", "2026-05-02", "approved");
        assert!(updated.contains("reviewed_by: pepe"));
        assert!(updated.ends_with("review_outcome: approved\n"));
    }

    #[test]
    fn append_body_section_inserts_before_template_marker() {
        let body = "\n# Body\n\nContent.\n\n---\n\n<!-- Template: StrayMark | https://strangedays.tech -->\n";
        let out = append_body_section(body, "pepe", "2026-05-02", "approved", None);
        let approval_pos = out.find("## Approval").unwrap();
        let marker_pos = out.find("<!-- Template: StrayMark").unwrap();
        assert!(approval_pos < marker_pos, "Approval should land before signature");
        assert!(out.contains("**Approved**: 2026-05-02 by `pepe`."));
    }

    #[test]
    fn append_body_section_includes_notes_when_provided() {
        let body = "# Body\n";
        let out = append_body_section(
            body,
            "pepe",
            "2026-05-02",
            "approved",
            Some("Looks good. Ship it."),
        );
        assert!(out.contains("Looks good. Ship it."));
    }

    #[test]
    fn append_body_section_handles_revisions_requested() {
        let body = "# Body\n";
        let out = append_body_section(body, "pepe", "2026-05-02", "revisions_requested", None);
        assert!(out.contains("**Revisions requested**: 2026-05-02"));
    }

    #[test]
    fn split_frontmatter_round_trips() {
        let raw = "---\nid: x\nstatus: accepted\n---\n\n# Body\n";
        let (fm, rest) = split_frontmatter(raw).unwrap();
        assert_eq!(fm, "id: x\nstatus: accepted\n");
        assert_eq!(rest, "\n\n# Body\n");
    }

    #[test]
    fn apply_approval_full_pipeline() {
        let raw = "---\nid: AIDEC-2026-05-02-001\ntitle: Test\nstatus: accepted\nreview_required: true\nrisk_level: medium\n---\n\n# AIDEC\n\nBody.\n";
        let out = apply_approval(raw, "pepe@example.com", "2026-05-02", "approved", None).unwrap();
        // Frontmatter has the three new fields.
        assert!(out.contains("reviewed_by: pepe@example.com"));
        assert!(out.contains("reviewed_at: 2026-05-02"));
        assert!(out.contains("review_outcome: approved"));
        // Body has the section.
        assert!(out.contains("## Approval"));
        assert!(out.contains("**Approved**: 2026-05-02 by `pepe@example.com`."));
        // Round-trips through YAML parser.
        let parsed: serde_yaml::Value = serde_yaml::from_str(
            out.split("---\n").nth(1).unwrap().split("\n---").next().unwrap(),
        )
        .unwrap();
        assert_eq!(
            parsed.get("review_outcome").and_then(|v| v.as_str()),
            Some("approved")
        );
    }

    // --- #378: status transition on approval ---

    #[test]
    fn transition_status_replaces_draft() {
        let fm = "id: AILOG-2026-07-28-001\nstatus: draft\ntitle: Test\n";
        let out = transition_status_to_accepted(fm);
        assert!(out.contains("status: accepted"));
        assert!(!out.contains("status: draft"));
    }

    #[test]
    fn transition_status_replaces_review() {
        let fm = "id: AILOG-x\nstatus: review\n";
        let out = transition_status_to_accepted(fm);
        assert!(out.contains("status: accepted"));
        assert!(!out.contains("status: review"));
    }

    #[test]
    fn transition_status_inserts_when_absent() {
        let fm = "id: AILOG-x\ntitle: No status field\n";
        let out = transition_status_to_accepted(fm);
        assert!(out.contains("status: accepted"));
        // Inserted after id: line.
        let lines: Vec<&str> = out.lines().collect();
        let id_pos = lines.iter().position(|l| l.starts_with("id:")).unwrap();
        let status_pos = lines.iter().position(|l| l.starts_with("status:")).unwrap();
        assert_eq!(status_pos, id_pos + 1);
    }

    #[test]
    fn apply_approval_transitions_draft_to_accepted() {
        let raw = "---\nid: AILOG-2026-07-28-001\ntitle: Test\nstatus: draft\nreview_required: true\n---\n\n# Body\n";
        let out = apply_approval(raw, "reviewer@x.com", "2026-07-29", "approved", None).unwrap();
        assert!(out.contains("status: accepted"));
        assert!(!out.contains("status: draft"));
        assert!(out.contains("review_outcome: approved"));
    }

    #[test]
    fn apply_approval_revisions_requested_does_not_change_status() {
        let raw = "---\nid: AILOG-2026-07-28-001\ntitle: Test\nstatus: draft\nreview_required: true\n---\n\n# Body\n";
        let out = apply_approval(raw, "reviewer@x.com", "2026-07-29", "revisions_requested", None).unwrap();
        // Status must remain draft — only `approved` transitions.
        assert!(out.contains("status: draft"));
        assert!(out.contains("review_outcome: revisions_requested"));
    }
}
