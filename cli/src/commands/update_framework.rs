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

    // Drop paths this release no longer distributes. Without this, retiring an
    // agent surface would leave it in every existing installation forever —
    // update_files() only ever copies, and only `straymark remove` deletes.
    let prune = prune_retired(&target, &manifest, &current_checksums)?;

    // Save manifest locally for future remove operations
    save_local_manifest(&target, &manifest)?;

    // Save new checksums. The distributed-hash overrides are load-bearing
    // (GH #388): user-kept files must carry the release hash as their
    // baseline, not the on-disk one.
    save_checksums(&target, &release.tag_name, &stats.distributed_hashes)?;

    // Print summary
    println!();
    utils::success("StrayMark framework updated successfully!");
    println!("  Files updated: {}", stats.updated);
    println!("  Files skipped (user-modified): {}", stats.skipped);
    for path in &stats.skipped_files {
        println!("    - {} (kept your version)", path.dimmed());
    }
    println!("  Files added: {}", stats.added);
    if !prune.removed.is_empty() {
        println!("  Files retired: {}", prune.removed.len());
    }
    for path in &prune.kept_modified {
        println!("    - {} (retired upstream, kept — you modified it)", path.dimmed());
    }
    for path in &prune.kept_foreign {
        println!("    - {} (retired upstream, kept — not installed by StrayMark)", path.dimmed());
    }

    Ok(())
}

struct UpdateStats {
    updated: usize,
    skipped: usize,
    added: usize,
    /// Relative paths the operator chose to keep. GH #388: named in the final
    /// report — a bare count is what let the config.yml overwrite go unnoticed.
    skipped_files: Vec<String>,
    /// Distributed (release) hash per relative path, for every file that now
    /// has a known baseline: copied-over, newly added, or user-kept. GH #388:
    /// the checksum store must record the *release* hash for user-kept files,
    /// not the on-disk one, or the next update sees them as unmodified.
    distributed_hashes: std::collections::HashMap<String, String>,
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
        skipped_files: Vec::new(),
        distributed_hashes: std::collections::HashMap::new(),
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
            stats.distributed_hashes.insert(
                relative.clone(),
                utils::file_hash(&source_path).unwrap_or_default(),
            );
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
            stats.distributed_hashes.insert(
                relative.clone(),
                utils::file_hash(&source_path).unwrap_or_default(),
            );
        } else {
            // User modified it — prompt for action
            let new_hash = utils::file_hash(&source_path).unwrap_or_default();
            if current_hash == new_hash {
                // Same content, no action needed
                stats
                    .distributed_hashes
                    .insert(relative.clone(), new_hash);
                continue;
            }

            utils::warn(&format!("User-modified file: {}", relative));
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What would you like to do?")
                .items(&["Keep my version", "Use new version", "Backup mine + use new"])
                .default(0)
                .interact()?;

            // GH #388: the baseline for the next update is the release hash in
            // every case. For "Keep my version" this is the load-bearing part:
            // the on-disk content stays the user's, so stamping the store with
            // the disk hash would launder the modification into the baseline
            // and the next update would overwrite it without prompting.
            stats
                .distributed_hashes
                .insert(relative.clone(), new_hash.clone());

            match selection {
                0 => {
                    stats.skipped += 1;
                    stats.skipped_files.push(relative.clone());
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
                    stats.skipped_files.push(relative.clone());
                }
            }
        }
    }

    Ok(stats)
}

/// Outcome of a retired-path sweep.
pub struct PruneStats {
    /// Relative paths deleted — files StrayMark installed and the operator
    /// never touched.
    pub removed: Vec<String>,
    /// Paths StrayMark installed but the operator has since edited.
    pub kept_modified: Vec<String>,
    /// Paths absent from the checksum store: operator-authored, or put there by
    /// something other than StrayMark. Reported separately because telling
    /// someone they "modified" a file they wrote themselves is just wrong.
    pub kept_foreign: Vec<String>,
}

