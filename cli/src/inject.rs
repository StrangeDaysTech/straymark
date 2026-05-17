use anyhow::{Context, Result};
use std::path::Path;

const MARKER_BEGIN: &str = "<!-- straymark:begin -->";
const MARKER_END: &str = "<!-- straymark:end -->";

/// Extract the marker block (begin marker through end marker, inclusive) from a template string.
/// Returns None if markers are not found.
fn extract_marker_block(template: &str) -> Option<String> {
    let (start, end) = find_canonical_block(template)?;
    Some(template[start..end].to_string())
}

/// Find the canonical StrayMark block in `content`.
///
/// Returns `(start_inclusive, end_exclusive)` spanning from the **first** BEGIN
/// marker to the **last** END marker in the file (whichever pair the host file
/// actually delimits). Returns `None` if there is no BEGIN or no END strictly
/// after the BEGIN.
///
/// The asymmetry (first BEGIN, last END) is load-bearing: several host files —
/// notably `.cursorrules` and `.cursor/rules/straymark.md` — embed `STRAYMARK.md`,
/// whose own documentation contains the literal strings `<!-- straymark:begin -->`
/// and `<!-- straymark:end -->` inside a markdown code block describing the
/// marker convention. Those literals sit between the real BEGIN and the real
/// END. Using `find(MARKER_END)` would stop at the in-docs literal and produce
/// a half-amputated block on every `update-framework`, leaving the real END
/// orphaned — the exact corruption observed in Sentinel. `rfind(MARKER_END)`
/// reliably picks the trailing marker that templates always write last.
///
/// For the rare patological case of two complete BEGIN..END pairs (e.g. an
/// old append-bug victim), this means the canonical span engulfs both pairs
/// and any text between them; the replacement substitutes the whole region.
/// That's the correct outcome: corrupted-marker regions are not safe to
/// preserve content from.
fn find_canonical_block(content: &str) -> Option<(usize, usize)> {
    let start = content.find(MARKER_BEGIN)?;
    let end_pos = content.rfind(MARKER_END)?;
    let end = end_pos + MARKER_END.len();
    if end <= start + MARKER_BEGIN.len() {
        return None;
    }
    Some((start, end))
}

/// Remove orphan markers and additional BEGIN..END blocks from `content`, preserving
/// only the first canonical block (if any) intact.
///
/// Used to auto-repair host files that accumulated malformed markers from prior
/// versions of the CLI or manual edits. The first valid BEGIN..END pair is the
/// canonical block; everything else (extra blocks, lone BEGIN/END markers) gets
/// stripped — lines containing only a marker are removed entirely; markers embedded
/// inside lines with other content have just the marker substring removed.
fn sanitize_orphan_markers(content: &str) -> String {
    match find_canonical_block(content) {
        Some((a, b)) => {
            let prefix = sanitize_region(&content[..a]);
            let block = &content[a..b];
            let suffix = sanitize_region(&content[b..]);
            join_regions(&prefix, block, &suffix)
        }
        None => sanitize_region(content),
    }
}

/// Clean all markers (lone and full blocks) from a region. Caller must ensure the
/// region does NOT contain the canonical block that should be preserved.
fn sanitize_region(text: &str) -> String {
    let mut s = text.to_string();
    while let Some((a, b)) = find_canonical_block(&s) {
        let (cut_start, cut_end) = expand_to_line_bounds(&s, a, b);
        s.replace_range(cut_start..cut_end, "");
    }
    s = strip_lone_marker(&s, MARKER_BEGIN);
    s = strip_lone_marker(&s, MARKER_END);
    collapse_blank_runs(&s)
}

/// Remove `marker` occurrences from `text`. Lines whose trimmed body is exactly
/// the marker are dropped entirely; lines with the marker embedded keep their
/// other content but lose the marker substring.
fn strip_lone_marker(text: &str, marker: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for segment in text.split_inclusive('\n') {
        let nl_len = if segment.ends_with("\r\n") {
            2
        } else if segment.ends_with('\n') {
            1
        } else {
            0
        };
        let body = &segment[..segment.len() - nl_len];
        let newline = &segment[segment.len() - nl_len..];
        if body.trim() == marker {
            continue;
        }
        if body.contains(marker) {
            result.push_str(&body.replace(marker, ""));
        } else {
            result.push_str(body);
        }
        result.push_str(newline);
    }
    result
}

