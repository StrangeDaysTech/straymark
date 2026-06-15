//! Low-level parsing helpers shared across the document model.
//!
//! Moved into `straymark-core` in Loom A1.0 (`002-architecture-plan`) so the
//! CLI's Charter/follow-ups parsers and the architecture-plan projection share
//! one definition of what a frontmatter block is.

/// Split a markdown document into (frontmatter, body) at the first pair of `---`
/// delimiters. Returns `None` if the document has no frontmatter block.
/// Accepts both `\n` and `\r\n` line endings on the closing delimiter.
/// Shared by the Charter parser ([`crate::charter`]) and, via a CLI re-export,
/// the follow-ups registry parser so both artifacts agree on what a frontmatter
/// block is.
pub fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    // Opening delimiter: must be at the very start of the file.
    let after_open = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;
    // Closing delimiter: find the first occurrence of `\n---\n` or `\r\n---\r\n`.
    let (end, delim_len) = if let Some(idx) = after_open.find("\n---\n") {
        (idx, 5)
    } else if let Some(idx) = after_open.find("\r\n---\r\n") {
        (idx, 7)
    } else if let Some(idx) = after_open.find("\n---\r\n") {
        (idx, 6)
    } else {
        return None;
    };
    let frontmatter = &after_open[..end];
    let body = &after_open[end + delim_len..];
    Some((frontmatter, body))
}
