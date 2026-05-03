use anyhow::{bail, Result};
use colored::Colorize;
use std::collections::BTreeMap;
use std::path::PathBuf;

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
                "DevTrail is not installed in {}",
                target.display()
            ));
            utils::info("Run 'devtrail init' to initialize DevTrail in this directory.");
            return Ok(());
        }
    };

    if resolved.is_fallback {
        utils::info(&format!(
            "Using DevTrail installation at repo root: {}",
            resolved.path.display()
        ));
    }

    let target = resolved.path;
    let devtrail_dir = target.join(".devtrail");

    // --staged mode: validate only git-staged .devtrail/ documents.
    // Charter validation in --staged mode is a Phase 2 enhancement; in v0
    // the flag is honored only in the all-mode path below.
    if staged {
        return run_staged(&target, &devtrail_dir);
    }

    // Header
    println!();
    println!("  {}", "DevTrail Validate".bold().cyan());
    println!("  {}", target.display().to_string().dimmed());
    println!();

    // Run validation
    let (mut result, mut doc_count) = validation::validate_all(&devtrail_dir);

    if include_charters {
        let (charter_result, charter_count) =
            validation::validate_charters(&target, &devtrail_dir);
        result.merge(charter_result);
        doc_count += charter_count;
    }

    if check_pending_reviews {
        for issue in validation::check_pending_reviews(&devtrail_dir, max_pending_days) {
            result.warnings.push(issue);
        }
    }

    if doc_count == 0 {
        utils::info("No documents found to validate.");
        println!(
            "  {} Create documents with {} or {}",
            "→".blue().bold(),
            "devtrail new".cyan(),
            "/devtrail-new".cyan()
        );
        println!();
        return Ok(());
    }

    // Apply fixes if requested
    if fix {
        apply_fixes(&devtrail_dir);
        // Re-validate after fixes
        let (mut result, mut doc_count) = validation::validate_all(&devtrail_dir);
        if include_charters {
            let (charter_result, charter_count) =
                validation::validate_charters(&target, &devtrail_dir);
            result.merge(charter_result);
            doc_count += charter_count;
        }
        if check_pending_reviews {
            for issue in validation::check_pending_reviews(&devtrail_dir, max_pending_days) {
                result.warnings.push(issue);
            }
        }
        print_results(&result, doc_count);
        return exit_with_code(&result);
    }

    print_results(&result, doc_count);
    exit_with_code(&result)
}

fn run_staged(project_root: &std::path::Path, devtrail_dir: &std::path::Path) -> Result<()> {
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
        .filter(|line| line.starts_with(".devtrail/") && line.ends_with(".md"))
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
    println!("  {}", "DevTrail Validate (staged)".bold().cyan());
    println!(
        "  {} file(s)",
        staged_paths.len().to_string().dimmed()
    );
    println!();

    let (result, doc_count) = validation::validate_paths(&staged_paths, devtrail_dir);

    if doc_count == 0 {
        println!(
            "  {} No DevTrail documents among staged files.",
            "✓".green().bold()
        );
        return Ok(());
    }

    print_results(&result, doc_count);
    exit_with_code(&result)
}

fn apply_fixes(devtrail_dir: &std::path::Path) {
    let paths = crate::document::discover_documents(devtrail_dir);
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
