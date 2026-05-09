use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::path::PathBuf;

use crate::document;
use crate::metrics_engine::{self, MetricsReport, Period, TrendDirection};
use crate::utils;

pub fn run(path: &str, period: &str, output: &str) -> Result<()> {
    let resolved = match utils::resolve_project_root(path) {
        Some(r) => r,
        None => {
            let target = PathBuf::from(path)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(path));
            utils::info(&format!(
                "StrayMark is not installed in {}",
                target.display()
            ));
            utils::info("Run 'straymark init' to initialize StrayMark in this directory.");
            return Ok(());
        }
    };

    if resolved.is_fallback {
        utils::info(&format!(
            "Using StrayMark installation at repo root: {}",
            resolved.path.display()
        ));
    }

    let target = resolved.path;
    let straymark_dir = target.join(".straymark");

    // Discover and parse all documents
    let paths = document::discover_documents(&straymark_dir);
    let docs: Vec<_> = paths
        .iter()
        .filter_map(|p| document::parse_document(p).ok())
        .collect();

    let period_enum = Period::from_str(period);
    let now = Local::now().date_naive();
    let report = metrics_engine::calculate_metrics(&docs, period_enum, now);

    match output {
        "json" => print_json(&report),
        "markdown" => print_markdown(&report),
        _ => print_text(&report, &target),
    }

    Ok(())
}

fn print_text(report: &MetricsReport, target: &std::path::Path) {
    println!();
    println!("  {}", "StrayMark Metrics".bold().cyan());
    println!("  {}", target.display().to_string().dimmed());
    println!(
        "  {} {} — {}",
        "Period:".dimmed(),
        report.period_label.dimmed(),
        format!("{} to {}", report.period_start, report.period_end).dimmed()
    );
    println!();

    // Document counts
    println!("  {}", "Documents by Type".bold());
    let active_counts: Vec<&(String, usize)> =
        report.doc_counts.iter().filter(|(_, c)| *c > 0).collect();

    if active_counts.is_empty() {
        println!("    {} No documents in this period", "→".blue().bold());
    } else {
        for (doc_type, count) in &active_counts {
            let bar = "█".repeat((*count).min(20));
            println!(
                "    {:>6} {} {}",
                doc_type.cyan().bold(),
                format!("{:>3}", count).bold(),
                bar.cyan()
            );
        }
    }
    println!();

    // Summary
    println!("  {}", "Summary".bold());
    println!(
        "    {} Total documents: {}",
        "→".blue().bold(),
        report.total_docs.to_string().bold()
    );

    // Review compliance
    let review_color = if report.review_compliance.rate >= 80.0 {
        format!("{:.0}%", report.review_compliance.rate)
            .green()
            .bold()
    } else if report.review_compliance.rate >= 50.0 {
        format!("{:.0}%", report.review_compliance.rate)
            .yellow()
            .bold()
    } else {
        format!("{:.0}%", report.review_compliance.rate)
            .red()
            .bold()
    };
    println!(
        "    {} Review compliance: {} ({}/{} reviewed)",
        "→".blue().bold(),
        review_color,
        report.review_compliance.reviewed,
        report.review_compliance.total_requiring_review
    );
    println!();

    // Risk distribution
    let has_risks = report.risk_distribution.iter().any(|(_, c)| *c > 0);
    if has_risks {
        println!("  {}", "Risk Distribution".bold());
        for (level, count) in &report.risk_distribution {
            if *count == 0 {
                continue;
            }
            let level_color = match level.as_str() {
                "critical" => level.red().bold(),
                "high" => level.red(),
                "medium" => level.yellow(),
                "low" => level.green(),
                _ => level.normal(),
            };
            println!("    {:>10} {}", level_color, count);
        }
        println!();
    }

    // Agent activity
    if !report.agent_activity.is_empty() {
        println!("  {}", "Agent Activity".bold());
        for (agent, count) in &report.agent_activity {
            println!("    {} {}", agent.cyan(), count);
        }
        println!();
    }

    // Trends
    if !report.trends.is_empty() {
        println!("  {}", "Trends".bold());
        for trend in &report.trends {
            let arrow = match trend.direction {
                TrendDirection::Up => trend.direction.symbol().green().bold(),
                TrendDirection::Down => trend.direction.symbol().red().bold(),
                TrendDirection::Stable => trend.direction.symbol().dimmed(),
            };
            println!(
                "    {} {} {} (was {})",
                arrow, trend.metric, trend.current, trend.previous
            );
        }
        println!();
    }
}

fn print_json(report: &MetricsReport) {
    let json = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into());
    println!("{}", json);
}

fn print_markdown(report: &MetricsReport) {
    println!("# StrayMark Metrics Report");
    println!();
    println!(
        "**Period:** {} ({} to {})",
        report.period_label, report.period_start, report.period_end
    );
    println!("**Total documents:** {}", report.total_docs);
    println!();

    // Document counts table
    let active: Vec<&(String, usize)> = report.doc_counts.iter().filter(|(_, c)| *c > 0).collect();
    if !active.is_empty() {
        println!("## Documents by Type");
        println!();
        println!("| Type | Count |");
        println!("|------|-------|");
        for (doc_type, count) in &active {
            println!("| {} | {} |", doc_type, count);
        }
        println!();
    }

    // Review compliance
    println!("## Review Compliance");
    println!();
    println!(
        "- **Rate:** {:.0}% ({}/{} reviewed)",
        report.review_compliance.rate,
        report.review_compliance.reviewed,
        report.review_compliance.total_requiring_review
    );
    println!();

    // Risk distribution
    let has_risks = report.risk_distribution.iter().any(|(_, c)| *c > 0);
    if has_risks {
        println!("## Risk Distribution");
        println!();
        println!("| Level | Count |");
        println!("|-------|-------|");
        for (level, count) in &report.risk_distribution {
            if *count > 0 {
                println!("| {} | {} |", level, count);
            }
        }
        println!();
    }

    // Agent activity
    if !report.agent_activity.is_empty() {
        println!("## Agent Activity");
        println!();
        println!("| Agent | Documents |");
        println!("|-------|-----------|");
        for (agent, count) in &report.agent_activity {
            println!("| {} | {} |", agent, count);
        }
        println!();
    }

    // Trends
    if !report.trends.is_empty() {
        println!("## Trends");
        println!();
        println!("| Metric | Current | Previous | Trend |");
        println!("|--------|---------|----------|-------|");
        for trend in &report.trends {
            println!(
                "| {} | {} | {} | {} |",
                trend.metric,
                trend.current,
                trend.previous,
                trend.direction.symbol()
            );
        }
    }
}
