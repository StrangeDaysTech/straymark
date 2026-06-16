//! Shared parser for an AILOG's `## Modified Files` section.
//!
//! Authored in Loom A1.1 (`002-architecture-plan`, the deferred A1.0 task T0.3).
//! The architecture "you are here" projection derives a component's
//! `implemented` state from the files a *closed* Charter's AILOGs actually
//! touched; the file list lives in the AILOG's `## Modified Files` markdown
//! table. This pure string parser extracts those paths so both the CLI
//! (`status --where`, A1.4) and the Loom server (A2) read AILOG file sets with
//! one extractor — the same "one parser, structurally no drift" discipline that
//! moved charter/drift into `core` in A1.0.
//!
//! The AILOG template ships English-first (`dist/.straymark/templates/
//! TEMPLATE-AILOG.md`), but adopters working in Spanish or Chinese translate
//! the section heading, and a literal-English match silently skips those AILOGs
//! — the implemented-state projection then under-reports for a non-English-first
//! corpus (#263). So, like [`crate::charter_files`], we recognize the heading in
//! all three shipped locales. Path extraction reuses the shared
//! [`crate::charter_files`] helpers so the two table extractors agree on what
//! counts as a path.
//!
//! Limitation: brace-expansion cells like `` `core/{Cargo.toml,src/lib.rs}` ``
//! are reported as the raw backtick token (not expanded). Callers matching such
//! a token against globs will simply not match — acceptable for the projection,
//! which treats a missing match as "not touched".

use crate::charter_files::{first_backtick_token, looks_like_path};
use std::path::{Path, PathBuf};

/// The `## Modified Files` heading in the three shipped locales (#263).
const SECTION_HEADINGS: &[&str] = &["Modified Files", "Archivos modificados", "修改的文件"];

/// Canonical sub-path (relative to project root) of the AILOG directory.
/// Moved to `core` in Loom A2.0 so the CLI (`charter drift`, `batch_complete`,
/// `status --where`) and the Loom server (A2) discover AILOGs one way.
pub fn agent_logs_dir(project_root: &Path) -> PathBuf {
    project_root
        .join(".straymark")
        .join("07-ai-audit")
        .join("agent-logs")
}

/// Find the AILOG file matching the given AILOG ID. Searches recursively
/// and matches by filename prefix. The id may be bare
/// (`AILOG-2026-05-02-028b`) or include a slug
/// (`AILOG-2026-05-02-028b-foo`); both resolve to the same file.
pub fn find_ailog_file(agent_logs_dir: &Path, ailog_id: &str) -> Option<PathBuf> {
    let prefix: String = ailog_id
        .split('-')
        .take(5) // "AILOG", "YYYY", "MM", "DD", "NNN[a-z]?"
        .collect::<Vec<_>>()
        .join("-");
    walk_for_prefix(agent_logs_dir, &prefix)
}

fn walk_for_prefix(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = walk_for_prefix(&path, prefix) {
                return Some(found);
            }
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(prefix) && name.ends_with(".md") {
                return Some(path);
            }
        }
    }
    None
}