/// Delete the paths a release declares under `retired:`.
///
/// Deletion is gated on provenance, not on the path pattern: a file goes only
/// when its current hash equals the one `.checksums.json` recorded for it —
/// i.e. StrayMark put it there and nobody has edited it since. Operator-edited
/// files and files absent from the store (operator-authored, or dropped in by
/// something else) are kept and named in the report, because a retirement
/// notice from upstream is not a licence to delete someone's work.
pub fn prune_retired(
    target: &Path,
    manifest: &DistManifest,
    checksums: &Checksums,
) -> Result<PruneStats> {
    let mut stats = PruneStats {
        removed: Vec::new(),
        kept_modified: Vec::new(),
        kept_foreign: Vec::new(),
    };

    for entry in &manifest.retired {
        let entry_path = target.join(entry.trim_end_matches('/'));
        if !entry_path.exists() {
            continue;
        }

        let files = if entry_path.is_dir() {
            walkdir(entry_path.clone())?
        } else {
            vec![entry_path.clone()]
        };

        for file in files {
            let relative = file
                .strip_prefix(target)
                .unwrap_or(&file)
                .display()
                .to_string()
                .replace('\\', "/");

            let current = utils::file_hash(&file);
            match checksums.files.get(&relative) {
                Some(stored) if current.as_deref() == Some(stored.as_str()) => {
                    std::fs::remove_file(&file)
                        .with_context(|| format!("Failed to remove {}", file.display()))?;
                    stats.removed.push(relative);
                }
                Some(_) => stats.kept_modified.push(relative),
                None => stats.kept_foreign.push(relative),
            }
        }

        // Sweep bottom-up: a skill directory whose only file was pruned is now
        // empty even when its parent still holds files the operator kept.
        prune_empty_subtree(&entry_path);
        remove_empty_dirs(&entry_path, target);
    }

    Ok(stats)
}

/// Remove every empty directory inside `root` (and `root` itself if it ends up
/// empty), deepest first. Best-effort: anything still holding a file stays.
fn prune_empty_subtree(root: &Path) {
    if !root.is_dir() {
        return;
    }
    let mut dirs = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
                dirs.push(path);
            }
        }
    }
    // Deepest first, so a parent is only considered once its children are gone.
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for dir in dirs {
        if std::fs::read_dir(&dir).map(|mut e| e.next().is_none()).unwrap_or(false) {
            let _ = std::fs::remove_dir(&dir);
        }
    }
}

/// Remove `dir` and any parent left empty, stopping at `stop_at` (exclusive).
/// Best-effort: a non-empty directory simply ends the walk.
fn remove_empty_dirs(dir: &Path, stop_at: &Path) {
    let mut current = dir.to_path_buf();
    while current.starts_with(stop_at) && current != stop_at {
        if !current.is_dir() {
            // Not a directory (or already gone) — climb to the parent anyway,
            // which is the case where `retired:` named a single file.
        } else {
            let empty = match std::fs::read_dir(&current) {
                Ok(mut entries) => entries.next().is_none(),
                Err(_) => return,
            };
            if !empty || std::fs::remove_dir(&current).is_err() {
                return;
            }
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return,
        }
    }
}

