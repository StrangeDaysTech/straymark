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

/// A declared path's exemption from the on-disk existence check (#215 Gap 3).
///
/// `New` keeps the original `(new)` semantics. The others let a *closed*
/// Charter's historical table legitimately carry paths that don't exist in the
/// adopter repo — without the parser-shaped de-backticking workaround adopters
/// reached for. A closed Charter's table is a historical record, not a forward
/// declaration; an unmarked missing path is still flagged (so substituted-late
/// `AILOG-YYYY-MM-NNN` placeholders keep being caught).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileExemption {
    /// Created by this Charter — Change column "New"/"Nuevo"/"新建" or a `(new)` tag.
    New,
    /// Lives in another repository (cross-repo Charter). Tagged `(external)`.
    External,
    /// Existed once / planned but deleted or never materialized. Tagged `(removed)`.
    Removed,
    /// Shipped under a different path; carries the new location.
    /// Tagged `(relocated: <path>)`.
    Relocated(String),
}

/// A file declared in a Charter's `## Files to modify` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredFile {
    /// The path as declared, with any trailing exemption tag stripped. May still
    /// contain a wildcard (`...` or `*`); callers that check disk existence
    /// must skip those (see [`is_wildcard`]).
    pub path: String,
    /// The exemption disposition, if the row carries one. `None` is a normal
    /// declared path subject to the existence check.
    pub exemption: Option<FileExemption>,
    /// The trimmed Change-column text (empty for bullet/prose declarations).
    pub raw_change: String,
}

impl DeclaredFile {
    /// True when the row marks the file as created by this Charter. Back-compat
    /// accessor (was a public field before #215).
    pub fn is_new(&self) -> bool {
        matches!(self.exemption, Some(FileExemption::New))
    }

    /// True when the declared path is exempt from the on-disk existence check
    /// (created here, or a marked cross-repo / removed / relocated path).
    pub fn is_existence_exempt(&self) -> bool {
        self.exemption.is_some()
    }
}