/// Parse the `## Modified Files` section of an AILOG body and return the paths
/// it lists. Recognizes the markdown-table form the template ships (column 1 =
/// backtick path, remaining columns = lines/description). Non-path tokens
/// (e.g. a backtick `cargo build` in a description) are filtered out via
/// [`crate::charter_files::looks_like_path`]. Returns the raw declared token —
/// wildcards (`*`, `...`) and brace groups are preserved for the caller.
pub fn parse_modified_files(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_section = false;

    for line in body.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("## ") {
            if in_section {
                // A new `## ` heading ends the section.
                break;
            }
            let title = trimmed.trim_start_matches('#').trim();
            if SECTION_HEADINGS.contains(&title) {
                in_section = true;
            }
            continue;
        }

        if !in_section || !trimmed.starts_with('|') {
            continue;
        }

        // Markdown table row. split('|') yields a leading empty element.
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 2 {
            continue;
        }
        let col1 = cols[1].trim();

        // Skip separator rows (only dashes/colons/spaces).
        if !col1.is_empty() && col1.chars().all(|c| matches!(c, '-' | ':' | ' ')) {
            continue;
        }
        // Skip the header row.
        let col1_plain = col1.trim_matches('*').trim();
        if matches!(col1_plain, "File" | "Files") {
            continue;
        }

        let Some(token) = first_backtick_token(col1) else {
            continue;
        };
        if !looks_like_path(token) {
            continue;
        }
        out.push(token.to_string());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_table_col1_backtick_paths() {
        let body = r#"## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `core/src/graph.rs` | +396/-0 | New — typed bidirectional graph |
| `cli/src/main.rs` | +2/-1 | Mechanical import churn |

## Decisions Made
"#;
        assert_eq!(
            parse_modified_files(body),
            vec!["core/src/graph.rs", "cli/src/main.rs"]
        );
    }

    #[test]
    fn ignores_annotations_and_non_path_backticks() {
        // A trailing "(root)" annotation does not break extraction; a backtick
        // command in the description column is not picked up as a path.
        let body = r#"## Modified Files

| File | Lines | Change Description |
|------|-------|--------------------|
| `Cargo.toml` (root) | +10/-0 | workspace members; run `cargo build` |
"#;
        assert_eq!(parse_modified_files(body), vec!["Cargo.toml"]);
    }

    #[test]
    fn skips_separator_and_header_rows() {
        let body = "## Modified Files\n\n| File | Lines |\n| --- | --- |\n| `a.rs` | +1 |\n";
        assert_eq!(parse_modified_files(body), vec!["a.rs"]);
    }

    #[test]
    fn stops_at_next_heading() {
        let body = r#"## Modified Files

| File | Lines |
|---|---|
| `in.rs` | +1 |

## Impact

- `out.rs` should not be captured
"#;
        assert_eq!(parse_modified_files(body), vec!["in.rs"]);
    }

    #[test]
    fn preserves_wildcards_raw() {
        // A wildcard token is kept verbatim as long as it still looks like a
        // path (ends in a recognized extension). A bare `dir/*` with no
        // extension is filtered out by `looks_like_path`.
        let body = "## Modified Files\n\n| File | Lines |\n|---|---|\n| `cli/src/commands/loom/*.rs` | +1 |\n";
        assert_eq!(parse_modified_files(body), vec!["cli/src/commands/loom/*.rs"]);
    }

    #[test]
    fn empty_when_section_absent() {
        let body = "## Summary\n\nNo modified-files section here.\n";
        assert!(parse_modified_files(body).is_empty());
    }

    #[test]
    fn recognizes_spanish_and_chinese_headings() {
        // A Spanish- or Chinese-first adopter translates the section heading; a
        // literal-English match would silently skip the table and under-report
        // implemented state (#263).
        let es = "## Archivos modificados\n\n| Archivo | Líneas |\n|---|---|\n| `core/src/x.rs` | +1 |\n";
        let zh = "## 修改的文件\n\n| 文件 | 行数 |\n|---|---|\n| `cli/src/y.rs` | +2 |\n";
        assert_eq!(parse_modified_files(es), vec!["core/src/x.rs"]);
        assert_eq!(parse_modified_files(zh), vec!["cli/src/y.rs"]);
    }

    #[test]
    fn agent_logs_dir_is_canonical_subpath() {
        let root = Path::new("/proj");
        assert_eq!(
            agent_logs_dir(root),
            Path::new("/proj/.straymark/07-ai-audit/agent-logs")
        );
    }

    #[test]
    fn find_ailog_file_matches_letter_suffix_id() {
        // A bare id resolves to its slugged file, recursively.
        let tmp = tempfile::TempDir::new().unwrap();
        let agent_logs = tmp.path().join("agent-logs");
        std::fs::create_dir_all(&agent_logs).unwrap();
        let path = agent_logs.join("AILOG-2026-05-02-028b-collision.md");
        std::fs::write(&path, "stub\n").unwrap();

        let found = find_ailog_file(&agent_logs, "AILOG-2026-05-02-028b").unwrap();
        assert_eq!(found, path);
        assert!(find_ailog_file(&agent_logs, "AILOG-2026-05-02-999").is_none());
    }
}
