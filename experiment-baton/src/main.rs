//! `straymark-baton` — Baton's experimental CLI.
//!
//! Phase 1 ships `inspect`: a read-only dump of the SpecKit intent inputs the
//! Coherence Bridge can see. The reconciling `coherence` command (finding
//! classes C1–C4) lands in batch B3.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

use straymark_baton::speckit;

#[derive(Parser)]
#[command(
    name = "straymark-baton",
    version,
    about = "Baton — coherence between SpecKit intent and StrayMark governance (experimental, read-only)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read-only dump of the SpecKit intent inputs (B1 adapter).
    Inspect {
        /// Project root (default: current directory).
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFmt::Text)]
        out: OutFmt,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum OutFmt {
    Text,
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { path, out } => inspect(path.unwrap_or_else(|| PathBuf::from(".")), out),
    }
}

fn inspect(root: PathBuf, out: OutFmt) -> anyhow::Result<()> {
    let source = speckit::SpecKitSource::discover(&root);
    if source.is_empty() {
        eprintln!(
            "{} no SpecKit project found under {} (looked for .specify/ and specs/)",
            "note:".yellow(),
            root.display()
        );
    }
    let artifacts = speckit::load_from(&source);

    match out {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&artifacts)?),
        OutFmt::Text => render_text(&artifacts),
    }
    Ok(())
}

fn render_text(a: &speckit::SpecKitArtifacts) {
    println!("{}", "SpecKit intent inputs (Baton B1, read-only)".bold());

    match &a.speckit_version {
        Some(v) if a.version_supported => println!("  SpecKit version: {v}"),
        Some(v) => println!(
            "  SpecKit version: {v} {}",
            "(untested by this adapter — parsing best-effort)".yellow()
        ),
        None => println!("  SpecKit version: {}", "unknown".dimmed()),
    }

    println!("\n  {} ({})", "Specs".bold(), a.specs.len());
    for s in &a.specs {
        let title = s.title.as_deref().unwrap_or("(untitled)");
        println!("    • {} — {title}", s.id.cyan());
        println!(
            "        {} requirements, {} backlog decisions, {} consume hints, {} contract files",
            s.requirements.len(),
            s.decisions.len(),
            s.consumes.len(),
            s.contract_files.len()
        );
        for d in &s.decisions {
            let refs = if d.references.is_empty() {
                String::new()
            } else {
                format!(" → {}", d.references.join(", "))
            };
            println!("          decision {}{}", d.id.magenta(), refs);
        }
    }

    println!(
        "\n  {} ({}) — from .specify/memory/",
        "Intended components".bold(),
        a.intended_components.len()
    );
    for c in &a.intended_components {
        println!("    • {} ({:?})", c.label.green(), c.kind);
    }
}
