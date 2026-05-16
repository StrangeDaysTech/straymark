//! `straymark charter audit` — orchestrate the dual-audit + calibrator cycle.
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
    prepare: bool,
    merge_reports: bool,
    calibrate: bool,
    finalize: bool,
    merge_into: Option<&str>,
) -> Result<()> {
    // --merge-into only makes sense with --merge-reports (or the deprecated
    // --finalize that aliases to it).
    if merge_into.is_some() && !merge_reports && !finalize {
        bail!("--merge-into is only valid with --merge-reports (or the deprecated --finalize)");
    }

    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;
    let straymark_dir = project_root.join(".straymark");

    // Resolve the Charter.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let charter = charter::find_by_id(&charters, charter_id)
        .ok_or_else(|| {
            anyhow!(
                "Charter {} not found in .straymark/charters/.\n  hint: run `straymark charter list` to see discovered Charters.",
                charter_id
            )
        })?
        .clone();

    let canonical_id = canonical_charter_id(&charter.frontmatter.charter_id);

    // v1 canonical path: .straymark/audits/<CHARTER-ID>/. The audit-prompt is
    // written directly to this dir; reports land here as report-*.md; the
    // future review.md consolidated by the audit-review skill lands here too.
    let audit_dir = straymark_dir.join("audits").join(&canonical_id);
    utils::ensure_dir(&audit_dir)?;

    let range = match range {
        Some(r) => r.to_string(),
        None => resolve_default_range(project_root),
    };

    // Deprecated v0 flag: --calibrate. v1 has no separate calibrate step
    // (the main agent fills the calibrator role via /straymark-audit-review
    // skill). Emit guidance and exit nonzero so callers notice.
    if calibrate {
        eprintln!(
            "{} --calibrate was the v0 way to resolve the calibrator prompt. \
             v1 of the audit flow eliminates that step — the main agent \
             reconciles N reports inline via the /straymark-audit-review skill. \
             To merge reports into telemetry, use --merge-reports.",
            "warn:".yellow().bold()
        );
        bail!("--calibrate is no longer supported in the v1 audit flow");
    }

    // Deprecated v0 alias: --finalize → --merge-reports.
    if finalize {
        eprintln!(
            "{} --finalize is the v0 name for the merge step. The v1 \
             equivalent is --merge-reports (now reading N reports from \
             {}/report-*.md instead of two fixed files).",
            "warn:".yellow().bold(),
            relative_path(project_root, &audit_dir).display()
        );
        return run_merge_reports(
            project_root,
            &straymark_dir,
            &audit_dir,
            &charter,
            &canonical_id,
            merge_into.map(Path::new),
        );
    }

    if merge_reports {
        return run_merge_reports(
            project_root,
            &straymark_dir,
            &audit_dir,
            &charter,
            &canonical_id,
            merge_into.map(Path::new),
        );
    }

    // Default action: prepare. The --prepare flag is accepted for
    // self-documenting invocations but is also the implicit default.
    let _ = prepare;
    run_prepare(project_root, &straymark_dir, &audit_dir, &charter, &range)
}

// ── Step 1: prepare ────────────────────────────────────────────────────────

