use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::config::Checksums;
use crate::download;
use crate::inject;
use crate::manifest::DistManifest;
use crate::utils;

pub fn run(
    path: &str,
    install_hooks: bool,
    install_merge_driver: bool,
    skip_merge_driver: bool,
) -> Result<()> {
    let target = PathBuf::from(path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path));

    println!(
        "{} StrayMark in {}",
        "Initializing".cyan().bold(),
        target.display()
    );

    // Check if already initialized
    if target.join(".straymark").exists() {
        bail!(
            ".straymark/ already exists. Use {} to update.",
            "straymark update".yellow()
        );
    }

    // Download latest release
    utils::info("Fetching latest release...");
    let release = download::get_latest_release()?;
    println!(
        "  {} {}",
        "Found version:".dimmed(),
        release.tag_name.green()
    );

    // Download ZIP to temp file
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
    let zip_path = temp_dir.path().join("straymark.zip");

    utils::info("Downloading...");
    download::download_zip(&release.zip_url, &zip_path)?;

    // Extract files according to manifest
    utils::info("Extracting files...");
    let (manifest, templates) = extract_distribution(&zip_path, &target)?;

    // Create empty directory structure with .gitkeep
    create_empty_dirs(&target)?;

    // Inject into directive files
    utils::info("Configuring AI agent directives...");
    inject_directives(&target, &manifest, &templates)?;

    // Save manifest locally for future remove operations
    save_local_manifest(&target, &manifest)?;

    // Save checksums
    save_initial_checksums(&target, &release.tag_name)?;

    // Auto-adoption safeguard S2: stamp the install with a provenance marker so
    // every command can tell an installed project apart from the framework
    // distribution source (dist/, caught by S1) or a test fixture. The
    // `framework_version` records the pinned release the project governs itself
    // with. Absent provenance is tolerated elsewhere (legacy installs).
    write_provenance(&target, &manifest.version, &release.tag_name)?;

    // Install pre-PR hook (opt-in via --hooks).
    if install_hooks {
        match install_pre_pr_hook(&target) {
            Ok(installed) => {
                if installed {
                    println!(
                        "  {} pre-PR hook installed at {}",
                        "✓".green().bold(),
                        ".git/hooks/pre-push".dimmed()
                    );
                }
            }
            Err(e) => {
                utils::warn(&format!(
                    "Failed to install pre-PR hook: {}. Continuing without it.",
                    e
                ));
            }
        }
    }

    // Wire the follow-ups registry merge driver (GH #391). Explicit flag wins;
    // otherwise offer it, but only when someone is there to answer — `init` runs
    // in CI and provisioning scripts, where a blocking prompt is a hang.
    let wants_driver = if skip_merge_driver {
        false
    } else if install_merge_driver {
        true
    } else {
        prompt_for_merge_driver(&target)
    };
    if wants_driver {
        match crate::commands::followups::install_merge_driver::install(&target) {
            Ok(outcome) => {
                if outcome.attributes_added || outcome.config_added {
                    println!(
                        "  {} follow-ups merge driver wired ({} + {})",
                        "✓".green().bold(),
                        ".gitattributes".dimmed(),
                        "git config".dimmed()
                    );
                }
            }
            Err(e) => {
                utils::warn(&format!(
                    "Could not wire the merge driver: {e}. Continuing without it;                      run `straymark followups install-merge-driver` later."
                ));
            }
        }
    }

    // Print summary
    println!();
    utils::success("StrayMark initialized successfully!");
    println!();
    println!("  {}", "Next steps:".bold());
    println!("    1. Review .straymark/config.yml for language settings");
    println!("    2. Check STRAYMARK.md for governance rules");
    println!(
        "    3. Run {} to validate your setup",
        "straymark validate".cyan()
    );
    println!(
        "    4. Commit: {}",
        "git add .straymark/ STRAYMARK.md && git commit -m \"chore: adopt StrayMark\"".dimmed()
    );

    Ok(())
}

/// Extract distributable files from the release ZIP and read templates into memory
fn extract_distribution(
    zip_path: &Path,
    target: &Path,
) -> Result<(DistManifest, HashMap<String, String>)> {
    let file = std::fs::File::open(zip_path).context("Failed to open ZIP file")?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;

    // Find the manifest inside the ZIP (it may be in a subdirectory like straymark-v2.0.0/)
    let mut manifest_content = None;
    let mut prefix = String::new();

    // First pass: find the manifest entry index
    let mut manifest_index = None;
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.ends_with("dist-manifest.yml") {
            if let Some(pos) = name.find("dist-manifest.yml") {
                prefix = name[..pos].to_string();
            }
            manifest_index = Some(i);
            break;
        }
    }

    // Second pass: read manifest content
    if let Some(idx) = manifest_index {
        let mut content = String::new();
        let mut entry = archive.by_index(idx)?;
        std::io::Read::read_to_string(&mut entry, &mut content)?;
        manifest_content = Some(content);
    }

    let manifest_str = manifest_content.context("dist-manifest.yml not found in release ZIP")?;
    let manifest = DistManifest::from_str(&manifest_str)?;

    // Extract each file listed in manifest
    for pattern in &manifest.files {
        extract_matching_files(&mut archive, &prefix, pattern, target)?;
    }

    // Read template files from ZIP into memory
    let mut templates: HashMap<String, String> = HashMap::new();
    for injection in &manifest.injections {
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

    Ok((manifest, templates))
}

