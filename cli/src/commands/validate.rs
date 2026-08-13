use anyhow::{bail, Result};
use colored::Colorize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::inject;
use crate::manifest::DistManifest;
use crate::utils;
use crate::validation::{self, Severity, ValidationIssue};

pub fn run(
    path: &str,
    fix: bool,
    staged: bool,
    agent: Option<&str>,
    include_charters: bool,
    check_pending_reviews: bool,
    max_pending_days: i64,
    commit_msg: Option<&str>,
) -> Result<()> {
    // Agent-targeted validation is a separate code path: it inspects an
    // external skills directory (e.g. ~/.codex/skills/), not StrayMark docs.
    if let Some(agent) = agent {
        return match agent {
            "codex" | "qoder" | "qwen" => validate_agent_skills(agent),
            other => bail!("unknown --agent: {other} (supported: codex, qoder, qwen)"),
        };
    }

    let resolved = match utils::resolve_project_root(path) {
        Some(r) => r,
        None => {
            let target = PathBuf::from(path)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(path));
            utils::info(&format!(
                "StrayMark is not installed in {}",
                target.display()
            ));
            utils::info("Run 'straymark init' to initialize StrayMark in this directory.");
            return Ok(());
        }
    };

    if resolved.is_fallback {
        utils::info(&format!(
            "Using StrayMark installation at repo root: {}",
            resolved.path.display()
        ));
    }

    let target = resolved.path;
    let straymark_dir = target.join(".straymark");

    // --commit-msg mode: id-shaped references in a commit message must resolve
    // (#419). Blocking; hook-shaped like --staged but for commit-msg hooks.
    if let Some(msg_file) = commit_msg {
        return run_commit_msg(&straymark_dir, msg_file);
    }

    // --staged mode: validate only git-staged .straymark/ documents.
    // Charter validation in --staged mode is a Phase 2 enhancement; in v0
    // the flag is honored only in the all-mode path below.
    if staged {
        return run_staged(&target, &straymark_dir);
    }

    // Header
    println!();
    println!("  {}", "StrayMark Validate".bold().cyan());
    println!("  {}", target.display().to_string().dimmed());
    println!();

    // Run validation
    let (mut result, mut doc_count) = validation::validate_all(&straymark_dir);

    if include_charters {
        let (charter_result, charter_count) =
            validation::validate_charters(&target, &straymark_dir);
        result.merge(charter_result);
        doc_count += charter_count;
    }

    if check_pending_reviews {
        for issue in validation::check_pending_reviews(&straymark_dir, max_pending_days) {
            result.warnings.push(issue);
        }
    }

    for issue in check_host_marker_health(&target, &straymark_dir) {
        result.warnings.push(issue);
    }

    if doc_count == 0 && result.errors.is_empty() && result.warnings.is_empty() {
        utils::info("No documents found to validate.");
        println!(
            "  {} Create documents with {} or {}",
            "→".blue().bold(),
            "straymark new".cyan(),
            "/straymark-new".cyan()
        );
        println!();
        return Ok(());
    }

    // Apply fixes if requested
    if fix {
        apply_fixes(&straymark_dir);
        // Re-validate after fixes
        let (mut result, mut doc_count) = validation::validate_all(&straymark_dir);
        if include_charters {
            let (charter_result, charter_count) =
                validation::validate_charters(&target, &straymark_dir);
            result.merge(charter_result);
            doc_count += charter_count;
        }
        if check_pending_reviews {
            for issue in validation::check_pending_reviews(&straymark_dir, max_pending_days) {
                result.warnings.push(issue);
            }
        }
        for issue in check_host_marker_health(&target, &straymark_dir) {
            result.warnings.push(issue);
        }
        print_results(&result, doc_count);
        return exit_with_code(&result);
    }

    print_results(&result, doc_count);
    exit_with_code(&result)
}

