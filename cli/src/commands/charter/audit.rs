//! `devtrail charter audit` — orchestrate the dual-audit + calibrator cycle.
//!
//! Phase 3 v0 is **orchestration-only**: the command resolves prompts, awaits
//! the operator's auditor responses, validates outputs against the schema,
//! and prints the consolidated findings ready to paste into the Charter
//! telemetry. The CLI does NOT invoke any LLM API directly — the operator
//! runs the prompts in their auditor of choice (Copilot, Gemini, Claude, etc.)
//! and saves the responses to canonical paths.
//!
//! Three steps, each invokable independently:
//!
//! 1. **Prepare** (default invocation): resolve `auditor-primary.prompt.md`
//!    and `auditor-secondary.prompt.md` against the Charter + git diff +
//!    AILOGs, write them under `audit/charters/<CHARTER-ID>/prompts/`.
//! 2. **Calibrate** (`--calibrate`): once both auditor responses exist at
//!    `audit/charters/<CHARTER-ID>/auditor-{primary,secondary}.md`, validate
//!    them against the schema and resolve the calibrator prompt against
//!    their findings.
//! 3. **Finalize** (`--finalize`): once the calibrator response exists,
//!    validate everything, print a YAML-formatted `external_audit` block
//!    for the operator to paste into the Charter telemetry, and print the
//!    calibrator's reconciliation summary.
//!
//! Per RFC #82 the resolved prompts are persisted BEFORE any external
//! action. Per principle #10 (honesty about what the tool does not do) the
//! CLI does not pretend to talk to LLMs.

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::audit_schema::AuditOutputSchema;
use crate::charter::{self, Charter};
use crate::utils;

const DEFAULT_RANGE: &str = "HEAD~1..HEAD";

pub fn run(
    path: &str,
    charter_id: &str,
    range: Option<&str>,
    calibrate: bool,
    finalize: bool,
) -> Result<()> {
    if calibrate && finalize {
        bail!("--calibrate and --finalize are mutually exclusive — run one at a time");
    }

    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("DevTrail not installed. Run 'devtrail init' first."))?;
    let project_root = &resolved.path;
    let devtrail_dir = project_root.join(".devtrail");

    // Resolve the Charter.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let charter = charter::find_by_id(&charters, charter_id)
        .ok_or_else(|| anyhow!("Charter {} not found in docs/charters/", charter_id))?
        .clone();

    let canonical_id = canonical_charter_id(&charter.frontmatter.charter_id);

    let audit_dir = project_root
        .join("audit")
        .join("charters")
        .join(&canonical_id);
    let prompts_dir = audit_dir.join("prompts");
    utils::ensure_dir(&prompts_dir)?;

    let range = range.unwrap_or(DEFAULT_RANGE).to_string();

    if finalize {
        return run_finalize(project_root, &devtrail_dir, &audit_dir, &charter);
    }
    if calibrate {
        return run_calibrate(
            project_root,
            &devtrail_dir,
            &audit_dir,
            &prompts_dir,
            &charter,
            &range,
        );
    }
    run_prepare(
        project_root,
        &devtrail_dir,
        &audit_dir,
        &prompts_dir,
        &charter,
        &range,
    )
}

// ── Step 1: prepare ────────────────────────────────────────────────────────

fn run_prepare(
    project_root: &Path,
    devtrail_dir: &Path,
    audit_dir: &Path,
    prompts_dir: &Path,
    charter: &Charter,
    range: &str,
) -> Result<()> {
    println!(
        "{} {} ({})",
        "Step 1/3:".cyan().bold(),
        "PREPARE".bold(),
        charter.frontmatter.charter_id.dimmed()
    );

    let context = build_audit_context(project_root, charter, range)?;

    for role in ["auditor-primary", "auditor-secondary"] {
        let template_path = devtrail_dir
            .join("audit-prompts")
            .join(format!("{role}.md"));
        let template = std::fs::read_to_string(&template_path).with_context(|| {
            format!(
                "Audit prompt template not found at {}. Run `devtrail repair` to restore framework files.",
                template_path.display()
            )
        })?;
        let resolved = resolve_audit_template(&template, &context, role);
        let out = prompts_dir.join(format!("{role}.prompt.md"));
        std::fs::write(&out, resolved)
            .with_context(|| format!("Failed to write resolved prompt to {}", out.display()))?;
        println!(
            "  {} Wrote {}",
            "✔".green().bold(),
            relative_path(project_root, &out).display()
        );
    }

    println!();
    println!("  {}", "Next:".bold());
    println!("    1. Paste each prompt into your auditor of choice (use a model");
    println!("       of a different family per auditor — see CLI-REFERENCE).");
    println!("    2. Save the auditor responses to:");
    println!(
        "         {}",
        audit_dir
            .join("auditor-primary.md")
            .strip_prefix(project_root)
            .unwrap_or_else(|_| audit_dir.as_ref())
            .display()
    );
    println!(
        "         {}",
        audit_dir
            .join("auditor-secondary.md")
            .strip_prefix(project_root)
            .unwrap_or_else(|_| audit_dir.as_ref())
            .display()
    );
    println!(
        "    3. Run: {} {} --calibrate",
        "devtrail charter audit".cyan(),
        charter.frontmatter.charter_id.cyan()
    );
    Ok(())
}

