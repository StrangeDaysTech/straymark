use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};

use straymark_core::charter::{self, CharterStatus};
use crate::config::StrayMarkConfig;
use crate::manifest::DistManifest;
use crate::utils::{self, pad_right_visual, visual_width};

/// Expected directories inside .straymark/
const EXPECTED_DIRS: &[&str] = &[
    "00-governance",
    "01-requirements",
    "02-design/decisions",
    "03-implementation",
    "04-testing",
    "05-operations/incidents",
    "05-operations/runbooks",
    "06-evolution/technical-debt",
    "07-ai-audit/agent-logs",
    "07-ai-audit/decisions",
    "07-ai-audit/ethical-reviews",
    "08-security",
    "09-ai-models",
    "templates",
];

/// Expected files (relative to project root)
const EXPECTED_FILES: &[(&str, &str)] = &[
    (".straymark/config.yml", "config.yml"),
    (".straymark/dist-manifest.yml", "dist-manifest.yml"),
    ("STRAYMARK.md", "STRAYMARK.md"),
];

/// Document type prefixes for counting
const DOC_TYPES: &[(&str, &str)] = &[
    ("ADR", "Architecture Decisions"),
    ("AIDEC", "AI Decisions"),
    ("AILOG", "AI Action Logs"),
    ("ETH", "Ethical Reviews"),
    ("INC", "Incident Post-mortems"),
    ("REQ", "Requirements"),
    ("TDE", "Technical Debt"),
    ("TES", "Test Plans"),
    ("SEC", "Security"),
    ("MCARD", "Model Cards"),
    ("SBOM", "Software Bill of Materials"),
    ("DPIA", "Data Protection Impact"),
];

