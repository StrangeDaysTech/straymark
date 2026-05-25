use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils;

/// Install StrayMark skills into an AI agent's user-level skills directory.
///
/// Currently supports `--agent codex` (skills land in `$CODEX_HOME/skills/`
/// or `$HOME/.codex/skills/`). Claude and Gemini consume skills directly
/// from the project tree (`.claude/skills/`, `.gemini/skills/`); for those
/// agents this command exits with an explanatory error.
pub fn run(agent: &str, project_path: &str, dry_run: bool, symlink: bool) -> Result<()> {
    match agent {
        "codex" => install_codex(project_path, dry_run, symlink),
        "claude" | "gemini" => {
            bail!(
                "Skills for {agent} are read directly from the project tree (.{agent}/skills/). \
                 No user-level install is required. Run `straymark init` or `straymark update` \
                 to refresh them in the project."
            );
        }
        other => bail!("unknown agent: {other} (supported: codex)"),
    }
}

fn install_codex(project_path: &str, dry_run: bool, symlink: bool) -> Result<()> {
    let project = PathBuf::from(project_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(project_path));

    let src = project.join(".codex").join("skills");
    if !src.is_dir() {
        bail!(
            "{} not found. Run `straymark init` (or `straymark update`) in this project first \
             so that the .codex/skills/ tree is materialized.",
            src.display()
        );
    }

    let codex_home = resolve_codex_home()?;
    let dst = codex_home.join("skills");
    if !dst.exists() {
        if dry_run {
            println!(
                "  {} would create {}",
                "→".blue().bold(),
                dst.display()
            );
        } else {
            fs::create_dir_all(&dst)
                .with_context(|| format!("create {}", dst.display()))?;
        }
    }

    println!();
    println!("  {}", "StrayMark Install Skills".bold().cyan());
    println!("  agent: {}", "codex".green());
    println!("  from:  {}", src.display().to_string().dimmed());
    println!("  to:    {}", dst.display().to_string().dimmed());
    if dry_run {
        println!("  mode:  {}", "dry-run".yellow());
    } else if symlink {
        println!("  mode:  {}", "symlink".cyan());
    }
    println!();

    let mut entries: Vec<_> = fs::read_dir(&src)
        .with_context(|| format!("read_dir {}", src.display()))?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut installed = 0usize;
    let mut replaced = 0usize;
    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let name = entry.file_name();
        let target = dst.join(&name);
        let existed = target.exists() || target.is_symlink();

        if dry_run {
            let verb = if existed { "would replace" } else { "would install" };
            let mode = if symlink { "symlink" } else { "copy" };
            println!(
                "  {} {} {} ({})",
                "→".blue().bold(),
                verb,
                name.to_string_lossy(),
                mode
            );
            installed += 1;
            if existed {
                replaced += 1;
            }
            continue;
        }

        if existed {
            // Remove regardless of file/dir/symlink kind so we can replace.
            remove_entry(&target)?;
            replaced += 1;
        }

        if symlink {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&path, &target)
                .with_context(|| format!("symlink {} → {}", target.display(), path.display()))?;
            #[cfg(not(unix))]
            bail!("--symlink is only supported on Unix platforms; rerun without --symlink");
        } else {
            copy_dir_recursive(&path, &target)
                .with_context(|| format!("copy {} → {}", path.display(), target.display()))?;
        }
        installed += 1;
        utils::success(&format!("{}", name.to_string_lossy()));
    }

    println!();
    if dry_run {
        println!(
            "  {} {} skill(s) would be installed ({} replacing existing). Re-run without --dry-run to apply.",
            "→".blue().bold(),
            installed,
            replaced
        );
    } else {
        println!(
            "  {} {} skill(s) installed in {} ({} replaced).",
            "✓".green().bold(),
            installed,
            dst.display(),
            replaced
        );
        println!(
            "  {} Codex will discover them on next session.",
            "→".blue().bold()
        );
    }
    println!();
    Ok(())
}

fn resolve_codex_home() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("CODEX_HOME") {
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    let home = std::env::var("HOME").context("$HOME is not set")?;
    Ok(PathBuf::from(home).join(".codex"))
}

fn remove_entry(p: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(p).with_context(|| format!("stat {}", p.display()))?;
    if meta.file_type().is_symlink() || meta.is_file() {
        fs::remove_file(p).with_context(|| format!("remove file {}", p.display()))?;
    } else if meta.is_dir() {
        fs::remove_dir_all(p).with_context(|| format!("remove dir {}", p.display()))?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("mkdir {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("read_dir {}", src.display()))? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)
                .with_context(|| format!("copy {} → {}", path.display(), target.display()))?;
        }
    }
    Ok(())
}
