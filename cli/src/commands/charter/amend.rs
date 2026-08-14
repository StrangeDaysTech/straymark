//! `straymark charter amend <id>` — scaffold a post-close Batch N.4 amendment.
//!
//! Pattern 2 in `dist/.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md`:
//! external audit findings (or other bounded triggers) arrive after the
//! Charter has been marked `status: closed`, the fix surface is small enough
//! that a new Charter would be overkill, and the operator wants the
//! remediation to ride on the original execute branch.
//!
//! This command does NOT touch git. It prepares three artifacts so the
//! operator can commit at their leisure:
//!
//! 1. A new AILOG stub in `.straymark/07-ai-audit/agent-logs/` with the
//!    convention `risk_level: high`, `review_required: true`, and an
//!    `amends:` field linking back to the most-recent originating AILOG of
//!    the Charter.
//! 2. A `## Historical correction (YYYY-MM-DD)` subsection appended to the
//!    located previous AILOG, pointing forward to the new id.
//! 3. A rendered `post_close_amendment:` YAML block, either printed to
//!    stdout for manual paste or merged into the Charter's `.telemetry.yaml`
//!    via `--merge-into`.
//!
//! fw-4.16.0+ (Issue #156).

use anyhow::{bail, Context, Result};
use chrono::Utc;
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::ailog::agent_logs_dir;
use straymark_core::charter::{discover_and_parse, find_by_id, CharterStatus};

pub fn run(
    project_path: &str,
    charter_id_input: &str,
    trigger: &str,
    findings_closed: u32,
    ailog_title: &str,
    merge_into: Option<&str>,
) -> Result<()> {
    let project_root = Path::new(project_path);
    let ailog_title = ailog_title.trim();
    if ailog_title.is_empty() {
        bail!("--ailog-title must be non-empty");
    }
    validate_trigger(trigger)?;

    // ── Charter lookup ───────────────────────────────────────────────────
    let (charters, errors) = discover_and_parse(project_root);
    for (path, err) in &errors {
        eprintln!(
            "  {} could not parse {} — {}",
            "⚠".yellow().bold(),
            path.display(),
            err
        );
    }
    let charter = find_by_id(&charters, charter_id_input).ok_or_else(|| {
        anyhow::anyhow!(
            "No Charter matches `{}` (tried full id, CHARTER-NN prefix, and bare NN).",
            charter_id_input
        )
    })?;
    if !matches!(charter.frontmatter.status, CharterStatus::Closed) {
        bail!(
            "Charter {} is not closed (status: {:?}). The amendment pattern \
             applies only AFTER `status: closed` — see STRAYMARK.md §15.B.",
            charter.frontmatter.charter_id,
            charter.frontmatter.status
        );
    }

    // ── Locate previous AILOG (optional — warn if absent) ────────────────
    let agent_logs = agent_logs_dir(project_root);
    let previous_ailog = locate_previous_ailog(&agent_logs, &charter.frontmatter.charter_id)?;
    let previous_ailog_id = previous_ailog
        .as_ref()
        .and_then(|p| extract_ailog_id_from_filename(p));

    // ── Mint new AILOG ───────────────────────────────────────────────────
    let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let nnn = next_ailog_nnn(&agent_logs, &today)?;
    let slug = slugify(ailog_title);
    if slug.is_empty() {
        bail!("--ailog-title slugifies to empty — pass a title with alphanumerics");
    }
    let new_ailog_id = format!("AILOG-{today}-{nnn:03}");
    let new_ailog_filename = format!("{new_ailog_id}-{slug}.md");
    let new_ailog_path = agent_logs.join(&new_ailog_filename);
    if new_ailog_path.exists() {
        bail!(
            "Refusing to overwrite existing AILOG at {}",
            new_ailog_path.display()
        );
    }
    if !agent_logs.exists() {
        std::fs::create_dir_all(&agent_logs).with_context(|| {
            format!("Failed to create {}", agent_logs.display())
        })?;
    }
    let new_ailog_body = render_new_ailog(
        &new_ailog_id,
        ailog_title,
        &charter.frontmatter.charter_id,
        trigger,
        findings_closed,
        previous_ailog_id.as_deref(),
        &today,
    );
    std::fs::write(&new_ailog_path, &new_ailog_body)
        .with_context(|| format!("Failed to write {}", new_ailog_path.display()))?;

    // ── Append Historical correction to previous AILOG (if found) ────────
    if let (Some(prev_path), Some(prev_id)) = (previous_ailog.as_ref(), previous_ailog_id.as_deref())
    {
        append_historical_correction(prev_path, &new_ailog_id, &today, trigger)?;
        println!(
            "  {} Appended `## Historical correction ({})` to {}",
            "✔".green().bold(),
            today,
            prev_path.display()
        );
        let _ = prev_id; // already in the appended block
    } else {
        println!(
            "  {} No prior AILOG mentioning {} found — skipped historical-correction step.",
            "⚠".yellow().bold(),
            charter.frontmatter.charter_id
        );
        println!(
            "  {}",
            "  Edit the new AILOG manually if you want to link back to a specific prior AILOG."
                .dimmed()
        );
    }
    println!(
        "  {} Created new AILOG at {}",
        "✔".green().bold(),
        new_ailog_path.display()
    );

    // ── Render post_close_amendment YAML block ───────────────────────────
    let yaml_block = render_post_close_amendment_yaml(
        trigger,
        &new_ailog_id,
        findings_closed,
    );

    if let Some(target) = merge_into {
        let telemetry_path = PathBuf::from(target);
        merge_post_close_amendment_into(&telemetry_path, &yaml_block)?;
        println!(
            "  {} Merged `post_close_amendment:` block into {}",
            "✔".green().bold(),
            telemetry_path.display()
        );
    } else {
        println!();
        println!(
            "  {}",
            "post_close_amendment YAML — paste under `charter_telemetry:`:".bold()
        );
        println!();
        println!("{yaml_block}");
        println!();
    }

    println!();
    println!(
        "  {}",
        "Next: review the new AILOG, edit it with concrete details, then commit \
         on the original execute branch (do NOT branch off main)."
            .dimmed()
    );

    Ok(())
}

