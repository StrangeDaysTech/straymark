use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::Checksums;
use crate::download;
use crate::inject;
use crate::manifest::{DistManifest, Injection};
use crate::utils;

/// Expected directories inside .straymark/ (same as init.rs)
const EXPECTED_DIRS: &[&str] = &[
    ".straymark/00-governance/exceptions",
    ".straymark/01-requirements",
    ".straymark/02-design/decisions",
    ".straymark/03-implementation",
    ".straymark/04-testing",
    ".straymark/05-operations/incidents",
    ".straymark/05-operations/runbooks",
    ".straymark/06-evolution/technical-debt",
    ".straymark/07-ai-audit/agent-logs",
    ".straymark/07-ai-audit/decisions",
    ".straymark/07-ai-audit/ethical-reviews",
    ".straymark/08-security",
    ".straymark/09-ai-models",
];

pub fn run(path: &str) -> Result<()> {
    let resolved = match utils::resolve_project_root(path) {
        Some(r) => r,
        None => {
            utils::warn("StrayMark is not installed in this directory or repo root.");
            utils::info("Run 'straymark init' to initialize StrayMark.");
            bail!("No StrayMark installation found");
        }
    };

    if resolved.is_fallback {
        utils::info(&format!(
            "Using StrayMark installation at repo root: {}",
            resolved.path.display()
        ));
    }

    let target = resolved.path;

    println!(
        "{} StrayMark in {}",
        "Repairing".cyan().bold(),
        target.display()
    );

    // Phase 1: Check what's missing
    let mut missing_dirs: Vec<&str> = Vec::new();
    for dir in EXPECTED_DIRS {
        let dir_path = target.join(dir);
        if !dir_path.exists() {
            missing_dirs.push(dir);
        }
    }

    // Check for missing framework files that require download
    let needs_download = check_needs_download(&target);

    let missing_dir_count = missing_dirs.len();
    let total_issues = missing_dir_count + if needs_download { 1 } else { 0 };

    if total_issues == 0 {
        utils::success("StrayMark structure is healthy, nothing to repair.");
        return Ok(());
    }

    println!(
        "  {} Found {} issue(s) to repair",
        "→".blue().bold(),
        total_issues
    );

    // Phase 2: Repair missing directories (no download needed)
    if !missing_dirs.is_empty() {
        utils::info(&format!(
            "Restoring {} missing director{}...",
            missing_dir_count,
            if missing_dir_count == 1 { "y" } else { "ies" }
        ));
        for dir in &missing_dirs {
            let dir_path = target.join(dir);
            utils::ensure_dir(&dir_path)?;
            let gitkeep = dir_path.join(".gitkeep");
            if !gitkeep.exists() {
                std::fs::write(&gitkeep, "")?;
            }
            utils::success(&format!("Restored {dir}/"));
        }
    }

    // Phase 3: Download framework and restore missing files if needed
    if needs_download {
        utils::info("Downloading framework to restore missing files...");
        let release = download::get_latest_release()?;
        println!(
            "  {} {}",
            "Using version:".dimmed(),
            release.tag_name.green()
        );

        let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
        let zip_path = temp_dir.path().join("straymark.zip");

        download::download_zip(&release.zip_url, &zip_path)?;

        restore_missing_files(&zip_path, &target)?;
    }

    // Phase 4: Recalculate checksums
    utils::info("Updating checksums...");
    let version = load_current_version(&target);
    save_checksums(&target, &version)?;

    println!();
    utils::success("StrayMark repaired successfully!");
    println!();

    Ok(())
}

/// Check if any framework files are missing and a download is needed
fn check_needs_download(target: &Path) -> bool {
    let checks = [
        ".straymark/config.yml",
        ".straymark/dist-manifest.yml",
        "STRAYMARK.md",
    ];

    // Check essential files
    for file in &checks {
        if !target.join(file).exists() {
            return true;
        }
    }

    // Check if templates dir is empty or missing
    let templates_dir = target.join(".straymark/templates");
    if !templates_dir.exists() {
        return true;
    }
    if let Ok(entries) = std::fs::read_dir(&templates_dir) {
        if entries
            .flatten()
            .filter(|e| {
                e.path().is_file()
                    && e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        == Some("md")
            })
            .count()
            == 0
        {
            return true;
        }
    }

    // Check governance docs
    let governance_dir = target.join(".straymark/00-governance");
    if governance_dir.exists() {
        let has_md_files = std::fs::read_dir(&governance_dir)
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .any(|e| {
                        e.path().is_file()
                            && e.path()
                                .extension()
                                .and_then(|ext| ext.to_str())
                                == Some("md")
                    })
            })
            .unwrap_or(false);
        if !has_md_files {
            return true;
        }
    }

    // Check injection targets declared in the local manifest (fw-4.16.2 / cli-3.14.1).
    // Before this version, `repair` only triggered the framework download when one
    // of the essential framework files was missing — a single deleted injection
    // target (e.g. `AGENTS.md` while `STRAYMARK.md` remained intact) was a silent
    // no-op. Walking `manifest.injections` here brings the download phase in line
    // with the restore phase, which now checks each target individually.
    let manifest_path = target.join(".straymark/dist-manifest.yml");
    if let Ok(manifest) = DistManifest::load(&manifest_path) {
        for injection in &manifest.injections {
            if !target.join(&injection.target).exists() {
                return true;
            }
        }
    }

    false
}

