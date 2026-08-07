//! `straymark followups install-merge-driver` — wire the registry merge driver
//! into a clone (GH #391 follow-up).
//!
//! The driver itself shipped in cli-3.41.0, but it only takes effect once the
//! clone declares it in two places: a `.gitattributes` line binding the
//! registry to a named driver, and a `git config` entry telling git what to run
//! for that name. Only the first half is committable — `.git/config` never is —
//! so a teammate who clones the repo inherits the `.gitattributes` line and no
//! driver. Git does **not** degrade gracefully there: it aborts the merge with
//! `fatal: custom merge driver straymark-followups lacks command line`.
//!
//! That asymmetry is why this exists as a command rather than as documentation.
//! The committable half is not merely inert without the other; it breaks merges
//! for anyone who has not run the setup, which makes "once per clone" something
//! the tool has to make easy rather than a note in a reference page.

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;

/// The named driver, as referenced from `.gitattributes` and `git config`.
const DRIVER_NAME: &str = "straymark-followups";
/// Path pattern bound to the driver. Registry location is fixed by the framework.
const ATTR_LINE: &str = ".straymark/follow-ups-backlog.md merge=straymark-followups";
const DRIVER_CMD: &str = "straymark followups merge-driver %O %A %B";

pub struct SetupOutcome {
    pub attributes_added: bool,
    pub config_added: bool,
    /// A driver was already configured under our name with a different command.
    /// Left untouched — overwriting someone's deliberate override is not ours
    /// to do silently.
    pub config_conflict: Option<String>,
}

impl SetupOutcome {
    pub fn changed_nothing(&self) -> bool {
        !self.attributes_added && !self.config_added
    }
}

pub fn run(project_path: &str) -> Result<()> {
    let target = crate::utils::resolve_project_root(project_path)
        .map(|r| r.path)
        .unwrap_or_else(|| Path::new(project_path).to_path_buf());

    let outcome = install(&target)?;
    report(&outcome, &target);
    Ok(())
}

/// Wire the driver into `target`. Idempotent: re-running is a no-op.
pub fn install(target: &Path) -> Result<SetupOutcome> {
    if !target.join(".git").exists() {
        bail!(
            "not a git repository ({}). The merge driver is per-clone git \
             configuration; run `git init` first.",
            target.display()
        );
    }

    let attributes_added = ensure_gitattributes(target)?;
    let (config_added, config_conflict) = ensure_git_config(target)?;

    Ok(SetupOutcome {
        attributes_added,
        config_added,
        config_conflict,
    })
}

/// Append the merge attribute unless the registry is already bound to a driver.
fn ensure_gitattributes(target: &Path) -> Result<bool> {
    let path = target.join(".gitattributes");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    // Any existing binding for the registry wins — including one pointing at a
    // different driver, which would be a deliberate adopter choice.
    if existing
        .lines()
        .any(|l| l.trim_start().starts_with(".straymark/follow-ups-backlog.md") && l.contains("merge="))
    {
        return Ok(false);
    }

    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n# StrayMark: structural three-way merge for the CLI-owned follow-ups\n");
    content.push_str("# registry (GH #391). This line alone is not enough: without the matching\n");
    content.push_str("# `git config merge.straymark-followups.driver`, git ABORTS any merge that\n");
    content.push_str("# touches the registry. Every clone must run, once:\n");
    content.push_str("#     straymark followups install-merge-driver\n");
    content.push_str(ATTR_LINE);
    content.push('\n');

    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

/// Set `merge.<name>.{name,driver}` in the clone's git config.
fn ensure_git_config(target: &Path) -> Result<(bool, Option<String>)> {
    let key = format!("merge.{DRIVER_NAME}.driver");

    let current = git(target, &["config", "--get", &key])?;
    if let Some(value) = current {
        if value.trim() == DRIVER_CMD {
            return Ok((false, None));
        }
        return Ok((false, Some(value.trim().to_string())));
    }

    git_expect(
        target,
        &[
            "config",
            &format!("merge.{DRIVER_NAME}.name"),
            "StrayMark follow-ups registry (structural three-way merge)",
        ],
    )?;
    git_expect(target, &["config", &key, DRIVER_CMD])?;
    Ok((true, None))
}

fn git(target: &Path, args: &[&str]) -> Result<Option<String>> {
    let out = std::process::Command::new("git")
        .current_dir(target)
        .args(args)
        .output()
        .context("Failed to run git — is it installed and on PATH?")?;
    if out.status.success() {
        Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()))
    } else {
        Ok(None)
    }
}

fn git_expect(target: &Path, args: &[&str]) -> Result<()> {
    let out = std::process::Command::new("git")
        .current_dir(target)
        .args(args)
        .output()
        .context("Failed to run git — is it installed and on PATH?")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

fn report(outcome: &SetupOutcome, target: &Path) {
    if let Some(existing) = &outcome.config_conflict {
        crate::utils::warn(&format!(
            "merge.{DRIVER_NAME}.driver is already set to something else — left as is:"
        ));
        println!("    {}", existing.dimmed());
        println!(
            "  {} To replace it: {}",
            "→".blue().bold(),
            format!("git config merge.{DRIVER_NAME}.driver '{DRIVER_CMD}'").cyan()
        );
        println!();
        return;
    }

    if outcome.changed_nothing() {
        crate::utils::success("Merge driver already wired — nothing to do.");
        println!();
        return;
    }

    if outcome.attributes_added {
        crate::utils::success(".gitattributes: registry bound to the merge driver");
    }
    if outcome.config_added {
        crate::utils::success(&format!("git config: merge.{DRIVER_NAME}.driver set"));
    }
    println!();
    if outcome.attributes_added {
        println!(
            "  {} Commit {} so teammates inherit the binding.",
            "→".blue().bold(),
            ".gitattributes".cyan()
        );
    }
    // The config half lives in .git/config, which is never committed. Saying so
    // is load-bearing: a teammate who pulls the .gitattributes line without
    // running this gets `fatal: ... lacks command line` on any merge touching
    // the registry — a hard stop, not a fallback to normal conflict markers.
    println!(
        "  {} Every clone must run {} once — {} is not committed,",
        "→".blue().bold(),
        "straymark followups install-merge-driver".cyan(),
        ".git/config".dimmed()
    );
    println!("    and git aborts the merge outright when the driver is missing.");
    println!("  {}", target.display().to_string().dimmed());
    println!();
}
