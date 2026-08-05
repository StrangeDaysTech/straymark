use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::path::Path;

use crate::inject;
use crate::manifest::DistManifest;
use crate::utils;

/// Legacy hardcoded targets for backwards compatibility with installations
/// that don't have a local dist-manifest.yml
const LEGACY_DIRECTIVE_TARGETS: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "GEMINI.md",
    ".github/copilot-instructions.md",
    ".cursorrules",
    ".cursor/rules/straymark.md",
];

pub fn run(full: bool) -> Result<()> {
    let target = std::env::current_dir().context("Failed to get current directory")?;

    if !target.join(".straymark").exists() {
        bail!("StrayMark is not installed in this directory.");
    }

    if full {
        println!(
            "{} This will remove ALL StrayMark files including your documents!",
            "WARNING:".red().bold()
        );
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you sure you want to remove everything?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }

        let double_confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("This will delete all your AILOG, AIDEC, ADR, and other documents. Really proceed?")
            .default(false)
            .interact()?;

        if !double_confirmed {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("{} StrayMark...", "Removing".red().bold());

    // Remove injections from directive files
    utils::info("Cleaning AI agent directives...");
    clean_directives(&target)?;

    // Remove framework files
    utils::info("Removing framework files...");

    if full {
        // Remove everything
        remove_dir_if_exists(&target.join(".straymark"))?;
    } else {
        // Selective removal: keep user documents, remove framework
        remove_framework_files(&target)?;
    }

    // Remove distributed files
    remove_file_if_exists(&target.join("STRAYMARK.md"))?;

    // Remove agent skills and workflows
    remove_dir_if_exists(&target.join(".claude/skills"))?;
    remove_dir_if_exists(&target.join(".gemini/skills"))?;
    remove_dir_if_exists(&target.join(".codex/skills"))?;
    remove_dir_if_exists(&target.join(".qoder/skills"))?;
    remove_dir_if_exists(&target.join(".agent/workflows"))?;

    // Clean up empty parent dirs
    remove_empty_dir(&target.join(".claude"))?;
    remove_empty_dir(&target.join(".gemini"))?;
    remove_empty_dir(&target.join(".codex"))?;
    remove_empty_dir(&target.join(".qoder"))?;
    remove_empty_dir(&target.join(".agent"))?;

    // Clean up .cursor directories (injections already handled by clean_directives)
    remove_empty_dir(&target.join(".cursor/rules"))?;
    remove_empty_dir(&target.join(".cursor"))?;

    // Legacy: clean up scripts from pre-CLI installations (removed in fw-5.0)
    let scripts = [
        "scripts/straymark-new.sh",
        "scripts/straymark-status.sh",
        "scripts/pre-commit-docs.sh",
        "scripts/validate-docs.ps1",
    ];
    for script in &scripts {
        remove_file_if_exists(&target.join(script))?;
    }
    remove_empty_dir(&target.join("scripts"))?;

    println!();
    utils::success("StrayMark removed successfully.");

    if !full {
        println!();
        println!(
            "  {} User-generated documents in .straymark/ were preserved.",
            "Note:".bold()
        );
        println!(
            "  Use {} to remove everything.",
            "straymark remove --full".yellow()
        );
    }

    Ok(())
}

fn clean_directives(target: &Path) -> Result<()> {
    // Try to load the local manifest for injection targets
    let manifest_path = target.join(".straymark/dist-manifest.yml");
    let directive_targets: Vec<String> = if manifest_path.exists() {
        match DistManifest::load(&manifest_path) {
            Ok(manifest) => manifest
                .injections
                .iter()
                .map(|inj| inj.target.clone())
                .collect(),
            Err(_) => {
                // Failed to parse — fall back to legacy list
                LEGACY_DIRECTIVE_TARGETS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            }
        }
    } else {
        // No local manifest — legacy installation
        LEGACY_DIRECTIVE_TARGETS
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    for directive_target in &directive_targets {
        let path = target.join(directive_target);
        if inject::remove_injection(&path)? {
            utils::success(&format!("Cleaned {}", directive_target));
        }

        // Clean up empty parent directories
        if let Some(parent) = path.parent() {
            if parent != target {
                remove_empty_dir(parent)?;
            }
        }
    }

    Ok(())
}

/// Remove framework files but keep user-generated documents
fn remove_framework_files(target: &Path) -> Result<()> {
    let straymark = target.join(".straymark");

    // Framework directories to remove entirely
    let framework_dirs = [
        "00-governance",
        "03-implementation",
        "templates",
    ];

    for dir in &framework_dirs {
        remove_dir_if_exists(&straymark.join(dir))?;
    }

    // Remove framework files but keep user documents in these dirs
    let mixed_dirs = [
        "01-requirements",
        "02-design/decisions",
        "04-testing",
        "05-operations/incidents",
        "05-operations/runbooks",
        "06-evolution/technical-debt",
        "07-ai-audit/agent-logs",
        "07-ai-audit/decisions",
        "07-ai-audit/ethical-reviews",
    ];

    for dir in &mixed_dirs {
        let dir_path = straymark.join(dir);
        if dir_path.is_dir() {
            // Only remove .gitkeep, keep user documents
            remove_file_if_exists(&dir_path.join(".gitkeep"))?;
        }
    }

    // Remove framework root files
    remove_file_if_exists(&straymark.join("config.yml"))?;
    remove_file_if_exists(&straymark.join("QUICK-REFERENCE.md"))?;
    remove_file_if_exists(&straymark.join(".checksums.json"))?;
    remove_file_if_exists(&straymark.join(".provenance.yml"))?;
    remove_file_if_exists(&straymark.join("dist-manifest.yml"))?;

    Ok(())
}

fn remove_file_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    }
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
    }
    Ok(())
}

fn remove_empty_dir(path: &Path) -> Result<()> {
    if path.is_dir() {
        if let Ok(mut entries) = std::fs::read_dir(path) {
            if entries.next().is_none() {
                std::fs::remove_dir(path).ok();
            }
        }
    }
    Ok(())
}