/// Restore missing files from the release ZIP
fn restore_missing_files(zip_path: &Path, target: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;

    // Find prefix in ZIP
    let mut prefix = String::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with("dist-manifest.yml") {
            if let Some(pos) = name.find("dist-manifest.yml") {
                prefix = name[..pos].to_string();
            }
            break;
        }
    }

    // Read manifest from ZIP
    let mut manifest_content = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name().ends_with("dist-manifest.yml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content)?;
            manifest_content = Some(content);
            break;
        }
    }

    let manifest_str = manifest_content.context("dist-manifest.yml not found in release ZIP")?;
    let manifest = DistManifest::from_str(&manifest_str)?;

    let mut restored = 0;

    // Extract only files that are missing
    for pattern in &manifest.files {
        restored += extract_missing_files(&mut archive, &prefix, pattern, target)?;
    }

    // Restore each missing injection target individually (fw-4.16.2 / cli-3.14.1).
    // Before this version, this block was gated on `!STRAYMARK.md.exists()` — which
    // meant a single missing injection target (e.g. `AGENTS.md` while `STRAYMARK.md`
    // remained intact) was silently skipped, contradicting the `straymark repair`
    // promise to restore broken installations. The per-target presence check below
    // brings repair in line with the documented intent and with what `straymark
    // update-framework` already does for the same set of files.
    let missing_injections: Vec<&Injection> = manifest
        .injections
        .iter()
        .filter(|inj| !target.join(&inj.target).exists())
        .collect();

    if !missing_injections.is_empty() {
        // Load only the templates we will actually re-inject.
        let mut templates: HashMap<String, String> = HashMap::new();
        for injection in &missing_injections {
            if templates.contains_key(&injection.template) {
                continue;
            }
            let zip_entry_name = format!("{}{}", prefix, injection.template);
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                if entry.name() == zip_entry_name {
                    let mut content = String::new();
                    std::io::Read::read_to_string(&mut entry, &mut content)?;
                    templates.insert(injection.template.clone(), content);
                    break;
                }
            }
        }

        // Re-inject only the missing ones.
        for injection in &missing_injections {
            let template_content = match templates.get(&injection.template) {
                Some(content) => content,
                None => continue,
            };

            let embed_content = if let Some(embed_file) = &injection.embed {
                let embed_path = target.join(embed_file);
                if embed_path.exists() {
                    Some(std::fs::read_to_string(&embed_path)?)
                } else {
                    continue;
                }
            } else {
                None
            };

            let target_path = target.join(&injection.target);
            inject::inject_directive(&target_path, template_content, embed_content.as_deref())?;
            utils::success(&format!("Restored {}", injection.target));
            restored += 1;
        }
    }

    // Save manifest
    let manifest_path = target.join(".straymark/dist-manifest.yml");
    if !manifest_path.exists() {
        let content = manifest.to_yaml()?;
        std::fs::write(&manifest_path, content)?;
        utils::success("Restored dist-manifest.yml");
        restored += 1;
    }

    if restored > 0 {
        utils::info(&format!("Restored {} file(s) from framework", restored));
    }

    Ok(())
}

/// Extract files from ZIP that don't exist in the target
fn extract_missing_files(
    archive: &mut zip::ZipArchive<std::fs::File>,
    prefix: &str,
    pattern: &str,
    target: &Path,
) -> Result<usize> {
    let pattern_with_prefix = format!("{}{}", prefix, pattern);
    let mut count = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        let matches = if pattern.ends_with('/') {
            name.starts_with(&pattern_with_prefix)
        } else {
            name == pattern_with_prefix
        };

        if matches && !entry.is_dir() {
            let relative = &name[prefix.len()..];
            let dest = target.join(relative);

            // Only restore if the file doesn't already exist
            if !dest.exists() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&dest)?;
                std::io::copy(&mut entry, &mut outfile)?;
                utils::success(&format!("Restored {relative}"));
                count += 1;
            }
        }
    }

    Ok(count)
}

fn load_current_version(target: &Path) -> String {
    let manifest_path = target.join(".straymark/dist-manifest.yml");
    match DistManifest::load(&manifest_path) {
        Ok(m) => m.version,
        Err(_) => "unknown".to_string(),
    }
}

fn save_checksums(target: &Path, version: &str) -> Result<()> {
    let mut checksums = Checksums {
        version: version.to_string(),
        files: std::collections::HashMap::new(),
    };

    if let Ok(entries) = walkdir(target.join(".straymark")) {
        for entry in entries {
            if let Some(hash) = utils::file_hash(&entry) {
                let relative = entry
                    .strip_prefix(target)
                    .unwrap_or(&entry)
                    .display()
                    .to_string();
                checksums.files.insert(relative, hash);
            }
        }
    }

    let straymark_path = target.join("STRAYMARK.md");
    if let Some(hash) = utils::file_hash(&straymark_path) {
        checksums.files.insert("STRAYMARK.md".to_string(), hash);
    }

    checksums.save(target)?;
    Ok(())
}

