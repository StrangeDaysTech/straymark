use anyhow::{bail, Result};
use colored::Colorize;
use std::path::PathBuf;

use crate::audit_engine::{self, AuditReport};
use crate::compliance::CheckStatus;
use straymark_core::document;
use crate::utils;

pub fn run(
    path: &str,
    from: Option<&str>,
    to: Option<&str>,
    system: Option<&str>,
    output: &str,
) -> Result<()> {
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

    // Parse date arguments
    let from_date = match from {
        Some(s) => {
            let d = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d");
            match d {
                Ok(date) => Some(date),
                Err(_) => bail!("Invalid --from date '{}'. Expected format: YYYY-MM-DD", s),
            }
        }
        None => None,
    };

    let to_date = match to {
        Some(s) => {
            let d = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d");
            match d {
                Ok(date) => Some(date),
                Err(_) => bail!("Invalid --to date '{}'. Expected format: YYYY-MM-DD", s),
            }
        }
        None => None,
    };

    // Discover and parse all documents
    let paths = document::discover_documents(&straymark_dir);
    let docs: Vec<_> = paths
        .iter()
        .filter_map(|p| document::parse_document(p).ok())
        .collect();

    let report = audit_engine::generate_audit(&docs, from_date, to_date, system, &straymark_dir);

    match output {
        "json" => print_json(&report),
        "markdown" => print_markdown(&report),
        "html" => print_html(&report),
        _ => print_text(&report, &target),
    }

    Ok(())
}

fn print_text(report: &AuditReport, target: &std::path::Path) {
    println!();
    println!("  {}", "StrayMark Audit Report".bold().cyan());
    println!("  {}", target.display().to_string().dimmed());
    println!(
        "  {} {} — {}",
        "Period:".dimmed(),
        report.period_start.dimmed(),
        report.period_end.dimmed()
    );
    if let Some(sys) = &report.system_filter {
        println!("  {} {}", "System:".dimmed(), sys.dimmed());
    }
    println!(
        "  {}",
        format!("{} document(s)", report.total_docs).dimmed()
    );
    println!();

    // Timeline
    if !report.timeline.is_empty() {
        println!("  {}", "Timeline".bold());
        for entry in &report.timeline {
            let risk_color = match entry.risk_level.as_str() {
                "critical" => entry.risk_level.red().bold(),
                "high" => entry.risk_level.red(),
                "medium" => entry.risk_level.yellow(),
                "low" => entry.risk_level.green(),
                _ => entry.risk_level.dimmed(),
            };
            println!(
                "    {} {} {} {}",
                entry.date.dimmed(),
                entry.doc_type.cyan().bold(),
                entry.title,
                risk_color
            );
            println!(
                "         {} {} {}",
                entry.id.dimmed(),
                "by".dimmed(),
                entry.agent.dimmed()
            );
        }
        println!();
    }

    // Traceability
    if !report.traceability_chains.is_empty() {
        println!("  {}", "Traceability Map".bold());
        for chain in &report.traceability_chains {
            print!(
                "    {} {}",
                chain.root.doc_type.cyan().bold(),
                chain.root.id
            );
            for node in &chain.chain {
                print!(" → {} {}", node.doc_type.cyan().bold(), node.id);
            }
            println!();
        }
        println!();
    }

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

    // Compliance summary
    if !report.compliance_summary.is_empty() {
        println!("  {}", "Compliance Summary".bold());
        for cr in &report.compliance_summary {
            let score_color = if cr.score >= 80.0 {
                format!("{:.0}%", cr.score).green().bold()
            } else if cr.score >= 50.0 {
                format!("{:.0}%", cr.score).yellow().bold()
            } else {
                format!("{:.0}%", cr.score).red().bold()
            };
            println!(
                "    {} {} {}",
                "■".cyan().bold(),
                cr.standard_label,
                score_color
            );
        }
        println!();
    }
}

fn print_json(report: &AuditReport) {
    let json = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into());
    println!("{}", json);
}