fn validate_trigger(trigger: &str) -> Result<()> {
    match trigger {
        "external_audit" | "production_incident" | "deferred_implementation" => Ok(()),
        other => bail!(
            "Invalid --trigger `{}` (allowed: external_audit | production_incident | deferred_implementation)",
            other
        ),
    }
}

/// Search recursively under `agent-logs/` for an AILOG file whose content
/// mentions the canonical charter_id (full form, e.g. `CHARTER-18-commshub-...`).
/// Returns the most-recently-modified match — a reasonable proxy for "the
/// AILOG that documented the original execute work".
fn locate_previous_ailog(
    agent_logs: &Path,
    canonical_charter_id: &str,
) -> Result<Option<PathBuf>> {
    if !agent_logs.exists() {
        return Ok(None);
    }
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    walk_files(agent_logs, &mut |path| {
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            return;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            return;
        };
        if !content.contains(canonical_charter_id) {
            return;
        }
        let Ok(meta) = std::fs::metadata(path) else {
            return;
        };
        let Ok(mtime) = meta.modified() else { return };
        match &best {
            Some((b_mtime, _)) if *b_mtime >= mtime => {}
            _ => best = Some((mtime, path.to_path_buf())),
        }
    });
    Ok(best.map(|(_, p)| p))
}

fn walk_files(dir: &Path, visit: &mut dyn FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, visit);
        } else {
            visit(&path);
        }
    }
}

fn extract_ailog_id_from_filename(path: &Path) -> Option<String> {
    let name = path.file_name().and_then(|n| n.to_str())?;
    let stem = name.strip_suffix(".md")?;
    // Take the first 5 dash-separated components: AILOG-YYYY-MM-DD-NNN[a-z]?
    let parts: Vec<&str> = stem.split('-').take(5).collect();
    if parts.len() < 5 {
        return None;
    }
    if parts[0] != "AILOG" {
        return None;
    }
    Some(parts.join("-"))
}

fn next_ailog_nnn(agent_logs: &Path, today: &str) -> Result<u32> {
    let prefix = format!("AILOG-{today}-");
    let mut max_seen: Option<u32> = None;
    if agent_logs.exists() {
        let mut paths: Vec<PathBuf> = Vec::new();
        walk_files(agent_logs, &mut |p| paths.push(p.to_path_buf()));
        for path in paths {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(rest) = name.strip_prefix(&prefix) else {
                continue;
            };
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                max_seen = Some(max_seen.map_or(n, |m| m.max(n)));
            }
        }
    }
    Ok(max_seen.map_or(1, |n| n + 1))
}

/// Lightweight slugifier — lowercased ascii alphanumerics + dashes, collapsed.
/// Mirrors `commands::charter::new::slugify` semantics (kebab, no truncation
/// beyond 60 chars).
fn slugify(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut last_dash = true;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.chars().count() > 60 {
        trimmed.chars().take(60).collect()
    } else {
        trimmed
    }
}

