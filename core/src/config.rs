//! Project language resolution, shared by the CLI and Loom so both agree on
//! which of StrayMark's supported locales (`en`, `es`, `zh-CN`) to display.
//!
//! Extracted into `straymark-core` in Loom M3: the CLI's
//! `StrayMarkConfig::resolve_language` delegates here, and Loom calls
//! [`resolve_language`] directly to localize its UI — one source of truth, no
//! drift (the same principle as the M0 parser extraction).

use std::path::Path;

use serde::Deserialize;

/// The default StrayMark UI language when nothing else resolves.
pub const DEFAULT_LANGUAGE: &str = "en";

/// Just the `language` key of `.straymark/config.yml`; the full config struct
/// (complexity, regional scope, …) stays in the CLI.
#[derive(Debug, Deserialize)]
struct LanguageOnly {
    #[serde(default)]
    language: Option<String>,
}

/// Resolve the effective display language for a project, applying all
/// fallbacks in order:
///
/// 1. If `.straymark/config.yml` exists on disk, the value of its `language`
///    key (defaulting to `"en"` when the field is absent or the file fails to
///    parse). A configured value — even the default `"en"` — is treated as an
///    explicit choice and is never overridden by env vars.
/// 2. If no config file exists, parse `$LC_ALL` / `$LANG` and map it onto a
///    supported locale (`en`, `es`, `zh-CN`).
/// 3. Final fallback: `"en"`.
pub fn resolve_language(project_root: &Path) -> String {
    let config_path = project_root.join(".straymark/config.yml");
    if config_path.exists() {
        return std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|contents| serde_yaml::from_str::<LanguageOnly>(&contents).ok())
            .and_then(|c| c.language)
            .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());
    }
    detect_os_locale().unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(parse_posix_locale("fr_FR.UTF-8"), None);
        assert_eq!(parse_posix_locale("ja_JP.UTF-8"), None);
        assert_eq!(parse_posix_locale(""), None);
    }

    #[test]
    fn parse_posix_locale_strips_charset_and_modifier() {
        assert_eq!(parse_posix_locale("es_ES@euro"), Some("es".into()));
    }

    #[test]
    fn resolve_language_reads_config_language() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
        std::fs::write(
            tmp.path().join(".straymark/config.yml"),
            "language: zh-CN\n",
        )
        .unwrap();
        assert_eq!(resolve_language(tmp.path()), "zh-CN");
    }

    #[test]
    fn resolve_language_config_without_language_key_defaults_en() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
        std::fs::write(
            tmp.path().join(".straymark/config.yml"),
            "complexity:\n  threshold: 5\n",
        )
        .unwrap();
        assert_eq!(resolve_language(tmp.path()), "en");
    }
}