/// Given a slice [start, end) of `s` whose surrounding line context is whitespace-only,
/// expand the cut to include the full line(s). Avoids leaving blank lines behind when
/// a block was the only thing on its line.
fn expand_to_line_bounds(s: &str, start: usize, end: usize) -> (usize, usize) {
    let line_start = s[..start].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_end = s[end..].find('\n').map(|p| end + p + 1).unwrap_or(s.len());
    let prefix_clean = s[line_start..start].chars().all(|c| c.is_whitespace());
    let suffix_clean = s[end..line_end].chars().all(|c| c.is_whitespace());
    let cut_start = if prefix_clean { line_start } else { start };
    let cut_end = if suffix_clean { line_end } else { end };
    (cut_start, cut_end)
}

/// Collapse runs of consecutive blank lines down to a single blank line.
fn collapse_blank_runs(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_blanks = 0;
    for segment in text.split_inclusive('\n') {
        let body = segment.trim_end_matches('\n').trim_end_matches('\r');
        if body.trim().is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= 1 {
                result.push_str(segment);
            }
        } else {
            consecutive_blanks = 0;
            result.push_str(segment);
        }
    }
    result
}

/// Reassemble sanitized prefix + canonical block + sanitized suffix with
/// appropriate single-newline separators between them.
fn join_regions(prefix: &str, block: &str, suffix: &str) -> String {
    let mut out = String::with_capacity(prefix.len() + block.len() + suffix.len() + 4);
    out.push_str(prefix);
    if !prefix.is_empty() && !prefix.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(block);
    if !suffix.is_empty() {
        if !block.ends_with('\n') && !suffix.starts_with('\n') {
            out.push('\n');
        }
        out.push_str(suffix);
    }
    collapse_blank_runs(&out)
}

/// Build the marker block content for an injection.
///
/// If `embed_content` is provided, wraps it between markers.
/// Otherwise, extracts the existing marker block from the template.
fn build_marker_block(template: &str, embed_content: Option<&str>) -> Result<String> {
    match embed_content {
        Some(content) => Ok(format!("{}\n{}\n{}", MARKER_BEGIN, content.trim(), MARKER_END)),
        None => extract_marker_block(template)
            .context("Template is missing straymark markers"),
    }
}

/// Build the full file content by replacing markers in the template with the marker block.
fn build_full_content(template: &str, marker_block: &str) -> String {
    if let Some(original_block) = extract_marker_block(template) {
        template.replace(&original_block, marker_block)
    } else {
        // Template has no markers — append block at the end
        format!("{}\n\n{}\n", template.trim_end(), marker_block)
    }
}

/// Unified injection: apply a template (with optional embed content) to a target file.
///
/// - If the target doesn't exist → create it with the full template content.
/// - If the target has any StrayMark marker (BEGIN or END, even malformed) →
///   replace the canonical block AND auto-repair orphan markers.
/// - If the target exists without any marker → append the marker block.
pub fn inject_directive(target: &Path, template_content: &str, embed_content: Option<&str>) -> Result<()> {
    let marker_block = build_marker_block(template_content, embed_content)?;
    let full_content = build_full_content(template_content, &marker_block);

    if target.exists() {
        let content = std::fs::read_to_string(target).context("Failed to read directive file")?;
        let has_any_marker = content.contains(MARKER_BEGIN) || content.contains(MARKER_END);

        if has_any_marker {
            // Replace canonical block + auto-sanitize any malformed markers.
            let new_content = replace_between_markers(&content, &marker_block);
            std::fs::write(target, new_content).context("Failed to write directive file")?;
        } else {
            // Append injection — no markers present, so nothing to sanitize.
            let new_content = format!("{}\n\n{}\n", content.trim_end(), marker_block);
            std::fs::write(target, new_content).context("Failed to write directive file")?;
        }
    } else {
        // Create new file with full template content
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).context("Failed to create directory")?;
        }
        std::fs::write(target, full_content).context("Failed to create directive file")?;
    }

    Ok(())
}

/// Remove StrayMark injection from a directive file
pub fn remove_injection(target: &Path) -> Result<bool> {
    if !target.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(target).context("Failed to read file")?;

    if !content.contains(MARKER_BEGIN) && !content.contains(MARKER_END) {
        return Ok(false);
    }

    let new_content = remove_between_markers(&content);
    let trimmed = new_content.trim();

    if trimmed.is_empty() {
        // File is empty after removal — delete it
        std::fs::remove_file(target).context("Failed to remove empty directive file")?;
    } else {
        std::fs::write(target, format!("{}\n", trimmed))
            .context("Failed to write updated file")?;
    }

    Ok(true)
}