// ── Step 2: calibrate ──────────────────────────────────────────────────────

fn run_calibrate(
    project_root: &Path,
    devtrail_dir: &Path,
    audit_dir: &Path,
    prompts_dir: &Path,
    charter: &Charter,
    range: &str,
) -> Result<()> {
    println!(
        "{} {} ({})",
        "Step 2/3:".cyan().bold(),
        "CALIBRATE".bold(),
        charter.frontmatter.charter_id.dimmed()
    );

    let primary_path = audit_dir.join("auditor-primary.md");
    let secondary_path = audit_dir.join("auditor-secondary.md");

    for (role, path) in [
        ("auditor-primary", &primary_path),
        ("auditor-secondary", &secondary_path),
    ] {
        if !path.exists() {
            bail!(
                "{} not found. Save the {} response to that path before running --calibrate.",
                path.display(),
                role
            );
        }
    }

    let schema = AuditOutputSchema::load(devtrail_dir)?;
    for path in [&primary_path, &secondary_path] {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let frontmatter = parse_frontmatter(&raw)
            .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;
        let issues = schema.validate(&frontmatter, path);
        if !issues.is_empty() {
            eprintln!(
                "{} validation issues in {}:",
                "error:".red().bold(),
                path.display()
            );
            for issue in &issues {
                eprintln!("  - {} [{}]", issue.message, issue.rule);
                if let Some(hint) = &issue.fix_hint {
                    eprintln!("    {} {}", "hint:".cyan(), hint);
                }
            }
            bail!("auditor output failed schema validation");
        }
        println!(
            "  {} Validated {}",
            "✔".green().bold(),
            relative_path(project_root, path).display()
        );
    }

    let primary_body = std::fs::read_to_string(&primary_path)?;
    let secondary_body = std::fs::read_to_string(&secondary_path)?;

    let mut context = build_audit_context(project_root, charter, range)?;
    context.auditor_primary_findings = primary_body;
    context.auditor_secondary_findings = secondary_body;

    let template_path = devtrail_dir
        .join("audit-prompts")
        .join("calibrator-reconciler.md");
    let template = std::fs::read_to_string(&template_path).with_context(|| {
        format!(
            "Calibrator prompt template not found at {}. Run `devtrail repair`.",
            template_path.display()
        )
    })?;
    let resolved = resolve_audit_template(&template, &context, "calibrator-reconciler");
    let out = prompts_dir.join("calibrator-reconciler.prompt.md");
    std::fs::write(&out, resolved)
        .with_context(|| format!("Failed to write {}", out.display()))?;
    println!(
        "  {} Wrote {}",
        "✔".green().bold(),
        relative_path(project_root, &out).display()
    );

    println!();
    println!("  {}", "Next:".bold());
    println!(
        "    1. Run the calibrator prompt in a model of your choice (calibrator may");
    println!("       be of any family per roadmap §5.2 — heterogeneity is for the");
    println!("       auditor pair, not the calibrator).");
    println!(
        "    2. Save the response to: {}",
        audit_dir
            .join("calibrator-reconciler.md")
            .strip_prefix(project_root)
            .unwrap_or_else(|_| audit_dir.as_ref())
            .display()
    );
    println!(
        "    3. Run: {} {} --finalize",
        "devtrail charter audit".cyan(),
        charter.frontmatter.charter_id.cyan()
    );
    Ok(())
}

// ── Step 3: finalize ───────────────────────────────────────────────────────

