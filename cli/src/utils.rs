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

    // Check the given path first. Never treat a `.straymark/` that is NOT an
    // installed project as one — the framework distribution source (`dist/`,
    // safeguard S1) or a directory whose provenance marks a non-install role
    // (safeguard S2). Operating on the distribution would mutate the shipped
    // template (write AILOGs/AIDECs into `dist/`, validate the source, …). Skip
    // it so resolution falls through to the real install (git root) or reports
    // not-installed.
    if target.join(".straymark").exists() {
        match non_project_reason(&target) {
            Some(reason) => warn_non_project_skipped(&reason),
            None => {
                return Some(ResolvedPath {
                    path: target,
                    is_fallback: false,
                })
            }
        }
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
            match non_project_reason(&root) {
                Some(reason) => warn_non_project_skipped(&reason),
                None => {
                    return Some(ResolvedPath {
                        path: root,
                        is_fallback: true,
                    })
                }
            }
        }
    }

    None
}

/// Reason a `.straymark/` at `dir` must NOT be treated as an installed project,
/// or `None` if it is operable. Refuses the framework distribution source (S1)
/// and any directory whose provenance records an explicit non-install role (S2).
/// An **absent** provenance file is operable — legacy installs (existing
/// adopters) carry none and must keep working.
fn non_project_reason(dir: &Path) -> Option<String> {
    if is_distribution_source(dir) {
        return Some(format!(
            "{} is the framework distribution source (a sibling `dist-manifest.yml` marks it)",
            dir.join(".straymark").display()
        ));
    }
    if let Some(role) = provenance_role(dir) {
        if role != "installed-project" {
            return Some(format!(
                "{} has provenance `role: {}` (not an installed project)",
                dir.join(".straymark").display(),
                role
            ));
        }
    }
    None
}

/// True when `dir` holds the framework **distribution source** rather than an
/// installed project: its `.straymark/` sits next to a `dist-manifest.yml`. That
/// manifest is unique to StrayMark's own `dist/` — no adopter project ships one
/// — so it is a precise, cheap signal. (Auto-adoption safeguard S1.)
pub fn is_distribution_source(dir: &Path) -> bool {
    dir.join("dist-manifest.yml").exists()
}

/// The `role:` recorded in `<dir>/.straymark/.provenance.yml` (auto-adoption
/// safeguard S2, written by `straymark init`), or `None` if the file is absent
/// (a legacy install) or has no `role:` line. Line-scanned rather than
/// YAML-parsed to keep the hot resolution path dependency-light.
pub fn provenance_role(dir: &Path) -> Option<String> {
    let content =
        std::fs::read_to_string(dir.join(".straymark").join(".provenance.yml")).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("role:") {
            let val = rest.trim().trim_matches(['"', '\'']).to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

fn warn_non_project_skipped(reason: &str) {
    eprintln!(
        "{} skipping {} — not an installed StrayMark project. Run from the project \
         root, or `straymark init` to install.",
        "note:".yellow().bold(),
        reason
    );
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

// `split_frontmatter` moved to `straymark_core::utils` in Loom A1.0 so the
// architecture-plan projection and the Loom server share one definition.
// Re-exported so the follow-ups registry parser (`crate::followups`) and
// `crate::commands::approve` keep their `crate::utils::split_frontmatter` paths.
pub use straymark_core::utils::split_frontmatter;

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

    // ── Auto-adoption safeguard S1: distribution-source guard ─────────────

    #[test]
    fn is_distribution_source_detects_dist_manifest_sibling() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        assert!(!is_distribution_source(dir));
        std::fs::write(dir.join("dist-manifest.yml"), "version: \"1.0.0\"\n").unwrap();
        assert!(is_distribution_source(dir));
    }

    #[test]
    fn resolve_project_root_refuses_distribution_source() {
        // A `dist/.straymark` with a sibling `dist-manifest.yml` must NEVER
        // resolve as a project (it is the shipped template). With no real
        // install to fall back to, resolution is None — not `dist/`.
        let tmp = tempfile::TempDir::new().unwrap();
        let dist = tmp.path().join("dist");
        std::fs::create_dir_all(dist.join(".straymark")).unwrap();
        std::fs::write(dist.join("dist-manifest.yml"), "version: \"1.0.0\"\n").unwrap();
        assert!(
            resolve_project_root(dist.to_str().unwrap()).is_none(),
            "distribution source must never resolve as an installed project"
        );
    }

    #[test]
    fn resolve_project_root_accepts_normal_install() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
        let resolved = resolve_project_root(tmp.path().to_str().unwrap())
            .expect("a plain .straymark/ (no dist-manifest, no provenance) must resolve");
        assert!(!resolved.is_fallback);
        assert!(resolved.path.join(".straymark").exists());
    }

    // ── Auto-adoption safeguard S2: provenance sentinel ───────────────────

    fn seed_straymark(dir: &Path, provenance: Option<&str>) {
        std::fs::create_dir_all(dir.join(".straymark")).unwrap();
        if let Some(p) = provenance {
            std::fs::write(dir.join(".straymark").join(".provenance.yml"), p).unwrap();
        }
    }

    #[test]
    fn provenance_role_reads_role_line_ignoring_comments() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(provenance_role(tmp.path()), None, "absent file → None");
        seed_straymark(
            tmp.path(),
            Some("# comment\nrole: installed-project\nframework_version: \"4.35.0\"\n"),
        );
        assert_eq!(provenance_role(tmp.path()).as_deref(), Some("installed-project"));
    }

    #[test]
    fn resolve_project_root_tolerates_absent_provenance_legacy() {
        // Existing adopters (Sentinel, lnxdrive) have a `.straymark/` with no
        // provenance file — they MUST keep resolving.
        let tmp = tempfile::TempDir::new().unwrap();
        seed_straymark(tmp.path(), None);
        assert!(resolve_project_root(tmp.path().to_str().unwrap()).is_some());
    }

    #[test]
    fn resolve_project_root_accepts_installed_project_role() {
        let tmp = tempfile::TempDir::new().unwrap();
        seed_straymark(tmp.path(), Some("role: installed-project\n"));
        assert!(resolve_project_root(tmp.path().to_str().unwrap()).is_some());
    }

    #[test]
    fn resolve_project_root_refuses_non_install_role() {
        // A `.straymark/` explicitly stamped as a non-install role (e.g. a test
        // fixture or a hand-marked distribution copy) must not resolve.
        let tmp = tempfile::TempDir::new().unwrap();
        seed_straymark(tmp.path(), Some("role: test-fixture\n"));
        assert!(
            resolve_project_root(tmp.path().to_str().unwrap()).is_none(),
            "a non-install provenance role must be refused"
        );
    }

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