/// `validate --commit-msg <file>` (#419): name resolution for commit messages.
/// Every id-shaped token (AILOG-*, FU-*, CHARTER-*, ...) must resolve to a
/// document, charter or follow-up in .straymark/ — a citation that resolves to
/// nothing is a phantom reference and blocks the commit. Designed to be wired
/// into a commit-msg hook:
///
/// ```sh
/// # .git/hooks/commit-msg
/// #!/bin/sh
/// straymark validate --commit-msg "$1"
/// ```
fn run_commit_msg(straymark_dir: &std::path::Path, msg_file: &str) -> Result<()> {
    let content = std::fs::read_to_string(msg_file).map_err(|e| {
        anyhow::anyhow!("Cannot read commit message file '{msg_file}': {e}")
    })?;

    println!();
    println!("  {}", "StrayMark Validate (commit-msg)".bold().cyan());
    println!("  {}", msg_file.dimmed());
    println!();

    let result = validation::validate_commit_msg(
        std::path::Path::new(msg_file),
        &content,
        straymark_dir,
    );

    if result.errors.is_empty() && result.warnings.is_empty() {
        println!(
            "  {} Every StrayMark id referenced resolves.",
            "✓".green().bold()
        );
        println!();
        return Ok(());
    }

    print_results(&result, 1);
    exit_with_code(&result)
}