fn run_finalize(
    project_root: &Path,
    devtrail_dir: &Path,
    audit_dir: &Path,
    charter: &Charter,
) -> Result<()> {
    println!(
        "{} {} ({})",
        "Step 3/3:".cyan().bold(),
        "FINALIZE".bold(),
        charter.frontmatter.charter_id.dimmed()
    );

    let primary_path = audit_dir.join("auditor-primary.md");
    let secondary_path = audit_dir.join("auditor-secondary.md");
    let calibrator_path = audit_dir.join("calibrator-reconciler.md");

    for (label, path) in [
        ("auditor-primary", &primary_path),
        ("auditor-secondary", &secondary_path),
        ("calibrator-reconciler", &calibrator_path),
    ] {
        if !path.exists() {
            bail!(
                "{} not found. {} must exist before --finalize. \
                 Re-run --calibrate if the calibrator step is incomplete.",
                path.display(),
                label
            );
        }
    }

    let schema = AuditOutputSchema::load(devtrail_dir)?;
    let mut auditor_summaries: Vec<AuditorSummary> = Vec::new();
    for path in [&primary_path, &secondary_path] {
        let raw = std::fs::read_to_string(path)?;
        let fm = parse_frontmatter(&raw)?;
        let issues = schema.validate(&fm, path);
        if !issues.is_empty() {
            eprintln!("{} {} failed schema validation", "error:".red().bold(), path.display());
            for issue in &issues {
                eprintln!("  - {}", issue.message);
            }
            bail!("auditor output failed schema validation");
        }
        let summary = AuditorSummary::from_frontmatter(&fm)?;
        println!(
            "  {} Validated {} ({} findings, prompt: {})",
            "✔".green().bold(),
            relative_path(project_root, path).display(),
            summary.findings_total,
            summary.prompt_used.dimmed()
        );
        auditor_summaries.push(summary);
    }

    let calibrator_raw = std::fs::read_to_string(&calibrator_path)?;
    let calibrator_fm = parse_frontmatter(&calibrator_raw)?;
    let issues = schema.validate(&calibrator_fm, &calibrator_path);
    if !issues.is_empty() {
        eprintln!("{} calibrator failed schema validation", "error:".red().bold());
        for issue in &issues {
            eprintln!("  - {}", issue.message);
        }
        bail!("calibrator output failed schema validation");
    }
    println!(
        "  {} Validated {}",
        "✔".green().bold(),
        relative_path(project_root, &calibrator_path).display()
    );

    println!();
    println!("  {}", "Charter audit complete.".green().bold());
    println!();
    println!("  {}", "external_audit YAML — paste into telemetry:".bold());
    println!("  {}", "(charter_telemetry.external_audit array)".dimmed());
    println!();
    println!("{}", render_external_audit_yaml(&auditor_summaries));
    println!();
    println!("  {}", "Calibrator summary (copy to outcome.scope_change_notes if relevant):".dimmed());
    println!(
        "  {}",
        relative_path(project_root, &calibrator_path).display().to_string().dimmed()
    );
    Ok(())
}

// ── Audit context + template resolution ────────────────────────────────────

struct AuditContext {
    charter_id: String,
    charter_title: String,
    charter_path: String,
    charter_content: String,
    git_range: String,
    git_diff: String,
    ailog_paths: String,
    ailog_contents: String,
    schema_path: String,
    auditor_primary_findings: String,
    auditor_secondary_findings: String,
}

fn build_audit_context(
    project_root: &Path,
    charter: &Charter,
    range: &str,
) -> Result<AuditContext> {
    let charter_content = std::fs::read_to_string(&charter.path)
        .with_context(|| format!("Failed to read {}", charter.path.display()))?;
    let charter_path_rel = relative_path(project_root, &charter.path)
        .display()
        .to_string();

    let (ailog_paths, ailog_contents) = read_originating_ailogs(project_root, charter)?;
    let git_diff = run_git_diff(project_root, range)?;

    Ok(AuditContext {
        charter_id: charter.frontmatter.charter_id.clone(),
        charter_title: charter::display_title(charter),
        charter_path: charter_path_rel,
        charter_content,
        git_range: range.to_string(),
        git_diff,
        ailog_paths,
        ailog_contents,
        schema_path: ".devtrail/schemas/audit-output.schema.v0.json".to_string(),
        auditor_primary_findings: String::new(),
        auditor_secondary_findings: String::new(),
    })
}

