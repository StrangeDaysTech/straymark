use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

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
