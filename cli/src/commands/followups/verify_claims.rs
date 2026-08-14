//! `straymark followups verify --claims` — batch re-derivation of the code
//! claims embedded in registry entries (#419, issue case 2).
//!
//! A follow-up entry is a dated hypothesis, and its prose often carries
//! *mechanical* claims: `` `src/foo.rs` `` exists, `` `bar` `` is the function
//! to change, "`baz` has no callers". Those claims decay silently — the file
//! is renamed, the symbol is deleted, someone wires the "dead" code — and the
//! registry keeps stating them as fact. The per-entry `verify` puts the
//! premise in front of a human; this mode re-derives the claims the tree can
//! answer on its own:
//!
//! - **CLAIM-PATH-GONE** — a backticked path (or bare filename) that no
//!   longer exists anywhere in the tree.
//! - **CLAIM-SYMBOL-GONE** — a backticked symbol with zero word-boundary
//!   occurrences outside `.straymark/`.
//! - **CLAIM-STALE-DEAD** — an entry asserting "no callers / not wired /
//!   unused / dead code" whose symbol is now mentioned by ≥2 files.
//!
//! Grep tier by design (design constraint 2): no AST, no language smarts —
//! high recall, modest precision, which is why the mode is **warn-first**
//! (design constraint 1): findings print, the exit code stays 0.
//!
//! Defect class: registry drift vs the tree.

use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use regex::Regex;

use crate::followups::{self, Entry, FuStatus};
use crate::tree_grep::{self, TextFile};
use crate::utils;

/// Phrases that assert a symbol is dead. Entry-level association: when one of
/// these appears anywhere in the entry's claim text, every symbol-like span
/// in that entry gets the callers re-check.
const DEAD_CLAIM: &str =
    r"(?i)(no callers?\b|nothing calls\b|not wired\b|never (called|invoked|used)\b|\bunused\b|dead code)";

struct ClaimWarning {
    rule: &'static str,
    message: String,
}

pub fn run(path: &str, fu_id: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let project_root = &resolved.path;

    let registry_path = followups::registry_path(project_root);
    if !registry_path.exists() {
        bail!(
            "No follow-ups registry at {}.\n  hint: see STRAYMARK.md §16 for the adoption walkthrough.",
            registry_path.display()
        );
    }
    let registry = followups::parse_registry(&registry_path)?;

    let entries: Vec<Entry> = match fu_id {
        Some(id) => vec![followups::find_entry_unique(&registry, id)?.clone()],
        None => registry
            .entries()
            .filter(|e| matches!(e.status, FuStatus::Open | FuStatus::InProgress))
            .cloned()
            .collect(),
    };

    let tree = tree_grep::read_text_tree(project_root);
    let dead_re = Regex::new(DEAD_CLAIM).expect("static regex");

    println!();
    println!("  {}", "StrayMark Followups — verify --claims".bold().cyan());
    println!("  {}", project_root.display().to_string().dimmed());
    println!();

    let mut total_findings = 0;
    let mut entries_with_findings = 0;

    for entry in &entries {
        let warnings = rederive_entry(project_root, &tree, &dead_re, entry);
        if warnings.is_empty() {
            continue;
        }
        entries_with_findings += 1;
        total_findings += warnings.len();
        println!(
            "  {} — {}",
            entry.fu_id.bold().cyan(),
            entry.description.bold()
        );
        for w in &warnings {
            println!("    {} [{}] {}", "warn".yellow(), w.rule, w.message);
        }
        println!();
    }

    if total_findings == 0 {
        println!(
            "  {} Every code claim in {} entr{} re-derived clean.",
            "✓".green().bold(),
            entries.len(),
            if entries.len() == 1 { "y" } else { "ies" },
        );
    } else {
        println!(
            "  {} {} claim(s) in {} entr{} no longer match the tree.",
            "→".yellow().bold(),
            total_findings.to_string().yellow().bold(),
            entries_with_findings,
            if entries_with_findings == 1 { "y" } else { "ies" },
        );
        println!(
            "  {}",
            "Warn-first: re-check the entry prose and reword, resolve, or close — the exit code stays 0."
                .dimmed()
        );
    }
    println!();

    Ok(())
}

