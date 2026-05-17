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
    include_charters: bool,
    check_pending_reviews: bool,
    max_pending_days: i64,
) -> Result<()> {
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
    let paths = crate::document::discover_documents(straymark_dir);
    let mut fixed_count = 0;

    for path in &paths {
        if let Ok(doc) = crate::document::parse_document(path) {
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
