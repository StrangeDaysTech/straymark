//! Shared parser for a Charter's `## Files to modify` section.
//!
//! Ported from the awk extraction in
//! `dist/.straymark/scripts/check-charter-drift.sh` (the bash drift script),
//! so the CLI's pure-Rust consumers (the `CHARTER-FILES-EXIST` validate rule,
//! the `charter new` reconnaissance nudge) agree byte-for-byte with the drift
//! script on what counts as a declared file.
//!
//! Two consumers, one source of truth:
//! - `validation.rs` — the `CHARTER-FILES-EXIST` rule warns when a declared
//!   path that is NOT tagged "new" does not exist on disk (Charter authored
//!   against assumed code — finding #210). This is a *static authoring* check,
//!   distinct from the drift script's *dynamic* git-range check.
//! - the `(new)` exemption keeps from flagging files the Charter itself creates.
//!
//! The drift script applies a recognized-extension / `.straymark/` filter via
//! grep; we mirror it in [`looks_like_path`] so a backtick token like
//! `` `cargo build` `` is not mistaken for a declared file.

/// A file declared in a Charter's `## Files to modify` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredFile {
    /// The path as declared, with any trailing `(new)` tag stripped. May still
    /// contain a wildcard (`...` or `*`); callers that check disk existence
    /// must skip those (see [`is_wildcard`]).
    pub path: String,
    /// True when the row marks the file as created by this Charter — either the
    /// Change column starts with "New"/"Nuevo"/"新建", or the path/cell carries
    /// a `(new)` tag. Existence-checking consumers skip these.
    pub is_new: bool,
    /// The trimmed Change-column text (empty for bullet/prose declarations).
    pub raw_change: String,
}

/// Recognized source-file extensions, mirroring the grep filter in
/// `check-charter-drift.sh`. A backtick token is treated as a declared path
/// only when it ends in one of these or contains `.straymark/`.
const RECOGNIZED_EXTENSIONS: &[&str] = &[
    ".go", ".sql", ".yaml", ".yml", ".md", ".sh", ".ts", ".tsx", ".js", ".jsx",
    ".rs", ".py", ".java", ".kt", ".rb", ".cs", ".cpp", ".c", ".h", ".hpp",
    ".swift", ".toml", ".json", ".tf",
];

/// The `## Files to modify` heading in the three shipped locales.
const SECTION_HEADINGS: &[&str] = &["Files to modify", "Archivos a modificar", "要修改的文件"];

/// Change-column prefixes that mark a file as newly created (locale variants).
const NEW_MARKERS: &[&str] = &["new", "nuevo", "新建"];

/// True if a declared path uses a wildcard form (`prefix...suffix` ellipsis or
/// `prefix*suffix` glob). Such paths are git-range patterns, not literal paths,
/// and cannot be existence-checked.
pub fn is_wildcard(path: &str) -> bool {
    path.contains("...") || path.contains('*')
}

/// True if a backtick token looks like a declared file path: it ends with a
/// recognized extension, or references something under `.straymark/`.
fn looks_like_path(token: &str) -> bool {
    if token.contains(".straymark/") {
        return true;
    }
    RECOGNIZED_EXTENSIONS.iter().any(|ext| token.ends_with(ext))
}

/// Extract the first backtick-quoted token from a string, if any.
fn first_backtick_token(s: &str) -> Option<&str> {
    let start = s.find('`')? + 1;
    let rest = &s[start..];
    let end = rest.find('`')?;
    Some(&rest[..end])
}

/// True when `cell`/`change` indicates the file is created by this Charter.
fn detect_new(col1_cell: &str, change: &str) -> bool {
    let change_lc = change.trim().to_lowercase();
    if NEW_MARKERS.iter().any(|m| change_lc.starts_with(m)) {
        return true;
    }
    // Fallback: a `(new)` tag anywhere in the path cell (table) or line (bullet).
    col1_cell.to_lowercase().contains("(new)")
}

/// Strip a trailing `(new)` tag (and surrounding whitespace) from a path.
fn strip_new_tag(path: &str) -> String {
    let trimmed = path.trim();
    // Tag may appear right after the token, e.g. `src/foo.rs (new)` once the
    // backticks are removed. Drop any parenthetical "(new)" suffix.
    let lower = trimmed.to_lowercase();
    if let Some(idx) = lower.rfind("(new)") {
        if trimmed[idx + 5..].trim().is_empty() {
            return trimmed[..idx].trim().to_string();
        }
    }
    trimmed.to_string()
}

