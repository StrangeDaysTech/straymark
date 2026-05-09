//! `straymark charter close` — record the post-execution telemetry of a Charter.
//!
//! Two operating modes:
//! - **Interactive** (default): walks the operator through the telemetry schema
//!   field by field with prompts. Target time: 5–10 min per Charter.
//! - **From-template** (`--from-template`): copies the YAML skeleton next to
//!   the Charter so the operator can edit it manually with their preferred
//!   editor. Combine with `--non-interactive` for CI / scripted use.
//!
//! After populating telemetry, the YAML is validated against
//! `.straymark/schemas/charter-telemetry.schema.v0.json`. The Charter
//! frontmatter `status` is bumped to `closed`.
//!
//! Storage: `.straymark/charters/CHARTER-NN.telemetry.yaml`. We do not embed
//! telemetry in the Charter frontmatter (see roadmap §A2): frontmatter is
//! declarative ex-ante; telemetry is ex-post and voluminous.

use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::charter::{self, charters_dir, Charter, CharterStatus};
use crate::prompts;
use crate::telemetry_schema::TelemetrySchema;
use crate::utils;

pub fn run(
    path: &str,
    charter_id: &str,
    from_template: bool,
    non_interactive: bool,
) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;
    let straymark_dir = project_root.join(".straymark");

    // Resolve the Charter.
    let (charters, _errors) = charter::discover_and_parse(project_root);
    let charter = charter::find_by_id(&charters, charter_id)
        .ok_or_else(|| {
            anyhow!(
                "Charter {} not found in .straymark/charters/.\n  hint: run `straymark charter list` to see discovered Charters.",
                charter_id
            )
        })?
        .clone();

    // Decide telemetry destination — declarative .md and telemetry .yaml share
    // the same directory by design (see charters_dir docstring).
    let telemetry_dir = charters_dir(project_root);
    utils::ensure_dir(&telemetry_dir)?;
    let telemetry_path = telemetry_path_for(&telemetry_dir, &charter);

    // F7 (cli-3.8.0): differentiate first-run vs subsequent-run output.
    // Pre-existence check happens BEFORE any write so we can report
    // accurately even though copy_template_for is idempotent.
    let telemetry_existed_before = telemetry_path.exists();

    // Mode dispatch.
    let yaml_text = if from_template {
        copy_template_for(&straymark_dir, &charter, &telemetry_path, non_interactive)?
    } else {
        prompts::require_interactive()?;
        let telemetry = drive_interactive_flow(&charter)?;
        std::fs::write(&telemetry_path, &telemetry).with_context(|| {
            format!("Failed to write telemetry to {}", telemetry_path.display())
        })?;
        telemetry
    };

    // Validate against schema. F7: on a subsequent --from-template run the
    // user has presumably edited the file, so we DO validate; only a true
    // first-run with --non-interactive (where the user hasn't seen the
    // template yet) skips validation since the placeholders would fail.
    let is_first_run_template = from_template && non_interactive && !telemetry_existed_before;
    if !is_first_run_template {
        let schema = TelemetrySchema::load(&straymark_dir)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_text)
            .with_context(|| format!("Telemetry YAML at {} is not valid YAML", telemetry_path.display()))?;
        let issues = schema.validate(&yaml_value, &telemetry_path);
        if !issues.is_empty() {
            eprintln!("{}", "Telemetry validation found issues:".yellow().bold());
            for issue in &issues {
                eprintln!("  - {} [{}]", issue.message, issue.rule);
                if let Some(hint) = &issue.fix_hint {
                    eprintln!("    {} {}", "hint:".cyan(), hint);
                }
            }
            bail!("telemetry written but failed schema validation; fix and re-run with --from-template to edit in place");
        }
    }

    // Bump Charter status to closed.
    update_charter_status_to_closed(&charter)?;

    // Summary. F7 (cli-3.8.0): differentiate the three operationally
    // distinct cases instead of printing identical output for all three:
    //
    //  - Interactive flow → telemetry was filled directly via prompts;
    //    print the standard "closed + telemetry path + status" trio.
    //  - --from-template, FIRST run: template was just dropped on disk
    //    with placeholders; user needs to edit + re-run. Tell them so.
    //  - --from-template, SUBSEQUENT run: schema passed; the file is
    //    valid telemetry. Charter close is finalized.
    println!(
        "{} Charter {} closed.",
        "✔".green().bold(),
        charter.frontmatter.charter_id.bold()
    );
    println!("  Telemetry: {}", telemetry_path.display());
    println!("  Status updated: in-progress/declared → closed");
    if from_template && non_interactive {
        if is_first_run_template {
            println!();
            println!(
                "  {} Telemetry template created with prefilled charter_id, title, and closed_at.",
                "→".blue().bold()
            );
            println!(
                "  {} Edit the YAML to fill in trigger, effort, agent_quality, outcome,",
                "→".blue().bold()
            );
            println!(
                "    and qualitative sections. Then re-run the same command to validate"
            );
            println!("    against the schema and finalize the close.");
        } else {
            println!();
            println!(
                "  {} Telemetry schema validation passed. Charter close finalized.",
                "✔".green().bold()
            );
        }
    }
    Ok(())
}

