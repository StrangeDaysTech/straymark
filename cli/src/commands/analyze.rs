use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::analysis_engine::{self, AnalysisReport, FunctionEntry};
use crate::config::StrayMarkConfig;
use crate::utils::{self, pad_right_visual, truncate_visual, visual_width};

pub fn run(path: &str, threshold: Option<u32>, output: &str, top: Option<usize>) -> Result<()> {
    let target = PathBuf::from(path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path));

    // Try to load config from .straymark/ if available
    let config_threshold = utils::resolve_project_root(path)
        .and_then(|resolved| StrayMarkConfig::load(&resolved.path).ok())
        .map(|c| c.complexity.threshold);

    // Priority: CLI flag > config > default (8)
    let resolved_threshold = threshold
        .or(config_threshold)
        .unwrap_or(8);

    let mut report = analysis_engine::analyze_path(&target, resolved_threshold)?;

    // Sort by cognitive complexity descending
    report.functions.sort_by(|a, b| b.cognitive.cmp(&a.cognitive));

    // Apply --top filter
    if let Some(n) = top {
        report.functions.truncate(n);
    }

    match output {
        "json" => print_json(&report),
        "markdown" => print_markdown(&report, &target),
        _ => print_text(&report, &target),
    }

    Ok(())
}

fn print_text(report: &AnalysisReport, target: &std::path::Path) {
    println!();
    println!("  {}", "StrayMark Analyze".bold().cyan());
    println!("  {}", target.display().to_string().dimmed());
    println!(
        "  {} cognitive complexity > {}",
        "Threshold:".dimmed(),
        report.threshold.to_string().bold()
    );
    println!();

    // Functions exceeding threshold
    let above: Vec<&FunctionEntry> = report
        .functions
        .iter()
        .filter(|f| f.cognitive > report.threshold)
        .collect();

    if above.is_empty() {
        println!(
            "  {} No functions exceed the threshold ({} functions analyzed)",
            "✓".green().bold(),
            report.summary.total_functions
        );
    } else {
        println!(
            "  {} ({} of {} total)",
            "Functions exceeding threshold".bold(),
            above.len().to_string().yellow().bold(),
            report.summary.total_functions
        );
        println!();
        println!(
            "    {} {} {:>5} {:>5} {:>5} {:>5}",
            pad_right_visual("FILE", 40).dimmed(),
            pad_right_visual("FUNCTION", 25).dimmed(),
            "LINE".dimmed(),
            "COGN".dimmed(),
            "CYCL".dimmed(),
            "SLOC".dimmed(),
        );

        for func in &above {
            let cogn_str = func.cognitive.to_string();
            let cogn_colored = if func.cognitive > report.threshold * 2 {
                cogn_str.red().bold()
            } else {
                cogn_str.yellow().bold()
            };
            println!(
                "    {} {} {:>5} {:>5} {:>5} {:>5}",
                truncate_path(&func.file, 40),
                pad_right_visual(&truncate_visual(&func.name, 25), 25),
                func.line,
                cogn_colored,
                func.cyclomatic,
                func.sloc,
            );
        }
    }

    println!();
    println!("  {}", "Summary".bold());
    println!(
        "    {} Files analyzed: {}",
        "→".blue().bold(),
        report.summary.files_analyzed.to_string().bold()
    );
    println!(
        "    {} Total functions: {}",
        "→".blue().bold(),
        report.summary.total_functions.to_string().bold()
    );

    if report.summary.above_threshold > 0 {
        println!(
            "    {} Above threshold: {} ({:.1}%)",
            "→".blue().bold(),
            report.summary.above_threshold.to_string().yellow().bold(),
            report.summary.above_threshold_pct
        );
    } else {
        println!(
            "    {} Above threshold: {}",
            "→".blue().bold(),
            "0".green().bold()
        );
    }

    if report.summary.total_functions > 0 {
        println!(
            "    {} Max cognitive complexity: {} ({})",
            "→".blue().bold(),
            report.summary.max_cognitive.to_string().bold(),
            report.summary.max_cognitive_location.dimmed()
        );
        println!(
            "    {} Average cognitive complexity: {:.1}",
            "→".blue().bold(),
            report.summary.avg_cognitive
        );
    }

    // Warnings
    if !report.warnings.is_empty() {
        println!();
        println!("  {}", "Warnings".bold().yellow());
        for w in &report.warnings {
            println!("    {} {}", "!".yellow().bold(), w);
        }
    }

    println!();
}

fn print_json(report: &AnalysisReport) {
    let json = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into());
    println!("{}", json);
}

fn print_markdown(report: &AnalysisReport, target: &std::path::Path) {
    println!("# StrayMark Analyze Report");
    println!();
    println!("**Path:** `{}`", target.display());
    println!("**Threshold:** cognitive complexity > {}", report.threshold);
    println!();

    let above: Vec<&FunctionEntry> = report
        .functions
        .iter()
        .filter(|f| f.cognitive > report.threshold)
        .collect();

    if !above.is_empty() {
        println!("## Functions Exceeding Threshold");
        println!();
        println!("| File | Function | Line | Cognitive | Cyclomatic | SLOC |");
        println!("|------|----------|------|-----------|------------|------|");
        for func in &above {
            println!(
                "| {} | {} | {} | {} | {} | {} |",
                func.file, func.name, func.line, func.cognitive, func.cyclomatic, func.sloc
            );
        }
        println!();
    }

    println!("## Summary");
    println!();
    println!("- **Files analyzed:** {}", report.summary.files_analyzed);
    println!("- **Total functions:** {}", report.summary.total_functions);
    println!(
        "- **Above threshold:** {} ({:.1}%)",
        report.summary.above_threshold, report.summary.above_threshold_pct
    );
    if report.summary.total_functions > 0 {
        println!(
            "- **Max cognitive complexity:** {} ({})",
            report.summary.max_cognitive, report.summary.max_cognitive_location
        );
        println!(
            "- **Average cognitive complexity:** {:.1}",
            report.summary.avg_cognitive
        );
    }

    if !report.warnings.is_empty() {
        println!();
        println!("## Warnings");
        println!();
        for w in &report.warnings {
            println!("- {}", w);
        }
    }
}

/// Truncate a path string to exactly `max` visual columns, preserving the
/// tail (most meaningful part of a path) with a leading "…". The result is
/// right-padded so the column is always `max` columns wide.
fn truncate_path(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if visual_width(s) <= max {
        return pad_right_visual(s, max);
    }
    // Walk the string from the end, accumulating chars until we've consumed
    // `max - 1` columns (leaving 1 column for the leading "…").
    let budget = max.saturating_sub(1);
    let mut used = 0usize;
    let mut keep_from = s.len();
    for (byte_idx, ch) in s.char_indices().rev() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > budget {
            break;
        }
        used += w;
        keep_from = byte_idx;
    }
    let mut out = String::with_capacity(s.len() - keep_from + 3);
    out.push('…');
    out.push_str(&s[keep_from..]);
    pad_right_visual(&out, max)
}