fn run_prepare(
    project_root: &Path,
    straymark_dir: &Path,
    audit_dir: &Path,
    charter: &Charter,
    range: &str,
) -> Result<()> {
    println!(
        "{} {} ({})",
        "PREPARE".cyan().bold(),
        "audit prompt".bold(),
        charter.frontmatter.charter_id.dimmed()
    );

    let context = build_audit_context(project_root, charter, range)?;

    let lang = crate::config::StrayMarkConfig::resolve_language(project_root);
    let template_path = crate::utils::resolve_localized_path(
        &straymark_dir.join("audit-prompts"),
        "audit-prompt.md",
        &lang,
    );
    let template = std::fs::read_to_string(&template_path).with_context(|| {
        format!(
            "Audit prompt template not found at {}. Run `straymark repair` to restore framework files.",
            template_path.display()
        )
    })?;
    let resolved = resolve_audit_template(&template, &context, "auditor");
    let out = audit_dir.join("audit-prompt.md");
    std::fs::write(&out, resolved)
        .with_context(|| format!("Failed to write resolved prompt to {}", out.display()))?;
    println!(
        "  {} Wrote {}",
        "✔".green().bold(),
        relative_path(project_root, &out).display()
    );

    println!();
    println!("  {}", "Next:".bold());
    println!(
        "    1. Open one or more auditor CLIs (gemini-cli, claude-cli, copilot-cli, etc.)"
    );
    println!("       in this repo and invoke {} in each.",
        format!("/straymark-audit-execute {}", charter.frontmatter.charter_id).cyan());
    println!(
        "       Recommended: at least 2 auditors of different model families."
    );
    println!("    2. Each auditor reads the prompt above, audits with tool use,");
    println!("       and writes its report to:");
    println!(
        "         {}",
        audit_dir
            .join("report-<sluggified-model-id>.md")
            .strip_prefix(project_root)
            .unwrap_or_else(|_| audit_dir.as_ref())
            .display()
    );
    println!(
        "    3. When ALL audits you commissioned have finished (NOT before),"
    );
    println!(
        "       return to this agent and run: {}",
        format!("/straymark-audit-review {}", charter.frontmatter.charter_id).cyan()
    );
    println!("    4. The review skill consolidates the reports and merges YAML");
    println!("       into telemetry.");
    Ok(())
}

// ── Step 2: merge reports ──────────────────────────────────────────────────
//
// v1 unifies the v0 `--calibrate` + `--finalize` two-step into a single
// `--merge-reports` action. The calibrator role (cross-finding reconciliation,
// severity recalibration, missed-finding detection, remediation plan) is now
// performed by the main agent via the /straymark-audit-review skill, which
// produces `.straymark/audits/<id>/review.md` consolidated. The CLI's job
// here is mechanical: validate each report against the schema, build per-
// auditor summaries, and emit the YAML block (or merge into telemetry).

fn run_merge_reports(
    project_root: &Path,
    straymark_dir: &Path,
    audit_dir: &Path,
    charter: &Charter,
    canonical_id: &str,
    merge_into: Option<&Path>,
) -> Result<()> {
    println!(
        "{} {} ({})",
        "MERGE-REPORTS".cyan().bold(),
        "audit cycle".bold(),
        charter.frontmatter.charter_id.dimmed()
    );

    // Gather report-*.md files under the audit dir (one per auditor).
    let mut report_paths: Vec<PathBuf> = Vec::new();
    if audit_dir.exists() {
        for entry in std::fs::read_dir(audit_dir)
            .with_context(|| format!("Failed to read {}", audit_dir.display()))?
        {
            let entry = entry?;
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let name = match p.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if name.starts_with("report-") && name.ends_with(".md") {
                report_paths.push(p);
            }
        }
    }
    report_paths.sort();

    if report_paths.is_empty() {
        bail!(
            "No reports found in {}. Expected one or more files matching report-*.md \
             written by the /straymark-audit-execute skill (or saved manually by the \
             operator). Run --prepare first if you have not generated the audit prompt.",
            relative_path(project_root, audit_dir).display()
        );
    }

    if report_paths.len() < 2 {
        eprintln!(
            "{} only one report found ({}). Cross-family heterogeneity is the \
             discovery mechanism for substantive findings — recommended minimum \
             is 2 auditors of different model families. Proceeding with the \
             single report you provided.",
            "warn:".yellow().bold(),
            relative_path(project_root, &report_paths[0]).display()
        );
    }

    let schema = AuditOutputSchema::load(straymark_dir)?;
    let mut auditor_summaries: Vec<AuditorSummary> = Vec::new();
    for path in &report_paths {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let fm = parse_frontmatter(&raw)
            .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;
        let issues = schema.validate(&fm, path);
        if !issues.is_empty() {
            eprintln!(
                "{} {} failed schema validation",
                "error:".red().bold(),
                path.display()
            );
            for issue in &issues {
                eprintln!("  - {}", issue.message);
            }
            bail!("auditor report failed schema validation");
        }
        let summary = AuditorSummary::from_frontmatter(&fm)?;
        println!(
            "  {} Validated {} ({} findings)",
            "✔".green().bold(),
            relative_path(project_root, path).display(),
            summary.findings_total
        );
        auditor_summaries.push(summary);
    }

    println!();
    println!("  {}", "Audit cycle merge complete.".green().bold());
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
    Ok(())
}