/// Build the `.straymark/charters/CHARTER-NN.telemetry.yaml` path for a Charter.
fn telemetry_path_for(charters_state_dir: &Path, charter: &Charter) -> PathBuf {
    // Strip optional slug suffix to keep the telemetry filename stable across
    // Charter renames: CHARTER-01-foo → CHARTER-01.telemetry.yaml.
    let id = &charter.frontmatter.charter_id;
    let canonical = id
        .split_once('-')
        .and_then(|(prefix, rest)| {
            // CHARTER-NN[-slug] → CHARTER-NN
            let nn = rest.split('-').next()?;
            Some(format!("{}-{}", prefix, nn))
        })
        .unwrap_or_else(|| id.clone());
    charters_state_dir.join(format!("{}.telemetry.yaml", canonical))
}

/// Copy the template YAML to the telemetry path. Refuses to overwrite an
/// existing file unless the operator confirms (interactive mode) or the file
/// is the template itself unchanged (resumable in `--from-template` flow).
fn copy_template_for(
    straymark_dir: &Path,
    charter: &Charter,
    dest: &Path,
    non_interactive: bool,
) -> Result<String> {
    let template_path = straymark_dir
        .join("templates")
        .join("charter-telemetry-template.yaml");
    let template = std::fs::read_to_string(&template_path).with_context(|| {
        format!(
            "Telemetry template not found at {}. Run `straymark repair` to restore framework files.",
            template_path.display()
        )
    })?;

    if dest.exists() {
        if non_interactive {
            // Idempotent: leave existing file in place.
            return std::fs::read_to_string(dest)
                .with_context(|| format!("Failed to read existing telemetry at {}", dest.display()));
        }
        prompts::require_interactive()?;
        let overwrite = prompts::prompt_bool(
            &format!("Telemetry file {} already exists — overwrite?", dest.display()),
            false,
        )?;
        if !overwrite {
            return std::fs::read_to_string(dest)
                .with_context(|| format!("Failed to read existing telemetry at {}", dest.display()));
        }
    }

    // Pre-fill the obvious fields (charter_id, charter_title, closed_at) so
    // the user lands on a less-empty starting point.
    let prefilled = template
        .replace("CHARTER-NN", &short_id(&charter.frontmatter.charter_id))
        .replace("<short title>", &one_line_title(charter))
        .replace("YYYY-MM-DD", &Local::now().format("%Y-%m-%d").to_string());
    std::fs::write(dest, &prefilled)
        .with_context(|| format!("Failed to write telemetry template to {}", dest.display()))?;
    Ok(prefilled)
}

fn short_id(charter_id: &str) -> String {
    charter_id
        .split_once('-')
        .and_then(|(prefix, rest)| Some(format!("{}-{}", prefix, rest.split('-').next()?)))
        .unwrap_or_else(|| charter_id.to_string())
}

