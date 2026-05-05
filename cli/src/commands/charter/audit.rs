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

/// Last-resort fallback when no upstream branch is reachable. Issued with a
/// warning that explains why the operator may want `--range` explicitly.
const FALLBACK_RANGE: &str = "HEAD~1..HEAD";

/// Resolve the default git range when `--range` is not provided.
///
/// Tries upstream branches in priority order (`origin/main` → `origin/master`)
/// to bound the audit at the point where the feature branch diverged from the
/// project's main line. Falls back to `HEAD~1..HEAD` (the v0 default) when no
/// upstream is reachable, with a warning to stderr — that path covers
/// freshly-cloned repos without remotes, disconnected branches, or repos
/// where the operator hasn't run `git fetch` yet.
///
/// Issue #102 R11(A): Sentinel CHARTER-07 was implemented as 8 commits on a
/// feature branch; the previous default `HEAD~1..HEAD` only sent the last
/// (metadata-only) commit to the auditors, which converged on "0 substantive
/// findings" vacuously because they never saw the migrations, SQLC, scaffolding,
/// or PII guard test. `origin/main..HEAD` captures the full implementation set.
fn resolve_default_range(project_root: &Path) -> String {
    for candidate in ["origin/main", "origin/master"] {
        let probe = std::process::Command::new("git")
            .args(["rev-parse", "--verify", "--quiet", candidate])
            .current_dir(project_root)
            .output();
        if let Ok(out) = probe {
            if out.status.success() {
                return format!("{candidate}..HEAD");
            }
        }
    }
    eprintln!(
        "{} no upstream branch reachable (tried origin/main, origin/master); \
         falling back to {}. For multi-commit feature branches, pass \
         --range <REV..REV> explicitly so the auditors see the full \
         implementation set, not just the last commit.",
        "warn:".yellow().bold(),
        FALLBACK_RANGE
    );
    FALLBACK_RANGE.to_string()
}