/// Replace the canonical marker block with `replacement`, auto-sanitizing any
/// orphan markers or additional blocks that may exist in `content`.
///
/// If sanitization removes every marker (e.g. the file only had orphans), the
/// replacement is appended at the end with blank-line separation.
fn replace_between_markers(content: &str, replacement: &str) -> String {
    let clean = sanitize_orphan_markers(content);
    match find_canonical_block(&clean) {
        Some((start, end)) => {
            format!("{}{}{}", &clean[..start], replacement, &clean[end..])
        }
        None => {
            if clean.trim().is_empty() {
                format!("{}\n", replacement)
            } else {
                format!("{}\n\n{}\n", clean.trim_end(), replacement)
            }
        }
    }
}

/// Remove the canonical marker block (and any orphan markers / additional blocks)
/// from `content`, including surrounding blank lines.
fn remove_between_markers(content: &str) -> String {
    let clean = sanitize_orphan_markers(content);
    match find_canonical_block(&clean) {
        Some((start, end)) => {
            let before = clean[..start].trim_end();
            let after = clean[end..].trim_start();
            if after.is_empty() {
                before.to_string()
            } else if before.is_empty() {
                after.to_string()
            } else {
                format!("{}\n\n{}", before, after)
            }
        }
        None => clean,
    }
}

/// Diagnostic report on the marker state of a host file.
///
/// The model treats the first BEGIN and the last END in the file as the canonical
/// markers (matching `find_canonical_block`'s asymmetric semantics — see its
/// docstring for why `rfind(END)` is used). Marker literals between the two
/// canonical positions are assumed to be embed content (e.g. STRAYMARK.md's own
/// documentation of the marker convention) and are NOT flagged as orphans.
///
/// Malformed signals:
/// - `begin_count != end_count`: asymmetric marker counts mean at least one
///   marker is unbalanced — the Sentinel case (an `<!-- straymark:end -->`
///   accumulated by the pre-3.15.0 buggy update) shows up as `end_count` one
///   greater than `begin_count`.
/// - `has_canonical_block == false` while markers exist: only orphans, no usable
///   block.
/// - `end_before_begin == true`: the file's first END appears before its first
///   BEGIN, which always indicates corruption.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarkerHealth {
    pub begin_count: usize,
    pub end_count: usize,
    pub has_canonical_block: bool,
    pub end_before_begin: bool,
}

impl MarkerHealth {
    pub fn is_malformed(&self) -> bool {
        if self.begin_count != self.end_count {
            return true;
        }
        if (self.begin_count > 0 || self.end_count > 0) && !self.has_canonical_block {
            return true;
        }
        if self.end_before_begin {
            return true;
        }
        false
    }
}

/// Inspect a host file's marker structure without mutating it. Used by
/// `straymark validate` to surface malformed injections before the user
/// re-runs an update that would auto-repair them.
pub fn inspect_marker_health(path: &Path) -> Result<MarkerHealth> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(analyze_markers(&content))
}