fn one_line_title(charter: &Charter) -> String {
    // Use the H1 title if available; fallback to the slug.
    crate::charter::display_title(charter)
}

/// Drive the operator through the full schema interactively. Returns the
/// rendered YAML (top-level `charter_telemetry:` map) ready to write.
fn drive_interactive_flow(charter: &Charter) -> Result<String> {
    println!(
        "{} {}",
        "Closing".cyan().bold(),
        charter.frontmatter.charter_id.bold()
    );
    println!(
        "  Title: {}",
        crate::charter::display_title(charter).dimmed()
    );
    println!("{}", "Press Enter to accept defaults; type to override.".dimmed());
    println!();

    let charter_id = short_id(&charter.frontmatter.charter_id);
    let charter_title = crate::charter::display_title(charter);
    let closed_at = Local::now().format("%Y-%m-%d").to_string();

    // ── Trigger ─────────────────────────────────────────────────────────
    println!("{}", "── Trigger ──".bold());
    let declared_kind = prompts::prompt_enum(
        "Declared trigger kind",
        &["event_trigger", "date", "metric_threshold", "infrastructure_milestone"],
        0,
    )?;
    let declared_description =
        prompts::prompt_string("Declared trigger description", None, true)?;
    let fired_at = prompts::prompt_string("Fired at (YYYY-MM-DD)", Some(&closed_at), false)?;
    let fire_clarity =
        prompts::prompt_enum("Trigger clarity", &["clear", "ambiguous", "manually_decided"], 0)?;
    let fire_clarity_notes = prompts::prompt_string("Trigger clarity notes", None, true)?;

    // ── Effort ──────────────────────────────────────────────────────────
    println!();
    println!("{}", "── Effort ──".bold());
    let estimated_effort =
        prompts::prompt_string("Estimated effort (e.g., M (~1.5h))", Some("M (~1.5h)"), false)?;
    let actual_effort = prompts::prompt_string(
        "Actual effort (e.g., M (~1.5h))",
        Some(&estimated_effort),
        false,
    )?;
    let drift_factor_raw = prompts::prompt_string(
        "Estimation drift factor (actual/estimated TIME, e.g., 1.0)",
        Some("1.0"),
        false,
    )?;
    let drift_factor: f64 = drift_factor_raw
        .trim()
        .parse()
        .map_err(|e| anyhow!("drift factor not a number: {e}"))?;
    let drift_reason = prompts::prompt_string("Estimation drift reason", None, true)?;

    // ── Agent quality ───────────────────────────────────────────────────
    println!();
    println!("{}", "── Agent quality ──".bold());
    let sessions_count = prompts::prompt_u32("Sessions count", 1)?;
    let hallucinations_caught = prompts::prompt_u32("Hallucinations caught", 0)?;
    let hallucination_categories = if hallucinations_caught > 0 {
        prompts::prompt_string_array("Hallucination categories")?
    } else {
        Vec::new()
    };
    let decisions_contradicting_prior_adrs =
        prompts::prompt_u32("Decisions contradicting prior ADRs", 0)?;
    let context_loaded_was_sufficient =
        prompts::prompt_bool("Context loaded was sufficient", true)?;
    let additional_context_loaded_manually =
        prompts::prompt_u32("Additional context files loaded manually", 0)?;
    let r_n_plus_one_emergent_count =
        prompts::prompt_u32("Emergent risks named during execution (R<N+1>)", 0)?;

    // ── Outcome ─────────────────────────────────────────────────────────
    println!();
    println!("{}", "── Outcome ──".bold());
    let completed_as_planned = prompts::prompt_bool("Completed as planned", true)?;
    let scope_changes = prompts::prompt_enum("Scope changes", &["ninguno", "menor", "mayor"], 0)?;
    let scope_change_notes = if scope_changes == "ninguno" {
        String::new()
    } else {
        prompts::prompt_string("Scope change notes (F1...FN encoding)", None, true)?
    };
    let new_followups_generated = prompts::prompt_u32("New follow-up AILOGs generated", 0)?;
    let new_charters_created = prompts::prompt_u32("New Charters created", 0)?;

    // ── Qualitative ─────────────────────────────────────────────────────
    println!();
    println!("{}", "── Qualitative ──".bold());
    let format_iteration = prompts::prompt_string("Format iteration (e.g., v4)", Some("v4"), false)?;
    let friction_points = prompts::prompt_string_array("Friction points")?;
    let wins = prompts::prompt_string_array("Wins")?;
    let overall_satisfaction = prompts::prompt_u32("Overall satisfaction (1-5)", 4)?;
    if !(1..=5).contains(&overall_satisfaction) {
        bail!("overall_satisfaction must be in 1..=5 (got {})", overall_satisfaction);
    }
    let would_repeat_format = prompts::prompt_bool("Would repeat this format", true)?;

    // ── Render YAML ─────────────────────────────────────────────────────
    let yaml = render_yaml(TelemetryDraft {
        charter_id,
        charter_title,
        closed_at,
        trigger_declared_kind: declared_kind,
        trigger_declared_description: declared_description,
        trigger_fired_at: fired_at,
        trigger_fire_clarity: fire_clarity,
        trigger_fire_clarity_notes: fire_clarity_notes,
        effort_estimated: estimated_effort,
        effort_actual: actual_effort,
        effort_drift_factor: drift_factor,
        effort_drift_reason: drift_reason,
        agent_sessions_count: sessions_count,
        agent_hallucinations_caught: hallucinations_caught,
        agent_hallucination_categories: hallucination_categories,
        agent_decisions_contradicting: decisions_contradicting_prior_adrs,
        agent_context_sufficient: context_loaded_was_sufficient,
        agent_additional_context_manual: additional_context_loaded_manually,
        agent_r_n_plus_one: r_n_plus_one_emergent_count,
        outcome_completed: completed_as_planned,
        outcome_scope_changes: scope_changes,
        outcome_scope_change_notes: scope_change_notes,
        outcome_new_followups: new_followups_generated,
        outcome_new_charters: new_charters_created,
        qualitative_format_iteration: format_iteration,
        qualitative_friction_points: friction_points,
        qualitative_wins: wins,
        qualitative_overall_satisfaction: overall_satisfaction,
        qualitative_would_repeat: would_repeat_format,
    });
    Ok(yaml)
}

