use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::{Path, PathBuf};

use crate::config::Checksums;
use crate::download;
use crate::inject;
use crate::manifest::DistManifest;
use crate::utils;

pub fn run() -> Result<()> {
    let target = std::env::current_dir().context("Failed to get current directory")?;

    // Verify StrayMark is installed
    if !target.join(".straymark").exists() {
        bail!(
            ".straymark/ not found. Use {} to initialize first.",
            "straymark init".yellow()
        );
    }

    // Load current checksums
    let current_checksums = Checksums::load(&target)?;
    if !current_checksums.version.is_empty() {
        utils::info(&format!("Current version: {}", current_checksums.version));
    }

    // Fetch latest release
    utils::info("Checking for updates...");
    let release = download::get_latest_release()?;
    let display_version = download::strip_tag_prefix(&release.tag_name);
    println!(
        "  {} {}",
        "Latest version:".dimmed(),
        release.tag_name.green()
    );

    // Compare versions — skip if already up to date
    let current_ver_str = download::strip_tag_prefix(&current_checksums.version);
    if !current_ver_str.is_empty() {
        if let (Ok(current), Ok(latest)) = (
            semver::Version::parse(current_ver_str),
            semver::Version::parse(display_version),
        ) {
            if latest <= current {
                utils::success(&format!(
                    "Framework is already at the latest version ({})",
                    current_checksums.version
                ));
                return Ok(());
            }
        }
    }

    // Download ZIP
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let zip_path = temp_dir.path().join("straymark.zip");

    utils::info("Downloading...");
    download::download_zip(&release.zip_url, &zip_path)?;

    // Extract to temp directory for comparison
    let extract_dir = temp_dir.path().join("extracted");
    std::fs::create_dir_all(&extract_dir)?;
    extract_all(&zip_path, &extract_dir)?;

    // Find source root within extracted content
    let source_root = find_source_root(&extract_dir)?;

    // Load manifest from extracted release
    let manifest = DistManifest::load(&source_root.join("dist-manifest.yml"))?;

    // Update framework files
    utils::info("Updating framework files...");
    let stats = update_files(&target, &source_root, &manifest, &current_checksums)?;

    // Update directive injections
    utils::info("Updating AI agent directives...");
    inject_directives(&target, &source_root, &manifest)?;

    // Save manifest locally for future remove operations
    save_local_manifest(&target, &manifest)?;

    // Save new checksums
    save_checksums(&target, &release.tag_name)?;

    // Print summary
    println!();
    utils::success("StrayMark framework updated successfully!");
    println!("  Files updated: {}", stats.updated);
    println!("  Files skipped (user-modified): {}", stats.skipped);
    println!("  Files added: {}", stats.added);

    Ok(())
}

struct UpdateStats {
    updated: usize,
    skipped: usize,
    added: usize,
}

/// Update files, respecting user modifications
fn update_files(
    target: &Path,
    source_root: &Path,
    manifest: &DistManifest,
    checksums: &Checksums,
) -> Result<UpdateStats> {
    let mut stats = UpdateStats {
        updated: 0,
        skipped: 0,
        added: 0,
    };

    // Walk extracted files
    let entries = walkdir(source_root.to_path_buf())?;

    for source_path in entries {
        let relative = source_path
            .strip_prefix(source_root)
            .unwrap_or(&source_path)
            .display()
            .to_string()
            .replace('\\', "/");

        // Only touch files declared by the release manifest. The release ZIP also
        // ships internal artifacts (`dist-manifest.yml`, `dist-templates/`) that
        // the CLI consumes from the temp dir but must never copy into the
        // adopter's project. Mirrors `init.rs::extract_matching_files`.
        if !matches_manifest(&relative, &manifest.files) {
            continue;
        }

        // Skip user-generated documents
        if utils::is_user_document(&source_path) {
            continue;
        }

        let target_path = target.join(&relative);

        if !target_path.exists() {
            // New file — just copy it
            if let Some(parent) = target_path.parent() {
                utils::ensure_dir(parent)?;
            }
            std::fs::copy(&source_path, &target_path)?;
            stats.added += 1;
            continue;
        }

        // File exists — check if user modified it
        let current_hash = utils::file_hash(&target_path).unwrap_or_default();
        let original_hash = checksums
            .files
            .get(&relative)
            .cloned()
            .unwrap_or_default();

        if current_hash == original_hash || original_hash.is_empty() {
            // User hasn't modified it (or no previous hash) — safe to overwrite
            std::fs::copy(&source_path, &target_path)?;
            stats.updated += 1;
        } else {
            // User modified it — prompt for action
            let new_hash = utils::file_hash(&source_path).unwrap_or_default();
            if current_hash == new_hash {
                // Same content, no action needed
                continue;
            }

            utils::warn(&format!("User-modified file: {}", relative));
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What would you like to do?")
                .items(&["Keep my version", "Use new version", "Backup mine + use new"])
                .default(0)
                .interact()?;

            match selection {
                0 => {
                    stats.skipped += 1;
                }
                1 => {
                    std::fs::copy(&source_path, &target_path)?;
                    stats.updated += 1;
                }
                2 => {
                    let backup = target_path.with_extension("md.bak");
                    std::fs::copy(&target_path, &backup)?;
                    std::fs::copy(&source_path, &target_path)?;
                    stats.updated += 1;
                    utils::info(&format!("Backup saved: {}", backup.display()));
                }
                _ => {
                    stats.skipped += 1;
                }
            }
        }
    }

    Ok(stats)
}