fn run_staged(project_root: &std::path::Path, straymark_dir: &std::path::Path) -> Result<()> {
    // Get staged files from git
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(project_root)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => {
            bail!("Not a git repository or git is not available. --staged requires a git repo.");
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let staged_paths: Vec<PathBuf> = stdout
        .lines()
        .filter(|line| line.starts_with(".straymark/") && line.ends_with(".md"))
        .map(|line| project_root.join(line))
        .collect();

    if staged_paths.is_empty() {
        println!(
            "  {} No staged documentation to validate.",
            "✓".green().bold()
        );
        return Ok(());
    }

    // Header
    println!();
    println!("  {}", "StrayMark Validate (staged)".bold().cyan());
    println!(
        "  {} file(s)",
        staged_paths.len().to_string().dimmed()
    );
    println!();

    let (result, doc_count) = validation::validate_paths(&staged_paths, straymark_dir);

    if doc_count == 0 {
        println!(
            "  {} No StrayMark documents among staged files.",
            "✓".green().bold()
        );
        return Ok(());
    }

    print_results(&result, doc_count);
    exit_with_code(&result)
}

fn apply_fixes(straymark_dir: &std::path::Path) {
    let paths = straymark_core::document::discover_documents(straymark_dir);
    let mut fixed_count = 0;

    for path in &paths {
        if let Ok(doc) = straymark_core::document::parse_document(path) {
            if let Some(new_content) = validation::apply_fixes(&doc) {
                if std::fs::write(path, new_content).is_ok() {
                    println!(
                        "  {} Fixed: {}",
                        "✓".green().bold(),
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?")
                    );
                    fixed_count += 1;
                }
            }
        }
    }

    if fixed_count > 0 {
        println!();
        println!(
            "  {} {} file(s) fixed automatically",
            "→".blue().bold(),
            fixed_count
        );
        println!();
    }
}

fn print_results(result: &validation::ValidationResult, doc_count: usize) {
    let all_issues: Vec<&ValidationIssue> = result
        .errors
        .iter()
        .chain(result.warnings.iter())
        .collect();

    if all_issues.is_empty() {
        println!(
            "  {} All {} document(s) passed validation",
            "✓".green().bold(),
            doc_count
        );
        println!();
        return;
    }

    // Group by file
    let mut by_file: BTreeMap<&PathBuf, Vec<&ValidationIssue>> = BTreeMap::new();
    for issue in &all_issues {
        by_file.entry(&issue.file).or_default().push(issue);
    }

    for (file, issues) in &by_file {
        let filename = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?");

        println!("  {}", filename.bold());

        for issue in issues {
            let severity_label = match issue.severity {
                Severity::Error => "error".red().bold(),
                Severity::Warning => "warn".yellow().bold(),
            };
            println!(
                "    {} [{}] {}",
                severity_label, issue.rule, issue.message
            );
            if let Some(hint) = &issue.fix_hint {
                println!("    {} {}", "hint:".dimmed(), hint.dimmed());
            }
        }
        println!();
    }

    // Summary
    let error_count = result.errors.len();
    let warning_count = result.warnings.len();

    let summary = format!(
        "  {} error(s), {} warning(s) in {} document(s)",
        error_count, warning_count, doc_count
    );

    if error_count > 0 {
        println!("{}", summary.red().bold());
    } else {
        println!("{}", summary.yellow());
    }
    println!();
}

fn exit_with_code(result: &validation::ValidationResult) -> Result<()> {
    if result.errors.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Per-agent shape of a user-level skills installation.
///
/// `env_var`/`default_dir` mirror the agent's own home resolution (see
/// `install_skills.rs`). `minimal_frontmatter` says whether the agent expects
/// the reduced Codex-style frontmatter (`name` + `description` only) or the
/// full Claude-style one: Qoder and Qwen Code both parse `allowed-tools`, so
/// for them its presence is correct rather than a copy-paste mistake.
struct AgentSkillsSpec {
    env_var: &'static str,
    default_dir: &'static str,
    minimal_frontmatter: bool,
}

fn agent_skills_spec(agent: &str) -> AgentSkillsSpec {
    match agent {
        "qoder" => AgentSkillsSpec {
            env_var: "QODER_CONFIG_DIR",
            default_dir: ".qoder",
            minimal_frontmatter: false,
        },
        "qwen" => AgentSkillsSpec {
            env_var: "QWEN_HOME",
            default_dir: ".qwen",
            minimal_frontmatter: false,
        },
        // codex
        _ => AgentSkillsSpec {
            env_var: "CODEX_HOME",
            default_dir: ".codex",
            minimal_frontmatter: true,
        },
    }
}

/// Validate an agent's user-level skills installation (e.g.
/// `$CODEX_HOME/skills/`, `$QODER_CONFIG_DIR/skills/`, `$QWEN_HOME/skills/`).
/// Checks every `straymark-*` skill for: presence of `SKILL.md`, parseable YAML
/// frontmatter, required `name` and `description`, and — for agents that expect
/// the minimal frontmatter — absence of Claude-only keys like `allowed-tools`
/// (whose presence indicates someone copied skills from `.claude/` by mistake).
fn validate_agent_skills(agent: &str) -> Result<()> {
    let spec = agent_skills_spec(agent);
    let home_fallback = || {
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(spec.default_dir)
    };
    let agent_home = match std::env::var(spec.env_var) {
        Ok(v) if !v.is_empty() => std::path::PathBuf::from(v),
        _ => home_fallback(),
    };
    let skills_dir = agent_home.join("skills");
    let install_hint = format!("straymark install-skills --agent {agent}");

    println!();
    println!("  {}", format!("StrayMark Validate ({agent})").bold().cyan());
    println!("  {}", skills_dir.display().to_string().dimmed());
    println!();

    if !skills_dir.is_dir() {
        utils::warn(&format!(
            "{} skills directory not found: {}",
            agent,
            skills_dir.display()
        ));
        println!(
            "  {} Run {} to populate it.",
            "→".blue().bold(),
            install_hint.cyan()
        );
        println!();
        std::process::exit(1);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&skills_dir)
        .map_err(|e| anyhow::anyhow!("read_dir {}: {e}", skills_dir.display()))?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_string_lossy()
                    .starts_with("straymark-")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        utils::warn(&format!(
            "No straymark-* skills found under {}",
            skills_dir.display()
        ));
        println!(
            "  {} Run {} to install them.",
            "→".blue().bold(),
            install_hint.cyan()
        );
        println!();
        std::process::exit(1);
    }

    let mut errors = 0usize;
    let mut warnings = 0usize;
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let skill_md = entry.path().join("SKILL.md");
        if !skill_md.exists() {
            println!("  {}", name.bold());
            println!("    {} [missing-skill-md] SKILL.md not found", "error".red().bold());
            println!();
            errors += 1;
            continue;
        }
        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
        let (fm, parse_err) = parse_frontmatter(&content);
        let mut file_issues: Vec<(bool, String, String)> = Vec::new();
        if let Some(err) = parse_err {
            file_issues.push((true, "frontmatter-invalid".into(), err));
        } else {
            let has_name = fm
                .as_ref()
                .and_then(|m| m.get("name"))
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            let has_desc = fm
                .as_ref()
                .and_then(|m| m.get("description"))
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            if !has_name {
                file_issues.push((true, "missing-name".into(), "frontmatter missing `name`".into()));
            }
            if !has_desc {
                file_issues.push((
                    true,
                    "missing-description".into(),
                    "frontmatter missing `description`".into(),
                ));
            }
            if spec.minimal_frontmatter {
                if let Some(map) = &fm {
                    for forbidden in &["allowed-tools", "argument-hint", "model"] {
                        if map.contains_key(*forbidden) {
                            file_issues.push((
                                false,
                                "claude-only-key".into(),
                                format!(
                                    "frontmatter contains `{forbidden}` — {agent} skills should keep only `name` and `description`"
                                ),
                            ));
                        }
                    }
                }
            }
        }
        if file_issues.is_empty() {
            continue;
        }
        println!("  {}", name.bold());
        for (is_error, rule, msg) in file_issues {
            let label = if is_error {
                "error".red().bold()
            } else {
                "warn".yellow().bold()
            };
            println!("    {} [{}] {}", label, rule, msg);
            if is_error {
                errors += 1;
            } else {
                warnings += 1;
            }
        }
        println!();
    }

    if errors == 0 && warnings == 0 {
        println!(
            "  {} All {} {} skill(s) passed validation",
            "✓".green().bold(),
            entries.len(),
            agent
        );
        println!();
        return Ok(());
    }

    let summary = format!(
        "  {} error(s), {} warning(s) across {} skill(s)",
        errors, warnings, entries.len()
    );
    if errors > 0 {
        println!("{}", summary.red().bold());
        println!();
        std::process::exit(1);
    } else {
        println!("{}", summary.yellow());
        println!();
        Ok(())
    }
}