struct TelemetryDraft {
    charter_id: String,
    charter_title: String,
    closed_at: String,
    trigger_declared_kind: String,
    trigger_declared_description: String,
    trigger_fired_at: String,
    trigger_fire_clarity: String,
    trigger_fire_clarity_notes: String,
    effort_estimated: String,
    effort_actual: String,
    effort_drift_factor: f64,
    effort_drift_reason: String,
    agent_sessions_count: u32,
    agent_hallucinations_caught: u32,
    agent_hallucination_categories: Vec<String>,
    agent_decisions_contradicting: u32,
    agent_context_sufficient: bool,
    agent_additional_context_manual: u32,
    agent_r_n_plus_one: u32,
    outcome_completed: bool,
    outcome_scope_changes: String,
    outcome_scope_change_notes: String,
    outcome_new_followups: u32,
    outcome_new_charters: u32,
    qualitative_format_iteration: String,
    qualitative_friction_points: Vec<String>,
    qualitative_wins: Vec<String>,
    qualitative_overall_satisfaction: u32,
    qualitative_would_repeat: bool,
}

/// Render a `TelemetryDraft` as canonical YAML. We hand-write this rather than
/// relying on `serde_yaml::to_string` so the output is stable, comment-free,
/// and visually consistent with `charter-telemetry-template.yaml`.
fn render_yaml(d: TelemetryDraft) -> String {
    let mut out = String::new();
    out.push_str("charter_telemetry:\n");
    out.push_str(&format!("  charter_id: \"{}\"\n", yaml_escape(&d.charter_id)));
    out.push_str(&format!("  charter_title: \"{}\"\n", yaml_escape(&d.charter_title)));
    out.push_str(&format!("  closed_at: \"{}\"\n", d.closed_at));

    out.push_str("\n  trigger:\n");
    out.push_str(&format!("    declared_kind: \"{}\"\n", d.trigger_declared_kind));
    out.push_str(&format!(
        "    declared_description: \"{}\"\n",
        yaml_escape(&d.trigger_declared_description)
    ));
    out.push_str(&format!("    fired_at: \"{}\"\n", d.trigger_fired_at));
    out.push_str(&format!("    fire_clarity: \"{}\"\n", d.trigger_fire_clarity));
    out.push_str(&format!(
        "    fire_clarity_notes: \"{}\"\n",
        yaml_escape(&d.trigger_fire_clarity_notes)
    ));

    out.push_str("\n  effort:\n");
    out.push_str(&format!("    estimated_effort: \"{}\"\n", d.effort_estimated));
    out.push_str(&format!("    actual_effort: \"{}\"\n", d.effort_actual));
    out.push_str(&format!(
        "    estimation_drift_factor: {}\n",
        format_float(d.effort_drift_factor)
    ));
    out.push_str(&format!(
        "    estimation_drift_reason: \"{}\"\n",
        yaml_escape(&d.effort_drift_reason)
    ));

    out.push_str("\n  agent_quality:\n");
    out.push_str(&format!("    sessions_count: {}\n", d.agent_sessions_count));
    out.push_str(&format!(
        "    hallucinations_caught: {}\n",
        d.agent_hallucinations_caught
    ));
    out.push_str("    hallucination_categories:");
    if d.agent_hallucination_categories.is_empty() {
        out.push_str(" []\n");
    } else {
        out.push('\n');
        for cat in &d.agent_hallucination_categories {
            out.push_str(&format!("      - \"{}\"\n", yaml_escape(cat)));
        }
    }
    out.push_str(&format!(
        "    decisions_contradicting_prior_adrs: {}\n",
        d.agent_decisions_contradicting
    ));
    out.push_str(&format!(
        "    context_loaded_was_sufficient: {}\n",
        d.agent_context_sufficient
    ));
    out.push_str(&format!(
        "    additional_context_loaded_manually: {}\n",
        d.agent_additional_context_manual
    ));
    out.push_str(&format!(
        "    r_n_plus_one_emergent_count: {}\n",
        d.agent_r_n_plus_one
    ));

    out.push_str("\n  outcome:\n");
    out.push_str(&format!("    completed_as_planned: {}\n", d.outcome_completed));
    out.push_str(&format!(
        "    scope_changes: \"{}\"\n",
        d.outcome_scope_changes
    ));
    if !d.outcome_scope_change_notes.is_empty() {
        out.push_str(&format!(
            "    scope_change_notes: \"{}\"\n",
            yaml_escape(&d.outcome_scope_change_notes)
        ));
    }
    out.push_str(&format!(
        "    new_followups_generated: {}\n",
        d.outcome_new_followups
    ));
    out.push_str(&format!("    new_charters_created: {}\n", d.outcome_new_charters));

    out.push_str("\n  qualitative:\n");
    out.push_str(&format!(
        "    format_iteration: \"{}\"\n",
        d.qualitative_format_iteration
    ));
    out.push_str("    friction_points:");
    if d.qualitative_friction_points.is_empty() {
        out.push_str(" []\n");
    } else {
        out.push('\n');
        for p in &d.qualitative_friction_points {
            out.push_str(&format!("      - \"{}\"\n", yaml_escape(p)));
        }
    }
    out.push_str("    wins:");
    if d.qualitative_wins.is_empty() {
        out.push_str(" []\n");
    } else {
        out.push('\n');
        for w in &d.qualitative_wins {
            out.push_str(&format!("      - \"{}\"\n", yaml_escape(w)));
        }
    }
    out.push_str(&format!(
        "    overall_satisfaction: {}\n",
        d.qualitative_overall_satisfaction
    ));
    out.push_str(&format!(
        "    would_repeat_format: {}\n",
        d.qualitative_would_repeat
    ));

    out
}

