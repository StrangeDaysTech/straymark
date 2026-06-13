use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::compliance::{self, CheckStatus, ComplianceReport, Standard};
use crate::config::StrayMarkConfig;
use straymark_core::document;
use crate::utils;

pub fn run(
    path: &str,
    standard: Option<&str>,
    region: Option<&str>,
    all: bool,
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

    // Discover and parse all documents
    let paths = document::discover_documents(&straymark_dir);
    let docs: Vec<_> = paths
        .iter()
        .filter_map(|p| document::parse_document(p).ok())
        .collect();

    let config = StrayMarkConfig::load(&target).unwrap_or_default();

    // Resolve which standards to run.
    let standards = resolve_standards(&config, standard, region, all);

    if standards.is_empty() {
        utils::warn(&format!(
            "No standards selected. regional_scope is {:?}. Use --standard, --region, or set regional_scope in .straymark/config.yml.",
            config.regional_scope
        ));
        return Ok(());
    }

    let mut reports: Vec<ComplianceReport> = Vec::new();
    for s in &standards {
        let r = match s {
            Standard::EuAiAct => compliance::check_eu_ai_act(&docs, &straymark_dir),
            Standard::Iso42001 => compliance::check_iso_42001(&docs, &straymark_dir),
            Standard::NistAiRmf => compliance::check_nist_ai_rmf(&docs, &straymark_dir),
            Standard::ChinaTc260 => compliance::check_china_tc260(&docs, &straymark_dir),
            Standard::ChinaPipl => compliance::check_china_pipl(&docs, &straymark_dir),
            Standard::ChinaGb45438 => compliance::check_china_gb45438(&docs, &straymark_dir),
            Standard::ChinaCac => compliance::check_china_cac(&docs, &straymark_dir),
            Standard::ChinaGb45652 => compliance::check_china_gb45652(&docs, &straymark_dir),
            Standard::ChinaCsl => compliance::check_china_csl(&docs, &straymark_dir),
        };
        reports.push(r);
    }

    // Output
    match output {
        "json" => print_json(&reports),
        "markdown" => print_markdown(&reports, docs.len()),
        _ => print_text(&reports, &target, docs.len()),
    }

    Ok(())
}

/// Decide which standards to run given CLI flags and the project's regional_scope.
///
/// Precedence:
/// 1. `--standard <name>` — single standard, always honored.
/// 2. `--all` — every standard known to StrayMark (independent of regional_scope).
/// 3. `--region <name>` — every standard whose `region()` matches the value
///    (`all` matches every region; `china` requires no opt-in for explicit overrides).
/// 4. Default — every standard whose region appears in `regional_scope`.
fn resolve_standards(
    config: &StrayMarkConfig,
    standard: Option<&str>,
    region: Option<&str>,
    all: bool,
) -> Vec<Standard> {
    if let Some(name) = standard {
        return match name {
            "eu-ai-act" => vec![Standard::EuAiAct],
            "iso-42001" => vec![Standard::Iso42001],
            "nist-ai-rmf" => vec![Standard::NistAiRmf],
            "china-tc260" => vec![Standard::ChinaTc260],
            "china-pipl" => vec![Standard::ChinaPipl],
            "china-gb45438" => vec![Standard::ChinaGb45438],
            "china-cac" => vec![Standard::ChinaCac],
            "china-gb45652" => vec![Standard::ChinaGb45652],
            "china-csl" => vec![Standard::ChinaCsl],
            _ => vec![],
        };
    }

    let all_standards = [
        Standard::EuAiAct,
        Standard::Iso42001,
        Standard::NistAiRmf,
        Standard::ChinaTc260,
        Standard::ChinaPipl,
        Standard::ChinaGb45438,
        Standard::ChinaCac,
        Standard::ChinaGb45652,
        Standard::ChinaCsl,
    ];

    if all {
        return all_standards.to_vec();
    }

    if let Some(r) = region {
        let r_lower = r.to_ascii_lowercase();
        if r_lower == "all" {
            return all_standards.to_vec();
        }
        return all_standards
            .iter()
            .copied()
            .filter(|s| s.region() == r_lower)
            .collect();
    }

    // Default: every standard whose region is in regional_scope.
    all_standards
        .iter()
        .copied()
        .filter(|s| config.has_region(s.region()))
        .collect()
}