pub fn run(path: &str) -> Result<()> {
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

    let version = load_version(&target);
    let language = load_language(&target);
    let cli_version = env!("CARGO_PKG_VERSION");

    // ── Header ──
    println!();
    println!("  {}", "StrayMark Status".bold().cyan());
    println!();

    // ── Project Info ──
    println!("  {}", "Project".bold());
    let project_rows: Vec<(&str, String)> = vec![
        ("Path", target.display().to_string()),
        ("Framework", format!("fw-{}", version)),
        ("CLI", format!("cli-{}", cli_version)),
        ("Language", language.clone()),
    ];
    let label_w = project_rows
        .iter()
        .map(|(l, _)| visual_width(l))
        .max()
        .unwrap_or(5);
    let value_w = project_rows
        .iter()
        .map(|(_, v)| visual_width(v))
        .max()
        .unwrap_or(10);
    print_border("  ┌", label_w, "┬", value_w, "┐");
    for (label, value) in &project_rows {
        println!(
            "  │ {} │ {} │",
            pad_right_visual(label, label_w).dimmed(),
            pad_right_visual(value, value_w),
        );
    }
    print_border("  └", label_w, "┴", value_w, "┘");

    // ── S3 (auto-adoption): framework skew ──
    // When this project is the StrayMark repo governing itself, a sibling
    // `dist/dist-manifest.yml` (the in-development framework) exists. Surface the
    // skew between the installed framework (pinned — the "yesterday's tail" the
    // repo is governed by) and what it is editing under `dist/`. Invisible for
    // normal adopters (no `dist/`).
    if let Some(dist_version) = load_dist_source_version(&target) {
        if dist_version != version {
            println!();
            println!(
                "  {} installed framework {} (pinned) vs dist/ in-development {}",
                "skew:".yellow().bold(),
                format!("fw-{version}").yellow(),
                format!("fw-{dist_version}").yellow(),
            );
            println!(
                "  {}",
                "        this repo is governed by the last release, not the framework it is editing."
                    .dimmed()
            );
        }
    }

    // ── Structure ──
    println!();
    println!("  {}", "Structure".bold());

    // Collect all structure items with their status
    let mut struct_items: Vec<(String, bool)> = Vec::new();
    for dir in EXPECTED_DIRS {
        let dir_path = straymark_dir.join(dir);
        struct_items.push((format!("{dir}/"), dir_path.exists()));
    }
    for &(rel_path, label) in EXPECTED_FILES {
        let file_path = target.join(rel_path);
        struct_items.push((label.to_string(), file_path.exists()));
    }

    let total_items = struct_items.len();
    let total_ok = struct_items.iter().filter(|(_, ok)| *ok).count();
    let total_missing = total_items - total_ok;

    if total_missing == 0 {
        println!(
            "  {} All {} items present",
            "✓".green().bold(),
            total_items
        );
    } else {
        println!(
            "  {} {}/{} items present ({} missing)",
            "!".yellow().bold(),
            total_ok,
            total_items,
            total_missing
        );
    }

    // Calculate column widths dynamically, measured in visual columns.
    let name_w = struct_items
        .iter()
        .map(|(name, _)| visual_width(name))
        .max()
        .unwrap_or(10)
        .max(visual_width("Directory / File"));
    let status_w = 6; // "✓ OK " or "✗ -- "

    println!();
    println!(
        "  {} {} {}",
        pad_right_visual("Directory / File", name_w).dimmed(),
        "│".dimmed(),
        pad_right_visual("Status", status_w).dimmed(),
    );
    println!(
        "  {}",
        format!("{}─┼─{}", "─".repeat(name_w), "─".repeat(status_w)).dimmed()
    );

    for (name, exists) in &struct_items {
        let status_text = if *exists { "✓ OK" } else { "✗ --" };
        let name_cell = pad_right_visual(name, name_w);
        let status_cell = pad_right_visual(status_text, status_w);
        if *exists {
            println!("  {} │ {}", name_cell, status_cell.green());
        } else {
            println!("  {} │ {}", name_cell.yellow(), status_cell.yellow());
        }
    }

    // ── Documentation ──
    let counts = count_documents(&straymark_dir);
    let total: usize = counts.iter().map(|(_, _, c)| c).sum();

    println!();
    println!("  {}", "Documentation".bold());

    let type_w = DOC_TYPES
        .iter()
        .map(|(p, l)| visual_width(&format!("{p:<6}{l}")))
        .max()
        .unwrap_or(20)
        .max(visual_width("Type"));
    let count_w = 5;

    println!();
    println!(
        "  {} {} {}",
        pad_right_visual("Type", type_w).dimmed(),
        "│".dimmed(),
        pad_right_visual("Count", count_w).dimmed(),
    );
    println!(
        "  {}",
        format!("{}─┼─{}", "─".repeat(type_w), "─".repeat(count_w)).dimmed()
    );

    for (prefix, label, count) in &counts {
        let display = format!("{prefix:<6}{label}");
        let count_str = format!("{count:>count_w$}");
        let padded = pad_right_visual(&display, type_w);
        if *count > 0 {
            println!("  {} │ {}", padded, count_str.green().bold());
        } else {
            println!("  {} │ {}", padded.dimmed(), count_str.dimmed());
        }
    }

    let total_str = format!("{total:>count_w$}");
    println!(
        "  {} │ {}",
        pad_right_visual("TOTAL", type_w).bold(),
        total_str.cyan().bold(),
    );
    println!();

    // ── Charters ──
    // Charters are an optional pattern (bounded units of work). Surfaced as a
    // dedicated block so adopters of `straymark charter new` see them in the
    // canonical health view; projects without Charters see a one-line hint.
    let charter_counts = count_charters(&target);
    print_charters_block(&charter_counts);

    // ── Follow-ups ──
    // The follow-ups backlog registry (first-class since fw-4.21.0 /
    // cli-3.19.0). Counts are recomputed from actual entry statuses —
    // never read from the (possibly stale) frontmatter counters.
    print_followups_block(&target);

    // ── Hints ──
    if total_missing > 0 {
        println!(
            "  {} Run {} to restore missing directories and files",
            "→".blue().bold(),
            "straymark repair".cyan().bold()
        );
    }
    if total > 0 {
        println!(
            "  {} Run {} to browse documentation interactively",
            "→".blue().bold(),
            "straymark explore".cyan().bold()
        );
    }
    if total_missing > 0 || total > 0 {
        println!();
    }

    Ok(())
}