/// Inject directives based on manifest and templates from the release
fn inject_directives(target: &Path, source_root: &Path, manifest: &DistManifest) -> Result<()> {
    for injection in &manifest.injections {
        let target_path = target.join(&injection.target);

        // In update mode, only update targets that already exist
        if !target_path.exists() {
            continue;
        }

        let template_path = source_root.join(&injection.template);
        let template_content = match std::fs::read_to_string(&template_path) {
            Ok(content) => content,
            Err(_) => {
                utils::warn(&format!(
                    "Template not found: {}",
                    injection.template
                ));
                continue;
            }
        };

        let embed_content = if let Some(embed_file) = &injection.embed {
            // Use the embed file from the release, not the local one
            let embed_path = source_root.join(embed_file);
            if embed_path.exists() {
                Some(std::fs::read_to_string(&embed_path).with_context(|| {
                    format!("Failed to read embed file: {}", embed_path.display())
                })?)
            } else {
                utils::warn(&format!(
                    "Embed file not found in release: {} (skipping {})",
                    embed_file, injection.target
                ));
                continue;
            }
        } else {
            None
        };

        inject::inject_directive(&target_path, &template_content, embed_content.as_deref())?;
    }

    Ok(())
}

/// Save the manifest locally for future remove operations
fn save_local_manifest(target: &Path, manifest: &DistManifest) -> Result<()> {
    let manifest_path = target.join(".straymark/dist-manifest.yml");
    let content = manifest.to_yaml()?;
    std::fs::write(&manifest_path, content)
        .context("Failed to save local dist-manifest.yml")?;
    Ok(())
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

fn extract_all(zip_path: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let path = dest.join(entry.name());

        if entry.is_dir() {
            std::fs::create_dir_all(&path)?;
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&path)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

fn find_source_root(extract_dir: &Path) -> Result<PathBuf> {
    // Check if dist-manifest.yml is directly in extract_dir
    if extract_dir.join("dist-manifest.yml").exists() {
        return Ok(extract_dir.to_path_buf());
    }

    // Check one level deep (GitHub ZIP archives nest in a directory)
    for entry in std::fs::read_dir(extract_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("dist-manifest.yml").exists() {
            return Ok(path);
        }
    }

    bail!("Could not find dist-manifest.yml in extracted archive");
}

/// Match a relative path (POSIX-style) against the manifest's `files` whitelist.
/// Patterns ending in `/` match any path under that directory; otherwise exact match.
fn matches_manifest(relative: &str, files: &[String]) -> bool {
    files.iter().any(|pat| {
        if pat.ends_with('/') {
            relative.starts_with(pat.as_str())
        } else {
            relative == pat
        }
    })
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
    use super::matches_manifest;

    fn manifest_files() -> Vec<String> {
        // Matches `dist/dist-manifest.yml` (fw-4.3.0).
        vec![
            ".straymark/".to_string(),
            "STRAYMARK.md".to_string(),
            ".claude/skills/".to_string(),
            ".gemini/skills/".to_string(),
            ".agent/workflows/".to_string(),
            ".github/workflows/docs-validation.yml".to_string(),
        ]
    }

    #[test]
    fn package_artifacts_are_rejected() {
        let files = manifest_files();
        // Regression for the bug where `straymark update` deposited these in the
        // adopter project. Both live at the ZIP root, neither is in `manifest.files`.
        assert!(!matches_manifest("dist-manifest.yml", &files));
        assert!(!matches_manifest("dist-templates/directives/CLAUDE.md", &files));
    }

    #[test]
    fn declared_files_and_directories_match() {
        let files = manifest_files();
        assert!(matches_manifest("STRAYMARK.md", &files));
        assert!(matches_manifest(".straymark/00-governance/AGENT-RULES.md", &files));
        assert!(matches_manifest(".claude/skills/straymark-new/SKILL.md", &files));
        assert!(matches_manifest(
            ".github/workflows/docs-validation.yml",
            &files
        ));
    }

    #[test]
    fn undeclared_paths_are_rejected() {
        let files = manifest_files();
        assert!(!matches_manifest("README.md", &files));
        assert!(!matches_manifest(".github/workflows/release-cli.yml", &files));
        assert!(!matches_manifest(".claude/agents/foo.md", &files));
    }
}