fn render_new_ailog(
    new_ailog_id: &str,
    title: &str,
    charter_id: &str,
    trigger: &str,
    findings_closed: u32,
    amends: Option<&str>,
    today: &str,
) -> String {
    let amends_line = amends
        .map(|a| format!("amends: {a}\n"))
        .unwrap_or_default();
    format!(
        "---\n\
id: {new_ailog_id}\n\
type: ailog\n\
title: \"{title}\"\n\
status: draft\n\
created: {today}\n\
agent: straymark-cli-amend  # replace with the executing agent's identity (e.g. claude-code-v1.0)\n\
confidence: medium  # re-assess once the amendment work is underway\n\
charter_id: {charter_id}\n\
risk_level: high\n\
review_required: true\n\
{amends_line}\
trigger: {trigger}\n\
findings_closed: {findings_closed}\n\
guard_closure:\n\
  # Per finding being closed: exactly one of `guard:` (the mechanical check that now\n\
  # prevents recurrence — a command, rule, or test) or `unguardable:` (a specific\n\
  # rationale; stock phrases like 'human review' trip GUARD-001). One item per finding.\n\
  - finding: F1  # replace with the real finding id from the audit review\n\
    guard: \"\"\n\
---\n\
\n\
# {title}\n\
\n\
> Post-close Batch N.4 amendment for {charter_id}. See [STRAYMARK.md §15.B](../../../STRAYMARK.md) and [CHARTER-CHAIN-EVOLUTION.md Pattern 2](../../00-governance/CHARTER-CHAIN-EVOLUTION.md).\n\
\n\
## Context\n\
\n\
- **Trigger**: `{trigger}`\n\
- **Charter**: `{charter_id}` (closed)\n\
- **Findings closed**: {findings_closed}\n\
{}\
\n\
*(Describe what surfaced after `status: closed` — Critical/High audit findings, production incident details, or the deferred-implementation gap being closed. Cite F1...Fn ids from the audit `review.md` where applicable.)*\n\
\n\
## Decision\n\
\n\
*(What you are changing now, scoped to the bounded fix surface. The amendment must remain in one cohesive PR (~< 25 files, no architectural reopen) — if scope grows, abandon this AILOG and open a new Charter instead.)*\n\
\n\
## Change list\n\
\n\
- *(Per-file or per-component description of the amendment commit.)*\n\
\n\
## Risks\n\
\n\
- *(R1: …)*\n\
\n\
## Validation\n\
\n\
- *(Tests, manual smoke, audit-finding-by-id closure evidence.)*\n\
\n\
## Telemetry\n\
\n\
This amendment populates `charter_telemetry.post_close_amendment:` in the Charter's `.telemetry.yaml`. Run `straymark charter amend {charter_id} --trigger {trigger} --merge-into .straymark/charters/CHARTER-NN.telemetry.yaml` (this command) with the appropriate path to auto-merge, or paste the YAML block printed by the CLI.\n\
",
        amends
            .map(|a| format!("- **Amends**: [{a}]({a}.md) (forward pointer; the original gets a `## Historical correction` subsection)\n"))
            .unwrap_or_default()
    )
}

fn append_historical_correction(
    prev_path: &Path,
    new_ailog_id: &str,
    today: &str,
    trigger: &str,
) -> Result<()> {
    let mut content = std::fs::read_to_string(prev_path)
        .with_context(|| format!("Failed to read {}", prev_path.display()))?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    if !content.ends_with("\n\n") {
        content.push('\n');
    }
    content.push_str(&format!(
        "## Historical correction ({today})\n\
\n\
This decision was amended by [{new_ailog_id}]({new_ailog_id}.md) after \
`status: closed`. Trigger: `{trigger}`. See Pattern 2 in \
`.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md`.\n"
    ));
    std::fs::write(prev_path, content)
        .with_context(|| format!("Failed to write {}", prev_path.display()))?;
    Ok(())
}

fn render_post_close_amendment_yaml(
    trigger: &str,
    new_ailog_id: &str,
    findings_closed: u32,
) -> String {
    format!(
        "  post_close_amendment:\n    \
applied: true\n    \
trigger: \"{trigger}\"\n    \
ailog_id: \"{new_ailog_id}\"\n    \
findings_closed: {findings_closed}\n    \
files_modified: 0\n    \
effort_hours: 0.0\n"
    )
}