/// Append a freshly-rendered `external_audit:` block to an existing Charter
/// telemetry YAML.
///
/// v0.2 (fw-4.16.0, Issue #156) — placeholder-aware merge:
/// - missing `external_audit:` key → append the new block (original behavior)
/// - empty placeholder `external_audit: []` → REPLACE it in place (so the
///   post-close audit cycle round-trips when close (or any tool) writes the
///   placeholder)
/// - populated `external_audit:` array with one or more entries → bail with
///   guidance (re-audit/append is still unsupported; operator reconciles
///   manually rather than silently duplicating findings)
fn merge_external_audit_into(
    telemetry_path: &Path,
    auditor_summaries: &[AuditorSummary],
    canonical_charter_id: &str,
) -> Result<()> {
    if !telemetry_path.exists() {
        bail!(
            "Telemetry file not found: {}\n  \
             Run `straymark charter close <CHARTER-ID>` first to create the telemetry,\n  \
             then re-run with --merge-into. Or omit --merge-into to print the YAML\n  \
             for manual paste.",
            telemetry_path.display()
        );
    }

    let mut content = std::fs::read_to_string(telemetry_path)
        .with_context(|| format!("Failed to read {}", telemetry_path.display()))?;

    // Parse the full document so we can inspect `charter_telemetry.external_audit`
    // semantically rather than via fragile string matching (fw-4.16.0).
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("{} is not valid YAML", telemetry_path.display()))?;

    let charter_telemetry = parsed
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("charter_telemetry".into())))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "{} does not have a top-level `charter_telemetry:` key — \
                 expected the standard charter close output shape.",
                telemetry_path.display()
            )
        })?;

    let existing_external_audit = charter_telemetry
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("external_audit".into())));

    let placeholder = match existing_external_audit {
        None => ExternalAuditState::Missing,
        Some(serde_yaml::Value::Sequence(seq)) if seq.is_empty() => {
            ExternalAuditState::EmptyPlaceholder
        }
        Some(serde_yaml::Value::Sequence(_)) => ExternalAuditState::Populated,
        Some(_) => {
            bail!(
                "{} has `charter_telemetry.external_audit` set to a non-array \
                 value — refusing to overwrite. Fix the YAML by hand or remove \
                 the key before re-running.",
                telemetry_path.display()
            );
        }
    };

    if matches!(placeholder, ExternalAuditState::Populated) {
        bail!(
            "{} already has a populated `external_audit:` block. Re-audit\n  \
             (appending to an existing array) is not supported in v0. Re-run\n  \
             `straymark charter audit <id> --merge-reports` (without --merge-into)\n  \
             to print the new YAML, then merge manually if you want to append.",
            telemetry_path.display()
        );
    }

    let rendered = render_external_audit_yaml(auditor_summaries, canonical_charter_id);

    match placeholder {
        ExternalAuditState::Missing => {
            while content.ends_with("\n\n") {
                content.pop();
            }
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("\n  external_audit:\n");
            content.push_str(&rendered);
        }
        ExternalAuditState::EmptyPlaceholder => {
            content = replace_empty_external_audit_placeholder(&content, &rendered)?;
        }
        ExternalAuditState::Populated => unreachable!("guarded above"),
    }

    std::fs::write(telemetry_path, &content)
        .with_context(|| format!("Failed to write {}", telemetry_path.display()))?;
    Ok(())
}