fn print_border(prefix: &str, w1: usize, mid: &str, w2: usize, suffix: &str) {
    println!(
        "{}",
        format!(
            "{}{}{}{}{}",
            prefix,
            "─".repeat(w1 + 2),
            mid,
            "─".repeat(w2 + 2),
            suffix
        )
        .dimmed()
    );
}

fn load_version(project_root: &std::path::Path) -> String {
    let manifest_path = project_root.join(".straymark/dist-manifest.yml");
    match DistManifest::load(&manifest_path) {
        Ok(m) => m.version,
        Err(_) => {
            utils::warn("Could not read dist-manifest.yml");
            "unknown".to_string()
        }
    }
}

/// The framework version declared by the in-development distribution source at
/// `<target>/dist/dist-manifest.yml`, if present. Only the StrayMark repo itself
/// (self-adoption) carries this; normal adopters return `None`. Drives the S3
/// skew line in `status`.
fn load_dist_source_version(target: &std::path::Path) -> Option<String> {
    let manifest_path = target.join("dist").join("dist-manifest.yml");
    if !manifest_path.exists() {
        return None;
    }
    DistManifest::load(&manifest_path).ok().map(|m| m.version)
}

#[cfg(test)]
mod s3_tests {
    use super::*;

    #[test]
    fn load_dist_source_version_only_for_self_adoption() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Normal adopter: no dist/ → None.
        assert_eq!(load_dist_source_version(tmp.path()), None);
        // StrayMark repo self-adopting: dist/dist-manifest.yml present.
        std::fs::create_dir_all(tmp.path().join("dist")).unwrap();
        std::fs::write(
            tmp.path().join("dist/dist-manifest.yml"),
            "version: \"9.9.9\"\ndescription: \"x\"\nfiles: []\n",
        )
        .unwrap();
        assert_eq!(load_dist_source_version(tmp.path()).as_deref(), Some("9.9.9"));
    }
}

fn load_language(project_root: &std::path::Path) -> String {
    // Use the same resolver as `explore` / `new` so all three commands
    // agree on the effective language (config when present, else OS locale,
    // else "en").
    StrayMarkConfig::resolve_language(project_root)
}

fn count_documents(straymark_dir: &std::path::Path) -> Vec<(&'static str, &'static str, usize)> {
    let files = walk_files(straymark_dir);
    DOC_TYPES
        .iter()
        .map(|&(doc_type, label)| {
            let prefix = format!("{}-", doc_type);
            let count = files
                .iter()
                .filter(|p| {
                    utils::is_user_document(p)
                        && p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.starts_with(&prefix))
                            .unwrap_or(false)
                })
                .count();
            (doc_type, label, count)
        })
        .collect()
}

