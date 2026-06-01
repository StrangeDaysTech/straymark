use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// StrayMark project configuration from .straymark/config.yml
#[derive(Debug, Deserialize, Serialize)]
pub struct StrayMarkConfig {
    /// Language setting: "en", "es", or "zh-CN"
    #[serde(default = "default_language")]
    pub language: String,
    /// Complexity analysis settings
    #[serde(default)]
    pub complexity: ComplexityConfig,
    /// Regional regulatory scope. Values: "global" (NIST + ISO 42001),
    /// "eu" (EU AI Act + GDPR), "china" (TC260, PIPL, GB 45438, CAC, GB/T 45652, CSL).
    /// Default `["global", "eu"]` preserves backward compatibility.
    #[serde(default = "default_regional_scope")]
    pub regional_scope: Vec<String>,
    /// Named profiles for `straymark analyze declared-vs-wired`. Empty by default.
    #[serde(default)]
    pub declared_vs_wired: DeclaredVsWiredConfig,
}

/// Configuration for `straymark analyze declared-vs-wired` — a set of named
/// profiles, each pairing a declared-side and a wired-side (glob + capture
/// regex). Lets adopters commit their stack's regexes once instead of passing
/// four flags every run.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DeclaredVsWiredConfig {
    #[serde(default)]
    pub profiles: Vec<DeclaredVsWiredProfile>,
}

/// One declared-vs-wired profile. The capture patterns must each expose a
/// capture group 1 whose value is the symbol name to compare across sides.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeclaredVsWiredProfile {
    pub name: String,
    /// Glob (relative to the analyzed path) for files holding declarations
    /// (e.g. a D-Bus proxy, a gRPC stub, a REST client).
    pub declared_glob: String,
    /// Regex matched against declared files; capture group 1 = symbol name.
    pub declared_pattern: String,
    /// Glob (relative to the analyzed path) for files holding implementations
    /// (e.g. the daemon/server interface).
    pub wired_glob: String,
    /// Regex matched against wired files; capture group 1 = symbol name.
    pub wired_pattern: String,
}

/// Configuration for the `straymark analyze` command
#[derive(Debug, Deserialize, Serialize)]
pub struct ComplexityConfig {
    /// Cognitive complexity threshold (default: 8)
    #[serde(default = "default_threshold")]
    pub threshold: u32,
}

fn default_threshold() -> u32 {
    8
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            threshold: default_threshold(),
        }
    }
}

fn default_language() -> String {
    "en".to_string()
}

fn default_regional_scope() -> Vec<String> {
    vec!["global".to_string(), "eu".to_string()]
}

impl Default for StrayMarkConfig {
    fn default() -> Self {
        Self {
            language: default_language(),
            complexity: ComplexityConfig::default(),
            regional_scope: default_regional_scope(),
            declared_vs_wired: DeclaredVsWiredConfig::default(),
        }
    }
}

impl StrayMarkConfig {
    /// Read config from .straymark/config.yml at the given project root
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join(".straymark/config.yml");
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let contents =
            std::fs::read_to_string(&config_path).context("Failed to read config.yml")?;
        let config: Self = serde_yaml::from_str(&contents).context("Failed to parse config.yml")?;
        Ok(config)
    }

    /// True if `regional_scope` includes the given region (case-insensitive).
    pub fn has_region(&self, region: &str) -> bool {
        self.regional_scope
            .iter()
            .any(|r| r.eq_ignore_ascii_case(region))
    }

    /// Resolve the effective display language for a project, applying all
    /// fallbacks in order:
    ///
    /// 1. If `.straymark/config.yml` exists on disk, the value of its
    ///    `language` key (defaulting to `"en"` when the field is absent).
    ///    A configured value — even the default `"en"` — is treated as an
    ///    explicit choice and is never overridden by env vars.
    /// 2. If no config file exists, parse `$LC_ALL` / `$LANG` and map it
    ///    onto a supported locale (`en`, `es`, `zh-CN`).
    /// 3. Final fallback: `"en"`.
    ///
    /// This is the single entry point used by `straymark explore`,
    /// `straymark new`, and `straymark status` so they all agree on which
    /// language to use.
    pub fn resolve_language(project_root: &Path) -> String {
        let config_path = project_root.join(".straymark/config.yml");
        if config_path.exists() {
            return Self::load(project_root)
                .map(|c| c.language)
                .unwrap_or_else(|_| default_language());
        }
        crate::utils::detect_os_locale().unwrap_or_else(default_language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_regional_scope() {
        let cfg = StrayMarkConfig::default();
        assert!(cfg.has_region("global"));
        assert!(cfg.has_region("eu"));
        assert!(!cfg.has_region("china"));
    }

    #[test]
    fn test_has_region_case_insensitive() {
        let cfg = StrayMarkConfig {
            regional_scope: vec!["China".into(), "GLOBAL".into()],
            ..Default::default()
        };
        assert!(cfg.has_region("china"));
        assert!(cfg.has_region("CHINA"));
        assert!(cfg.has_region("global"));
        assert!(!cfg.has_region("eu"));
    }

    #[test]
    fn resolve_language_uses_config_value_when_present() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dt = tmp.path().join(".straymark");
        std::fs::create_dir_all(&dt).unwrap();
        std::fs::write(dt.join("config.yml"), "language: zh-CN\n").unwrap();

        // Even if $LANG is set to something else, the file's explicit
        // value must win — config is treated as a deliberate user choice.
        let prev = std::env::var("LANG").ok();
        unsafe { std::env::set_var("LANG", "fr_FR.UTF-8"); }
        let lang = StrayMarkConfig::resolve_language(tmp.path());
        if let Some(p) = prev {
            unsafe { std::env::set_var("LANG", p); }
        } else {
            unsafe { std::env::remove_var("LANG"); }
        }
        assert_eq!(lang, "zh-CN");
    }

    #[test]
    fn resolve_language_falls_back_to_default_when_no_config_no_env() {
        let tmp = tempfile::TempDir::new().unwrap();
        // No .straymark/config.yml in tmp.
        // Clear env vars so the OS locale path can't return a real value.
        let prev_all = std::env::var("LC_ALL").ok();
        let prev_lang = std::env::var("LANG").ok();
        unsafe {
            std::env::remove_var("LC_ALL");
            std::env::set_var("LANG", "C");
        }
        let lang = StrayMarkConfig::resolve_language(tmp.path());
        // Restore env.
        unsafe {
            if let Some(p) = prev_all {
                std::env::set_var("LC_ALL", p);
            }
            if let Some(p) = prev_lang {
                std::env::set_var("LANG", p);
            } else {
                std::env::remove_var("LANG");
            }
        }
        // "C" maps to "en", so the result is "en".
        assert_eq!(lang, "en");
    }
}

/// Checksums tracking file for detecting user modifications
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Checksums {
    pub version: String,
    pub files: std::collections::HashMap<String, String>,
}

impl Checksums {
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join(".straymark/.checksums.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents =
            std::fs::read_to_string(&path).context("Failed to read .checksums.json")?;
        let checksums: Self =
            serde_json::from_str(&contents).context("Failed to parse .checksums.json")?;
        Ok(checksums)
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = project_root.join(".straymark/.checksums.json");
        let contents =
            serde_json::to_string_pretty(self).context("Failed to serialize checksums")?;
        std::fs::write(&path, contents).context("Failed to write .checksums.json")?;
        Ok(())
    }
}