/// Merge a rendered `post_close_amendment:` block into an existing telemetry
/// YAML. Refuses to overwrite an existing populated block.
fn merge_post_close_amendment_into(telemetry_path: &Path, rendered: &str) -> Result<()> {
    if !telemetry_path.exists() {
        bail!(
            "Telemetry file not found: {}\n  \
             Run `straymark charter close <CHARTER-ID>` first.",
            telemetry_path.display()
        );
    }
    let mut content = std::fs::read_to_string(telemetry_path)
        .with_context(|| format!("Failed to read {}", telemetry_path.display()))?;

    let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("{} is not valid YAML", telemetry_path.display()))?;
    let charter_telemetry = parsed
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("charter_telemetry".into())))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} does not have a top-level `charter_telemetry:` key.",
                telemetry_path.display()
            )
        })?;
    if charter_telemetry
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("post_close_amendment".into())))
        .is_some()
    {
        bail!(
            "{} already has a `post_close_amendment:` block. Refusing to\n  \
             overwrite. Edit the YAML by hand if you genuinely intend to\n  \
             update an existing amendment record.",
            telemetry_path.display()
        );
    }

    while content.ends_with("\n\n") {
        content.pop();
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(rendered);

    std::fs::write(telemetry_path, &content)
        .with_context(|| format!("Failed to write {}", telemetry_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_kebab_and_clamp() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  --weird/--TITLE--  "), "weird-title");
        let long = slugify(&"a-b-".repeat(40));
        assert!(long.chars().count() <= 60);
    }

    #[test]
    fn next_ailog_nnn_increments_correctly() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("AILOG-2026-05-15-001-foo.md"), "x").unwrap();
        std::fs::write(dir.join("AILOG-2026-05-15-003-bar.md"), "x").unwrap();
        std::fs::write(dir.join("AILOG-2026-05-14-007-baz.md"), "x").unwrap();
        assert_eq!(next_ailog_nnn(dir, "2026-05-15").unwrap(), 4);
        assert_eq!(next_ailog_nnn(dir, "2026-05-16").unwrap(), 1);
    }

    #[test]
    fn validate_trigger_accepts_canonical_values() {
        assert!(validate_trigger("external_audit").is_ok());
        assert!(validate_trigger("production_incident").is_ok());
        assert!(validate_trigger("deferred_implementation").is_ok());
        assert!(validate_trigger("misc").is_err());
    }

    #[test]
    fn append_historical_correction_appends_block() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("AILOG-2026-05-14-049-x.md");
        std::fs::write(&path, "---\nid: AILOG-2026-05-14-049\n---\n\nBody.\n").unwrap();
        append_historical_correction(&path, "AILOG-2026-05-15-050", "2026-05-15", "external_audit")
            .unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("## Historical correction (2026-05-15)"));
        assert!(content.contains("AILOG-2026-05-15-050"));
        assert!(content.contains("external_audit"));
    }

    #[test]
    fn merge_post_close_refuses_existing_block() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("t.yaml");
        std::fs::write(
            &path,
            "charter_telemetry:\n  charter_id: x\n  post_close_amendment:\n    applied: true\n",
        )
        .unwrap();
        let err = merge_post_close_amendment_into(&path, "  post_close_amendment:\n    applied: true\n").unwrap_err();
        assert!(err.to_string().contains("already has"));
    }

    #[test]
    fn merge_post_close_appends_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("t.yaml");
        std::fs::write(
            &path,
            "charter_telemetry:\n  charter_id: x\n  outcome:\n    completed_as_planned: true\n",
        )
        .unwrap();
        merge_post_close_amendment_into(
            &path,
            "  post_close_amendment:\n    applied: true\n    trigger: \"external_audit\"\n    ailog_id: \"AILOG-2026-05-15-050\"\n    findings_closed: 0\n    files_modified: 0\n    effort_hours: 0.0\n",
        )
        .unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("post_close_amendment:"));
        assert!(content.contains("AILOG-2026-05-15-050"));
        // Must still be valid YAML after the merge.
        let _: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    }

    // GH #390 — the scaffold must pass the CLI's own `validate` (META-001):
    // every required field present, with enum-valid status/confidence.
    #[test]
    fn render_new_ailog_satisfies_meta_001() {
        let rendered = render_new_ailog(
            "AILOG-2026-08-04-001",
            "post-close remediation",
            "CHARTER-57-x",
            "external_audit",
            3,
            Some("AILOG-2026-07-30-002"),
            "2026-08-04",
        );
        let yaml = rendered
            .strip_prefix("---\n")
            .and_then(|rest| rest.split_once("\n---\n"))
            .map(|(fm, _)| fm)
            .expect("frontmatter delimiters");
        let fm: serde_yaml::Mapping = serde_yaml::from_str(yaml).expect("valid YAML frontmatter");
        let get = |k: &str| {
            fm.get(&serde_yaml::Value::String(k.to_string()))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("missing field `{k}`"))
                .to_string()
        };
        for field in [
            "id",
            "title",
            "status",
            "created",
            "agent",
            "confidence",
            "risk_level",
        ] {
            let _ = get(field);
        }
        assert!(
            fm.get(&serde_yaml::Value::String("review_required".to_string()))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "review_required must be present and true"
        );
        assert_eq!(get("created"), "2026-08-04");
        assert!(
            ["draft", "review", "accepted"].contains(&get("status").as_str()),
            "status must be lifecycle-valid"
        );
        assert!(
            ["low", "medium", "high"].contains(&get("confidence").as_str()),
            "confidence must be enum-valid"
        );
        assert_eq!(get("amends"), "AILOG-2026-07-30-002");
    }
}