/// Parse the `## Files to modify` section of a Charter body and return the
/// declared files. Recognizes the markdown-table form (column 1 = backtick
/// path, column 2 = change) and the legacy bullet/prose form (any backtick
/// token on a line). Non-path tokens are filtered out (see [`looks_like_path`]).
pub fn parse_files_to_modify(body: &str) -> Vec<DeclaredFile> {
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

        if !in_section {
            continue;
        }

        if trimmed.starts_with('|') {
            // Markdown table row. split('|') yields a leading empty element.
            let cols: Vec<&str> = line.split('|').collect();
            if cols.len() < 2 {
                continue;
            }
            let col1 = cols[1].trim();
            let col2 = cols.get(2).map(|c| c.trim()).unwrap_or("");

            // Skip separator rows (only dashes/colons/spaces) and header rows.
            if !col1.is_empty()
                && col1.chars().all(|c| matches!(c, '-' | ':' | ' '))
            {
                continue;
            }
            let col1_plain = col1.trim_matches('*').trim();
            if matches!(col1_plain, "File" | "Archivo" | "文件") {
                continue;
            }

            let Some(token) = first_backtick_token(col1) else {
                continue;
            };
            if !looks_like_path(token) {
                continue;
            }
            let is_new = detect_new(col1, col2);
            out.push(DeclaredFile {
                path: strip_new_tag(token),
                is_new,
                raw_change: col2.to_string(),
            });
        } else {
            // Bullet / prose: extract every backtick token on the line.
            let is_new_line = trimmed.to_lowercase().contains("(new)");
            let mut rest = line;
            while let Some(start) = rest.find('`') {
                let after = &rest[start + 1..];
                let Some(end) = after.find('`') else { break };
                let token = &after[..end];
                if looks_like_path(token) {
                    out.push(DeclaredFile {
                        path: strip_new_tag(token),
                        is_new: is_new_line,
                        raw_change: String::new(),
                    });
                }
                rest = &after[end + 1..];
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(files: &[DeclaredFile]) -> Vec<&str> {
        files.iter().map(|f| f.path.as_str()).collect()
    }

    #[test]
    fn parses_table_col1_backtick_paths() {
        let body = r#"## Files to modify

| File | Change |
|---|---|
| `src/main.rs` | Adds routing |
| `src/config.rs` | New field `foo` |

## Verification
"#;
        let files = parse_files_to_modify(body);
        assert_eq!(paths(&files), vec!["src/main.rs", "src/config.rs"]);
        // The backtick `foo` in the Change column must NOT be picked up as a path.
        assert!(files.iter().all(|f| f.path != "foo"));
    }

    #[test]
    fn detects_new_via_change_column() {
        let body = r#"## Files to modify

| File | Change |
|---|---|
| `src/old.rs` | Refactor |
| `.straymark/07-ai-audit/agent-logs/AILOG-x.md` | New, `risk_level: low` |
| `src/created.rs` | Nuevo módulo |
"#;
        let files = parse_files_to_modify(body);
        let by_path = |p: &str| files.iter().find(|f| f.path == p).unwrap();
        assert!(!by_path("src/old.rs").is_new);
        assert!(by_path(".straymark/07-ai-audit/agent-logs/AILOG-x.md").is_new);
        assert!(by_path("src/created.rs").is_new); // "Nuevo" prefix
    }

    #[test]
    fn detects_new_via_path_tag() {
        let body = r#"## Files to modify

- `src/foo.rs` (new) — created here
- `src/bar.rs` — existing
"#;
        let files = parse_files_to_modify(body);
        let by_path = |p: &str| files.iter().find(|f| f.path == p).unwrap();
        assert!(by_path("src/foo.rs").is_new);
        assert!(!by_path("src/bar.rs").is_new);
        // The `(new)` tag is stripped from the stored path.
        assert!(by_path("src/foo.rs").path == "src/foo.rs");
    }

    #[test]
    fn skips_separator_and_header_rows() {
        let body = r#"## Files to modify

| File | Change |
| --- | --- |
| `a.rs` | x |
"#;
        let files = parse_files_to_modify(body);
        assert_eq!(paths(&files), vec!["a.rs"]);
    }

    #[test]
    fn recognizes_spanish_and_chinese_headings() {
        let es = "## Archivos a modificar\n\n| File | Change |\n|---|---|\n| `x.rs` | y |\n";
        let zh = "## 要修改的文件\n\n| File | Change |\n|---|---|\n| `z.go` | w |\n";
        assert_eq!(paths(&parse_files_to_modify(es)), vec!["x.rs"]);
        assert_eq!(paths(&parse_files_to_modify(zh)), vec!["z.go"]);
    }

    #[test]
    fn preserves_wildcards_for_caller_to_skip() {
        let body = "## Files to modify\n\n| File | Change |\n|---|---|\n| `AILOG-*.md` | bulk |\n| `.straymark/07-ai-audit/agent-logs/AILOG-...md` | log |\n";
        let files = parse_files_to_modify(body);
        assert!(files.iter().any(|f| is_wildcard(&f.path)));
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn stops_at_next_heading() {
        let body = r#"## Files to modify

| File | Change |
|---|---|
| `in.rs` | x |

## Risks

- `out.rs` should not be captured
"#;
        let files = parse_files_to_modify(body);
        assert_eq!(paths(&files), vec!["in.rs"]);
    }

    #[test]
    fn empty_when_section_absent() {
        let body = "## Context\n\nNo files section here.\n";
        assert!(parse_files_to_modify(body).is_empty());
    }
}