/// Recognized source-file extensions, mirroring the grep filter in
/// `check-charter-drift.sh`. A backtick token is treated as a declared path
/// only when it ends in one of these or contains `.straymark/`.
///
/// Shared with [`crate::ailog`] (the AILOG "Modified Files" parser) so the two
/// markdown-table extractors agree on what counts as a path.
pub(crate) const RECOGNIZED_EXTENSIONS: &[&str] = &[
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
pub(crate) fn looks_like_path(token: &str) -> bool {
    if token.contains(".straymark/") {
        return true;
    }
    RECOGNIZED_EXTENSIONS.iter().any(|ext| token.ends_with(ext))
}

/// Extract the first backtick-quoted token from a string, if any.
pub(crate) fn first_backtick_token(s: &str) -> Option<&str> {
    let start = s.find('`')? + 1;
    let rest = &s[start..];
    let end = rest.find('`')?;
    Some(&rest[..end])
}

/// Parse a `(relocated: <path>)` tag from a path cell, returning the target path.
fn parse_relocated(cell: &str) -> Option<String> {
    let lc = cell.to_lowercase();
    let idx = lc.find("(relocated:")?;
    let after = &cell[idx + "(relocated:".len()..];
    let end = after.find(')')?;
    Some(after[..end].trim().to_string())
}

/// Determine a declared path's exemption from `cell`/`change`. Explicit
/// parenthetical tags on the path cell win; otherwise a Change column starting
/// with a "new" locale variant marks the file as created here.
fn detect_exemption(col1_cell: &str, change: &str) -> Option<FileExemption> {
    if let Some(reloc) = parse_relocated(col1_cell) {
        return Some(FileExemption::Relocated(reloc));
    }
    let cell_lc = col1_cell.to_lowercase();
    if cell_lc.contains("(external)") {
        return Some(FileExemption::External);
    }
    if cell_lc.contains("(removed)") {
        return Some(FileExemption::Removed);
    }
    if cell_lc.contains("(new)") {
        return Some(FileExemption::New);
    }
    let change_lc = change.trim().to_lowercase();
    if NEW_MARKERS.iter().any(|m| change_lc.starts_with(m)) {
        return Some(FileExemption::New);
    }
    None
}

/// Strip a trailing exemption tag — `(new)`, `(external)`, `(removed)`,
/// `(relocated: …)` — and surrounding whitespace from a path.
fn strip_exemption_tag(path: &str) -> String {
    let trimmed = path.trim();
    if let Some(open) = trimmed.rfind('(') {
        if let Some(close_rel) = trimmed[open..].find(')') {
            let close = open + close_rel;
            // Only treat it as a tag when nothing but whitespace follows the ')'.
            if trimmed[close + 1..].trim().is_empty() {
                let tag = trimmed[open + 1..close].trim().to_lowercase();
                if tag == "new"
                    || tag == "external"
                    || tag == "removed"
                    || tag.starts_with("relocated:")
                {
                    return trimmed[..open].trim().to_string();
                }
            }
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
            let exemption = detect_exemption(col1, col2);
            out.push(DeclaredFile {
                path: strip_exemption_tag(token),
                exemption,
                raw_change: col2.to_string(),
            });
        } else {
            // Bullet / prose: extract every backtick token on the line. Any
            // exemption tag on the line applies to the tokens on it.
            let line_exemption = detect_exemption(line, "");
            let mut rest = line;
            while let Some(start) = rest.find('`') {
                let after = &rest[start + 1..];
                let Some(end) = after.find('`') else { break };
                let token = &after[..end];
                if looks_like_path(token) {
                    out.push(DeclaredFile {
                        path: strip_exemption_tag(token),
                        exemption: line_exemption.clone(),
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
        assert!(!by_path("src/old.rs").is_new());
        assert!(by_path(".straymark/07-ai-audit/agent-logs/AILOG-x.md").is_new());
        assert!(by_path("src/created.rs").is_new()); // "Nuevo" prefix
    }

    #[test]
    fn detects_new_via_path_tag() {
        let body = r#"## Files to modify

- `src/foo.rs` (new) — created here
- `src/bar.rs` — existing
"#;
        let files = parse_files_to_modify(body);
        let by_path = |p: &str| files.iter().find(|f| f.path == p).unwrap();
        assert!(by_path("src/foo.rs").is_new());
        assert!(!by_path("src/bar.rs").is_new());
        // The `(new)` tag is stripped from the stored path.
        assert!(by_path("src/foo.rs").path == "src/foo.rs");
    }

    #[test]
    fn detects_exemption_markers() {
        let body = r#"## Files to modify

| File | Change |
|---|---|
| `dist/.straymark/templates/charter-template.md` (external) | cross-repo |
| `interfaces/module.go` (removed) | never materialized |
| `src/old-name.rs` (relocated: src/new-name.rs) | renamed at impl |
| `src/normal.rs` | edit |
"#;
        let files = parse_files_to_modify(body);
        let by_path = |p: &str| files.iter().find(|f| f.path == p).unwrap();
        assert_eq!(
            by_path("dist/.straymark/templates/charter-template.md").exemption,
            Some(FileExemption::External)
        );
        assert_eq!(
            by_path("interfaces/module.go").exemption,
            Some(FileExemption::Removed)
        );
        assert_eq!(
            by_path("src/old-name.rs").exemption,
            Some(FileExemption::Relocated("src/new-name.rs".to_string()))
        );
        assert_eq!(by_path("src/normal.rs").exemption, None);
        // All marked rows are existence-exempt; the plain one is not.
        assert!(by_path("dist/.straymark/templates/charter-template.md").is_existence_exempt());
        assert!(by_path("interfaces/module.go").is_existence_exempt());
        assert!(by_path("src/old-name.rs").is_existence_exempt());
        assert!(!by_path("src/normal.rs").is_existence_exempt());
        // Tags are stripped from the stored path.
        assert!(by_path("src/old-name.rs").path == "src/old-name.rs");
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