/// Extract files from ZIP matching a manifest pattern
fn extract_matching_files(
    archive: &mut zip::ZipArchive<std::fs::File>,
    prefix: &str,
    pattern: &str,
    target: &Path,
) -> Result<()> {
    let pattern_with_prefix = format!("{}{}", prefix, pattern);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        // Check if this entry matches the pattern
        let matches = if pattern.ends_with('/') {
            // Directory pattern: match anything inside it
            name.starts_with(&pattern_with_prefix)
        } else {
            // Exact file match
            name == pattern_with_prefix
        };

        if matches && !entry.is_dir() {
            // Compute relative path (strip the prefix)
            let relative = &name[prefix.len()..];
            let dest = target.join(relative);

            // Create parent directories
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write file
            let mut outfile = std::fs::File::create(&dest)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
    }

    Ok(())
}

/// Create the empty directory structure with .gitkeep files
fn create_empty_dirs(target: &Path) -> Result<()> {
    let dirs = [
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
        ".straymark/00-governance/exceptions",
    ];

    for dir in &dirs {
        let dir_path = target.join(dir);
        utils::ensure_dir(&dir_path)?;
        let gitkeep = dir_path.join(".gitkeep");
        if !gitkeep.exists() {
            std::fs::write(&gitkeep, "")?;
        }
    }

    Ok(())
}

/// Inject StrayMark directives based on manifest and templates
fn inject_directives(
    target: &Path,
    manifest: &DistManifest,
    templates: &HashMap<String, String>,
) -> Result<()> {
    for injection in &manifest.injections {
        let template_content = match templates.get(&injection.template) {
            Some(content) => content,
            None => {
                utils::warn(&format!(
                    "Template not found in ZIP: {}",
                    injection.template
                ));
                continue;
            }
        };

        let embed_content = if let Some(embed_file) = &injection.embed {
            let embed_path = target.join(embed_file);
            if embed_path.exists() {
                Some(std::fs::read_to_string(&embed_path).with_context(|| {
                    format!("Failed to read embed file: {}", embed_path.display())
                })?)
            } else {
                utils::warn(&format!(
                    "Embed file not found: {} (skipping {})",
                    embed_file, injection.target
                ));
                continue;
            }
        } else {
            None
        };

        let target_path = target.join(&injection.target);
        inject::inject_directive(
            &target_path,
            template_content,
            embed_content.as_deref(),
        )?;
        utils::success(&format!("Configured {}", injection.target));
    }

    Ok(())
}

/// Ask whether to wire the merge driver, when a human is present to answer.
///
/// Returns false without prompting when stdin is not a TTY or the project is
/// not a git repo — `init` is routinely scripted, and a prompt that blocks a
/// provisioning run is worse than the friction it saves.
fn prompt_for_merge_driver(target: &Path) -> bool {
    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() || !target.join(".git").exists() {
        return false;
    }

    println!();
    println!(
        "  {} The follow-ups registry is a guaranteed conflict between parallel PRs,",
        "?".yellow().bold()
    );
    println!("    and resolving it by hand silently reverts closures (GH #391).");
    println!(
        "    StrayMark ships a git merge driver that resolves it structurally."
    );

    dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Wire it into this clone now?")
        .default(true)
        .interact()
        .unwrap_or(false)
}

/// Save the manifest locally for future remove operations
fn save_local_manifest(target: &Path, manifest: &DistManifest) -> Result<()> {
    let manifest_path = target.join(".straymark/dist-manifest.yml");
    let content = manifest.to_yaml()?;
    std::fs::write(&manifest_path, content)
        .context("Failed to save local dist-manifest.yml")?;
    Ok(())
}

/// Save initial checksums for all framework files
fn save_initial_checksums(target: &Path, version: &str) -> Result<()> {
    let mut checksums = Checksums {
        version: version.to_string(),
        files: std::collections::HashMap::new(),
    };

    // Walk .straymark/ and hash all files
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

    // Also hash STRAYMARK.md
    let straymark_path = target.join("STRAYMARK.md");
    if let Some(hash) = utils::file_hash(&straymark_path) {
        checksums.files.insert("STRAYMARK.md".to_string(), hash);
    }

    checksums.save(target)?;
    Ok(())
}