/// Inject directives based on manifest and templates from the release
fn inject_directives(target: &Path, source_root: &Path, manifest: &DistManifest) -> Result<()> {
    for injection in &manifest.injections {
        let target_path = target.join(&injection.target);

        // Missing targets are created, not skipped. A release that adds a new
        // agent surface (e.g. QWEN.md in fw-4.41.0) has to reach existing
        // installations through `update`, not only through `repair` — which is
        // what STRAYMARK.md § "Directive Injection Markers" already promises.
        // `inject::inject_directive` writes the full template when the file is
        // absent and manages only the marker block when it is not.
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

fn save_checksums(
    target: &Path,
    version: &str,
    distributed_hashes: &std::collections::HashMap<String, String>,
) -> Result<()> {
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

    // Override the disk-walk stamps with the release hashes. For files the
    // operator kept, the disk stamp is the user's content — stamping it as
    // the baseline would make the next update treat the file as unmodified
    // and overwrite it silently (GH #388).
    for (relative, hash) in distributed_hashes {
        checksums.files.insert(relative.clone(), hash.clone());
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
    use crate::config::Checksums;
    use crate::manifest::DistManifest;

    fn manifest_with_retired(retired: &str) -> DistManifest {
        DistManifest::from_str(&format!(
            "version: \"4.42.0\"\ndescription: \"test\"\nrepository: \"x\"\n\
             files: []\ninjections: []\nretired:\n  - {retired}\n"
        ))
        .unwrap()
    }

    fn seed(root: &std::path::Path, rel: &str, body: &str) -> String {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
        crate::utils::file_hash(&path).unwrap()
    }

    // Retiring a distributed path used to be impossible to finish: update_files
    // only ever copies, so a directory dropped from `files:` survived in every
    // existing installation until someone ran `straymark remove`. Deletion is
    // provenance-gated — a retirement notice upstream is not a licence to
    // delete work the operator did inside the path.
    #[test]
    fn retired_directory_is_pruned_but_operator_work_survives() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path();

        // Three files under the retired path, three different provenances.
        let pristine = seed(target, ".gemini/skills/straymark-new/SKILL.md", "shipped\n");
        let edited = seed(target, ".gemini/skills/straymark-adr/SKILL.md", "shipped\n");
        seed(target, ".gemini/skills/my-own/SKILL.md", "the operator's own\n");

        let mut checksums = Checksums::default();
        checksums
            .files
            .insert(".gemini/skills/straymark-new/SKILL.md".into(), pristine);
        checksums
            .files
            .insert(".gemini/skills/straymark-adr/SKILL.md".into(), edited);
        // `my-own` is deliberately absent from the store: never ours.

        // The operator edits one of the shipped files after installation.
        std::fs::write(
            target.join(".gemini/skills/straymark-adr/SKILL.md"),
            "shipped, then edited by me\n",
        )
        .unwrap();

        let manifest = manifest_with_retired(".gemini/skills/");
        let stats = super::prune_retired(target, &manifest, &checksums).unwrap();

        assert_eq!(
            stats.removed,
            vec![".gemini/skills/straymark-new/SKILL.md".to_string()],
            "only the untouched shipped file may be deleted"
        );
        assert!(
            !target.join(".gemini/skills/straymark-new").exists(),
            "the emptied skill directory should be cleaned up too"
        );

        assert_eq!(
            stats.kept_modified,
            vec![".gemini/skills/straymark-adr/SKILL.md".to_string()],
            "a file we shipped and the operator edited is kept as *modified*"
        );
        assert_eq!(
            stats.kept_foreign,
            vec![".gemini/skills/my-own/SKILL.md".to_string()],
            "a file we never installed is kept, but must not be reported as \"you modified it\""
        );
        assert!(target.join(".gemini/skills/straymark-adr/SKILL.md").exists());
        assert!(target.join(".gemini/skills/my-own/SKILL.md").exists());
    }

    #[test]
    fn fully_pruned_directory_disappears_including_its_parents() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path();

        let mut checksums = Checksums::default();
        for rel in [
            ".agent/workflows/straymark-new.md",
            ".agent/workflows/straymark-adr.md",
        ] {
            checksums.files.insert(rel.into(), seed(target, rel, "x\n"));
        }

        let manifest = manifest_with_retired(".agent/workflows/");
        let stats = super::prune_retired(target, &manifest, &checksums).unwrap();

        assert_eq!(stats.removed.len(), 2);
        assert!(stats.kept_modified.is_empty() && stats.kept_foreign.is_empty());
        assert!(
            !target.join(".agent").exists(),
            "an emptied parent must go too, or the adopter keeps a bare .agent/"
        );
    }

    #[test]
    fn retiring_a_path_that_is_already_gone_is_a_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = manifest_with_retired(".gemini/skills/");
        let stats =
            super::prune_retired(tmp.path(), &manifest, &Checksums::default()).unwrap();
        assert!(stats.removed.is_empty() && stats.kept_modified.is_empty());
    }

    /// Every `.straymark/dist-manifest.yml` written before fw-4.42.0 lacks the
    /// key, and `repair` / `remove` re-read those copies. Without
    /// `#[serde(default)]` this turns every pre-4.42.0 installation into a
    /// parse error.
    #[test]
    fn manifest_without_the_retired_key_still_parses() {
        let manifest = DistManifest::from_str(
            "version: \"4.41.0\"\ndescription: \"test\"\nrepository: \"x\"\n\
             files:\n  - STRAYMARK.md\ninjections: []\n",
        )
        .unwrap();
        assert!(manifest.retired.is_empty());
        assert_eq!(manifest.version, "4.41.0");
    }

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

    // GH #388 — a user-kept file must carry the release hash as its baseline,
    // not the on-disk (user) hash, or the next update sees it as unmodified
    // and silently overwrites it.
    #[test]
    fn save_checksums_prefers_distributed_hashes_over_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path();
        std::fs::create_dir_all(target.join(".straymark")).unwrap();
        std::fs::write(target.join(".straymark/config.yml"), "language: es\n").unwrap();

        let mut overrides = std::collections::HashMap::new();
        overrides.insert(
            ".straymark/config.yml".to_string(),
            "release-hash".to_string(),
        );
        super::save_checksums(target, "fw-4.38.2", &overrides).unwrap();

        let checksums = crate::config::Checksums::load(target).unwrap();
        assert_eq!(
            checksums
                .files
                .get(".straymark/config.yml")
                .map(String::as_str),
            Some("release-hash"),
            "the release hash must win over the on-disk (user) hash"
        );
        assert_eq!(checksums.version, "fw-4.38.2");
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

    /// A release that introduces a new agent surface must reach *existing*
    /// installations through `update`, not only through `repair`. Until
    /// fw-4.41.0 / cli-3.42.0 this loop skipped every target that was absent
    /// on disk, so `QWEN.md` would only ever have landed on fresh `init`s.
    #[test]
    fn update_creates_injection_targets_that_are_missing_on_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("project");
        let source_root = tmp.path().join("release");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::create_dir_all(source_root.join("dist-templates/directives")).unwrap();
        std::fs::write(
            source_root.join("dist-templates/directives/QWEN.md"),
            "# StrayMark - Qwen Code Configuration\n\n\
             <!-- straymark:begin -->\n> rules\n<!-- straymark:end -->\n",
        )
        .unwrap();

        // An existing target is refreshed in place; a missing one is created.
        std::fs::write(target.join("CLAUDE.md"), "# My own notes\n").unwrap();
        std::fs::write(
            source_root.join("dist-templates/directives/CLAUDE.md"),
            "# StrayMark - Claude Code Configuration\n\n\
             <!-- straymark:begin -->\n> rules\n<!-- straymark:end -->\n",
        )
        .unwrap();

        let manifest = crate::manifest::DistManifest::from_str(
            "version: \"4.41.0\"\ndescription: \"test\"\nrepository: \"x\"\nfiles: []\n\
             injections:\n\
             \x20 - target: CLAUDE.md\n    template: dist-templates/directives/CLAUDE.md\n\
             \x20 - target: QWEN.md\n    template: dist-templates/directives/QWEN.md\n",
        )
        .unwrap();

        super::inject_directives(&target, &source_root, &manifest).unwrap();

        let qwen = target.join("QWEN.md");
        assert!(
            qwen.exists(),
            "update must create the newly-declared QWEN.md target"
        );
        let qwen_content = std::fs::read_to_string(&qwen).unwrap();
        assert!(qwen_content.contains("<!-- straymark:begin -->"));
        assert!(qwen_content.contains("Qwen Code Configuration"));

        let claude_content = std::fs::read_to_string(target.join("CLAUDE.md")).unwrap();
        assert!(
            claude_content.contains("# My own notes"),
            "a pre-existing target keeps the operator's content"
        );
        assert!(claude_content.contains("<!-- straymark:begin -->"));
    }
}
