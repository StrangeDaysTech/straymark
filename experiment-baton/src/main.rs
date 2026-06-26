//! `straymark-baton` — Baton's experimental CLI.
//!
//! Phase 1 ships `inspect`: a read-only dump of the SpecKit intent inputs the
//! Coherence Bridge can see. The reconciling `coherence` command (finding
//! classes C1–C4) lands in batch B3.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

use straymark_baton::classify::classify;
use straymark_baton::coherence::{CoherenceReport, Severity};
use straymark_baton::intent::{Confidence, IntentModel};
use straymark_baton::overlay::{IntentState, OverlayReport};
use straymark_baton::signals::signals_for;
use straymark_baton::speckit;
use straymark_baton::telemetry::{build_report, EconomicTelemetry, UnitRouting};
use straymark_baton::tiers::Policy;
use straymark_baton::units::{inventory, Granularity};

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
    /// Read-only intent model: contracts + provenance edges (B2).
    Intent {
        /// Project root (default: current directory).
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFmt::Text)]
        out: OutFmt,
    },
    /// Read-only coherence report: findings C1–C4 (B3). Exit 1 if any blocking
    /// finding is reported (CI-gateable).
    Coherence {
        /// Project root (default: current directory).
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFmt::Text)]
        out: OutFmt,
        /// Only report findings at or above this confidence.
        #[arg(long, value_enum, default_value_t = MinConf::Low)]
        min_confidence: MinConf,
        /// Scope to one feature/spec id (only contracts that feature consumes) —
        /// the authoring-time view a SpecKit `before_implement` hook wants.
        #[arg(long)]
        spec: Option<String>,
    },
    /// Read-only intent overlay for Loom: per-component intended vs implemented (B4).
    Overlay {
        /// Project root (default: current directory).
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFmt::Text)]
        out: OutFmt,
    },
    /// Read-only task classification of recorded work units (Phase 2, B3).
    Classify {
        /// Project root (default: current directory).
        path: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutFmt::Text)]
        out: OutFmt,
        /// Granularity: charter|batch|followup|task|all.
        #[arg(long, default_value = "all")]
        granularity: String,
    },
    /// Read-only DRY-RUN tier router + economic telemetry (Phase 2, B4).
    /// Recommends a tier per unit and reports the §4.2 saving-vs-overhead verdict.
    /// Never executes a model — `--dry-run` is mandatory in Phase 2.
    Route {
        /// Project root (default: current directory).
        path: Option<PathBuf>,
        /// Required: Phase 2 only recommends; there is no execution path.
        #[arg(long)]
        dry_run: bool,
        #[arg(long, value_enum, default_value_t = OutFmt::Text)]
        out: OutFmt,
        /// Granularity: charter|batch|followup|task|all.
        #[arg(long, default_value = "all")]
        granularity: String,
        /// Path to a config file with a `baton:` block (default:
        /// `<root>/.straymark/config.yml`; falls back to illustrative defaults).
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum OutFmt {
    Text,
    Json,
    Markdown,
}

#[derive(Copy, Clone, ValueEnum)]
enum MinConf {
    Low,
    Medium,
    High,
}

impl From<MinConf> for Confidence {
    fn from(m: MinConf) -> Self {
        match m {
            MinConf::Low => Confidence::Low,
            MinConf::Medium => Confidence::Medium,
            MinConf::High => Confidence::High,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { path, out } => inspect(path.unwrap_or_else(|| PathBuf::from(".")), out),
        Command::Intent { path, out } => intent(path.unwrap_or_else(|| PathBuf::from(".")), out),
        Command::Coherence {
            path,
            out,
            min_confidence,
            spec,
        } => coherence(
            path.unwrap_or_else(|| PathBuf::from(".")),
            out,
            min_confidence.into(),
            spec,
        ),
        Command::Overlay { path, out } => overlay(path.unwrap_or_else(|| PathBuf::from(".")), out),
        Command::Classify {
            path,
            out,
            granularity,
        } => classify_cmd(path.unwrap_or_else(|| PathBuf::from(".")), out, &granularity),
        Command::Route {
            path,
            dry_run,
            out,
            granularity,
            config,
        } => route_cmd(
            path.unwrap_or_else(|| PathBuf::from(".")),
            dry_run,
            out,
            &granularity,
            config,
        ),
    }
}

fn parse_granularity(s: &str) -> anyhow::Result<Option<Granularity>> {
    Granularity::parse(s).ok_or_else(|| {
        anyhow::anyhow!("unknown --granularity '{s}' (expected charter|batch|followup|task|all)")
    })
}

fn classify_cmd(root: PathBuf, out: OutFmt, granularity: &str) -> anyhow::Result<()> {
    let only = parse_granularity(granularity)?;
    let units = inventory(&root, only);

    #[derive(serde::Serialize)]
    struct Row {
        id: String,
        granularity: Granularity,
        class: &'static str,
        confidence: Confidence,
        rationale: String,
        title: String,
    }
    let rows: Vec<Row> = units
        .iter()
        .map(|u| {
            let c = classify(&signals_for(u));
            Row {
                id: u.id.clone(),
                granularity: u.granularity,
                class: c.class.as_str(),
                confidence: c.confidence,
                rationale: c.rationale,
                title: u.title.clone(),
            }
        })
        .collect();

    match out {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
        _ => {
            println!(
                "{} ({} units)",
                "Task classification (Baton Phase 2, read-only)".bold(),
                rows.len()
            );
            for r in &rows {
                let title: String = r.title.chars().take(64).collect();
                println!(
                    "  [{}] {}  {} ({:?})  — {}",
                    r.granularity.as_str().dimmed(),
                    r.id.cyan(),
                    r.class.green(),
                    r.confidence,
                    title
                );
            }
        }
    }
    Ok(())
}

fn route_cmd(
    root: PathBuf,
    dry_run: bool,
    out: OutFmt,
    granularity: &str,
    config: Option<PathBuf>,
) -> anyhow::Result<()> {
    if !dry_run {
        eprintln!(
            "{} Phase 2 only recommends — pass {} (there is no execution path).",
            "error:".red(),
            "--dry-run".bold()
        );
        std::process::exit(2);
    }
    let only = parse_granularity(granularity)?;
    let units = inventory(&root, only);
    let policy = Policy::load(&root, config.as_deref());
    if policy.using_defaults {
        eprintln!(
            "{} no `baton:` config block found — using built-in illustrative tier costs.",
            "note:".yellow()
        );
    }
    let (routings, reports) = build_report(&units, &policy);

    #[derive(serde::Serialize)]
    struct RouteOut<'a> {
        units: &'a [UnitRouting],
        telemetry: &'a [EconomicTelemetry],
    }
    match out {
        OutFmt::Json => println!(
            "{}",
            serde_json::to_string_pretty(&RouteOut {
                units: &routings,
                telemetry: &reports,
            })?
        ),
        _ => render_route_text(&reports),
    }
    Ok(())
}

fn render_route_text(reports: &[EconomicTelemetry]) {
    println!(
        "{}",
        "Dry-run router — economic telemetry (Baton Phase 2, illustrative costs)".bold()
    );
    for t in reports {
        let label = match t.granularity {
            Some(g) => g.as_str(),
            None => "ALL",
        };
        let verdict = if t.routable {
            "routable".green()
        } else {
            "not routable".red()
        };
        let pct = |f: f64| format!("{:.0}%", f * 100.0);
        println!("\n  {} ({} units) — {}", label.bold(), t.units_total, verdict);
        let tiers: Vec<String> = t
            .tier_counts
            .iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect();
        println!("    tiers: {}", tiers.join(" · "));
        println!(
            "    cost: all-frontier {:.2} → routed {:.2}  (gross saving {:.2})",
            t.cost_all_frontier, t.cost_routed, t.gross_savings
        );
        println!(
            "    overhead {:.2} → net saving {:.2}",
            t.classification_overhead, t.net_savings
        );
        println!(
            "    caveats: {} low-confidence, {} of saving on low-confidence routing, {} conflicted",
            pct(t.low_confidence_fraction),
            pct(t.low_confidence_savings_fraction),
            pct(t.conflict_fraction)
        );
        println!(
            "    sensitivity: breakeven overhead/unit {:.3}, robust at 2× overhead: {}",
            t.sensitivity.breakeven_overhead_per_unit, t.sensitivity.robust_at_2x_overhead
        );
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
        _ => render_text(&artifacts),
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

fn intent(root: PathBuf, out: OutFmt) -> anyhow::Result<()> {
    let model = IntentModel::build(&root);
    match out {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&model)?),
        _ => render_intent(&model),
    }
    Ok(())
}

fn coherence(root: PathBuf, out: OutFmt, min: Confidence, spec: Option<String>) -> anyhow::Result<()> {
    let report = CoherenceReport::build_scoped(&root, spec.as_deref()).filtered(min);
    match out {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutFmt::Markdown => render_coherence_md(&report),
        OutFmt::Text => render_coherence_text(&report),
    }
    // CI gate: any blocking finding fails the run.
    if report.blocking_count() > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn overlay(root: PathBuf, out: OutFmt) -> anyhow::Result<()> {
    let report = OverlayReport::build(&root);
    match out {
        OutFmt::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        _ => render_overlay(&report),
    }
    Ok(())
}

fn render_overlay(r: &OverlayReport) {
    println!("{}", "Intent overlay (Baton B4, read-only)".bold());
    if !r.model_found {
        eprintln!(
            "  {} no architecture model.yml found — overlay limited to intended components",
            "note:".yellow()
        );
    }
    if r.components.is_empty() {
        println!("\n  (nothing to overlay)");
        return;
    }
    for c in &r.components {
        let (mark, label) = match c.state {
            IntentState::IntendedAndImplemented => ("✓".green(), "intended & implemented".green()),
            IntentState::IntendedNotImplemented => ("!".red(), "intended, NOT implemented".red()),
            IntentState::ImplementedNotIntended => {
                ("?".yellow(), "implemented, NOT intended".yellow())
            }
        };
        let modeled = if c.modeled { "" } else { " (not modeled)" };
        println!("  {} {} — {}{}", mark, c.label.bold(), label, modeled.dimmed());
    }
}

fn sev_label(s: Severity) -> colored::ColoredString {
    match s {
        Severity::Blocking => "BLOCKING".red().bold(),
        Severity::Warning => "WARNING".yellow(),
        Severity::Info => "INFO".dimmed(),
    }
}

fn conf_label(c: Confidence) -> &'static str {
    match c {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}

fn render_coherence_text(r: &CoherenceReport) {
    println!("{}", "Coherence report (Baton B3, read-only)".bold());
    if r.findings.is_empty() {
        println!("\n  {} no findings", "✓".green());
        return;
    }
    println!(
        "\n  {} finding(s) — {} blocking\n",
        r.findings.len(),
        r.blocking_count()
    );
    for f in &r.findings {
        println!(
            "  [{}] {} {} ({} confidence)",
            f.class.code().cyan(),
            sev_label(f.severity),
            f.id.dimmed(),
            conf_label(f.confidence)
        );
        println!("      {}", f.message);
        for loc in &f.locations {
            let sym = loc.symbol.as_deref().map(|s| format!(" [{s}]")).unwrap_or_default();
            println!("        ↳ {}{}", loc.file.dimmed(), sym.dimmed());
        }
    }
}

fn render_coherence_md(r: &CoherenceReport) {
    println!("# Coherence report (Baton)\n");
    println!(
        "{} finding(s), {} blocking.\n",
        r.findings.len(),
        r.blocking_count()
    );
    if r.findings.is_empty() {
        println!("No findings.");
        return;
    }
    println!("| Class | Severity | Confidence | Finding |");
    println!("|---|---|---|---|");
    for f in &r.findings {
        let sev = match f.severity {
            Severity::Blocking => "blocking",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        println!(
            "| {} | {} | {} | {} |",
            f.class.code(),
            sev,
            conf_label(f.confidence),
            f.message.replace('|', "\\|")
        );
    }
}

fn render_intent(m: &IntentModel) {
    println!("{}", "Intent model (Baton B2, read-only)".bold());

    println!("\n  {} ({})", "Contracts".bold(), m.contracts.len());
    for c in &m.contracts {
        let ep = c.endpoint.as_deref().unwrap_or("");
        println!("    • {} {}", c.id.cyan(), ep.dimmed());
        if let Some(p) = &c.producer {
            println!(
                "        producer: {} ({} fields, {} enum defs)",
                p.source.file,
                p.fields.len(),
                p.enums.len()
            );
        }
        for cons in &c.consumers {
            println!(
                "        consumer: {} ({} fields)",
                cons.source.file,
                cons.fields.len()
            );
        }
        if !c.defined_by.is_empty() {
            let ds: Vec<String> = c.defined_by.iter().map(|d| d.id.clone()).collect();
            println!("        defined by: {}", ds.join(", "));
        }
    }

    println!("\n  {} ({})", "Provenance edges".bold(), m.provenance.len());
    for e in &m.provenance {
        let conf = match e.confidence {
            Confidence::High => "HIGH".green(),
            Confidence::Medium => "MED".yellow(),
            Confidence::Low => "LOW".dimmed(),
        };
        let producer = e
            .producer
            .as_ref()
            .map(|p| p.file.as_str())
            .unwrap_or("(none)");
        println!(
            "    [{}] {} --consumes--> {} <--defines-- {}",
            conf,
            e.consumer.file,
            e.contract.magenta(),
            producer
        );
    }
}
