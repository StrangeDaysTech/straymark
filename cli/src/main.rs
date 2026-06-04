use clap::{Parser, Subcommand};
use colored::Colorize;

#[cfg(feature = "analyze")]
mod analysis_engine;
mod ailog;
mod audit_engine;
mod audit_schema;
mod charter;
mod charter_files;
mod charter_schema;
mod commands;
mod compliance;
mod config;
mod document;
mod download;
mod followups;
mod followups_schema;
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

/// StrayMark CLI - Documentation Governance for AI-Assisted Development
#[derive(Parser)]
#[command(name = "straymark", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize StrayMark in a project directory
    Init {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// After init, install the framework's pre-PR hook (runs
        /// `straymark charter drift` automatically before `git push`).
        /// Opt-in per principle #6 — friction with consent. Requires the
        /// project to be a git repository.
        #[arg(long)]
        hooks: bool,
    },
    /// Update both framework and CLI to the latest version
    Update {
        /// Update method for the CLI binary: auto, github, or cargo
        #[arg(long, default_value = "auto", value_parser = ["auto", "github", "cargo"])]
        method: String,
    },
    /// Update the StrayMark framework to the latest version
    UpdateFramework,
    /// Update the CLI binary to the latest version
    UpdateCli {
        /// Update method: auto (detect), github (prebuilt binary), or cargo (compile from source)
        #[arg(long, default_value = "auto", value_parser = ["auto", "github", "cargo"])]
        method: String,
    },
    /// Remove StrayMark from the project
    Remove {
        /// Remove everything including user-generated documents (requires confirmation)
        #[arg(long)]
        full: bool,
    },
    /// Show StrayMark installation status and documentation statistics
    Status {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Repair StrayMark structure by restoring missing directories and files
    Repair {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Install StrayMark skills into an AI agent's user-level skills directory
    /// (currently only `--agent codex`, which targets `$CODEX_HOME/skills/` or
    /// `$HOME/.codex/skills/`). Claude and Gemini read skills directly from the
    /// project tree (`.claude/skills/`, `.gemini/skills/`) and do not require
    /// this command.
    InstallSkills {
        /// AI agent whose user-level skills directory we should populate.
        #[arg(long, value_parser = ["codex", "claude", "gemini"])]
        agent: String,
        /// Project directory (default: current directory). The source of the
        /// skills is `<path>/.codex/skills/` (materialized by `straymark init`
        /// or `straymark update`).
        #[arg(long = "path", default_value = ".")]
        path: String,
        /// Print what would be installed without writing anything.
        #[arg(long)]
        dry_run: bool,
        /// Symlink each skill instead of copying it. Unix-only; useful for
        /// framework devs working on skill content.
        #[arg(long)]
        symlink: bool,
    },
    /// Validate StrayMark documents for compliance and correctness
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
        /// Inspect an AI agent's user-level skills installation instead of
        /// validating documents. Currently supports `codex` — checks
        /// `~/.codex/skills/straymark-*` for presence, parseable frontmatter,
        /// required `name`/`description`, and absence of Claude-only keys.
        #[arg(long, value_parser = ["codex"])]
        agent: Option<String>,
        /// Also validate Charters in .straymark/charters/ against the Charter schema
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
    /// Create a new StrayMark document from a template
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
        /// Re-apply approval on a document that already has one. Required for
        /// the revisions_requested → approved iteration cycle and for
        /// multi-reviewer hand-offs. Without this flag, re-running approve
        /// on an already-approved document is a no-op (cli-3.7.1+).
        #[arg(long)]
        force: bool,
        /// Suppress the per-document success and idempotent-skip messages.
        /// Useful for bulk approve runs (the operator just wants the
        /// exit code). High-risk warnings (`risk_level: high|critical`)
        /// are NOT silenceable by this flag — bulk-approving high-risk
        /// docs without seeing the warning is exactly the failure mode
        /// the warning exists to prevent (cli-3.8.0+, F5 of issue #81).
        #[arg(long)]
        quiet: bool,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Analyze code: complexity metrics (default) or declared-vs-wired contract checks
    #[cfg(feature = "analyze")]
    #[command(args_conflicts_with_subcommands = true)]
    Analyze {
        /// Sub-analysis to run. Omit for the default complexity analysis.
        #[command(subcommand)]
        command: Option<AnalyzeCommands>,
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
    /// Explore StrayMark documentation interactively
    #[cfg(feature = "tui")]
    Explore {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Display language override (e.g., en, es, zh-CN). Defaults to
        /// `language` from .straymark/config.yml.
        #[arg(long)]
        lang: Option<String>,
    },
    /// Manage Charters: bounded units of work declared ex-ante and audited ex-post
    Charter {
        #[command(subcommand)]
        command: CharterCommands,
    },
    /// Manage the follow-ups backlog registry (.straymark/follow-ups-backlog.md)
    Followups {
        #[command(subcommand)]
        command: FollowupsCommands,
    },
}

#[derive(Subcommand)]
enum FollowupsCommands {
    /// List registry entries with optional filters
    List {
        /// Filter by bucket (ready, time-triggered, charter-triggered,
        /// phase-blocked, operational)
        #[arg(long)]
        bucket: Option<String>,
        /// Filter by entry status (open, in-progress, suspected-closed,
        /// closed, superseded, promoted)
        #[arg(long)]
        status: Option<String>,
        /// Filter by severity (normal, blocking)
        #[arg(long)]
        severity: Option<String>,
        /// Filter by label (case-insensitive exact match on one tag)
        #[arg(long)]
        label: Option<String>,
        /// Project directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Show the registry pulse (counters recomputed on the fly), or one
    /// entry's detail when an FU id is given
    Status {
        /// Entry identifier (FU-NNN or just NNN)
        fu_id: Option<String>,
        /// Project directory (default: current directory).
        /// A flag (rather than positional) so it cannot be confused with
        /// the optional fu_id positional.
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Detect AILOGs with follow-up content not yet extracted into the
    /// registry. Native replacement (cli-3.19.0) for the deprecated
    /// adopter-side check-followups-drift.sh. Exit 1 on drift unless --apply.
    Drift {
        /// Extract the missing entries into `## Bucket: ready`, append the
        /// AILOG ids to fully_extracted_ailogs, recompute the CLI-owned
        /// counters, and upgrade v0 registries to v1 in place. Entries whose
        /// source AILOG carries a closure marker (`closed in-Charter`,
        /// `fixed in batch N`, a commit hash) land as `suspected-closed`.
        #[arg(long)]
        apply: bool,
        /// Sweep every AILOG in the project instead of the git range.
        #[arg(long)]
        scan_all: bool,
        /// Git revision range for the default scan (default:
        /// origin/main..HEAD, falling back to origin/master..HEAD, then
        /// HEAD~1..HEAD).
        #[arg(long, conflicts_with = "scan_all")]
        range: Option<String>,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Recompute the CLI-owned frontmatter counters from actual entry
    /// statuses, without scanning AILOGs. The §13-compliant way to reconcile
    /// counters after a manual-triage session (statuses flipped by hand,
    /// nothing left to extract or promote). Idempotent; upgrades v0
    /// registries to v1 in place like every other write command.
    Recount {
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Promote an entry to a TDE document (creates the TDE with
    /// promoted_from_followup traceability, marks the entry `promoted`,
    /// recomputes counters). Non-interactive; prioritization stays human.
    Promote {
        /// Entry identifier (FU-NNN or just NNN)
        fu_id: String,
        /// TDE title override (default: the FU description)
        #[arg(long)]
        title: Option<String>,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
}

/// Sub-analyses under `straymark analyze`.
#[cfg(feature = "analyze")]
#[derive(Subcommand)]
enum AnalyzeCommands {
    /// Flag declared symbols (e.g. client IPC/RPC proxy methods) that have no
    /// wiring counterpart on the implementation side — the "surface declaration
    /// without wiring" anti-pattern (POLISH-CHARTER-PATTERN.md sub-class 5).
    /// Config-driven: supply --profile (from .straymark/config.yml) or the four
    /// inline globs/patterns. Capture group 1 of each pattern is the symbol name.
    DeclaredVsWired {
        /// Target directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Named profile from `.straymark/config.yml` (`declared_vs_wired.profiles`)
        #[arg(long)]
        profile: Option<String>,
        /// Glob (relative to path) for files holding declarations
        #[arg(long)]
        declared_glob: Option<String>,
        /// Glob (relative to path) for files holding implementations
        #[arg(long)]
        wired_glob: Option<String>,
        /// Regex over declared files; capture group 1 = symbol name
        #[arg(long)]
        declared_pattern: Option<String>,
        /// Regex over wired files; capture group 1 = symbol name
        #[arg(long)]
        wired_pattern: Option<String>,
        /// Also report symbols wired but never declared (W \ D)
        #[arg(long)]
        show_orphans: bool,
        /// Output format
        #[arg(long, default_value = "text", value_parser = ["text", "json", "markdown"])]
        output: String,
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
        /// Explicit slug override (cli-3.7.2+). Use when the title-derived
        /// slug would lose meaningful context (e.g., a long title with a
        /// trailing reference like `… Plan 04 F3` whose suffix would be
        /// truncated). Input is normalized through the slugifier.
        #[arg(long)]
        slug: Option<String>,
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
    /// Mark a Charter batch as complete in the AILOG `## Batch Ledger`.
    /// For multi-batch Charters (3+ batches or >1 day): substitutes the
    /// `(pending)` placeholder under `### Batch <N>` with batch notes
    /// captured interactively (default) or via --note (one-shot / scripted).
    /// The `straymark charter drift` close-time gate rejects any batch left
    /// as `(pending)`.
    BatchComplete {
        /// Charter identifier (CHARTER-NN, CHARTER-NN-slug, or just NN)
        charter_id: String,
        /// Batch number (1-based) — matches the `### Batch <N>` heading in the
        /// AILOG `## Batch Ledger` section.
        batch_number: u32,
        /// Pre-filled batch note body. With this flag, the command writes the
        /// note non-interactively and skips prompts. Use for scripts / agents.
        #[arg(long)]
        note: Option<String>,
        /// Skip prompts. Requires --note (no batch content can be inferred).
        #[arg(long, requires = "note")]
        non_interactive: bool,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Orchestrate a multi-model external audit cycle (3-step: prepare,
    /// calibrate, finalize). Phase 3 v0 is orchestration-only — the CLI
    /// resolves prompts, validates auditor outputs, and prints findings
    /// for telemetry. It does NOT invoke LLM APIs; the operator runs the
    /// prompts in their auditor of choice (Copilot, Gemini, Claude, etc.)
    /// and saves responses to canonical paths.
    Audit {
        /// Charter identifier (CHARTER-NN, CHARTER-NN-slug, or just NN)
        charter_id: String,
        /// Git revision range (default: origin/main..HEAD with fallback to
        /// origin/master..HEAD; falls back to HEAD~1..HEAD with warning when
        /// no upstream is reachable). Override with explicit value as needed.
        #[arg(long)]
        range: Option<String>,
        /// Generate the unified audit prompt and write it to
        /// .straymark/audits/CHARTER-NN/audit-prompt.md. Default action when
        /// no other flag is passed. Equivalent to the v0 PREPARE step.
        #[arg(long, conflicts_with_all = ["merge_reports", "calibrate", "finalize"])]
        prepare: bool,
        /// Read all report-*.md files in .straymark/audits/CHARTER-NN/,
        /// validate them against audit-output.schema.v0.json, and emit the
        /// external_audit YAML block. Combine with --merge-into to append
        /// the block directly into the Charter's telemetry YAML.
        #[arg(long, conflicts_with_all = ["prepare", "calibrate", "finalize"])]
        merge_reports: bool,
        /// Deprecated v0 flag. The v1 flow does not have a separate calibrate
        /// step — the calibrator role is handled by the main agent via the
        /// /straymark-audit-review skill. Emits a warning and exits.
        #[arg(long, hide = true, conflicts_with_all = ["prepare", "merge_reports", "finalize"])]
        calibrate: bool,
        /// Deprecated v0 flag. Use --merge-reports instead. Emits a
        /// deprecation warning and routes through the new path.
        #[arg(long, hide = true, conflicts_with_all = ["prepare", "merge_reports", "calibrate"])]
        finalize: bool,
        /// With --merge-reports (or deprecated --finalize): append the
        /// external_audit array directly into the Charter's telemetry YAML
        /// at the given path instead of printing it to stdout. Re-audit
        /// (file already has external_audit) is rejected with a clear error.
        #[arg(long)]
        merge_into: Option<String>,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Pre-declare SpecKit refresh recommendation for a multi-Charter module.
    /// Read-only. Computes the rolling mean of
    /// `agent_quality.r_n_plus_one_emergent_count` across the last 3 closed
    /// Charters matching the module and compares against a threshold (default
    /// 6). See STRAYMARK.md §15.A and
    /// `.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md` Pattern 1.
    /// fw-4.16.0+ (Issue #156).
    RefreshSuggest {
        /// Module name to match (case-insensitive substring on charter_id).
        /// Example: `commshub` matches `CHARTER-18-commshub-us5-...`.
        module: String,
        /// Override the rolling-mean threshold (default: 6).
        #[arg(long)]
        threshold: Option<u32>,
        /// Project directory (default: current directory)
        #[arg(long = "path", default_value = ".")]
        path: String,
    },
    /// Scaffold a post-close Batch N.4 amendment for a closed Charter
    /// (audit-driven remediation, production incident, or deferred
    /// implementation). Creates a NEW AILOG stub, edits the most-recent
    /// originating AILOG with a `## Historical correction` subsection, and
    /// prints the `post_close_amendment:` YAML block ready for paste (or
    /// auto-merged into the telemetry with `--merge-into`). Does not touch
    /// git — operator decides when to commit. See STRAYMARK.md §15.B and
    /// `.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md` Pattern 2.
    /// fw-4.16.0+ (Issue #156).
    Amend {
        /// Charter identifier (CHARTER-NN, CHARTER-NN-slug, or just NN).
        /// Must be already closed.
        charter_id: String,
        /// What surfaced the need for the amendment.
        #[arg(long, value_parser = ["external_audit", "production_incident", "deferred_implementation"])]
        trigger: String,
        /// Number of findings closed by the amendment (default: 0).
        #[arg(long, default_value_t = 0)]
        findings_closed: u32,
        /// Short title for the new AILOG (drives filename slug).
        #[arg(long)]
        ailog_title: String,
        /// Optional: merge the rendered `post_close_amendment:` block directly
        /// into the Charter's telemetry YAML at the given path instead of
        /// printing it.
        #[arg(long)]
        merge_into: Option<String>,
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
        /// Disable the Batch Ledger gate (do not fail on `### Batch N (pending)`
        /// entries in AILOGs referenced by this Charter). Use only when the
        /// adopter has explicitly opted out of the ledger pattern for this
        /// Charter (e.g., consolidating post-close).
        #[arg(long)]
        no_batch_ledger_check: bool,
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
        Commands::Init { path, hooks } => commands::init::run(&path, hooks),
        Commands::Update { method } => commands::update::run(&method),
        Commands::UpdateFramework => commands::update_framework::run(),
        Commands::UpdateCli { method } => commands::update_cli::run(&method),
        Commands::Remove { full } => commands::remove::run(full),
        Commands::InstallSkills {
            agent,
            path,
            dry_run,
            symlink,
        } => commands::install_skills::run(&agent, &path, dry_run, symlink),
        Commands::Validate {
            path,
            fix,
            staged,
            agent,
            include_charters,
            check_pending_reviews,
            max_pending_days,
        } => commands::validate::run(
            &path,
            fix,
            staged,
            agent.as_deref(),
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
            force,
            quiet,
            path,
        } => commands::approve::run(
            &path,
            &doc_id,
            outcome.as_deref(),
            reviewer.as_deref(),
            at.as_deref(),
            notes.as_deref(),
            force,
            quiet,
        ),
        #[cfg(feature = "analyze")]
        Commands::Analyze {
            command,
            path,
            threshold,
            output,
            top,
        } => match command {
            Some(AnalyzeCommands::DeclaredVsWired {
                path,
                profile,
                declared_glob,
                wired_glob,
                declared_pattern,
                wired_pattern,
                show_orphans,
                output,
            }) => commands::analyze_declared_vs_wired::run(
                commands::analyze_declared_vs_wired::Args {
                    path,
                    profile,
                    declared_glob,
                    wired_glob,
                    declared_pattern,
                    wired_pattern,
                    show_orphans,
                    output,
                },
            ),
            None => commands::analyze::run(&path, threshold, &output, top),
        },
        #[cfg(feature = "tui")]
        Commands::Explore { path, lang } => commands::explore::run(&path, lang.as_deref()),
        Commands::Charter { command } => match command {
            CharterCommands::New {
                effort,
                from_ailog,
                from_spec,
                title,
                slug,
                path,
            } => commands::charter::new::run(
                &path,
                effort.as_deref(),
                from_ailog.as_deref(),
                from_spec.as_deref(),
                title.as_deref(),
                slug.as_deref(),
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
                no_batch_ledger_check,
                path,
            } => commands::charter::drift::run(
                &path,
                &charter_id,
                range.as_deref(),
                no_ailog_suppress,
                no_batch_ledger_check,
            ),
            CharterCommands::BatchComplete {
                charter_id,
                batch_number,
                note,
                non_interactive,
                path,
            } => commands::charter::batch_complete::run(
                &path,
                &charter_id,
                batch_number,
                note.as_deref(),
                non_interactive,
            ),
            CharterCommands::Audit {
                charter_id,
                range,
                prepare,
                merge_reports,
                calibrate,
                finalize,
                merge_into,
                path,
            } => commands::charter::audit::run(
                &path,
                &charter_id,
                range.as_deref(),
                prepare,
                merge_reports,
                calibrate,
                finalize,
                merge_into.as_deref(),
            ),
            CharterCommands::RefreshSuggest {
                module,
                threshold,
                path,
            } => commands::charter::refresh_suggest::run(&path, &module, threshold),
            CharterCommands::Amend {
                charter_id,
                trigger,
                findings_closed,
                ailog_title,
                merge_into,
                path,
            } => commands::charter::amend::run(
                &path,
                &charter_id,
                &trigger,
                findings_closed,
                &ailog_title,
                merge_into.as_deref(),
            ),
        },
        Commands::Followups { command } => match command {
            FollowupsCommands::List {
                bucket,
                status,
                severity,
                label,
                path,
            } => commands::followups::list::run(
                &path,
                &commands::followups::list::Filters {
                    bucket: bucket.as_deref(),
                    status: status.as_deref(),
                    severity: severity.as_deref(),
                    label: label.as_deref(),
                },
            ),
            FollowupsCommands::Status { fu_id, path } => {
                commands::followups::status::run(&path, fu_id.as_deref())
            }
            FollowupsCommands::Drift {
                apply,
                scan_all,
                range,
                path,
            } => commands::followups::drift::run(&path, apply, scan_all, range.as_deref()),
            FollowupsCommands::Recount { path } => commands::followups::recount::run(&path),
            FollowupsCommands::Promote { fu_id, title, path } => {
                commands::followups::promote::run(&path, &fu_id, title.as_deref())
            }
        },
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