/// Install `.straymark/hooks/pre-pr.sh` as `.git/hooks/pre-push`. Returns
/// `Ok(true)` on success, `Ok(false)` if the project is not a git repo
/// (so the caller can skip silently). Errors only on actual filesystem
/// failures (the hook source is missing, the destination can't be written,
/// etc.).
fn install_pre_pr_hook(target: &Path) -> Result<bool> {
    let git_dir = target.join(".git");
    if !git_dir.exists() {
        utils::warn(
            "Skipping --hooks: not a git repository (no .git/ directory). Run 'git init' first.",
        );
        return Ok(false);
    }

    let source = target.join(".straymark/hooks/pre-pr.sh");
    if !source.exists() {
        bail!(
            "pre-PR hook source not found at {}. The framework distribution may be incomplete.",
            source.display()
        );
    }

    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("Failed to create {}", hooks_dir.display()))?;

    let dest = hooks_dir.join("pre-push");
    if dest.exists() {
        utils::warn(&format!(
            "Refusing to overwrite existing hook at {}. Move or remove it, then re-run with --hooks.",
            dest.display()
        ));
        return Ok(false);
    }

    std::fs::copy(&source, &dest)
        .with_context(|| format!("Failed to copy hook to {}", dest.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }
    Ok(true)
}

/// Render the `.provenance.yml` body (auto-adoption safeguard S2). Pure so it
/// can be unit-tested without touching the filesystem or the clock.
fn render_provenance(framework_version: &str, installed_at: &str, source_release: &str) -> String {
    format!(
        "# StrayMark install provenance (auto-adoption safeguard S2). Written by\n\
         # `straymark init`. Marks this .straymark/ as an installed project — as\n\
         # opposed to the framework distribution source (dist/) or a test fixture —\n\
         # and records the pinned framework release the project governs itself with.\n\
         # Do not edit by hand.\n\
         role: installed-project\n\
         framework_version: \"{framework_version}\"\n\
         installed_at: \"{installed_at}\"\n\
         source_release: \"{source_release}\"\n",
    )
}

/// Write `.straymark/.provenance.yml` for a freshly-installed project (S2).
fn write_provenance(target: &Path, framework_version: &str, source_release: &str) -> Result<()> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let body = render_provenance(framework_version, &today, source_release);
    let dest = target.join(".straymark").join(".provenance.yml");
    std::fs::write(&dest, body)
        .with_context(|| format!("Failed to write {}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    #[test]
    fn render_provenance_is_installed_project_and_parses() {
        let body = render_provenance("4.35.0", "2026-07-13", "fw-4.35.0");
        // The provenance role reader (utils) must see installed-project.
        assert!(body.contains("role: installed-project"));
        assert!(body.contains("framework_version: \"4.35.0\""));
        assert!(body.contains("source_release: \"fw-4.35.0\""));
        // Round-trips through the utils line-scanner used at resolution time.
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
        std::fs::write(tmp.path().join(".straymark/.provenance.yml"), &body).unwrap();
        assert_eq!(
            crate::utils::provenance_role(tmp.path()).as_deref(),
            Some("installed-project")
        );
    }
}

#[cfg(test)]
mod hook_tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_tempdir_with_hook_source(tmp: &Path) {
        std::fs::create_dir_all(tmp.join(".git/objects")).unwrap();
        std::fs::create_dir_all(tmp.join(".straymark/hooks")).unwrap();
        std::fs::write(
            tmp.join(".straymark/hooks/pre-pr.sh"),
            "#!/usr/bin/env bash\necho hook\n",
        )
        .unwrap();
    }

    #[test]
    fn install_pre_pr_hook_copies_and_makes_executable() {
        let tmp = TempDir::new().unwrap();
        setup_tempdir_with_hook_source(tmp.path());

        let installed = install_pre_pr_hook(tmp.path()).unwrap();
        assert!(installed);

        let dest = tmp.path().join(".git/hooks/pre-push");
        assert!(dest.exists());
        let body = std::fs::read_to_string(&dest).unwrap();
        assert!(body.contains("hook"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&dest).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111, "hook must be executable");
        }
    }

    #[test]
    fn install_pre_pr_hook_skips_when_not_a_git_repo() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark/hooks")).unwrap();
        std::fs::write(
            tmp.path().join(".straymark/hooks/pre-pr.sh"),
            "#!/usr/bin/env bash\nexit 0\n",
        )
        .unwrap();

        let installed = install_pre_pr_hook(tmp.path()).unwrap();
        assert!(!installed);
    }

    #[test]
    fn install_pre_pr_hook_refuses_to_overwrite_existing() {
        let tmp = TempDir::new().unwrap();
        setup_tempdir_with_hook_source(tmp.path());
        let dest = tmp.path().join(".git/hooks/pre-push");
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::write(&dest, "#!/bin/sh\necho existing\n").unwrap();

        let installed = install_pre_pr_hook(tmp.path()).unwrap();
        assert!(!installed);

        // Original content is preserved.
        let body = std::fs::read_to_string(&dest).unwrap();
        assert!(body.contains("existing"));
    }

    #[test]
    fn install_pre_pr_hook_errors_when_source_missing() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".git/objects")).unwrap();
        // Note: no .straymark/hooks/pre-pr.sh

        let result = install_pre_pr_hook(tmp.path());
        assert!(result.is_err());
        let msg = format!("{:?}", result.unwrap_err());
        assert!(msg.contains("pre-PR hook source not found"));
    }
}

/// Simple recursive directory walker
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