fn walk_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_files(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

/// Counts of Charters by lifecycle status. `total` includes parseable charters
/// only; `unparseable` reports files that look like charters by filename but
/// failed schema parse, so the user knows to fix them.
struct CharterCounts {
    declared: usize,
    in_progress: usize,
    closed: usize,
    unparseable: usize,
    total: usize,
}

fn count_charters(project_root: &Path) -> CharterCounts {
    let (charters, errors) = charter::discover_and_parse(project_root);
    let mut declared = 0;
    let mut in_progress = 0;
    let mut closed = 0;
    for c in &charters {
        match c.frontmatter.status {
            CharterStatus::Declared => declared += 1,
            CharterStatus::InProgress => in_progress += 1,
            CharterStatus::Closed => closed += 1,
        }
    }
    CharterCounts {
        declared,
        in_progress,
        closed,
        unparseable: errors.len(),
        total: charters.len(),
    }
}

fn print_charters_block(c: &CharterCounts) {
    println!("  {}", "Charters".bold());
    if c.total == 0 && c.unparseable == 0 {
        println!(
            "  {} {}",
            "·".dimmed(),
            "No Charters yet — run `straymark charter new` to declare one (see STRAYMARK.md §15).".dimmed(),
        );
        println!();
        return;
    }

    let rows: Vec<(&str, usize, colored::Color)> = vec![
        ("declared", c.declared, colored::Color::White),
        ("in-progress", c.in_progress, colored::Color::Yellow),
        ("closed", c.closed, colored::Color::Green),
    ];
    let label_w = rows
        .iter()
        .map(|(l, _, _)| visual_width(l))
        .max()
        .unwrap_or(11);
    let count_w = 5;

    println!();
    println!(
        "  {} {} {}",
        pad_right_visual("Status", label_w).dimmed(),
        "│".dimmed(),
        pad_right_visual("Count", count_w).dimmed(),
    );
    println!(
        "  {}",
        format!("{}─┼─{}", "─".repeat(label_w), "─".repeat(count_w)).dimmed()
    );

    for (label, count, color) in &rows {
        let count_str = format!("{count:>count_w$}");
        let padded = pad_right_visual(label, label_w);
        if *count > 0 {
            println!("  {} │ {}", padded, count_str.color(*color).bold());
        } else {
            println!("  {} │ {}", padded.dimmed(), count_str.dimmed());
        }
    }

    let total_str = format!("{:>count_w$}", c.total);
    println!(
        "  {} │ {}",
        pad_right_visual("TOTAL", label_w).bold(),
        total_str.cyan().bold(),
    );

    if c.unparseable > 0 {
        println!(
            "  {} {} unparseable Charter file{} — run `straymark charter list` to see the warning detail.",
            "!".yellow().bold(),
            c.unparseable,
            if c.unparseable == 1 { "" } else { "s" },
        );
    }
    println!();
}

fn print_followups_block(project_root: &Path) {
    println!("  {}", "Follow-ups".bold());
    let registry_path = crate::followups::registry_path(project_root);
    if !registry_path.exists() {
        println!(
            "  {} {}",
            "·".dimmed(),
            "No follow-ups registry yet — adopt the pattern at ~20+ AILOGs (see STRAYMARK.md §16).".dimmed(),
        );
        println!();
        return;
    }
    let registry = match crate::followups::parse_registry(&registry_path) {
        Ok(r) => r,
        Err(e) => {
            println!("  {} registry unreadable: {}", "!".yellow().bold(), e);
            println!();
            return;
        }
    };
    let c = crate::followups::compute_counters(&registry);

    let rows: Vec<(&str, u32, colored::Color)> = vec![
        ("open", c.open, colored::Color::White),
        ("in-progress", c.in_progress, colored::Color::Yellow),
        ("suspected-closed", c.suspected_closed, colored::Color::Magenta),
        ("closed + superseded", c.closed_cumulative, colored::Color::Green),
        ("promoted", c.promoted, colored::Color::Cyan),
    ];
    let label_w = rows
        .iter()
        .map(|(l, _, _)| visual_width(l))
        .max()
        .unwrap_or(11);
    let count_w = 5;

    println!();
    println!(
        "  {} {} {}",
        pad_right_visual("Status", label_w).dimmed(),
        "│".dimmed(),
        pad_right_visual("Count", count_w).dimmed(),
    );
    println!(
        "  {}",
        format!("{}─┼─{}", "─".repeat(label_w), "─".repeat(count_w)).dimmed()
    );
    for (label, count, color) in &rows {
        let count_str = format!("{count:>count_w$}");
        let padded = pad_right_visual(label, label_w);
        if *count > 0 {
            println!("  {} │ {}", padded, count_str.color(*color).bold());
        } else {
            println!("  {} │ {}", padded.dimmed(), count_str.dimmed());
        }
    }
    println!(
        "  {} │ {}",
        pad_right_visual("TOTAL", label_w).bold(),
        format!("{:>count_w$}", c.total).cyan().bold(),
    );
    if c.blocking_open > 0 {
        println!(
            "  {} {} open blocking entr{} — `straymark followups list --severity blocking`.",
            "!".red().bold(),
            c.blocking_open,
            if c.blocking_open == 1 { "y" } else { "ies" },
        );
    }
    println!();
}