pub fn run(
    path: &str,
    charter_id: &str,
    range: Option<&str>,
    calibrate: bool,
    finalize: bool,
    merge_into: Option<&str>,
) -> Result<()> {
    if calibrate && finalize {
        bail!("--calibrate and --finalize are mutually exclusive — run one at a time");
    }
    if merge_into.is_some() && !finalize {
        bail!("--merge-into is only valid with --finalize");
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

    let range = match range {
        Some(r) => r.to_string(),
        None => resolve_default_range(project_root),
    };

    if finalize {
        return run_finalize(
            project_root,
            &devtrail_dir,
            &audit_dir,
            &charter,
            &canonical_id,
            merge_into.map(Path::new),
        );
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
    canonical_id: &str,
    merge_into: Option<&Path>,
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

    if let Some(target) = merge_into {
        merge_external_audit_into(target, &auditor_summaries, canonical_id)?;
        println!(
            "  {} Merged external_audit array into {}",
            "✔".green().bold(),
            relative_path(project_root, target).display()
        );
        println!();
        println!(
            "  {}",
            "Run `git diff` on the telemetry file to review the merge before commit.".dimmed()
        );
    } else {
        println!("  {}", "external_audit YAML — paste into telemetry:".bold());
        println!("  {}", "(charter_telemetry.external_audit array)".dimmed());
        println!();
        println!("{}", render_external_audit_yaml(&auditor_summaries, canonical_id));
        println!();
    }
    println!(
        "  {}",
        "Calibrator summary (copy to outcome.scope_change_notes if relevant):".dimmed()
    );
    println!(
        "  {}",
        relative_path(project_root, &calibrator_path).display().to_string().dimmed()
    );
    Ok(())
}

/// Append a freshly-rendered `external_audit:` block to an existing Charter
/// telemetry YAML. v0 deliberately rejects re-audit (file already has the
/// key) so the operator can reconcile manually rather than silently
/// duplicating findings.
fn merge_external_audit_into(
    telemetry_path: &Path,
    auditor_summaries: &[AuditorSummary],
    canonical_charter_id: &str,
) -> Result<()> {
    if !telemetry_path.exists() {
        bail!(
            "Telemetry file not found: {}\n  \
             Run `devtrail charter close <CHARTER-ID>` first to create the telemetry,\n  \
             then re-run with --merge-into. Or omit --merge-into to print the YAML\n  \
             for manual paste.",
            telemetry_path.display()
        );
    }

    let mut content = std::fs::read_to_string(telemetry_path)
        .with_context(|| format!("Failed to read {}", telemetry_path.display()))?;

    // Sanity: must parse as YAML.
    let _: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("{} is not valid YAML", telemetry_path.display()))?;

    if !content.contains("charter_telemetry:") {
        bail!(
            "{} does not have a top-level `charter_telemetry:` key — \
             expected the standard charter close output shape.",
            telemetry_path.display()
        );
    }

    // v0: re-audit (appending to existing array) is not supported. Detect
    // the key at indent 0 or 2 and bail with guidance.
    if content.contains("\n  external_audit:")
        || content.contains("\nexternal_audit:")
        || content.starts_with("  external_audit:")
        || content.starts_with("external_audit:")
    {
        bail!(
            "{} already has an `external_audit:` block. Re-audit (appending\n  \
             to an existing array) is not supported in v0. Re-run\n  \
             `devtrail charter audit <id> --finalize` (without --merge-into) to\n  \
             print the new YAML, then merge manually if you want to append.",
            telemetry_path.display()
        );
    }

    while content.ends_with("\n\n") {
        content.pop();
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }

    content.push_str("\n  external_audit:\n");
    content.push_str(&render_external_audit_yaml(
        auditor_summaries,
        canonical_charter_id,
    ));

    std::fs::write(telemetry_path, &content)
        .with_context(|| format!("Failed to write {}", telemetry_path.display()))?;
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
///
/// Placeholder replacement is **scoped to non-comment regions**: any
/// `{{placeholder}}` inside a `<!-- ... -->` HTML comment block is preserved
/// as-is. This prevents the documentation header of a template (which lists
/// available placeholders with their literal `{{name}}` syntax for human
/// reference) from being weaponized as a content duplicator. Reported in
/// issue #102 (R10) — Sentinel observed an inflated 1300-line resolved prompt
/// where Charter content was emitted twice (once expanded inside the comment,
/// once in the body proper). Unclosed comments (no matching `-->`) terminate
/// the scan early and the remaining tail is treated as non-comment.
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

    // Find all <!-- ... --> ranges so we can skip placeholder replacement
    // inside them. Each range is (start_byte, end_byte_exclusive) where
    // end_byte points just past the closing `-->`.
    let mut comment_ranges: Vec<(usize, usize)> = Vec::new();
    let mut search_from = 0;
    while let Some(rel_start) = template[search_from..].find("<!--") {
        let abs_start = search_from + rel_start;
        match template[abs_start + 4..].find("-->") {
            Some(rel_end) => {
                let abs_end = abs_start + 4 + rel_end + 3; // include closing -->
                comment_ranges.push((abs_start, abs_end));
                search_from = abs_end;
            }
            None => {
                // Unclosed comment — leave the rest as comment-region to
                // mirror typical HTML/markdown rendering, and stop scanning.
                comment_ranges.push((abs_start, template.len()));
                break;
            }
        }
    }

    let replace_all = |segment: &str| -> String {
        let mut s = segment.to_string();
        for (placeholder, value) in pairs {
            s = s.replace(placeholder, value);
        }
        s
    };

    let mut out = String::with_capacity(template.len());
    let mut cursor = 0;
    for (start, end) in &comment_ranges {
        // Replace placeholders in the segment before this comment.
        out.push_str(&replace_all(&template[cursor..*start]));
        // Append the comment range verbatim — no placeholder substitution.
        out.push_str(&template[*start..*end]);
        cursor = *end;
    }
    // Trailing segment after the last comment (or the whole template if
    // there were no comments).
    out.push_str(&replace_all(&template[cursor..]));
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

fn render_external_audit_yaml(summaries: &[AuditorSummary], canonical_charter_id: &str) -> String {
    let mut out = String::new();
    for (idx, s) in summaries.iter().enumerate() {
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
        // First summary maps to auditor-primary.md, second to
        // auditor-secondary.md — that's the order finalize reads them in.
        let role_file = if idx == 0 {
            "auditor-primary"
        } else {
            "auditor-secondary"
        };
        out.push_str(&format!(
            "      audit_notes: \"see audit/charters/{}/{}.md\"\n",
            canonical_charter_id, role_file
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

    // ── R10 regression tests (issue #102) ──────────────────────────────────
    //
    // Before the fix, the resolver did `String::replace` globally without
    // distinguishing whether a placeholder was inside an HTML comment. The
    // documentation header of `auditor-primary.md` has lines like
    // `  {{charter_content}} — full body of the Charter doc` whose intent
    // is purely descriptive — but the global replace expanded each one,
    // duplicating ~30k tokens of payload (Charter + AILOG + diff) inside
    // the comment block. After the fix, comments are preserved verbatim.

    fn r10_test_ctx() -> AuditContext {
        AuditContext {
            charter_id: "CHARTER-07".into(),
            charter_title: "T".into(),
            charter_path: "p".into(),
            charter_content: "REAL_CHARTER_BODY".into(),
            git_range: "main..HEAD".into(),
            git_diff: "REAL_DIFF".into(),
            ailog_paths: "(none)".into(),
            ailog_contents: "REAL_AILOGS".into(),
            schema_path: "s".into(),
            auditor_primary_findings: String::new(),
            auditor_secondary_findings: String::new(),
        }
    }

    #[test]
    fn resolve_template_skips_placeholder_inside_html_comment() {
        let template = "<!-- doc: {{charter_id}} stays literal --><body>{{charter_id}}</body>";
        let out = resolve_audit_template(template, &r10_test_ctx(), "auditor");
        assert_eq!(
            out,
            "<!-- doc: {{charter_id}} stays literal --><body>CHARTER-07</body>",
            "placeholder inside <!-- ... --> must be preserved verbatim; only the body occurrence is replaced"
        );
    }

    #[test]
    fn resolve_template_preserves_documentation_block_with_multiple_placeholders() {
        // Mirrors the real auditor-primary.md template header structure that
        // triggered R10 on Sentinel CHARTER-07.
        let template = "<!--\n\
            Placeholders:\n  \
            {{charter_id}}      — e.g., CHARTER-05\n  \
            {{charter_content}} — full body\n  \
            {{git_diff}}        — output of git diff\n\
        -->\n\
        \n\
        # Audit for {{charter_id}}\n\
        \n\
        Charter: {{charter_content}}\n\
        Diff: {{git_diff}}\n";

        let out = resolve_audit_template(template, &r10_test_ctx(), "auditor");

        // Comment block stays as documentation (no expansion inside).
        assert!(
            out.contains("{{charter_id}}      — e.g., CHARTER-05"),
            "documentation header must stay literal: got {out:?}"
        );
        assert!(
            out.contains("{{charter_content}} — full body"),
            "documentation header must stay literal"
        );

        // Body lines (outside comment) are expanded normally.
        assert!(out.contains("# Audit for CHARTER-07"));
        assert!(out.contains("Charter: REAL_CHARTER_BODY"));
        assert!(out.contains("Diff: REAL_DIFF"));

        // Critically: REAL_CHARTER_BODY should appear exactly once
        // (the body line), not twice (which would happen if the
        // documentation line `{{charter_content}} — full body` had been
        // expanded into `REAL_CHARTER_BODY — full body`).
        assert_eq!(
            out.matches("REAL_CHARTER_BODY").count(),
            1,
            "expected exactly one occurrence of REAL_CHARTER_BODY (R10 dedup)"
        );
        assert_eq!(
            out.matches("REAL_DIFF").count(),
            1,
            "expected exactly one occurrence of REAL_DIFF (R10 dedup)"
        );
    }

    #[test]
    fn resolve_template_handles_multiple_comment_blocks() {
        let template = "<!-- A: {{charter_id}} -->{{charter_id}}<!-- B: {{charter_id}} -->{{charter_id}}";
        let out = resolve_audit_template(template, &r10_test_ctx(), "auditor");
        assert_eq!(
            out,
            "<!-- A: {{charter_id}} -->CHARTER-07<!-- B: {{charter_id}} -->CHARTER-07"
        );
    }

    #[test]
    fn resolve_template_handles_unclosed_comment_gracefully() {
        // Edge case: malformed template with an unclosed `<!--`. The resolver
        // must not loop forever; it conservatively treats the rest as
        // comment-region (no replacement) and stops scanning.
        let template = "{{charter_id}} <!-- forever {{charter_id}}";
        let out = resolve_audit_template(template, &r10_test_ctx(), "auditor");
        assert_eq!(
            out,
            "CHARTER-07 <!-- forever {{charter_id}}",
            "first placeholder (before `<!--`) is replaced; tail after unclosed comment is preserved"
        );
    }

    #[test]
    fn resolve_template_no_comments_behaves_like_global_replace() {
        // When the template has no <!-- ... --> blocks, behavior is
        // backwards-compatible with the pre-R10 resolver.
        let template = "id={{charter_id}}; range={{git_range}}; id_again={{charter_id}}";
        let out = resolve_audit_template(template, &r10_test_ctx(), "auditor");
        assert_eq!(
            out,
            "id=CHARTER-07; range=main..HEAD; id_again=CHARTER-07"
        );
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