fn read_originating_ailogs(project_root: &Path, charter: &Charter) -> Result<(String, String)> {
    let ailog_ids = match &charter.frontmatter.originating_ailogs {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => return Ok(("(none)".to_string(), "(none)".to_string())),
    };
    let agent_logs = project_root
        .join(".devtrail")
        .join("07-ai-audit")
        .join("agent-logs");
    let mut paths = Vec::new();
    let mut contents = String::new();
    for id in &ailog_ids {
        let prefix = id.split('-').take(5).collect::<Vec<_>>().join("-");
        if let Some(found) = walk_for_prefix(&agent_logs, &prefix) {
            paths.push(
                relative_path(project_root, &found)
                    .display()
                    .to_string(),
            );
            if let Ok(body) = std::fs::read_to_string(&found) {
                contents.push_str(&format!("--- {} ---\n", id));
                contents.push_str(&body);
                contents.push('\n');
            }
        } else {
            paths.push(format!("{} (NOT FOUND)", id));
        }
    }
    Ok((paths.join("\n"), contents))
}

fn walk_for_prefix(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = walk_for_prefix(&path, prefix) {
                return Some(found);
            }
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(prefix) && name.ends_with(".md") {
                return Some(path);
            }
        }
    }
    None
}

fn run_git_diff(project_root: &Path, range: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["diff", range])
        .current_dir(project_root)
        .output()
        .with_context(|| format!("Failed to invoke git diff {range}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("git diff {range} failed: {err}");
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Substitute `{{placeholder}}` tokens in `template` with values from `ctx`.
/// `audit_role` is overridden per-call so the same context can be used for
/// primary, secondary, and calibrator passes.
fn resolve_audit_template(template: &str, ctx: &AuditContext, audit_role: &str) -> String {
    let pairs: &[(&str, &str)] = &[
        ("{{charter_id}}", &ctx.charter_id),
        ("{{charter_title}}", &ctx.charter_title),
        ("{{charter_path}}", &ctx.charter_path),
        ("{{charter_content}}", &ctx.charter_content),
        ("{{git_range}}", &ctx.git_range),
        ("{{git_diff}}", &ctx.git_diff),
        ("{{ailog_paths}}", &ctx.ailog_paths),
        ("{{ailog_contents}}", &ctx.ailog_contents),
        ("{{audit_role}}", audit_role),
        ("{{schema_path}}", &ctx.schema_path),
        ("{{auditor_primary_findings}}", &ctx.auditor_primary_findings),
        (
            "{{auditor_secondary_findings}}",
            &ctx.auditor_secondary_findings,
        ),
    ];
    let mut out = template.to_string();
    for (placeholder, value) in pairs {
        out = out.replace(placeholder, value);
    }
    out
}

// ── Frontmatter parsing + auditor summary ──────────────────────────────────

fn parse_frontmatter(raw: &str) -> Result<serde_yaml::Value> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    let after = trimmed
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow!("audit output does not start with `---` frontmatter delimiter"))?;
    let end = after
        .find("\n---")
        .ok_or_else(|| anyhow!("frontmatter is not terminated by `---`"))?;
    let yaml_str = &after[..end];
    Ok(serde_yaml::from_str(yaml_str)?)
}

struct AuditorSummary {
    auditor: String,
    findings_total: u64,
    findings_by_category: std::collections::BTreeMap<String, u64>,
    audit_quality: Option<String>,
    prompt_used: String,
}

impl AuditorSummary {
    fn from_frontmatter(fm: &serde_yaml::Value) -> Result<Self> {
        let auditor = fm
            .get("auditor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("auditor field missing"))?
            .to_string();
        let findings_total = fm
            .get("findings_total")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("findings_total missing"))?;
        let findings_by_category = match fm.get("findings_by_category").and_then(|v| v.as_mapping())
        {
            Some(map) => map
                .iter()
                .filter_map(|(k, v)| {
                    Some((k.as_str()?.to_string(), v.as_u64().unwrap_or(0)))
                })
                .collect(),
            None => Default::default(),
        };
        let audit_quality = fm
            .get("audit_quality")
            .and_then(|v| v.as_str())
            .map(String::from);
        let prompt_used = fm
            .get("prompt_used")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(Self {
            auditor,
            findings_total,
            findings_by_category,
            audit_quality,
            prompt_used,
        })
    }
}

