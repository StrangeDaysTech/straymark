use anyhow::{Context, Result};
use std::path::Path;

const MARKER_BEGIN: &str = "<!-- straymark:begin -->";
const MARKER_END: &str = "<!-- straymark:end -->";

/// Extract the marker block (begin marker through end marker, inclusive) from a template string.
/// Returns None if markers are not found.
fn extract_marker_block(template: &str) -> Option<String> {
    let start = template.find(MARKER_BEGIN)?;
    let end = template.find(MARKER_END)?;
    let end = end + MARKER_END.len();
    Some(template[start..end].to_string())
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
/// - If the target exists and has markers → replace the marker block.
/// - If the target exists without markers → append the marker block.
pub fn inject_directive(target: &Path, template_content: &str, embed_content: Option<&str>) -> Result<()> {
    let marker_block = build_marker_block(template_content, embed_content)?;
    let full_content = build_full_content(template_content, &marker_block);

    if target.exists() {
        let content = std::fs::read_to_string(target).context("Failed to read directive file")?;

        if content.contains(MARKER_BEGIN) {
            // Replace existing injection
            let new_content = replace_between_markers(&content, &marker_block);
            std::fs::write(target, new_content).context("Failed to write directive file")?;
        } else {
            // Append injection
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

    if !content.contains(MARKER_BEGIN) {
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

/// Replace content between markers (inclusive)
fn replace_between_markers(content: &str, replacement: &str) -> String {
    if let (Some(start), Some(end)) = (content.find(MARKER_BEGIN), content.find(MARKER_END)) {
        let end = end + MARKER_END.len();
        format!("{}{}{}", &content[..start], replacement, &content[end..])
    } else {
        content.to_string()
    }
}

/// Remove content between markers (inclusive), including surrounding blank lines
fn remove_between_markers(content: &str) -> String {
    if let (Some(start), Some(end)) = (content.find(MARKER_BEGIN), content.find(MARKER_END)) {
        let end = end + MARKER_END.len();
        let before = content[..start].trim_end();
        let after = content[end..].trim_start();
        if after.is_empty() {
            before.to_string()
        } else {
            format!("{}\n\n{}", before, after)
        }
    } else {
        content.to_string()
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
    }

    #[test]
    fn test_remove_between_markers() {
        let content = "header\n\n<!-- straymark:begin -->\nstuff\n<!-- straymark:end -->\n\nfooter";
        let result = remove_between_markers(content);
        assert!(result.contains("header"));
        assert!(result.contains("footer"));
        assert!(!result.contains("stuff"));
    }
}