fn print_text(reports: &[ComplianceReport], target: &std::path::Path, doc_count: usize) {
    println!();
    println!("  {}", "StrayMark Compliance".bold().cyan());
    println!("  {}", target.display().to_string().dimmed());
    println!(
        "  {}",
        format!("{} document(s) analyzed", doc_count).dimmed()
    );
    println!();

    for report in reports {
        let score_color = if report.score >= 80.0 {
            format!("{:.0}%", report.score).green().bold()
        } else if report.score >= 50.0 {
            format!("{:.0}%", report.score).yellow().bold()
        } else {
            format!("{:.0}%", report.score).red().bold()
        };

        println!(
            "  {} {} {}",
            "■".cyan().bold(),
            report.standard_label.bold(),
            score_color
        );

        for check in &report.checks {
            let status_icon = match check.status {
                CheckStatus::Pass => "✓".green().bold(),
                CheckStatus::Partial => "~".yellow().bold(),
                CheckStatus::Fail => "✗".red().bold(),
            };

            println!("    {} [{}] {}", status_icon, check.id, check.description);

            if !check.evidence.is_empty() && check.status != CheckStatus::Fail {
                let evidence_str = if check.evidence.len() <= 3 {
                    check.evidence.join(", ")
                } else {
                    format!(
                        "{}, ... (+{} more)",
                        check.evidence[..3].join(", "),
                        check.evidence.len() - 3
                    )
                };
                println!("      {}", evidence_str.dimmed());
            }

            if let Some(remediation) = &check.remediation {
                if check.status != CheckStatus::Pass {
                    println!("      {} {}", "fix:".dimmed(), remediation.dimmed());
                }
            }
        }
        println!();
    }

    // Overall summary
    if reports.len() > 1 {
        let avg_score: f64 = reports.iter().map(|r| r.score).sum::<f64>() / reports.len() as f64;
        let summary_color = if avg_score >= 80.0 {
            format!("  Overall compliance: {:.0}%", avg_score)
                .green()
                .bold()
        } else if avg_score >= 50.0 {
            format!("  Overall compliance: {:.0}%", avg_score)
                .yellow()
                .bold()
        } else {
            format!("  Overall compliance: {:.0}%", avg_score)
                .red()
                .bold()
        };
        println!("{}", summary_color);
        println!();
    }
}

fn print_json(reports: &[ComplianceReport]) {
    let json = serde_json::to_string_pretty(reports).unwrap_or_else(|_| "[]".into());
    println!("{}", json);
}

fn print_markdown(reports: &[ComplianceReport], doc_count: usize) {
    println!("# StrayMark Compliance Report");
    println!();
    println!("**Documents analyzed:** {}", doc_count);
    println!();

    for report in reports {
        println!("## {} — {:.0}%", report.standard_label, report.score);
        println!();
        println!("| Check | Status | Description |");
        println!("|-------|--------|-------------|");

        for check in &report.checks {
            let status_emoji = match check.status {
                CheckStatus::Pass => "✅",
                CheckStatus::Partial => "⚠️",
                CheckStatus::Fail => "❌",
            };
            println!(
                "| {} | {} | {} |",
                check.id, status_emoji, check.description
            );
        }
        println!();
    }

    if reports.len() > 1 {
        let avg_score: f64 = reports.iter().map(|r| r.score).sum::<f64>() / reports.len() as f64;
        println!("---");
        println!();
        println!("**Overall compliance: {:.0}%**", avg_score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(scope: &[&str]) -> StrayMarkConfig {
        StrayMarkConfig {
            regional_scope: scope.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn standard_flag_overrides_scope() {
        let resolved = resolve_standards(&cfg(&["global"]), Some("china-pipl"), None, false);
        assert_eq!(resolved, vec![Standard::ChinaPipl]);
    }

    #[test]
    fn region_flag_filters_by_region() {
        let resolved = resolve_standards(&cfg(&["global"]), None, Some("china"), false);
        assert!(resolved.contains(&Standard::ChinaTc260));
        assert!(resolved.contains(&Standard::ChinaCsl));
        assert!(!resolved.contains(&Standard::EuAiAct));
    }

    #[test]
    fn all_flag_includes_china_even_without_scope() {
        let resolved = resolve_standards(&cfg(&["global", "eu"]), None, None, true);
        assert_eq!(resolved.len(), 9);
    }

    #[test]
    fn default_filters_by_regional_scope_excluding_china() {
        let resolved = resolve_standards(&cfg(&["global", "eu"]), None, None, false);
        assert!(resolved.contains(&Standard::EuAiAct));
        assert!(resolved.contains(&Standard::Iso42001));
        assert!(resolved.contains(&Standard::NistAiRmf));
        assert!(!resolved.contains(&Standard::ChinaTc260));
        assert!(!resolved.contains(&Standard::ChinaPipl));
    }

    #[test]
    fn default_includes_china_when_in_scope() {
        let resolved = resolve_standards(&cfg(&["global", "china"]), None, None, false);
        assert!(resolved.contains(&Standard::ChinaTc260));
        assert!(resolved.contains(&Standard::ChinaPipl));
        assert!(resolved.contains(&Standard::ChinaCsl));
        assert!(!resolved.contains(&Standard::EuAiAct));
    }
}
