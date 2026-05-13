//! AILOG helpers: file discovery and `## Batch Ledger` parsing/editing.
//!
//! Shared between `charter::drift` (close-time gate) and
//! `charter::batch_complete` (per-batch update). The ledger format
//! canonized in `dist/.straymark/templates/TEMPLATE-AILOG.md`:
//!
//! ```text
//! ## Batch Ledger
//!
//! > (guidance blockquote — ignored by the parser)
//!
//! ### Batch 1 — [name from Charter §Tasks]
//!
//! (pending)
//!
//! ### Batch 2 — [name]
//!
//! (pending)
//! ```
//!
//! A batch is "pending" iff its body — every line between the
//! `### Batch <N>` heading and the next `###`/`##`/EOF, trimmed of
//! surrounding whitespace — equals `(pending)` exactly, or is empty.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Canonical sub-path (relative to project root) of the AILOG directory.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEntry {
    pub n: u32,
    /// Full heading line, e.g. `### Batch 5 — Migration 022 + handlers`.
    pub heading_line: String,
    /// Byte offset of the first character of the body (line after the
    /// heading + its trailing newline). Used by `write_batch_section`.
    pub body_start: usize,
    /// Byte offset one past the last character of the body (start of the
    /// next `###`/`##` heading, or end-of-file).
    pub body_end: usize,
    /// Body content (between heading and next section), trimmed of
    /// leading/trailing blank lines.
    pub body: String,
    pub is_pending: bool,
}

/// Parse the `## Batch Ledger` section. Returns `None` if the section
/// is absent (signal to callers that the AILOG opted out of the ledger
/// pattern).
pub fn parse_batch_ledger(content: &str) -> Option<Vec<BatchEntry>> {
    // Locate `## Batch Ledger` start.
    let header_idx = find_section_offset(content, "## Batch Ledger")?;

    // Locate end of the ledger section = next `## ` at column 0, or EOF.
    let after_header = &content[header_idx..];
    let ledger_end_rel = after_header
        .match_indices('\n')
        .filter_map(|(i, _)| {
            // Skip the header's own line.
            if i == 0 {
                return None;
            }
            let line_start = i + 1;
            let rest = &after_header[line_start..];
            if rest.starts_with("## ") {
                Some(line_start)
            } else {
                None
            }
        })
        .next()
        .unwrap_or(after_header.len());
    let ledger_section = &after_header[..ledger_end_rel];

    // Walk for `### Batch <N> — ...` sub-headings.
    let mut entries: Vec<BatchEntry> = Vec::new();
    let mut last_heading: Option<(u32, String, usize)> = None; // (n, heading_line, body_start_abs)

    for (line_start_rel, line) in line_offsets(ledger_section) {
        let abs_offset = header_idx + line_start_rel;
        if let Some((n, _)) = parse_batch_heading(line) {
            // Close the previous entry (if any) at this line's start.
            if let Some((prev_n, prev_heading, prev_body_start)) = last_heading.take() {
                let body_end_abs = abs_offset;
                push_entry(&mut entries, content, prev_n, prev_heading, prev_body_start, body_end_abs);
            }
            // Body starts at the line AFTER the heading line.
            let body_start_abs = abs_offset + line.len() + 1; // +1 for '\n'
            last_heading = Some((n, line.to_string(), body_start_abs));
        }
    }
    // Close the final entry at ledger_end.
    if let Some((prev_n, prev_heading, prev_body_start)) = last_heading.take() {
        let body_end_abs = header_idx + ledger_end_rel;
        push_entry(&mut entries, content, prev_n, prev_heading, prev_body_start, body_end_abs);
    }

    Some(entries)
}

fn push_entry(
    entries: &mut Vec<BatchEntry>,
    content: &str,
    n: u32,
    heading_line: String,
    body_start: usize,
    body_end: usize,
) {
    let raw = &content[body_start..body_end];
    let trimmed = raw.trim_matches(|c: char| c == '\n' || c == '\r').trim();
    let is_pending = trimmed.is_empty() || trimmed == "(pending)";
    entries.push(BatchEntry {
        n,
        heading_line,
        body_start,
        body_end,
        body: trimmed.to_string(),
        is_pending,
    });
}

/// Return the batch numbers whose entries are still `(pending)`.
pub fn pending_batches(content: &str) -> Vec<u32> {
    match parse_batch_ledger(content) {
        Some(entries) => entries.iter().filter(|e| e.is_pending).map(|e| e.n).collect(),
        None => Vec::new(),
    }
}

/// Replace the body of `### Batch <batch_number>` with `new_body` and write
/// the file. Returns the previous body so the caller can decide whether to
/// reject an overwrite.
pub fn write_batch_section(
    ailog_path: &Path,
    batch_number: u32,
    new_body: &str,
) -> Result<BatchEntry> {
    let content = std::fs::read_to_string(ailog_path)
        .with_context(|| format!("Failed to read AILOG at {}", ailog_path.display()))?;

    let entries = parse_batch_ledger(&content).ok_or_else(|| {
        anyhow::anyhow!(
            "AILOG at {} has no `## Batch Ledger` section.\n  hint: add the section (see TEMPLATE-AILOG.md) before running batch-complete.",
            ailog_path.display()
        )
    })?;

    let target = entries
        .iter()
        .find(|e| e.n == batch_number)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No `### Batch {}` heading in AILOG {}.\n  hint: add the heading to `## Batch Ledger` (the body should start as `(pending)`).",
                batch_number,
                ailog_path.display()
            )
        })?
        .clone();

    // Normalize body: ensure it starts with a blank line separator and ends
    // with a trailing newline so the next section heading stays on its own
    // line. The body slice [body_start..body_end] in the original content
    // already includes the surrounding newlines; we replace it fully with
    // "\n<new_body>\n\n".
    let mut formatted = String::with_capacity(new_body.len() + 4);
    formatted.push('\n');
    formatted.push_str(new_body.trim());
    formatted.push_str("\n\n");

    let mut updated = String::with_capacity(content.len() + formatted.len());
    updated.push_str(&content[..target.body_start]);
    updated.push_str(&formatted);
    updated.push_str(&content[target.body_end..]);

    std::fs::write(ailog_path, updated)
        .with_context(|| format!("Failed to write AILOG at {}", ailog_path.display()))?;
    Ok(target)
}