enum ExternalAuditState {
    Missing,
    EmptyPlaceholder,
    Populated,
}

/// Replace a `  external_audit: []` placeholder line (at indent 2 inside
/// `charter_telemetry:`) with a fully-rendered block. Tolerates whitespace
/// variations (`[]`, `[ ]`) and trailing comments. Bails if the placeholder
/// line cannot be located — that means the YAML parsed an empty array but
/// the source uses flow/block syntax we cannot rewrite safely.
fn replace_empty_external_audit_placeholder(content: &str, rendered: &str) -> Result<String> {
    let mut out = String::with_capacity(content.len() + rendered.len());
    let mut replaced = false;

    for line in content.split_inclusive('\n') {
        if !replaced && is_empty_external_audit_placeholder_line(line) {
            // Capture the original indentation so we preserve the file's
            // existing style (charter_telemetry sub-keys are indented by 2
            // spaces by close.rs, but we don't want to hardcode that).
            let indent: String = line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
            out.push_str(&indent);
            out.push_str("external_audit:\n");
            out.push_str(rendered);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            replaced = true;
        } else {
            out.push_str(line);
        }
    }

    if !replaced {
        bail!(
            "Could not locate `external_audit: []` placeholder line in the\n  \
             telemetry YAML to replace. The YAML parses as an empty array but\n  \
             the source uses a block/multi-line form this CLI does not yet\n  \
             rewrite. Remove the `external_audit:` key by hand and re-run, or\n  \
             omit --merge-into to print the YAML for manual merge."
        );
    }

    Ok(out)
}

/// Returns true for lines like `  external_audit: []`, `external_audit:[]`,
/// `  external_audit: [ ]  # comment`, etc. Stays strictly on the empty-array
/// placeholder form — any inline element (e.g. `[ {auditor: ...} ]`) returns
/// false and falls back to the populated-array bail path.
fn is_empty_external_audit_placeholder_line(line: &str) -> bool {
    let trimmed = line.trim_end_matches(['\n', '\r']);
    let after_indent = trimmed.trim_start_matches([' ', '\t']);
    let Some(rest) = after_indent.strip_prefix("external_audit:") else {
        return false;
    };
    let mut rest = rest.trim_start_matches([' ', '\t']);
    if let Some(after_open) = rest.strip_prefix('[') {
        rest = after_open.trim_start_matches([' ', '\t']);
    } else {
        return false;
    }
    let Some(after_close) = rest.strip_prefix(']') else {
        return false;
    };
    let tail = after_close.trim_start_matches([' ', '\t']);
    tail.is_empty() || tail.starts_with('#')
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
    project_context: String,
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
        schema_path: ".straymark/schemas/audit-output.schema.v0.json".to_string(),
        // {{project_context}} is intentionally empty by default. Adopters
        // who want to give auditors a project-stack hint can edit the
        // template to substitute it manually, or a future release may
        // derive it from CLAUDE.md / config.yml.
        project_context: String::new(),
    })
}

fn read_originating_ailogs(project_root: &Path, charter: &Charter) -> Result<(String, String)> {
    let ailog_ids = match &charter.frontmatter.originating_ailogs {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => return Ok(("(none)".to_string(), "(none)".to_string())),
    };
    let agent_logs = project_root
        .join(".straymark")
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
        ("{{project_context}}", &ctx.project_context),
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
    /// Read but currently unused outside the existing unit test that asserts
    /// it parses correctly. Kept on the struct so the schema validation
    /// covers it.
    #[allow(dead_code)]
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

            project_context: String::new(),
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

            project_context: String::new(),
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

            project_context: String::new(),
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