/// Re-derive every mechanical claim in one entry against the tree.
fn rederive_entry(
    project_root: &std::path::Path,
    tree: &[TextFile],
    dead_re: &Regex,
    entry: &Entry,
) -> Vec<ClaimWarning> {
    let mut out = Vec::new();
    let text = claim_text(entry);
    let spans = backticked_spans(&text);

    let mut symbols: Vec<&str> = Vec::new();
    for span in &spans {
        match classify(span) {
            Span::Path(rel) => {
                if !project_root.join(rel).exists() {
                    out.push(ClaimWarning {
                        rule: "CLAIM-PATH-GONE",
                        message: format!("`{rel}` no longer exists in the tree"),
                    });
                }
            }
            Span::FileName(name) => {
                let bare = format!("/{name}");
                let found = tree
                    .iter()
                    .any(|f| f.rel_path == *name || f.rel_path.ends_with(&bare));
                if !found {
                    out.push(ClaimWarning {
                        rule: "CLAIM-PATH-GONE",
                        message: format!("no file named `{name}` exists in the tree"),
                    });
                }
            }
            Span::Symbol(leaf) => {
                symbols.push(leaf);
                let (files, _) = tree_grep::symbol_occurrences(tree, leaf);
                if files == 0 {
                    out.push(ClaimWarning {
                        rule: "CLAIM-SYMBOL-GONE",
                        message: format!("`{leaf}` appears nowhere in the tree"),
                    });
                }
            }
            Span::Skip => {}
        }
    }

    if dead_re.is_match(&text) {
        for sym in symbols {
            let (files, _) = tree_grep::symbol_occurrences(tree, sym);
            if files >= 2 {
                out.push(ClaimWarning {
                    rule: "CLAIM-STALE-DEAD",
                    message: format!(
                        "entry claims `{sym}` is dead (no callers / not wired / unused), but {files} files now mention it"
                    ),
                });
            }
        }
    }

    out
}

/// The prose fields a claim can live in.
fn claim_text(entry: &Entry) -> String {
    let mut parts = vec![entry.description.as_str()];
    if let Some(p) = &entry.premise {
        parts.push(p);
    }
    if let Some(n) = &entry.notes {
        parts.push(n);
    }
    parts.join("\n")
}

/// Extract `` `...` `` spans, deduped, in order of first appearance.
fn backticked_spans(text: &str) -> Vec<&str> {
    let re = Regex::new(r"`([^`\n]+)`").expect("static regex");
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for caps in re.captures_iter(text) {
        let span = caps.get(1).expect("capture group").as_str().trim();
        if seen.insert(span) {
            out.push(span);
        }
    }
    out
}

enum Span<'a> {
    /// `cli/src/validation.rs` — checked against the filesystem.
    Path(&'a str),
    /// `validation.rs` — checked against every relative path in the tree.
    FileName(&'a str),
    /// `parse_registry` or `followups::parse_registry` — the leaf segment is
    /// searched word-boundary across the tree.
    Symbol(&'a str),
    Skip,
}

fn classify(span: &str) -> Span<'_> {
    // Not claims about the tree: flags, placeholders, URLs, prose fragments.
    if span.starts_with('-')
        || span.contains('<')
        || span.contains('>')
        || span.contains(char::is_whitespace)
        || span.contains("://")
    {
        return Span::Skip;
    }
    if let Some(rest) = span.strip_prefix("./") {
        return Span::Path(rest);
    }
    if span.contains('/') && !span.contains(':') {
        return Span::Path(span);
    }
    // Symbol, possibly `::`-qualified — check the leaf segment.
    let leaf = span.rsplit("::").next().unwrap_or(span);
    if !leaf.is_empty()
        && leaf
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        && leaf.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        // Dashed shapes are framework ids (FU-007, CHARTER-02) or rule ids
        // (REF-003) — the validate rules own those, not this mode.
        if leaf.contains('-') {
            return Span::Skip;
        }
        // Bare filename with an extension (`config.yml`, `validation.rs`):
        // at least one dot, an alphabetic stem, and a short alphanumeric
        // extension. All-digit stems (`3.46.0`) are versions, not files.
        if let Some((stem, ext)) = leaf.rsplit_once('.') {
            if !ext.is_empty()
                && ext.len() <= 6
                && ext.chars().all(|c| c.is_ascii_alphanumeric())
                && stem.chars().any(|c| c.is_ascii_alphabetic() || c == '_')
            {
                return Span::FileName(leaf);
            }
        }
        return Span::Symbol(leaf);
    }
    Span::Skip
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_skips_non_claims() {
        assert!(matches!(classify("--staged"), Span::Skip));
        assert!(matches!(classify("<name>"), Span::Skip));
        assert!(matches!(classify("straymark validate ."), Span::Skip));
        assert!(matches!(classify("https://example.com/x"), Span::Skip));
        assert!(matches!(classify("FU-007"), Span::Skip));
        assert!(matches!(classify("CHARTER-02"), Span::Skip));
        assert!(matches!(classify("REF-003"), Span::Skip));
        assert!(matches!(classify("2026-08-13"), Span::Skip));
    }

    #[test]
    fn classify_paths_filenames_symbols() {
        assert!(matches!(classify("cli/src/validation.rs"), Span::Path(_)));
        assert!(matches!(classify("./cli/src/main.rs"), Span::Path(_)));
        assert!(matches!(classify("validation.rs"), Span::FileName(_)));
        assert!(matches!(classify("config.yml"), Span::FileName(_)));
        assert!(matches!(classify("parse_registry"), Span::Symbol("parse_registry")));
        assert!(matches!(
            classify("followups::parse_registry"),
            Span::Symbol("parse_registry")
        ));
        // Versions are not filenames.
        assert!(matches!(classify("3.46.0"), Span::Skip));
    }

    #[test]
    fn backticked_spans_dedupes() {
        let spans = backticked_spans("see `foo` and `bar`, then `foo` again");
        assert_eq!(spans, vec!["foo", "bar"]);
    }
}