/// Tiny frontmatter parser for SKILL.md files. Returns the key→value map (or
/// None if no frontmatter) and an optional parse error string.
fn parse_frontmatter(content: &str) -> (Option<BTreeMap<String, String>>, Option<String>) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return (None, Some("no opening `---` fence".into()));
    }
    let close = match lines.iter().enumerate().skip(1).find(|(_, l)| l.trim() == "---") {
        Some((i, _)) => i,
        None => return (None, Some("no closing `---` fence".into())),
    };
    let body = lines[1..close].join("\n");
    match serde_yaml::from_str::<serde_yaml::Value>(&body) {
        Ok(serde_yaml::Value::Mapping(m)) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                if let Some(ks) = k.as_str() {
                    let vs = match v {
                        serde_yaml::Value::String(s) => s,
                        other => serde_yaml::to_string(&other).unwrap_or_default().trim().to_string(),
                    };
                    out.insert(ks.to_string(), vs);
                }
            }
            (Some(out), None)
        }
        Ok(_) => (None, Some("frontmatter is not a mapping".into())),
        Err(e) => (None, Some(format!("YAML parse error: {e}"))),
    }
}

/// Inspect every injection target declared in the local `dist-manifest.yml` and
/// emit a warning for each host file (`.cursorrules`, `CLAUDE.md`, etc.) whose
/// StrayMark marker block is malformed (duplicated, orphan, or inverted).
///
/// Reparation is the job of `straymark update-framework` / `repair` (which call
/// `inject::inject_directive` and auto-sanitize). This check is purely diagnostic.
///
/// If the manifest cannot be loaded (missing or malformed), the check is silently
/// skipped — a corrupt manifest is reported elsewhere and we don't want to
/// double-fail or block `validate` on it.
fn check_host_marker_health(project_root: &Path, straymark_dir: &Path) -> Vec<ValidationIssue> {
    let manifest_path = straymark_dir.join("dist-manifest.yml");
    let manifest = match DistManifest::load(&manifest_path) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    let mut issues = Vec::new();
    for injection in &manifest.injections {
        let target = project_root.join(&injection.target);
        if !target.exists() {
            continue;
        }
        let health = match inject::inspect_marker_health(&target) {
            Ok(h) => h,
            Err(_) => continue,
        };
        if !health.is_malformed() {
            continue;
        }
        let mut parts = Vec::new();
        if health.begin_count != health.end_count {
            parts.push(format!(
                "{} begin / {} end marker(s) (counts must match)",
                health.begin_count, health.end_count
            ));
        }
        if !health.has_canonical_block && (health.begin_count > 0 || health.end_count > 0) {
            parts.push("no canonical block (only orphan markers)".to_string());
        }
        if health.end_before_begin {
            parts.push("end marker before begin marker".to_string());
        }
        let detail = parts.join(", ");
        issues.push(ValidationIssue {
            file: target,
            rule: "host-marker-health".to_string(),
            message: format!("Malformed StrayMark markers ({detail})."),
            severity: Severity::Warning,
            fix_hint: Some(
                "Run 'straymark update-framework' or 'straymark repair' to auto-repair.".to_string(),
            ),
        });
    }
    issues
}