fn walkdir(dir: PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir(path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Injection;
    use tempfile::TempDir;

    fn write_manifest(target: &Path, injection_targets: &[&str]) {
        let manifest_path = target.join(".straymark/dist-manifest.yml");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        let injections_yaml: String = injection_targets
            .iter()
            .map(|t| {
                format!(
                    "  - target: {t}\n    template: dist-templates/directives/dummy.md\n"
                )
            })
            .collect();
        let body = format!(
            "version: \"4.16.2\"\ndescription: \"test\"\nrepository: \"x\"\nfiles: []\ninjections:\n{injections_yaml}"
        );
        std::fs::write(&manifest_path, body).unwrap();
    }

    fn write_essential_framework_files(target: &Path) {
        // Files `check_needs_download` requires to be present in order to NOT
        // already short-circuit to true for other reasons (we want to isolate
        // the "missing injection target" branch).
        std::fs::create_dir_all(target.join(".straymark/templates")).unwrap();
        std::fs::write(target.join(".straymark/templates/template.md"), "x").unwrap();
        std::fs::create_dir_all(target.join(".straymark/00-governance")).unwrap();
        std::fs::write(target.join(".straymark/00-governance/AGENT-RULES.md"), "x").unwrap();
        std::fs::write(target.join(".straymark/config.yml"), "language: en\n").unwrap();
        std::fs::write(target.join("STRAYMARK.md"), "# StrayMark\n").unwrap();
    }

    #[test]
    fn check_needs_download_true_when_injection_target_missing() {
        let dir = TempDir::new().unwrap();
        write_essential_framework_files(dir.path());
        write_manifest(dir.path(), &["AGENTS.md", "CLAUDE.md"]);
        // Pre-create one target, leave the other missing.
        std::fs::write(dir.path().join("AGENTS.md"), "existing\n").unwrap();

        assert!(
            check_needs_download(dir.path()),
            "must trigger download when an injection target listed in the manifest is missing"
        );
    }

    #[test]
    fn check_needs_download_false_when_all_injection_targets_present() {
        let dir = TempDir::new().unwrap();
        write_essential_framework_files(dir.path());
        write_manifest(dir.path(), &["AGENTS.md", "CLAUDE.md"]);
        std::fs::write(dir.path().join("AGENTS.md"), "x").unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "x").unwrap();

        assert!(
            !check_needs_download(dir.path()),
            "must NOT trigger download when nothing is missing"
        );
    }

    #[test]
    fn check_needs_download_true_when_essential_file_missing_independent_of_injections() {
        // Regression: pre-fw-4.16.2 behavior must still hold for the essential-file path.
        let dir = TempDir::new().unwrap();
        write_essential_framework_files(dir.path());
        write_manifest(dir.path(), &[]);
        std::fs::remove_file(dir.path().join("STRAYMARK.md")).unwrap();

        assert!(
            check_needs_download(dir.path()),
            "missing STRAYMARK.md must still trigger a download"
        );
    }

    #[test]
    fn missing_injections_filter_identifies_per_target_gaps() {
        // Exercises the same filter expression used in `restore_missing_files`
        // (and verifies it does not regress to the pre-fw-4.16.2 STRAYMARK.md gate).
        let dir = TempDir::new().unwrap();
        write_essential_framework_files(dir.path());
        std::fs::write(dir.path().join("AGENTS.md"), "x").unwrap();
        // CLAUDE.md intentionally missing.

        let injections = vec![
            Injection {
                target: "AGENTS.md".to_string(),
                template: "dist-templates/directives/AGENTS.md".to_string(),
                embed: None,
            },
            Injection {
                target: "CLAUDE.md".to_string(),
                template: "dist-templates/directives/CLAUDE.md".to_string(),
                embed: None,
            },
        ];

        let missing: Vec<&Injection> = injections
            .iter()
            .filter(|inj| !dir.path().join(&inj.target).exists())
            .collect();

        assert_eq!(missing.len(), 1, "exactly one target should be flagged missing");
        assert_eq!(missing[0].target, "CLAUDE.md");
    }

    #[test]
    fn missing_injections_filter_is_empty_when_all_present() {
        let dir = TempDir::new().unwrap();
        write_essential_framework_files(dir.path());
        std::fs::write(dir.path().join("AGENTS.md"), "x").unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "x").unwrap();

        let injections = vec![
            Injection {
                target: "AGENTS.md".to_string(),
                template: "x".to_string(),
                embed: None,
            },
            Injection {
                target: "CLAUDE.md".to_string(),
                template: "x".to_string(),
                embed: None,
            },
        ];

        let missing: Vec<&Injection> = injections
            .iter()
            .filter(|inj| !dir.path().join(&inj.target).exists())
            .collect();

        assert!(missing.is_empty());
    }
}
