use colored::Colorize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "→".blue().bold(), msg);
}

/// Print a warning message
pub fn warn(msg: &str) {
    println!("{} {}", "!".yellow().bold(), msg);
}

/// Compute SHA-256 hash of a file's contents
pub fn file_hash(path: &Path) -> Option<String> {
    let content = std::fs::read(path).ok()?;
    let hash = Sha256::digest(&content);
    Some(format!("{:x}", hash))
}

/// Check if a path looks like a user-generated StrayMark document
/// (matches pattern: *-YYYY-MM-DD-NNN-*.md)
pub fn is_user_document(path: &Path) -> bool {
    let name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    // Check for patterns like AILOG-2025-01-27-001-description.md
    let prefixes = [
        "AILOG-", "AIDEC-", "ETH-", "ADR-", "REQ-", "TES-", "INC-", "TDE-",
        "SEC-", "MCARD-", "SBOM-", "DPIA-",
    ];

    prefixes.iter().any(|p| name.starts_with(p))
}

/// Ensure a directory exists, creating it if needed
pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Result of resolving the StrayMark project root
pub struct ResolvedPath {
    /// The resolved project root where .straymark/ exists
    pub path: std::path::PathBuf,
    /// Whether we fell back to the git repo root (not the original path)
    pub is_fallback: bool,
}

/// Resolve the StrayMark project root from a given path.
///
/// 1. If `path` has `.straymark/`, use it directly
/// 2. If not, try the git repo root
/// 3. If neither has `.straymark/`, return None
pub fn resolve_project_root(path: &str) -> Option<ResolvedPath> {
    let target = std::path::PathBuf::from(path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(path));

    // Check the given path first
    if target.join(".straymark").exists() {
        return Some(ResolvedPath {
            path: target,
            is_fallback: false,
        });
    }

    // Try git repo root
    let git_root = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&target)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Some(std::path::PathBuf::from(root))
            } else {
                None
            }
        });

    if let Some(root) = git_root {
        // Don't fallback to the same path we already checked
        if root != target && root.join(".straymark").exists() {
            return Some(ResolvedPath {
                path: root,
                is_fallback: true,
            });
        }
    }

    None
}

/// Read `$LC_ALL` (preferred when set) or `$LANG` and map a POSIX locale
/// string like `zh_CN.UTF-8` or `es_MX` to one of the languages StrayMark
/// supports (`en`, `es`, `zh-CN`). Returns `None` when no env var is set
/// or when the territory points at an unsupported variant (e.g.,
/// Traditional Chinese in `zh_TW` / `zh_HK`). Callers fall back to `"en"`.
pub fn detect_os_locale() -> Option<String> {
    let raw = std::env::var("LC_ALL")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("LANG").ok().filter(|v| !v.is_empty()))?;
    parse_posix_locale(&raw)
}

/// Parse a POSIX locale string (e.g. `zh_CN.UTF-8`, `es`, `C`) and map it
/// to a StrayMark-supported language code. Public for unit testing.
pub fn parse_posix_locale(raw: &str) -> Option<String> {
    // Strip charset (`.UTF-8`) and modifier (`@euro`) suffixes first.
    let trimmed = raw.split('.').next()?.split('@').next()?;
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.splitn(2, '_');
    let lang = parts.next()?;
    let territory = parts.next();
    match (lang, territory) {
        ("zh", Some("CN")) | ("zh", Some("SG")) | ("zh", None) => Some("zh-CN".to_string()),
        // Traditional Chinese (TW / HK / MO) — StrayMark only ships zh-CN.
        ("zh", _) => None,
        ("es", _) => Some("es".to_string()),
        ("en", _) | ("C", _) | ("POSIX", _) => Some("en".to_string()),
        _ => None,
    }
}