fn print_markdown(report: &AuditReport) {
    println!("# StrayMark Audit Report");
    println!();

    // Executive summary
    println!("## Executive Summary");
    println!();
    println!(
        "- **Period:** {} to {}",
        report.period_start, report.period_end
    );
    if let Some(sys) = &report.system_filter {
        println!("- **System filter:** {}", sys);
    }
    println!("- **Total documents:** {}", report.total_docs);
    println!();

    // Timeline
    if !report.timeline.is_empty() {
        println!("## Timeline");
        println!();
        println!("| Date | Type | ID | Title | Agent | Risk |");
        println!("|------|------|----|-------|-------|------|");
        for entry in &report.timeline {
            println!(
                "| {} | {} | {} | {} | {} | {} |",
                entry.date,
                entry.doc_type,
                entry.id,
                entry.title,
                entry.agent,
                entry.risk_level
            );
        }
        println!();
    }

    // Traceability
    if !report.traceability_chains.is_empty() {
        println!("## Traceability Map");
        println!();
        for chain in &report.traceability_chains {
            print!(
                "- **{}** `{}`",
                chain.root.doc_type, chain.root.id
            );
            for node in &chain.chain {
                print!(" → **{}** `{}`", node.doc_type, node.id);
            }
            println!();
        }
        println!();
    }

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

    // Compliance summary
    if !report.compliance_summary.is_empty() {
        println!("## Compliance Summary");
        println!();
        println!("| Standard | Score | Checks |");
        println!("|----------|-------|--------|");
        for cr in &report.compliance_summary {
            let passed = cr
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Pass)
                .count();
            println!(
                "| {} | {:.0}% | {}/{} passed |",
                cr.standard_label,
                cr.score,
                passed,
                cr.checks.len()
            );
        }
    }
}