fn analyze_markers(content: &str) -> MarkerHealth {
    let begin_count = content.matches(MARKER_BEGIN).count();
    let end_count = content.matches(MARKER_END).count();
    let first_begin = content.find(MARKER_BEGIN);
    let first_end = content.find(MARKER_END);

    let end_before_begin = match (first_begin, first_end) {
        (Some(b), Some(e)) => e < b,
        _ => false,
    };

    let has_canonical_block = find_canonical_block(content).is_some();

    MarkerHealth {
        begin_count,
        end_count,
        has_canonical_block,
        end_before_begin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_marker_block() {
        let template = "# Header\n\n<!-- straymark:begin -->\nsome content\n<!-- straymark:end -->\n\nfooter";
        let block = extract_marker_block(template).unwrap();
        assert_eq!(block, "<!-- straymark:begin -->\nsome content\n<!-- straymark:end -->");
    }

    #[test]
    fn test_extract_marker_block_missing() {
        let template = "# Header\n\nno markers here";
        assert!(extract_marker_block(template).is_none());
    }

    #[test]
    fn test_build_marker_block_with_embed() {
        let template = "# Header\n\n<!-- straymark:begin -->\n<!-- straymark:end -->\n";
        let block = build_marker_block(template, Some("embedded content")).unwrap();
        assert_eq!(block, "<!-- straymark:begin -->\nembedded content\n<!-- straymark:end -->");
    }

    #[test]
    fn test_build_marker_block_without_embed() {
        let template = "# Header\n\n<!-- straymark:begin -->\nstatic ref\n<!-- straymark:end -->\n";
        let block = build_marker_block(template, None).unwrap();
        assert_eq!(block, "<!-- straymark:begin -->\nstatic ref\n<!-- straymark:end -->");
    }

    #[test]
    fn test_inject_directive_creates_file() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("CLAUDE.md");
        let template = "# StrayMark - Claude Code Configuration\n\n<!-- straymark:begin -->\n> **Read rules**\n<!-- straymark:end -->\n";

        inject_directive(&target, template, None).unwrap();

        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("# StrayMark - Claude Code Configuration"));
        assert!(content.contains("<!-- straymark:begin -->"));
        assert!(content.contains("> **Read rules**"));
        assert!(content.contains("<!-- straymark:end -->"));
    }

    #[test]
    fn test_inject_directive_with_embed_creates_file() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join(".cursorrules");
        let template = "# StrayMark - Cursor Configuration\n\n<!-- straymark:begin -->\n<!-- straymark:end -->\n";

        inject_directive(&target, template, Some("# My Rules\nRule 1\nRule 2")).unwrap();

        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("# StrayMark - Cursor Configuration"));
        assert!(content.contains("# My Rules"));
        assert!(content.contains("Rule 1"));
    }

    #[test]
    fn test_inject_directive_appends_to_existing() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("CLAUDE.md");
        std::fs::write(&target, "# My Project\n\nCustom config here\n").unwrap();

        let template = "# StrayMark\n\n<!-- straymark:begin -->\n> **Read rules**\n<!-- straymark:end -->\n";
        inject_directive(&target, template, None).unwrap();

        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("# My Project"));
        assert!(content.contains("Custom config here"));
        assert!(content.contains("<!-- straymark:begin -->"));
        assert!(content.contains("> **Read rules**"));
    }

    #[test]
    fn test_inject_directive_replaces_existing_markers() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("CLAUDE.md");
        std::fs::write(&target, "# My Project\n\n<!-- straymark:begin -->\nold content\n<!-- straymark:end -->\n\nfooter\n").unwrap();

        let template = "# StrayMark\n\n<!-- straymark:begin -->\nnew content\n<!-- straymark:end -->\n";
        inject_directive(&target, template, None).unwrap();

        let content = std::fs::read_to_string(&target).unwrap();
        assert!(content.contains("new content"));
        assert!(!content.contains("old content"));
        assert!(content.contains("# My Project"));
        assert!(content.contains("footer"));
    }

    #[test]
    fn test_inject_directive_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join(".github/copilot-instructions.md");
        let template = "# Config\n\n<!-- straymark:begin -->\nref\n<!-- straymark:end -->\n";

        inject_directive(&target, template, None).unwrap();
        assert!(target.exists());
    }

    #[test]
    fn test_replace_between_markers() {
        let content = "before\n<!-- straymark:begin -->\nold\n<!-- straymark:end -->\nafter";
        let result = replace_between_markers(
            content,
            "<!-- straymark:begin -->\nnew\n<!-- straymark:end -->",
        );
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
        assert!(result.contains("before"));
        assert!(result.contains("after"));
        // No leftover/duplicated markers.
        assert_eq!(result.matches(MARKER_BEGIN).count(), 1);
        assert_eq!(result.matches(MARKER_END).count(), 1);
    }

    #[test]
    fn test_remove_between_markers() {
        let content = "header\n\n<!-- straymark:begin -->\nstuff\n<!-- straymark:end -->\n\nfooter";
        let result = remove_between_markers(content);
        assert!(result.contains("header"));
        assert!(result.contains("footer"));
        assert!(!result.contains("stuff"));
    }

    // --- Auto-sanitization tests (Sentinel #157 regression coverage) ---

    #[test]
    fn test_replace_with_orphan_end_before_begin() {
        // Reproduces the Sentinel scenario: an orphan <!-- end --> sits above a
        // valid begin..end block. The old buggy logic produced output with
        // duplicated end markers; the fix must collapse to exactly one pair.
        let content = "header\n\n<!-- straymark:end -->\n\ncontext line\n<!-- straymark:begin -->\nold\n<!-- straymark:end -->\nfooter\n";
        let result = replace_between_markers(
            content,
            "<!-- straymark:begin -->\nnew\n<!-- straymark:end -->",
        );
        assert_eq!(result.matches(MARKER_BEGIN).count(), 1, "exactly one BEGIN: {}", result);
        assert_eq!(result.matches(MARKER_END).count(), 1, "exactly one END: {}", result);
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
        assert!(result.contains("header"));
        assert!(result.contains("context line"));
        assert!(result.contains("footer"));
    }

    #[test]
    fn test_replace_with_duplicate_complete_blocks() {
        // Two complete BEGIN..END pairs (pathological corruption). Under the
        // first-BEGIN/last-END model the canonical span engulfs both pairs +
        // any text between them. Trade-off documented on find_canonical_block:
        // corrupted-marker regions are not safe to preserve content from.
        let content = "A\n<!-- straymark:begin -->\nfirst\n<!-- straymark:end -->\nB\n<!-- straymark:begin -->\nsecond\n<!-- straymark:end -->\nC\n";
        let result = replace_between_markers(
            content,
            "<!-- straymark:begin -->\nnew\n<!-- straymark:end -->",
        );
        assert_eq!(result.matches(MARKER_BEGIN).count(), 1, "exactly one BEGIN: {}", result);
        assert_eq!(result.matches(MARKER_END).count(), 1, "exactly one END: {}", result);
        assert!(result.contains("new"));
        assert!(!result.contains("first"));
        assert!(!result.contains("second"));
        assert!(result.contains("A"));
        assert!(result.contains("C"));
    }

    #[test]
    fn test_replace_with_orphan_begin_no_end() {
        // A lone BEGIN with no matching END — saneamiento lo elimina y el
        // bloque nuevo se appendea al final.
        let content = "header\n<!-- straymark:begin -->\nstuck content\n";
        let result = replace_between_markers(
            content,
            "<!-- straymark:begin -->\nnew\n<!-- straymark:end -->",
        );
        assert_eq!(result.matches(MARKER_BEGIN).count(), 1);
        assert_eq!(result.matches(MARKER_END).count(), 1);
        assert!(result.contains("new"));
        assert!(result.contains("header"));
        assert!(result.contains("stuck content"));
    }

    #[test]
    fn test_replace_idempotent_on_malformed() {
        // Running replace twice on a malformed input must converge to a stable
        // output after the first pass.
        let content = "<!-- straymark:end -->\n\nA\n<!-- straymark:begin -->\nold\n<!-- straymark:end -->\nB\n";
        let replacement = "<!-- straymark:begin -->\nnew\n<!-- straymark:end -->";
        let first = replace_between_markers(content, replacement);
        let second = replace_between_markers(&first, replacement);
        assert_eq!(first, second, "second pass diverged:\nfirst:\n{}\nsecond:\n{}", first, second);
    }

    #[test]
    fn test_inject_directive_repairs_corrupted_cursorrules() {
        // End-to-end Sentinel scenario via the public API. The corrupted
        // .cursorrules has a duplicate end marker; update-framework (which
        // calls inject_directive) must auto-repair to a single canonical pair.
        let dir = TempDir::new().unwrap();
        let target = dir.path().join(".cursorrules");
        std::fs::write(
            &target,
            "# User Cursor Config\n\n<!-- straymark:end -->\n\n<!-- straymark:begin -->\nold STRAYMARK.md content\n<!-- straymark:end -->\n",
        )
        .unwrap();

        let template = "# Cursor Config\n\n<!-- straymark:begin -->\n<!-- straymark:end -->\n";
        inject_directive(&target, template, Some("# STRAYMARK.md\nnew content")).unwrap();

        let content = std::fs::read_to_string(&target).unwrap();
        assert_eq!(content.matches(MARKER_BEGIN).count(), 1, "repaired file: {}", content);
        assert_eq!(content.matches(MARKER_END).count(), 1, "repaired file: {}", content);
        assert!(content.contains("# User Cursor Config"));
        assert!(content.contains("new content"));
        assert!(!content.contains("old STRAYMARK.md content"));
    }

    #[test]
    fn test_inject_directive_repairs_orphan_end_only_file() {
        // The file has ONLY an orphan end marker (no begin). The injection
        // must remove the orphan and create a fresh block.
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("CLAUDE.md");
        std::fs::write(&target, "# Project\n\n<!-- straymark:end -->\n\nMore prose.\n").unwrap();

        let template = "# StrayMark\n\n<!-- straymark:begin -->\nrules\n<!-- straymark:end -->\n";
        inject_directive(&target, template, None).unwrap();

        let content = std::fs::read_to_string(&target).unwrap();
        assert_eq!(content.matches(MARKER_BEGIN).count(), 1);
        assert_eq!(content.matches(MARKER_END).count(), 1);
        assert!(content.contains("# Project"));
        assert!(content.contains("More prose."));
        assert!(content.contains("rules"));
    }

    // --- MarkerHealth tests ---

    #[test]
    fn test_inspect_marker_health_healthy() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("f.md");
        std::fs::write(&target, "header\n<!-- straymark:begin -->\nx\n<!-- straymark:end -->\nfooter\n").unwrap();
        let h = inspect_marker_health(&target).unwrap();
        assert_eq!(h.begin_count, 1);
        assert_eq!(h.end_count, 1);
        assert!(h.has_canonical_block);
        assert!(!h.end_before_begin);
        assert!(!h.is_malformed());
    }

    #[test]
    fn test_inspect_marker_health_no_markers() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("f.md");
        std::fs::write(&target, "just prose, no markers\n").unwrap();
        let h = inspect_marker_health(&target).unwrap();
        assert_eq!(h, MarkerHealth::default());
        assert!(!h.is_malformed());
    }

    #[test]
    fn test_inspect_marker_health_healthy_with_marker_literals_in_embed() {
        // Critical: `.cursorrules` (and `.cursor/rules/straymark.md`) embed
        // `STRAYMARK.md`, whose own documentation contains the literal strings
        // `<!-- straymark:begin -->` / `<!-- straymark:end -->` inside a fenced
        // code block. The model must NOT flag such an embed as malformed.
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("cursorrules");
        std::fs::write(
            &target,
            "# Header\n\n<!-- straymark:begin -->\nintro\n\n```\n<!-- straymark:begin -->\n... managed content ...\n<!-- straymark:end -->\n```\n\nmore text\n<!-- straymark:end -->\n",
        )
        .unwrap();
        let h = inspect_marker_health(&target).unwrap();
        assert_eq!(h.begin_count, 2);
        assert_eq!(h.end_count, 2);
        assert!(h.has_canonical_block);
        assert!(!h.end_before_begin);
        assert!(!h.is_malformed(), "embed with literal markers must not flag malformed");
    }

    #[test]
    fn test_inspect_marker_health_end_before_begin() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("f.md");
        std::fs::write(&target, "<!-- straymark:end -->\n<!-- straymark:begin -->\nx\n<!-- straymark:end -->\n").unwrap();
        let h = inspect_marker_health(&target).unwrap();
        assert!(h.is_malformed());
        assert!(h.end_before_begin);
        assert_eq!(h.begin_count, 1);
        assert_eq!(h.end_count, 2);
    }

    #[test]
    fn test_inspect_marker_health_extra_orphan_end() {
        // Sentinel scenario: an extra `<!-- straymark:end -->` accumulated
        // after the canonical block (the bug class fixed in cli-3.15.0).
        // Detected as asymmetric counts: end_count > begin_count.
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("f.md");
        std::fs::write(
            &target,
            "# Project\n\n<!-- straymark:begin -->\ncontent\n<!-- straymark:end -->\n\nstale text\n<!-- straymark:end -->\n",
        )
        .unwrap();
        let h = inspect_marker_health(&target).unwrap();
        assert!(h.is_malformed());
        assert_eq!(h.begin_count, 1);
        assert_eq!(h.end_count, 2);
    }

    #[test]
    fn test_inspect_marker_health_orphan_begin() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("f.md");
        std::fs::write(&target, "<!-- straymark:begin -->\ncontent\n").unwrap();
        let h = inspect_marker_health(&target).unwrap();
        assert!(h.is_malformed());
        assert_eq!(h.begin_count, 1);
        assert_eq!(h.end_count, 0);
        assert!(!h.has_canonical_block);
    }
}