/// Reject overwrite of an already-completed batch.
pub fn ensure_pending(entry: &BatchEntry) -> Result<()> {
    if !entry.is_pending {
        bail!(
            "Batch {} is already completed (body: `{}`). Refusing to overwrite — edit the AILOG manually if a correction is needed.",
            entry.n,
            truncate(&entry.body, 80)
        );
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}

fn find_section_offset(content: &str, heading: &str) -> Option<usize> {
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        let trimmed_newline = line.trim_end_matches('\n').trim_end_matches('\r');
        if trimmed_newline == heading {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn line_offsets(s: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    s.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line.trim_end_matches('\n').trim_end_matches('\r'))
    })
}

fn parse_batch_heading(line: &str) -> Option<(u32, &str)> {
    // Match `### Batch <N>` followed by space and optional ` — ...` or anything.
    let rest = line.strip_prefix("### Batch ")?;
    // The number is the leading digit run.
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let n: u32 = digits.parse().ok()?;
    Some((n, line))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# AILOG: Test

## Actions Performed

1. Stuff.

## Batch Ledger

> Use this section ...

### Batch 1 — Setup

Done on 2026-05-10. Files touched: a.rs, b.rs. Tests passing.

### Batch 2 — Impl

(pending)

### Batch 3 — Tests

(pending)

## Modified Files

| File | Lines |
"#;

    #[test]
    fn parse_finds_three_batches() {
        let entries = parse_batch_ledger(SAMPLE).expect("ledger present");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].n, 1);
        assert_eq!(entries[1].n, 2);
        assert_eq!(entries[2].n, 3);
    }

    #[test]
    fn parse_marks_pending_correctly() {
        let entries = parse_batch_ledger(SAMPLE).unwrap();
        assert!(!entries[0].is_pending, "Batch 1 is completed");
        assert!(entries[1].is_pending, "Batch 2 is pending");
        assert!(entries[2].is_pending, "Batch 3 is pending");
    }

    #[test]
    fn pending_batches_returns_pending_only() {
        assert_eq!(pending_batches(SAMPLE), vec![2, 3]);
    }

    #[test]
    fn parse_returns_none_when_ledger_absent() {
        let content = "# AILOG\n\n## Actions Performed\n\n1. Stuff.\n";
        assert!(parse_batch_ledger(content).is_none());
        assert!(pending_batches(content).is_empty());
    }

    #[test]
    fn write_batch_section_replaces_pending_body() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("AILOG-2026-05-10-001-test.md");
        std::fs::write(&path, SAMPLE).unwrap();

        let prev = write_batch_section(&path, 2, "Implemented X and Y. Files: handler.go. Tests passing.").unwrap();
        assert!(prev.is_pending);

        let updated = std::fs::read_to_string(&path).unwrap();
        let entries = parse_batch_ledger(&updated).unwrap();
        assert!(!entries[1].is_pending, "Batch 2 should now be completed");
        assert!(entries[1].body.contains("Implemented X"));
        // Batch 3 must still be pending.
        assert!(entries[2].is_pending, "Batch 3 must remain pending");
        // Batch 1's body must still mention "Done on 2026-05-10".
        assert!(entries[0].body.contains("Done on 2026-05-10"));
    }

    #[test]
    fn write_batch_section_errors_when_batch_absent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("AILOG-2026-05-10-001-test.md");
        std::fs::write(&path, SAMPLE).unwrap();

        let err = write_batch_section(&path, 99, "x").unwrap_err();
        assert!(err.to_string().contains("Batch 99"));
    }

    #[test]
    fn write_batch_section_errors_when_ledger_absent() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("AILOG-2026-05-10-001-test.md");
        std::fs::write(&path, "# AILOG\n\nNo ledger.\n").unwrap();

        let err = write_batch_section(&path, 1, "x").unwrap_err();
        assert!(err.to_string().contains("Batch Ledger"));
    }

    #[test]
    fn ensure_pending_rejects_completed_batch() {
        let entry = BatchEntry {
            n: 5,
            heading_line: "### Batch 5".to_string(),
            body_start: 0,
            body_end: 0,
            body: "Already done.".to_string(),
            is_pending: false,
        };
        assert!(ensure_pending(&entry).is_err());
    }

    #[test]
    fn find_ailog_file_matches_letter_suffix_id() {
        let tmp = tempfile::TempDir::new().unwrap();
        let agent_logs = tmp.path().join("agent-logs");
        std::fs::create_dir_all(&agent_logs).unwrap();
        let path = agent_logs.join("AILOG-2026-05-02-028b-collision.md");
        std::fs::write(&path, "stub\n").unwrap();

        let found = find_ailog_file(&agent_logs, "AILOG-2026-05-02-028b").unwrap();
        assert_eq!(found, path);
    }
}