fn render_external_audit_yaml(summaries: &[AuditorSummary]) -> String {
    let mut out = String::new();
    for s in summaries {
        out.push_str(&format!("    - auditor: \"{}\"\n", s.auditor));
        out.push_str(&format!("      findings_total: {}\n", s.findings_total));
        out.push_str("      findings_by_category:\n");
        for cat in [
            "hallucination",
            "implementation_gap",
            "real_debt",
            "false_positive",
        ] {
            let count = s.findings_by_category.get(cat).copied().unwrap_or(0);
            out.push_str(&format!("        {}: {}\n", cat, count));
        }
        if let Some(quality) = &s.audit_quality {
            out.push_str(&format!("      audit_quality: \"{}\"\n", quality));
        }
        out.push_str(&format!(
            "      audit_notes: \"see audit/charters/{}/{}.md\"\n",
            "<charter-id>",
            if s.auditor.contains("primary") || s.findings_total > 0 {
                "auditor-primary"
            } else {
                "auditor-secondary"
            }
        ));
    }
    out
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn canonical_charter_id(charter_id: &str) -> String {
    // CHARTER-NN[-slug] → CHARTER-NN.
    charter_id
        .split_once('-')
        .and_then(|(prefix, rest)| Some(format!("{}-{}", prefix, rest.split('-').next()?)))
        .unwrap_or_else(|| charter_id.to_string())
}

fn relative_path(project_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(project_root)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_id_strips_slug() {
        assert_eq!(canonical_charter_id("CHARTER-05"), "CHARTER-05");
        assert_eq!(canonical_charter_id("CHARTER-05-baseline"), "CHARTER-05");
        assert_eq!(
            canonical_charter_id("CHARTER-12-batching-listtimeseries-04-f3"),
            "CHARTER-12"
        );
    }

    #[test]
    fn resolve_template_substitutes_known_placeholders() {
        let template = "id: {{charter_id}}\nrole: {{audit_role}}\nrange: {{git_range}}\n";
        let ctx = AuditContext {
            charter_id: "CHARTER-01".into(),
            charter_title: "T".into(),
            charter_path: "p".into(),
            charter_content: "c".into(),
            git_range: "HEAD~1..HEAD".into(),
            git_diff: "d".into(),
            ailog_paths: "(none)".into(),
            ailog_contents: "(none)".into(),
            schema_path: "s".into(),
            auditor_primary_findings: String::new(),
            auditor_secondary_findings: String::new(),
        };
        let out = resolve_audit_template(template, &ctx, "auditor-primary");
        assert_eq!(
            out,
            "id: CHARTER-01\nrole: auditor-primary\nrange: HEAD~1..HEAD\n"
        );
    }

    #[test]
    fn resolve_template_leaves_unknown_placeholders_intact() {
        // If a template uses {{foo}} that isn't in our list, it stays as-is.
        let template = "{{charter_id}} -- {{unknown_token}}";
        let ctx = AuditContext {
            charter_id: "CHARTER-01".into(),
            charter_title: "".into(),
            charter_path: "".into(),
            charter_content: "".into(),
            git_range: "".into(),
            git_diff: "".into(),
            ailog_paths: "".into(),
            ailog_contents: "".into(),
            schema_path: "".into(),
            auditor_primary_findings: String::new(),
            auditor_secondary_findings: String::new(),
        };
        let out = resolve_audit_template(template, &ctx, "x");
        assert_eq!(out, "CHARTER-01 -- {{unknown_token}}");
    }

    #[test]
    fn parse_frontmatter_extracts_yaml_block() {
        let raw = "---\nfoo: bar\nlist:\n  - 1\n---\n\nbody\n";
        let v = parse_frontmatter(raw).unwrap();
        assert_eq!(v.get("foo").and_then(|v| v.as_str()), Some("bar"));
    }

    #[test]
    fn auditor_summary_extracts_fields() {
        let yaml: serde_yaml::Value = serde_yaml::from_str(
            r#"
auditor: copilot-v1.0.37
findings_total: 5
findings_by_category:
  hallucination: 0
  implementation_gap: 2
  real_debt: 2
  false_positive: 1
audit_quality: high
prompt_used: prompts/auditor-primary.prompt.md
"#,
        )
        .unwrap();
        let s = AuditorSummary::from_frontmatter(&yaml).unwrap();
        assert_eq!(s.auditor, "copilot-v1.0.37");
        assert_eq!(s.findings_total, 5);
        assert_eq!(s.findings_by_category.get("implementation_gap"), Some(&2));
        assert_eq!(s.audit_quality.as_deref(), Some("high"));
        assert_eq!(s.prompt_used, "prompts/auditor-primary.prompt.md");
    }
}
