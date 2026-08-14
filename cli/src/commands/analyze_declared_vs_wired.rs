//! `straymark analyze declared-vs-wired` — config-driven set-difference check
//! for the "surface declaration without wiring" anti-pattern (sub-class 5:
//! client-side IPC/RPC proxy method declared vs server interface implemented).
//!
//! See `.straymark/00-governance/POLISH-CHARTER-PATTERN.md`. This is the v0
//! crystallization gated on the LNXDrive N=2 validation (findings #209/#210).
//!
//! ## Why config-driven, not AST-based
//!
//! The five sub-classes are language/IPC-specific; a generic AST analyzer is not
//! tractable cross-stack. Instead the operator supplies the stack-specific
//! knowledge as two (glob, regex) pairs — a *declared* side and a *wired* side —
//! and the command reports the set difference of the captured symbol names:
//!
//! - **D \ W** (declared but not wired): a proxy method the client declares that
//!   no implemented interface provides — the LNXDrive regression. Always reported.
//! - **W \ D** (wired but not declared): an implemented method no client calls —
//!   reported only with `--show-orphans` (often benign).
//!
//! Each pattern's **capture group 1** is the symbol name compared across sides.
//!
//! ## Exit codes
//! - `0` — no declared-but-not-wired symbols (clean)
//! - `1` — at least one declared symbol has no wiring counterpart (a finding),
//!   or a usage error (missing profile/flags, bad regex, unreadable config) —
//!   the latter surfaces via the top-level `error:` handler in `main`.

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use regex::Regex;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::config::{DeclaredVsWiredProfile, StrayMarkConfig};
use crate::tree_grep::collect_symbols;
use crate::utils;

/// Parsed CLI inputs for the subcommand.
pub struct Args {
    pub path: String,
    pub profile: Option<String>,
    pub declared_glob: Option<String>,
    pub wired_glob: Option<String>,
    pub declared_pattern: Option<String>,
    pub wired_pattern: Option<String>,
    pub show_orphans: bool,
    pub output: String,
}

#[derive(Serialize)]
struct Finding {
    symbol: String,
    /// First file the symbol was found in (relative to the analyzed path).
    source: String,
}

#[derive(Serialize)]
struct Report {
    path: String,
    profile: String,
    declared_count: usize,
    wired_count: usize,
    declared_not_wired: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wired_not_declared: Option<Vec<Finding>>,
}

