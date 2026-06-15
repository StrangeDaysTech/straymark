//! `straymark charter batch-complete <CHARTER-ID> <N>` — mark a Charter
//! batch as complete in the AILOG `## Batch Ledger`.
//!
//! Default mode is interactive (TTY-only): prompts for files-touched,
//! tests, and a design note, then formats the trio under the `### Batch <N>`
//! heading. With `--note "..."` the command runs one-shot and writes the
//! given string verbatim — designed for agents and scripts. With
//! `--non-interactive` (requires `--note`), prompts are disabled outright
//! so a missing `--note` aborts cleanly instead of hanging.
//!
//! Reads the originating AILOG from the Charter frontmatter
//! `originating_ailogs[0]`. Rejects with a clear error if:
//! - the Charter has no originating AILOG (cannot resolve target file)
//! - the AILOG file is not found under `.straymark/07-ai-audit/agent-logs/`
//! - the AILOG has no `## Batch Ledger` section
//! - no `### Batch <N>` heading exists in the ledger
//! - the target batch is already completed (refuses overwrite)
//!
//! Companion of `charter::drift`, which gates Charter close on any
//! `### Batch N` left as `(pending)`.

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;

use crate::ailog;
use straymark_core::charter;
use crate::prompts;
use crate::utils;

pub fn run(
    path: &str,
    charter_id: &str,
    batch_number: u32,
    note: Option<&str>,
    non_interactive: bool,
) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    // Resolve Charter by ID.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let charter = charter::find_by_id(&charters, charter_id)
        .ok_or_else(|| {
            anyhow!(
                "Charter {} not found in .straymark/charters/.\n  hint: run `straymark charter list` to see discovered Charters.",
                charter_id
            )
        })?
        .clone();

    let ailog_ids = match &charter.frontmatter.originating_ailogs {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => bail!(
            "Charter {} has no `originating_ailogs` in frontmatter.\n  hint: batch-complete writes to the originating AILOG; add it to the Charter or run the command directly on the AILOG file.",
            charter.frontmatter.charter_id
        ),
    };
    let ailog_id = &ailog_ids[0];
    if ailog_ids.len() > 1 {
        eprintln!(
            "{} Charter has {} originating AILOGs; using the first one ({}).",
            "note:".cyan().bold(),
            ailog_ids.len(),
            ailog_id
        );
    }

    let agent_logs_dir = ailog::agent_logs_dir(project_root);
    if !agent_logs_dir.exists() {
        bail!(
            "AILOG directory not found at {}. Run `straymark repair` to restore framework files.",
            agent_logs_dir.display()
        );
    }
    let ailog_path = ailog::find_ailog_file(&agent_logs_dir, ailog_id).ok_or_else(|| {
        anyhow!(
            "AILOG file for {} not found under {}.\n  hint: the file must be named `{}-<slug>.md`.",
            ailog_id,
            agent_logs_dir.display(),
            ailog_id
        )
    })?;

    let ailog_path_rel = ailog_path
        .strip_prefix(project_root)
        .unwrap_or(&ailog_path)
        .to_path_buf();

    // Pre-flight: confirm the batch exists AND is pending. We do this before
    // any user input so the operator does not waste effort on a rejected
    // batch.
    let pre_content = std::fs::read_to_string(&ailog_path)
        .with_context(|| format!("Failed to read AILOG at {}", ailog_path.display()))?;
    let entries = ailog::parse_batch_ledger(&pre_content).ok_or_else(|| {
        anyhow!(
            "AILOG at {} has no `## Batch Ledger` section.\n  hint: add the section to the AILOG (see TEMPLATE-AILOG.md) before running batch-complete.\n  This Charter does not need the ledger if it is a single-batch effort — `## Actions Performed` is sufficient.",
            ailog_path_rel.display()
        )
    })?;
    let target = entries
        .iter()
        .find(|e| e.n == batch_number)
        .ok_or_else(|| {
            anyhow!(
                "No `### Batch {}` heading in AILOG {}.\n  hint: add `### Batch {} — <name>` followed by `(pending)` to the `## Batch Ledger` section.\n  Existing batches: {}",
                batch_number,
                ailog_path_rel.display(),
                batch_number,
                if entries.is_empty() {
                    "(none)".to_string()
                } else {
                    entries.iter().map(|e| format!("Batch {}", e.n)).collect::<Vec<_>>().join(", ")
                }
            )
        })?
        .clone();
    ailog::ensure_pending(&target)?;

    println!(
        "{} AILOG: {}",
        "✔".green().bold(),
        ailog_path_rel.display().to_string().dimmed()
    );
    println!(
        "{} {}",
        "✔".green().bold(),
        target.heading_line.dimmed()
    );

    // Collect the new body.
    let body = if let Some(n) = note {
        n.to_string()
    } else if non_interactive {
        // --non-interactive requires --note; clap enforces this via `requires`,
        // but we keep a defensive bail just in case.
        bail!("--non-interactive requires --note");
    } else {
        prompts::require_interactive()?;
        collect_interactively()?
    };

    if body.trim().is_empty() {
        bail!(
            "Refusing to write an empty batch body — that would leave the batch indistinguishable from `(pending)`. Pass `--note \"...\"` or fill the prompts."
        );
    }
    if body.trim() == "(pending)" {
        bail!(
            "Refusing to write a `(pending)` body verbatim — that defeats the gate. Either supply real content or skip the command for this batch."
        );
    }

    let _prev = ailog::write_batch_section(&ailog_path, batch_number, &body)?;
    println!();
    println!(
        "{} `### Batch {}` written.",
        "OK".green().bold(),
        batch_number
    );
    println!(
        "{} `git add {}` before pushing the batch commit so the AILOG ledger update lands atomically with the work it documents.",
        "Reminder:".yellow().bold(),
        ailog_path_rel.display()
    );

    Ok(())
}

/// Interactive flow: three short prompts (files, tests, design note), joined
/// into a single batch body. Empty fields are skipped in the output.
fn collect_interactively() -> Result<String> {
    println!();
    println!("{}", "Collecting batch notes — press Enter on blank line to skip a section.".dimmed());
    println!();

    let files = prompts::prompt_string(
        "Files touched (one-line summary, e.g. `migrations/022.sql, handlers/x.go`)",
        None,
        true,
    )?;
    let tests = prompts::prompt_string(
        "Tests added or status (e.g. `handler_x_test.go +12 -0, passing`)",
        None,
        true,
    )?;
    println!(
        "{}",
        "Design note / lessons (multiline; finish with `.` on a line by itself, or Ctrl-D):"
            .dimmed()
    );
    let design = prompts::prompt_multiline("note")?;

    let mut body = String::new();
    let mut wrote = false;
    if !files.trim().is_empty() {
        body.push_str(&format!("- **Files**: {}", files.trim()));
        wrote = true;
    }
    if !tests.trim().is_empty() {
        if wrote {
            body.push('\n');
        }
        body.push_str(&format!("- **Tests**: {}", tests.trim()));
        wrote = true;
    }
    if !design.trim().is_empty() {
        if wrote {
            body.push_str("\n\n");
        }
        body.push_str(design.trim());
    }
    Ok(body)
}