fn print_html(report: &AuditReport) {
    println!(
        "<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta charset=\"UTF-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
<title>StrayMark Audit Report</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #1a1b26; color: #a9b1d6; max-width: 960px; margin: 0 auto; padding: 2rem; }}
  h1 {{ color: #7aa2f7; border-bottom: 2px solid #3b4261; padding-bottom: 0.5rem; }}
  h2 {{ color: #bb9af7; margin-top: 2rem; }}
  table {{ width: 100%; border-collapse: collapse; margin: 1rem 0; }}
  th {{ background: #24283b; color: #7aa2f7; text-align: left; padding: 0.6rem 0.8rem; border-bottom: 2px solid #3b4261; }}
  td {{ padding: 0.5rem 0.8rem; border-bottom: 1px solid #3b4261; }}
  tr:hover {{ background: #24283b; }}
  .summary {{ background: #24283b; border-radius: 8px; padding: 1rem 1.5rem; margin: 1rem 0; }}
  .summary p {{ margin: 0.3rem 0; }}
  .risk-low {{ color: #9ece6a; }}
  .risk-medium {{ color: #e0af68; }}
  .risk-high {{ color: #f7768e; }}
  .risk-critical {{ color: #f7768e; font-weight: bold; }}
  .chain {{ background: #24283b; border-radius: 6px; padding: 0.6rem 1rem; margin: 0.4rem 0; font-family: monospace; }}
  .chain-arrow {{ color: #7aa2f7; }}
  .score-good {{ color: #9ece6a; font-weight: bold; }}
  .score-mid {{ color: #e0af68; font-weight: bold; }}
  .score-low {{ color: #f7768e; font-weight: bold; }}
  svg {{ display: block; margin: 1rem auto; }}
  .legend {{ display: flex; gap: 1.5rem; justify-content: center; margin-top: 0.5rem; }}
  .legend-item {{ display: flex; align-items: center; gap: 0.4rem; }}
  .legend-color {{ width: 14px; height: 14px; border-radius: 3px; display: inline-block; }}
</style>
</head>
<body>
<h1>StrayMark Audit Report</h1>"
    );

    // Summary
    println!("<div class=\"summary\">");
    println!(
        "<p><strong>Period:</strong> {} to {}</p>",
        report.period_start, report.period_end
    );
    if let Some(sys) = &report.system_filter {
        println!("<p><strong>System filter:</strong> {}</p>", escape_html(sys));
    }
    println!(
        "<p><strong>Total documents:</strong> {}</p>",
        report.total_docs
    );
    println!("</div>");

    // Timeline
    if !report.timeline.is_empty() {
        println!("<h2>Timeline</h2>");
        println!("<table><thead><tr><th>Date</th><th>Type</th><th>ID</th><th>Title</th><th>Agent</th><th>Risk</th></tr></thead><tbody>");
        for entry in &report.timeline {
            let risk_class = match entry.risk_level.as_str() {
                "critical" => "risk-critical",
                "high" => "risk-high",
                "medium" => "risk-medium",
                "low" => "risk-low",
                _ => "",
            };
            println!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
                entry.date,
                entry.doc_type,
                escape_html(&entry.id),
                escape_html(&entry.title),
                escape_html(&entry.agent),
                risk_class,
                entry.risk_level
            );
        }
        println!("</tbody></table>");
    }

    // Traceability
    if !report.traceability_chains.is_empty() {
        println!("<h2>Traceability Map</h2>");
        for chain in &report.traceability_chains {
            print!(
                "<div class=\"chain\"><strong>{}</strong> {}",
                chain.root.doc_type,
                escape_html(&chain.root.id)
            );
            for node in &chain.chain {
                print!(
                    " <span class=\"chain-arrow\">→</span> <strong>{}</strong> {}",
                    node.doc_type,
                    escape_html(&node.id)
                );
            }
            println!("</div>");
        }
    }

    // Risk distribution with SVG pie chart
    let risk_total: usize = report.risk_distribution.iter().map(|(_, c)| *c).sum();
    if risk_total > 0 {
        println!("<h2>Risk Distribution</h2>");
        print_svg_pie(&report.risk_distribution, risk_total);

        println!("<table><thead><tr><th>Level</th><th>Count</th></tr></thead><tbody>");
        for (level, count) in &report.risk_distribution {
            if *count > 0 {
                let risk_class = match level.as_str() {
                    "critical" => "risk-critical",
                    "high" => "risk-high",
                    "medium" => "risk-medium",
                    "low" => "risk-low",
                    _ => "",
                };
                println!(
                    "<tr><td class=\"{}\">{}</td><td>{}</td></tr>",
                    risk_class, level, count
                );
            }
        }
        println!("</tbody></table>");
    }

    // Compliance summary
    if !report.compliance_summary.is_empty() {
        println!("<h2>Compliance Summary</h2>");
        println!("<table><thead><tr><th>Standard</th><th>Score</th><th>Checks</th></tr></thead><tbody>");
        for cr in &report.compliance_summary {
            let score_class = if cr.score >= 80.0 {
                "score-good"
            } else if cr.score >= 50.0 {
                "score-mid"
            } else {
                "score-low"
            };
            let passed = cr
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Pass)
                .count();
            println!(
                "<tr><td>{}</td><td class=\"{}\">{:.0}%</td><td>{}/{} passed</td></tr>",
                cr.standard_label,
                score_class,
                cr.score,
                passed,
                cr.checks.len()
            );
        }
        println!("</tbody></table>");
    }

    println!("</body>\n</html>");
}

/// Generate an SVG pie chart for risk distribution
fn print_svg_pie(distribution: &[(String, usize)], total: usize) {
    let colors = [
        ("low", "#9ece6a"),
        ("medium", "#e0af68"),
        ("high", "#f7768e"),
        ("critical", "#db4b4b"),
    ];

    let cx: f64 = 80.0;
    let cy: f64 = 80.0;
    let r: f64 = 70.0;

    println!("<svg width=\"160\" height=\"160\" viewBox=\"0 0 160 160\" xmlns=\"http://www.w3.org/2000/svg\">");

    if total == 0 {
        println!("</svg>");
        return;
    }

    // If only one non-zero segment, draw a full circle
    let non_zero: Vec<_> = distribution.iter().filter(|(_, c)| *c > 0).collect();
    if non_zero.len() == 1 {
        let color = colors
            .iter()
            .find(|(l, _)| *l == non_zero[0].0)
            .map(|(_, c)| *c)
            .unwrap_or("#7aa2f7");
        println!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
            cx, cy, r, color
        );
    } else {
        let mut start_angle: f64 = -std::f64::consts::FRAC_PI_2; // start at top

        for (level, count) in distribution {
            if *count == 0 {
                continue;
            }

            let fraction = *count as f64 / total as f64;
            let sweep = fraction * 2.0 * std::f64::consts::PI;
            let end_angle = start_angle + sweep;

            let x1 = cx + r * start_angle.cos();
            let y1 = cy + r * start_angle.sin();
            let x2 = cx + r * end_angle.cos();
            let y2 = cy + r * end_angle.sin();

            let large_arc = if sweep > std::f64::consts::PI {
                1
            } else {
                0
            };

            let color = colors
                .iter()
                .find(|(l, _)| *l == level.as_str())
                .map(|(_, c)| *c)
                .unwrap_or("#7aa2f7");

            println!(
                "<path d=\"M {cx} {cy} L {x1:.2} {y1:.2} A {r} {r} 0 {large_arc} 1 {x2:.2} {y2:.2} Z\" fill=\"{color}\"/>"
            );

            start_angle = end_angle;
        }
    }

    println!("</svg>");

    // Legend
    println!("<div class=\"legend\">");
    for (level, count) in distribution {
        if *count == 0 {
            continue;
        }
        let color = colors
            .iter()
            .find(|(l, _)| *l == level.as_str())
            .map(|(_, c)| *c)
            .unwrap_or("#7aa2f7");
        println!(
            "<span class=\"legend-item\"><span class=\"legend-color\" style=\"background:{}\"></span>{} ({})</span>",
            color, level, count
        );
    }
    println!("</div>");
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