pub fn run(args: Args) -> Result<()> {
    let base = PathBuf::from(&args.path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&args.path));

    let (profile_name, profile) = resolve_profile(&args, &base)?;

    let declared_re = compile(&profile.declared_pattern, "declared")?;
    let wired_re = compile(&profile.wired_pattern, "wired")?;

    let declared = collect_symbols(&base, &profile.declared_glob, &declared_re)
        .with_context(|| format!("scanning declared glob `{}`", profile.declared_glob))?;
    let wired = collect_symbols(&base, &profile.wired_glob, &wired_re)
        .with_context(|| format!("scanning wired glob `{}`", profile.wired_glob))?;

    let declared_not_wired: Vec<Finding> = declared
        .iter()
        .filter(|(sym, _)| !wired.contains_key(*sym))
        .map(|(sym, src)| Finding { symbol: sym.clone(), source: src.clone() })
        .collect();

    let wired_not_declared = if args.show_orphans {
        Some(
            wired
                .iter()
                .filter(|(sym, _)| !declared.contains_key(*sym))
                .map(|(sym, src)| Finding { symbol: sym.clone(), source: src.clone() })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let report = Report {
        path: base.display().to_string(),
        profile: profile_name,
        declared_count: declared.len(),
        wired_count: wired.len(),
        declared_not_wired,
        wired_not_declared,
    };

    match args.output.as_str() {
        "json" => print_json(&report),
        "markdown" => print_markdown(&report),
        _ => print_text(&report),
    }

    if !report.declared_not_wired.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

/// Resolve the effective profile from either a named config profile
/// (`--profile`) or the four inline flags. The two are mutually exclusive in
/// practice: a named profile wins, inline flags fill the gap otherwise.
fn resolve_profile(args: &Args, base: &Path) -> Result<(String, DeclaredVsWiredProfile)> {
    if let Some(name) = &args.profile {
        let project_root = utils::resolve_project_root(&args.path)
            .map(|r| r.path)
            .unwrap_or_else(|| base.to_path_buf());
        let config = StrayMarkConfig::load(&project_root)
            .context("loading .straymark/config.yml for --profile lookup")?;
        let profile = config
            .declared_vs_wired
            .profiles
            .into_iter()
            .find(|p| p.name == *name)
            .ok_or_else(|| {
                anyhow!(
                    "profile '{}' not found in .straymark/config.yml under `declared_vs_wired.profiles`.\n  \
                     hint: define it there, or pass --declared-glob/--wired-glob/--declared-pattern/--wired-pattern inline.",
                    name
                )
            })?;
        return Ok((name.clone(), profile));
    }

    // Inline mode: all four flags required.
    match (
        &args.declared_glob,
        &args.wired_glob,
        &args.declared_pattern,
        &args.wired_pattern,
    ) {
        (Some(dg), Some(wg), Some(dp), Some(wp)) => Ok((
            "inline".to_string(),
            DeclaredVsWiredProfile {
                name: "inline".to_string(),
                declared_glob: dg.clone(),
                declared_pattern: dp.clone(),
                wired_glob: wg.clone(),
                wired_pattern: wp.clone(),
            },
        )),
        _ => bail!(
            "declared-vs-wired needs either --profile <name> (from .straymark/config.yml) \
             or all four of --declared-glob, --wired-glob, --declared-pattern, --wired-pattern."
        ),
    }
}

/// Compile a capture regex, requiring at least one capture group (group 1 is the
/// symbol name).
fn compile(pattern: &str, side: &str) -> Result<Regex> {
    let re = Regex::new(pattern)
        .with_context(|| format!("invalid {side} regex: `{pattern}`"))?;
    if re.captures_len() < 2 {
        bail!(
            "the {side} pattern `{}` has no capture group — wrap the symbol name in parentheses, \
             e.g. `fn (\\w+)`.",
            pattern
        );
    }
    Ok(re)
}

fn print_text(report: &Report) {
    println!();
    println!("  {}", "StrayMark Analyze — declared-vs-wired".bold().cyan());
    println!("  {}", report.path.dimmed());
    println!(
        "  {} {}   {} {} declared / {} wired",
        "Profile:".dimmed(),
        report.profile.bold(),
        "Symbols:".dimmed(),
        report.declared_count.to_string().bold(),
        report.wired_count.to_string().bold(),
    );
    println!();

    if report.declared_not_wired.is_empty() {
        println!(
            "  {} Every declared symbol has a wiring counterpart.",
            "✓".green().bold()
        );
    } else {
        println!(
            "  {} {} declared symbol(s) with NO wiring counterpart:",
            "✗".red().bold(),
            report.declared_not_wired.len().to_string().red().bold(),
        );
        for f in &report.declared_not_wired {
            println!(
                "    {} {}  {}",
                "-".red(),
                f.symbol.bold(),
                format!("({})", f.source).dimmed()
            );
        }
        println!();
        println!(
            "  {} a declared surface with no implementation is the \"surface declaration",
            "→".yellow().bold()
        );
        println!("    without wiring\" anti-pattern (POLISH-CHARTER-PATTERN.md sub-class 5).");
    }

    if let Some(orphans) = &report.wired_not_declared {
        println!();
        if orphans.is_empty() {
            println!(
                "  {} No wired-but-undeclared symbols.",
                "·".dimmed()
            );
        } else {
            println!(
                "  {} {} wired symbol(s) no declaration references (often benign):",
                "·".dimmed(),
                orphans.len()
            );
            for f in orphans {
                println!("    {} {}  {}", "-".dimmed(), f.symbol, format!("({})", f.source).dimmed());
            }
        }
    }

    println!();
}

fn print_json(report: &Report) {
    let json = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into());
    println!("{}", json);
}

fn print_markdown(report: &Report) {
    println!("# StrayMark Analyze — declared-vs-wired");
    println!();
    println!("**Path:** `{}`", report.path);
    println!("**Profile:** `{}`", report.profile);
    println!(
        "**Symbols:** {} declared / {} wired",
        report.declared_count, report.wired_count
    );
    println!();

    println!("## Declared but not wired");
    println!();
    if report.declared_not_wired.is_empty() {
        println!("_None — every declared symbol has a wiring counterpart._");
    } else {
        println!("| Symbol | Declared in |");
        println!("|--------|-------------|");
        for f in &report.declared_not_wired {
            println!("| `{}` | `{}` |", f.symbol, f.source);
        }
    }

    if let Some(orphans) = &report.wired_not_declared {
        println!();
        println!("## Wired but not declared");
        println!();
        if orphans.is_empty() {
            println!("_None._");
        } else {
            println!("| Symbol | Wired in |");
            println!("|--------|----------|");
            for f in orphans {
                println!("| `{}` | `{}` |", f.symbol, f.source);
            }
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_rejects_pattern_without_capture_group() {
        assert!(compile(r"fn \w+", "declared").is_err());
        assert!(compile(r"fn (\w+)", "declared").is_ok());
    }

    #[test]
    fn resolve_profile_requires_all_four_inline_flags() {
        let base = PathBuf::from(".");
        let args = Args {
            path: ".".into(),
            profile: None,
            declared_glob: Some("a/*.rs".into()),
            wired_glob: None, // missing
            declared_pattern: Some(r"fn (\w+)".into()),
            wired_pattern: Some(r"fn (\w+)".into()),
            show_orphans: false,
            output: "text".into(),
        };
        assert!(resolve_profile(&args, &base).is_err());
    }
}