/// Resolve `<dir>/<filename>` honoring an optional translation under
/// `<dir>/i18n/<lang>/<filename>`. When `lang` is `"en"` (or any value where
/// the localized variant is absent), returns the root path unchanged. This is
/// the single source of truth for i18n file resolution shared by `straymark
/// new` (templates) and `straymark explore` (governance docs).
pub fn resolve_localized_path(dir: &Path, filename: &str, lang: &str) -> PathBuf {
    if lang != "en" {
        let candidate = dir.join("i18n").join(lang).join(filename);
        if candidate.exists() {
            return candidate;
        }
    }
    dir.join(filename)
}

/// Visual width of a string in terminal columns, accounting for double-wide
/// characters (CJK, some emoji). This is the unit every TUI layout should
/// use — `.len()` measures bytes and `.chars().count()` measures code points,
/// neither of which matches how a terminal renders text.
pub fn visual_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Truncate `s` to fit within `max_cols` visual columns, appending "…"
/// (1 column) when truncation happens. Guarantees the returned string's
/// `visual_width()` is `<= max_cols` and that every byte offset used is a
/// valid UTF-8 char boundary.
#[cfg_attr(not(any(feature = "tui", feature = "analyze")), allow(dead_code))]
pub fn truncate_visual(s: &str, max_cols: usize) -> String {
    if max_cols == 0 {
        return String::new();
    }
    if visual_width(s) <= max_cols {
        return s.to_string();
    }
    // Reserve 1 column for the ellipsis when there's room for it.
    let budget = max_cols.saturating_sub(1);
    let mut used = 0usize;
    let mut cut_at = 0usize;
    for (byte_idx, ch) in s.char_indices() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > budget {
            cut_at = byte_idx;
            break;
        }
        used += w;
        cut_at = byte_idx + ch.len_utf8();
    }
    let mut out = String::with_capacity(cut_at + 3);
    out.push_str(&s[..cut_at]);
    out.push('…');
    out
}

