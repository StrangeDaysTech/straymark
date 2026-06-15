//! `straymark architecture validate` (Loom A1.3, Spec 002 §3.3, FR9) — report
//! the model↔layout integrity signals.
//!
//! Parses `model.yml`, the `straymark_component_id` cell ids in `plan.drawio`,
//! and the on-disk source inventory, then reports the §3.3 signals via
//! `straymark_core::architecture::validate_model`:
//! - **undrawn** — component in `model.yml` with no DrawIO cell;
//! - **unmodeled** — a DrawIO cell id absent from `model.yml`;
//! - **empty** — globs match no files on disk.
//!
//! ## Exit codes
//! - `0` — model is consistent (no signals).
//! - `1` — at least one integrity signal, a structurally invalid `model.yml`,
//!   or a missing `model.yml` (usage error surfaces via the top-level handler).
//!
//! When `plan.drawio` is absent the command degrades to glob coverage only
//! (drops `undrawn`/`unmodeled`, which are meaningless without a drawing).

use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::Serialize;
use straymark_core::architecture::{parse_model, validate_model, IntegritySignal};

use super::common;
use crate::utils;

#[derive(Serialize)]
struct ValidateReport {
    model: String,
    plan: Option<String>,
    components: usize,
    signals: Vec<SignalRow>,
    ok: bool,
}

#[derive(Serialize)]
struct SignalRow {
    kind: &'static str,
    target: String,
    message: String,
}

pub fn run(path: &str, out: Option<&str>, output: &str) -> Result<()> {
    let root = common::resolve_root(path);
    let (_out_dir, model_path, drawio_path) = common::artifact_paths(&root, out);

    if !model_path.exists() {
        bail!(
            "no architecture model at {} — run {} first",
            model_path.display(),
            "straymark architecture generate".cyan()
        );
    }
    let model = parse_model(&model_path)
        .with_context(|| format!("parsing {}", model_path.display()))?;

    let has_plan = drawio_path.exists();
    let cell_ids: Vec<String> = if has_plan {
        let xml = std::fs::read_to_string(&drawio_path)
            .with_context(|| format!("reading {}", drawio_path.display()))?;
        super::drawio::parse_component_ids(&xml)
    } else {
        Vec::new()
    };

    let on_disk: Vec<String> = common::collect_source_files(&root)
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();

    let mut signals = validate_model(&model, &cell_ids, &on_disk);
    // Degraded mode (no plan.drawio): undrawn/unmodeled are meaningless.
    if !has_plan {
        signals.retain(|s| matches!(s, IntegritySignal::Empty(_)));
    }

    let rows: Vec<SignalRow> = signals.iter().map(signal_row).collect();
    let report = ValidateReport {
        model: model_path.display().to_string(),
        plan: has_plan.then(|| drawio_path.display().to_string()),
        components: model.components.len(),
        ok: rows.is_empty(),
        signals: rows,
    };

    match output {
        "json" => print_json(&report),
        "markdown" => print_markdown(&report, has_plan),
        _ => print_text(&report, has_plan),
    }

    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

fn signal_row(s: &IntegritySignal) -> SignalRow {
    match s {
        IntegritySignal::Undrawn(id) => SignalRow {
            kind: "undrawn",
            target: id.clone(),
            message: format!("component `{id}` has no cell in plan.drawio"),
        },
        IntegritySignal::Unmodeled(id) => SignalRow {
            kind: "unmodeled",
            target: id.clone(),
            message: format!("plan.drawio cell `{id}` is not in model.yml"),
        },
        IntegritySignal::Empty(id) => SignalRow {
            kind: "empty",
            target: id.clone(),
            message: format!("component `{id}` globs match no files on disk"),
        },
    }
}

fn print_json(report: &ValidateReport) {
    println!(
        "{}",
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into())
    );
}

fn print_text(report: &ValidateReport, has_plan: bool) {
    if !has_plan {
        utils::warn("no plan.drawio — checked glob coverage only (run `architecture generate`).");
    }
    if report.ok {
        utils::success(&format!(
            "Architecture model is consistent ({} components).",
            report.components
        ));
        return;
    }
    utils::warn(&format!(
        "{} integrity signal{} in {}:",
        report.signals.len(),
        common::plural(report.signals.len()),
        report.model
    ));
    for row in &report.signals {
        println!("  {} {}", format!("[{}]", row.kind).yellow(), row.message);
    }
}

fn print_markdown(report: &ValidateReport, has_plan: bool) {
    println!("# Architecture validation\n");
    println!("- Model: `{}`", report.model);
    match &report.plan {
        Some(p) => println!("- Plan: `{p}`"),
        None => println!("- Plan: _absent — glob coverage only_"),
    }
    println!("- Components: {}", report.components);
    println!();
    if report.ok {
        println!("✅ No integrity signals.");
        return;
    }
    let _ = has_plan;
    println!("| Signal | Target | Detail |");
    println!("|--------|--------|--------|");
    for row in &report.signals {
        println!("| {} | `{}` | {} |", row.kind, row.target, row.message);
    }
}
