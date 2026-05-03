use clap::{Parser, Subcommand};
use colored::Colorize;

#[cfg(feature = "analyze")]
mod analysis_engine;
mod audit_engine;
mod charter;
mod charter_schema;
mod commands;
mod compliance;
mod config;
mod document;
mod download;
mod inject;
mod manifest;
mod metrics_engine;
mod platform;
mod prompts;
mod self_update;
mod telemetry_schema;
#[cfg(feature = "tui")]
mod tui;
mod utils;
mod validation;

/// DevTrail CLI - Documentation Governance for AI-Assisted Development
#[derive(Parser)]
#[command(name = "devtrail", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize DevTrail in a project directory
    Init {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Update both framework and CLI to the latest version
    Update {
        /// Update method for the CLI binary: auto, github, or cargo
        #[arg(long, default_value = "auto", value_parser = ["auto", "github", "cargo"])]
        method: String,
    },
    /// Update the DevTrail framework to the latest version
    UpdateFramework,
    /// Update the CLI binary to the latest version
    UpdateCli {
        /// Update method: auto (detect), github (prebuilt binary), or cargo (compile from source)
        #[arg(long, default_value = "auto", value_parser = ["auto", "github", "cargo"])]
        method: String,
    },
    /// Remove DevTrail from the project
    Remove {
        /// Remove everything including user-generated documents (requires confirmation)
        #[arg(long)]
        full: bool,
    },
    /// Show DevTrail installation status and documentation statistics
    Status {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Repair DevTrail structure by restoring missing directories and files
    Repair {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Validate DevTrail documents for compliance and correctness
    Validate {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Automatically fix simple issues
        #[arg(long)]
        fix: bool,
        /// Validate only git-staged files (for pre-commit hooks)
        #[arg(long)]
        staged: bool,
        /// Also validate Charters in docs/charters/ against the Charter schema
        /// and referential integrity (originating_ailogs IDs exist;
        /// originating_spec path exists). Default: false, to avoid breaking
        /// projects that don't yet use the Charter pattern.
        #[arg(long)]
        include_charters: bool,
        /// Surface documents with `review_required: true` and no
        /// `review_outcome` that are older than --max-pending-days. Warn-only
        /// (never fails the validate exit code); useful for CI dashboards of
        /// the approval backlog. See DOCUMENTATION-POLICY.md §3.5.
        #[arg(long)]
        check_pending_reviews: bool,
        /// Threshold for --check-pending-reviews (default: 14 days).
        #[arg(long, default_value_t = 14)]
        max_pending_days: i64,
    },
    /// Check regulatory compliance (EU, NIST, ISO; China standards opt-in via regional_scope)
    Compliance {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Check a specific standard
        #[arg(long, value_parser = [
            "eu-ai-act", "iso-42001", "nist-ai-rmf",
            "china-tc260", "china-pipl", "china-gb45438",
            "china-cac", "china-gb45652", "china-csl",
        ])]
        standard: Option<String>,
        /// Run all standards in a region: global, eu, china, or all
        #[arg(long, value_parser = ["global", "eu", "china", "all"])]
        region: Option<String>,
        /// Check all standards (equivalent to --region all, regardless of regional_scope)
        #[arg(long)]
        all: bool,
        /// Output format
        #[arg(long, default_value = "text", value_parser = ["text", "markdown", "json"])]
        output: String,
    },
    /// Generate audit trail reports with timeline and traceability
    Audit {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Start date for audit period (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// End date for audit period (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
        /// Filter by system/component name
        #[arg(long)]
        system: Option<String>,
        /// Output format
        #[arg(long, default_value = "text", value_parser = ["text", "markdown", "json", "html"])]
        output: String,
    },
    /// Show governance metrics and documentation statistics
    Metrics {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Time period for metrics
        #[arg(long, default_value = "last-30-days", value_parser = ["last-7-days", "last-30-days", "last-90-days", "all"])]
        period: String,
        /// Output format
        #[arg(long, default_value = "text", value_parser = ["text", "markdown", "json"])]
        output: String,
    },
    /// Create a new DevTrail document from a template
    New {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Document type (e.g., ailog, adr, sec)
        #[arg(long, short = 't')]
        doc_type: Option<String>,
        /// Document title
        #[arg(long)]
        title: Option<String>,
    },
    /// Show version, author, and license information
    About,
    /// Record a formal human approval on a `review_required: true` document.
    /// Writes the reviewed_by/reviewed_at/review_outcome frontmatter fields
    /// and appends the canonical `## Approval` body section in one edit.
    Approve {
        /// Document ID (e.g., AIDEC-2026-05-02-001 or with slug)
        doc_id: String,
        /// Outcome of the review
        #[arg(long, value_parser = ["approved", "revisions_requested", "rejected"])]
        outcome: Option<String>,
        /// Reviewer identity: email | github-handle | DID
        #[arg(long)]
        reviewer: Option<String>,
        /// Approval date (default: today, format YYYY-MM-DD)
        #[arg(long)]
        at: Option<String>,
        /// Optional reviewer notes (appended in the body section)
        #[arg(long)]
        notes: Option<String>,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Analyze code complexity using cognitive and cyclomatic metrics
    #[cfg(feature = "analyze")]
    Analyze {
        /// Target directory or file (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Cognitive complexity threshold (default: from config or 8)
        #[arg(long)]
        threshold: Option<u32>,
        /// Output format
        #[arg(long, default_value = "text", value_parser = ["text", "json", "markdown"])]
        output: String,
        /// Show only top N most complex functions
        #[arg(long)]
        top: Option<usize>,
    },
    /// Explore DevTrail documentation interactively
    #[cfg(feature = "tui")]
    Explore {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Display language override (e.g., en, es, zh-CN). Defaults to
        /// `language` from .devtrail/config.yml.
        #[arg(long)]
        lang: Option<String>,
    },
    /// Manage Charters: bounded units of work declared ex-ante and audited ex-post
    Charter {
        #[command(subcommand)]
        command: CharterCommands,
    },
}

#[derive(Subcommand)]
enum CharterCommands {
    /// Scaffold a new Charter from the framework template
    New {
        /// Effort estimate (defaults to M if absent)
        #[arg(long = "type", short = 't', value_parser = ["XS", "S", "M", "L"])]
        effort: Option<String>,
        /// Originating AILOG ID (e.g., AILOG-2026-04-28-021).
        /// Mutually exclusive with --from-spec.
        #[arg(long, conflicts_with = "from_spec")]
        from_ailog: Option<String>,
        /// Originating SpecKit spec path (e.g., specs/001-feature/spec.md).
        /// Mutually exclusive with --from-ailog.
        #[arg(long, conflicts_with = "from_ailog")]
        from_spec: Option<String>,
        /// Charter title (used to build the slug and filename)
        #[arg(long)]
        title: Option<String>,
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// List Charters with optional status / origin filter
    List {
        /// Filter by lifecycle status
        #[arg(long, default_value = "all", value_parser = ["declared", "in-progress", "closed", "all"])]
        status: String,
        /// Filter by origin type
        #[arg(long, value_parser = ["ailog", "spec", "any"])]
        origin: Option<String>,
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Show Charter detail (or last 5 Charters if no ID is given)
    Status {
        /// Charter identifier (CHARTER-NN, CHARTER-NN-slug, or just NN)
        charter_id: Option<String>,
        /// Project directory (default: current directory).
        /// Use a flag (rather than positional) so it cannot be confused
        /// with the optional charter_id positional.
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Record post-execution telemetry and bump status to `closed`
    Close {
        /// Charter identifier (CHARTER-NN, CHARTER-NN-slug, or just NN)
        charter_id: String,
        /// Copy the telemetry template next to the Charter for manual editing
        /// instead of running the interactive flow. Combine with
        /// --non-interactive for CI / scripted use.
        #[arg(long)]
        from_template: bool,
        /// Skip prompts. Requires --from-template (the schema cannot be
        /// validated until the user has filled in the YAML).
        #[arg(long, requires = "from_template")]
        non_interactive: bool,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Detect file-vs-commit drift at Charter close (declared-but-not-modified
    /// files; scope expansion). Suppresses alerts on paths already documented
    /// as risks in the Charter's originating AILOGs.
    Drift {
        /// Charter identifier (CHARTER-NN, CHARTER-NN-slug, or just NN)
        charter_id: String,
        /// Git revision range (default: HEAD~1..HEAD)
        #[arg(long)]
        range: Option<String>,
        /// Disable AILOG-aware suppression (show every declared-omitted path
        /// even if documented in an AILOG).
        #[arg(long)]
        no_ailog_suppress: bool,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
}

fn main() {
    // Clean up leftover binary from previous update
    self_update::cleanup_old_binary();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path } => commands::init::run(&path),
        Commands::Update { method } => commands::update::run(&method),
        Commands::UpdateFramework => commands::update_framework::run(),
        Commands::UpdateCli { method } => commands::update_cli::run(&method),
        Commands::Remove { full } => commands::remove::run(full),
        Commands::Validate {
            path,
            fix,
            staged,
            include_charters,
            check_pending_reviews,
            max_pending_days,
        } => commands::validate::run(
            &path,
            fix,
            staged,
            include_charters,
            check_pending_reviews,
            max_pending_days,
        ),
        Commands::Audit {
            path,
            from,
            to,
            system,
            output,
        } => commands::audit::run(&path, from.as_deref(), to.as_deref(), system.as_deref(), &output),
        Commands::Compliance {
            path,
            standard,
            region,
            all,
            output,
        } => commands::compliance::run(&path, standard.as_deref(), region.as_deref(), all, &output),
        Commands::Metrics {
            path,
            period,
            output,
        } => commands::metrics::run(&path, &period, &output),
        Commands::New {
            path,
            doc_type,
            title,
        } => commands::new::run(&path, doc_type.as_deref(), title.as_deref()),
        Commands::Status { path } => commands::status::run(&path),
        Commands::Repair { path } => commands::repair::run(&path),
        Commands::About => commands::about::run(),
        Commands::Approve {
            doc_id,
            outcome,
            reviewer,
            at,
            notes,
            path,
        } => commands::approve::run(
            &path,
            &doc_id,
            outcome.as_deref(),
            reviewer.as_deref(),
            at.as_deref(),
            notes.as_deref(),
        ),
        #[cfg(feature = "analyze")]
        Commands::Analyze {
            path,
            threshold,
            output,
            top,
        } => commands::analyze::run(&path, threshold, &output, top),
        #[cfg(feature = "tui")]
        Commands::Explore { path, lang } => commands::explore::run(&path, lang.as_deref()),
        Commands::Charter { command } => match command {
            CharterCommands::New {
                effort,
                from_ailog,
                from_spec,
                title,
                path,
            } => commands::charter::new::run(
                &path,
                effort.as_deref(),
                from_ailog.as_deref(),
                from_spec.as_deref(),
                title.as_deref(),
            ),
            CharterCommands::List {
                status,
                origin,
                path,
            } => commands::charter::list::run(&path, &status, origin.as_deref()),
            CharterCommands::Status { charter_id, path } => {
                commands::charter::status::run(&path, charter_id.as_deref())
            }
            CharterCommands::Close {
                charter_id,
                from_template,
                non_interactive,
                path,
            } => commands::charter::close::run(&path, &charter_id, from_template, non_interactive),
            CharterCommands::Drift {
                charter_id,
                range,
                no_ailog_suppress,
                path,
            } => commands::charter::drift::run(
                &path,
                &charter_id,
                range.as_deref(),
                no_ailog_suppress,
            ),
        },
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