/// Right-pad `s` with ASCII spaces so its visual width is exactly `cols`.
/// If `s` is already at least that wide, return it unchanged. Unlike
/// `format!("{:<N$}", ...)`, this counts terminal columns, not chars.
pub fn pad_right_visual(s: &str, cols: usize) -> String {
    let w = visual_width(s);
    if w >= cols {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + (cols - w));
    out.push_str(s);
    out.extend(std::iter::repeat_n(' ', cols - w));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_width_ascii() {
        assert_eq!(visual_width("hello"), 5);
        assert_eq!(visual_width(""), 0);
    }

    #[test]
    fn visual_width_accents_one_col_each() {
        assert_eq!(visual_width("áéíóú"), 5);
    }

    #[test]
    fn visual_width_cjk_two_cols_each() {
        assert_eq!(visual_width("数据"), 4);
    }

    #[test]
    fn truncate_visual_short_returns_as_is() {
        assert_eq!(truncate_visual("hello", 10), "hello");
    }

    #[test]
    fn truncate_visual_ascii_truncates_with_ellipsis() {
        let out = truncate_visual("hello world", 8);
        assert!(visual_width(&out) <= 8);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn truncate_visual_cjk_respects_double_width() {
        // 数据表格 (4 ideograms, visual width 8). Budget 5 → must fit with ellipsis.
        let out = truncate_visual("数据表格", 5);
        assert!(visual_width(&out) <= 5);
        assert!(std::str::from_utf8(out.as_bytes()).is_ok());
    }

    #[test]
    fn truncate_visual_em_dash_no_panic() {
        let s = "Partially mitigated — RLS is not active until middleware";
        for w in [5usize, 10, 20, 67] {
            let out = truncate_visual(s, w);
            assert!(visual_width(&out) <= w, "{out:?} too wide for {w}");
        }
    }

    #[test]
    fn truncate_visual_zero_width() {
        assert_eq!(truncate_visual("anything", 0), "");
    }

    #[test]
    fn pad_right_visual_ascii() {
        assert_eq!(pad_right_visual("hi", 5), "hi   ");
    }

    #[test]
    fn pad_right_visual_cjk_counts_two_columns() {
        // "数" has visual width 2. Padding to 5 should add 3 spaces.
        let out = pad_right_visual("数", 5);
        assert_eq!(visual_width(&out), 5);
        assert!(out.ends_with("   "));
    }

    #[test]
    fn pad_right_visual_already_wider_returns_as_is() {
        assert_eq!(pad_right_visual("hello", 3), "hello");
    }

    #[test]
    fn resolve_localized_path_uses_translation_when_present() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        let translated = dir.join("i18n").join("zh-CN");
        std::fs::create_dir_all(&translated).unwrap();
        std::fs::write(dir.join("FOO.md"), "english").unwrap();
        std::fs::write(translated.join("FOO.md"), "中文").unwrap();

        let resolved = resolve_localized_path(dir, "FOO.md", "zh-CN");
        assert_eq!(resolved, translated.join("FOO.md"));
    }

    #[test]
    fn resolve_localized_path_falls_back_to_english_when_translation_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        std::fs::write(dir.join("FOO.md"), "english").unwrap();

        let resolved = resolve_localized_path(dir, "FOO.md", "zh-CN");
        assert_eq!(resolved, dir.join("FOO.md"));
    }

    #[test]
    fn parse_posix_locale_zh_cn() {
        assert_eq!(parse_posix_locale("zh_CN.UTF-8"), Some("zh-CN".into()));
        assert_eq!(parse_posix_locale("zh_CN"), Some("zh-CN".into()));
        assert_eq!(parse_posix_locale("zh_SG.UTF-8"), Some("zh-CN".into()));
        // Bare "zh" with no territory: assume Simplified.
        assert_eq!(parse_posix_locale("zh"), Some("zh-CN".into()));
    }

    #[test]
    fn parse_posix_locale_traditional_chinese_unsupported() {
        // We don't ship Traditional translations — those should fall back
        // through to "en" via the caller's None handling, not silently
        // claim Simplified.
        assert_eq!(parse_posix_locale("zh_TW.UTF-8"), None);
        assert_eq!(parse_posix_locale("zh_HK.UTF-8"), None);
    }

    #[test]
    fn parse_posix_locale_spanish_any_territory() {
        assert_eq!(parse_posix_locale("es_MX.UTF-8"), Some("es".into()));
        assert_eq!(parse_posix_locale("es_ES"), Some("es".into()));
        assert_eq!(parse_posix_locale("es_AR.UTF-8"), Some("es".into()));
    }

    #[test]
    fn parse_posix_locale_english_and_pseudo() {
        assert_eq!(parse_posix_locale("en_US.UTF-8"), Some("en".into()));
        assert_eq!(parse_posix_locale("en"), Some("en".into()));
        assert_eq!(parse_posix_locale("C"), Some("en".into()));
        assert_eq!(parse_posix_locale("POSIX"), Some("en".into()));
    }

    #[test]
    fn parse_posix_locale_unsupported_returns_none() {
        // French isn't translated yet; caller must fall back to "en".
        assert_eq!(parse_posix_locale("fr_FR.UTF-8"), None);
        assert_eq!(parse_posix_locale("ja_JP.UTF-8"), None);
        assert_eq!(parse_posix_locale(""), None);
    }

    #[test]
    fn parse_posix_locale_strips_charset_and_modifier() {
        // `@modifier` after the charset (or before it) appears in some
        // locales; we strip whatever follows the first `@` or `.`.
        assert_eq!(parse_posix_locale("es_ES@euro"), Some("es".into()));
    }

    #[test]
    fn resolve_localized_path_for_english_skips_lookup() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        // Even if a stale i18n/en/ folder existed, "en" must always return root.
        let stale = dir.join("i18n").join("en");
        std::fs::create_dir_all(&stale).unwrap();
        std::fs::write(stale.join("FOO.md"), "should not be picked").unwrap();
        std::fs::write(dir.join("FOO.md"), "english").unwrap();

        let resolved = resolve_localized_path(dir, "FOO.md", "en");
        assert_eq!(resolved, dir.join("FOO.md"));
    }
}