/// Escape a string for use inside a double-quoted YAML scalar. Handles `\` and `"`.
fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Render a float without trailing zeros for clean YAML.
fn format_float(f: f64) -> String {
    if f.fract() == 0.0 {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

/// Update the Charter file: bump `status: ...` to `status: closed` in
/// frontmatter, AND sync the prose status mirror line in the body. No-op
/// when already closed.
///
/// F6 (cli-3.7.1): the previous version updated frontmatter only, leaving the
/// body's "Status (mirrored from frontmatter — source of truth is above):"
/// line stale. The template's own promise that the body line mirrors the
/// frontmatter was broken by the CLI. Now both representations are synced.
fn update_charter_status_to_closed(charter: &Charter) -> Result<()> {
    if matches!(charter.frontmatter.status, CharterStatus::Closed) {
        return Ok(());
    }
    let raw = std::fs::read_to_string(&charter.path).with_context(|| {
        format!("Failed to read Charter file at {}", charter.path.display())
    })?;

    // F8 (cli-3.7.2): also write closed_at: <today> alongside status: closed.
    // Per Sentinel CHARTER-02..05 telemetry the field had to be added manually
    // 4× consecutively. Schema permits arbitrary additional fields; the test
    // suite for fw-4.6.0 confirmed the validator passes through unknown keys.
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Walk the frontmatter in a single pass:
    //   - Replace the first `status:` line with `status: closed`.
    //   - Replace an existing `closed_at:` line with today's date, if present.
    //   - If `closed_at:` is absent, queue it for insertion right after the
    //     replaced `status:` line so it sits in a logical place.
    let mut updated = String::with_capacity(raw.len() + 32);
    let mut in_frontmatter = false;
    let mut status_replaced = false;
    let mut closed_at_replaced = false;
    let mut closed_at_indent: Option<String> = None;
    let mut delim_count = 0;
    for line in raw.lines() {
        if line.trim() == "---" {
            delim_count += 1;
            in_frontmatter = delim_count == 1;
            updated.push_str(line);
            updated.push('\n');
            continue;
        }
        if in_frontmatter && !status_replaced && line.trim_start().starts_with("status:") {
            let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            closed_at_indent = Some(leading_ws.clone());
            updated.push_str(&format!("{leading_ws}status: closed\n"));
            status_replaced = true;
            continue;
        }
        if in_frontmatter && !closed_at_replaced && line.trim_start().starts_with("closed_at:") {
            let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            updated.push_str(&format!("{leading_ws}closed_at: {today}\n"));
            closed_at_replaced = true;
            continue;
        }
        updated.push_str(line);
        updated.push('\n');
    }
    if !status_replaced {
        bail!(
            "Charter at {} has no `status:` line in frontmatter — cannot bump to closed",
            charter.path.display()
        );
    }

    // If closed_at was absent in the original frontmatter, insert it right
    // after the replaced status line. We rebuild `updated` rather than tracking
    // an insertion point above, since insertion at arbitrary line indices is
    // simpler than splicing an index.
    let updated = if !closed_at_replaced {
        let indent = closed_at_indent.unwrap_or_default();
        let needle = format!("{indent}status: closed\n");
        let insert = format!("{indent}closed_at: {today}\n");
        match updated.find(&needle) {
            Some(pos) => {
                let after = pos + needle.len();
                let mut out = String::with_capacity(updated.len() + insert.len());
                out.push_str(&updated[..after]);
                out.push_str(&insert);
                out.push_str(&updated[after..]);
                out
            }
            // Defensive — shouldn't happen since we just wrote it.
            None => updated,
        }
    } else {
        updated
    };

    // Sync the body status mirror line. Best-effort: if the line is absent or
    // has been edited beyond recognition, we leave it alone — the schema has
    // already authoritatively recorded the status in frontmatter.
    let synced = sync_body_status_mirror(&updated, "closed");

    std::fs::write(&charter.path, synced).with_context(|| {
        format!("Failed to write updated status to {}", charter.path.display())
    })?;
    Ok(())
}

/// Sync the body's prose status mirror line to `new_status`. Recognizes the
/// canonical EN/ES forms generated by the framework templates:
///
/// EN: `> **Status (mirrored from frontmatter — source of truth is above):** declared. Effort: ...`
/// ES: `> **Status (espejado del frontmatter — la fuente de verdad está arriba):** declared. Esfuerzo: ...`
///
/// (zh-CN templates currently use the same `**Status (...)** WORD` shape; if a
/// future translation diverges, extend the matcher with a third needle.)
///
/// The matcher anchors on the substring `mirrored from frontmatter` /
/// `espejado del frontmatter` to avoid touching unrelated `**Status:**` lines
/// that adopters may have added to their Charter body. We only rewrite the
/// status word that appears immediately after `):**` + whitespace; the rest
/// of the line (Effort, anything else) is preserved verbatim.
fn sync_body_status_mirror(text: &str, new_status: &str) -> String {
    let needles = [
        "mirrored from frontmatter",
        "espejado del frontmatter",
    ];
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let is_mirror = needles.iter().any(|n| line.contains(n));
        if !is_mirror {
            out.push_str(line);
            continue;
        }
        // Find the `):**` marker. After it, skip whitespace, replace the
        // following [A-Za-z-]+ token with `new_status`. If the shape doesn't
        // match (corrupted line), preserve as-is.
        match line.find("):**") {
            None => out.push_str(line),
            Some(marker_idx) => {
                let after_marker = marker_idx + "):**".len();
                let rest = &line[after_marker..];
                let leading_ws: String = rest.chars().take_while(|c| c.is_whitespace()).collect();
                let after_ws = &rest[leading_ws.len()..];
                let token_end = after_ws
                    .find(|c: char| !c.is_ascii_alphabetic() && c != '-')
                    .unwrap_or(after_ws.len());
                if token_end == 0 {
                    out.push_str(line);
                    continue;
                }
                let tail = &after_ws[token_end..];
                out.push_str(&line[..after_marker]);
                out.push_str(&leading_ws);
                out.push_str(new_status);
                out.push_str(tail);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_id_strips_slug() {
        assert_eq!(short_id("CHARTER-01"), "CHARTER-01");
        assert_eq!(short_id("CHARTER-01-foo-bar"), "CHARTER-01");
        assert_eq!(short_id("CHARTER-12"), "CHARTER-12");
        assert_eq!(short_id("CHARTER-12-baseline-recompute"), "CHARTER-12");
    }

    #[test]
    fn yaml_escape_handles_quotes_and_backslashes() {
        assert_eq!(yaml_escape("hello"), "hello");
        assert_eq!(yaml_escape(r#"a "b" c"#), r#"a \"b\" c"#);
        assert_eq!(yaml_escape(r"a\b"), r"a\\b");
    }

    #[test]
    fn format_float_keeps_one_decimal_for_whole_numbers() {
        assert_eq!(format_float(1.0), "1.0");
        assert_eq!(format_float(1.33), "1.33");
        assert_eq!(format_float(2.5), "2.5");
    }

    #[test]
    fn render_yaml_produces_parseable_output() {
        let draft = TelemetryDraft {
            charter_id: "CHARTER-01".into(),
            charter_title: "Test charter".into(),
            closed_at: "2026-05-02".into(),
            trigger_declared_kind: "event_trigger".into(),
            trigger_declared_description: "first ticket".into(),
            trigger_fired_at: "2026-05-02".into(),
            trigger_fire_clarity: "clear".into(),
            trigger_fire_clarity_notes: String::new(),
            effort_estimated: "M (~1.5h)".into(),
            effort_actual: "M (~1.5h)".into(),
            effort_drift_factor: 1.0,
            effort_drift_reason: String::new(),
            agent_sessions_count: 1,
            agent_hallucinations_caught: 0,
            agent_hallucination_categories: Vec::new(),
            agent_decisions_contradicting: 0,
            agent_context_sufficient: true,
            agent_additional_context_manual: 0,
            agent_r_n_plus_one: 0,
            outcome_completed: true,
            outcome_scope_changes: "ninguno".into(),
            outcome_scope_change_notes: String::new(),
            outcome_new_followups: 0,
            outcome_new_charters: 0,
            qualitative_format_iteration: "v4".into(),
            qualitative_friction_points: Vec::new(),
            qualitative_wins: Vec::new(),
            qualitative_overall_satisfaction: 4,
            qualitative_would_repeat: true,
        };
        let yaml = render_yaml(draft);
        let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        assert!(parsed.get("charter_telemetry").is_some());
        let ct = parsed.get("charter_telemetry").unwrap();
        assert_eq!(
            ct.get("charter_id").and_then(|v| v.as_str()),
            Some("CHARTER-01")
        );
        assert_eq!(
            ct.get("outcome")
                .and_then(|o| o.get("completed_as_planned"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            ct.get("effort")
                .and_then(|e| e.get("estimation_drift_factor"))
                .and_then(|v| v.as_f64()),
            Some(1.0)
        );
    }

    // ── F6 (cli-3.7.1): body status mirror sync ───────────────────────────

    #[test]
    fn sync_body_status_mirror_replaces_en_form() {
        let body = "# Charter: foo\n\n> **Status (mirrored from frontmatter — source of truth is above):** declared. Effort: M (~1.5h).\n>\n> **Origin:** ...\n";
        let out = sync_body_status_mirror(body, "closed");
        assert!(out.contains(":** closed. Effort: M (~1.5h)."), "got:\n{out}");
        assert!(!out.contains(":** declared."), "got:\n{out}");
    }

    #[test]
    fn sync_body_status_mirror_replaces_es_form() {
        let body = "# Charter: foo\n\n> **Status (espejado del frontmatter — la fuente de verdad está arriba):** in-progress. Esfuerzo: S.\n";
        let out = sync_body_status_mirror(body, "closed");
        assert!(out.contains(":** closed. Esfuerzo: S."), "got:\n{out}");
        assert!(!out.contains("in-progress."), "got:\n{out}");
    }

    #[test]
    fn sync_body_status_mirror_handles_in_progress_to_closed() {
        // Status word with hyphen should be replaced cleanly.
        let body = "> **Status (mirrored from frontmatter — source of truth is above):** in-progress. Effort: L.\n";
        let out = sync_body_status_mirror(body, "closed");
        assert_eq!(
            out,
            "> **Status (mirrored from frontmatter — source of truth is above):** closed. Effort: L.\n"
        );
    }

    #[test]
    fn sync_body_status_mirror_leaves_unrelated_status_lines_alone() {
        // An adopter may legitimately have other "**Status**" markers in
        // their body (e.g., status of a sub-task). Without the canonical
        // anchor, those lines must NOT be touched.
        let body = "# Charter\n\n**Status of sub-task:** in-progress (no mirror anchor).\n\n> **Status (mirrored from frontmatter — source of truth is above):** declared.\n";
        let out = sync_body_status_mirror(body, "closed");
        assert!(out.contains("**Status of sub-task:** in-progress"), "got:\n{out}");
        assert!(out.contains(":** closed."), "got:\n{out}");
    }

    #[test]
    fn sync_body_status_mirror_is_noop_when_anchor_absent() {
        let body = "# Charter\n\n> **Origin:** stuff.\n";
        let out = sync_body_status_mirror(body, "closed");
        assert_eq!(out, body);
    }

    #[test]
    fn sync_body_status_mirror_preserves_corrupted_lines() {
        // If the canonical `:**` marker is missing on the anchored line,
        // we don't try to guess — leave it for the operator to fix manually.
        let body = "> **Status (mirrored from frontmatter without colon-marker)** declared\n";
        let out = sync_body_status_mirror(body, "closed");
        assert_eq!(out, body);
    }
}
