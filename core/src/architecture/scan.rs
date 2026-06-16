//! Project-structure scanning config for the architecture seed (Loom A1, #279).
//!
//! `architecture generate` mines the codebase into a *first-draft* component
//! map. The mining is governed by three lists that used to be hard-coded and
//! tuned to Go/Rust/JS conventions — so a Ruby project produced an empty map and
//! a Java/Maven tree collapsed into one `main` box. This module makes them
//! configurable per project via an `architecture:` section in
//! `.straymark/config.yml`, with built-in defaults that cover the common
//! ecosystems (including multi-module Maven/Gradle out of the box).
//!
//! Config lists are **additive** — they extend the built-in defaults rather than
//! replacing them, so adding `rb` keeps Rust/Go/… working. The projection /
//! overlay is already language-agnostic (it matches globs against paths); this
//! only shapes the *seed*, which the human/agent then refines.

use std::path::Path;

use serde::Deserialize;

/// Extensions that mark a file as source worth seeding from. Built-in defaults;
/// extended by `architecture.source_extensions` in config.
pub const DEFAULT_SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "jsx", "tsx", "java", "go", "cs", "cpp", "cc",
    "cxx", "c", "h", "hpp", "php", "kt", "swift", "rb", "ex", "exs", "scala",
    "dart", "clj", "cljs", "mm", "vue", "svelte", "lua", "jl", "hs", "erl", "ml",
];

/// Directories never scanned for source (build output, VCS, deps, docs).
/// Built-in defaults; extended by `architecture.excluded_dirs`. Kept
/// conservative — `bin`/`obj` are *not* excluded by default since they collide
/// with legitimate source dirs (Rust `src/bin/`); add them per project if your
/// stack uses them as build output.
pub const DEFAULT_EXCLUDED_DIRS: &[&str] = &[
    ".straymark", ".git", "node_modules", "target", "vendor", "dist", "build",
    ".venv", "__pycache__", ".github", "docs", ".idea", ".vscode",
];

/// Organizational directories that hold *other* components rather than being one
/// themselves — the seed descends through them one level. Built-in defaults;
/// extended by `architecture.container_dirs`. `components` is deliberately *not*
/// a default (it would split a React app into one box per UI component); add it
/// per project if you want that granularity.
pub const DEFAULT_CONTAINER_DIRS: &[&str] =
    &["internal", "src", "pkg", "lib", "libs", "app", "apps", "modules", "packages"];

/// Multi-segment build scaffolding to skip wholesale (e.g. Maven/Gradle
/// `src/main/java`). When such a prefix sits *inside* a path, the directory
/// **before** it is the component (multi-module layout: `billing/src/main/java/…`
/// → `billing`); when it sits at the start (single module), the seed descends
/// into the package below it. Built-in defaults; extended by
/// `architecture.scaffolding_prefixes`.
pub const DEFAULT_SCAFFOLDING_PREFIXES: &[&str] = &[
    "src/main/java", "src/main/kotlin", "src/main/scala", "src/main/groovy",
    "src/test/java", "src/test/kotlin", "src/main/resources",
];

/// Resolved scanning configuration: built-in defaults ∪ the project's
/// `architecture:` config section.
#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub source_extensions: Vec<String>,
    pub excluded_dirs: Vec<String>,
    pub container_dirs: Vec<String>,
    pub scaffolding_prefixes: Vec<String>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        ScanConfig {
            source_extensions: to_vec(DEFAULT_SOURCE_EXTENSIONS),
            excluded_dirs: to_vec(DEFAULT_EXCLUDED_DIRS),
            container_dirs: to_vec(DEFAULT_CONTAINER_DIRS),
            scaffolding_prefixes: to_vec(DEFAULT_SCAFFOLDING_PREFIXES),
        }
    }
}

impl ScanConfig {
    pub fn is_source_ext(&self, ext: &str) -> bool {
        self.source_extensions.iter().any(|e| e == ext)
    }
    pub fn is_excluded_dir(&self, name: &str) -> bool {
        self.excluded_dirs.iter().any(|d| d == name)
    }
    fn is_container(&self, name: &str) -> bool {
        self.container_dirs.iter().any(|d| d == name)
    }
}

fn to_vec(s: &[&str]) -> Vec<String> {
    s.iter().map(|x| x.to_string()).collect()
}

/// Just the `architecture:` section of `.straymark/config.yml`.
#[derive(Debug, Default, Deserialize)]
struct ArchConfigFile {
    #[serde(default)]
    architecture: Option<ArchSection>,
}

#[derive(Debug, Default, Deserialize)]
struct ArchSection {
    #[serde(default)]
    source_extensions: Vec<String>,
    #[serde(default)]
    excluded_dirs: Vec<String>,
    #[serde(default)]
    container_dirs: Vec<String>,
    #[serde(default)]
    scaffolding_prefixes: Vec<String>,
}

/// Resolve the scan config for `root`: built-in defaults, extended (additively)
/// by `.straymark/config.yml`'s `architecture:` section when present. A missing
/// file or section, or a parse error, yields the pure defaults.
pub fn resolve_scan_config(root: &Path) -> ScanConfig {
    let mut cfg = ScanConfig::default();
    let path = root.join(".straymark/config.yml");
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return cfg;
    };
    let Ok(parsed) = serde_yaml::from_str::<ArchConfigFile>(&contents) else {
        return cfg;
    };
    if let Some(section) = parsed.architecture {
        extend_dedup(&mut cfg.source_extensions, section.source_extensions);
        extend_dedup(&mut cfg.excluded_dirs, section.excluded_dirs);
        extend_dedup(&mut cfg.container_dirs, section.container_dirs);
        extend_dedup(&mut cfg.scaffolding_prefixes, section.scaffolding_prefixes);
    }
    cfg
}

fn extend_dedup(base: &mut Vec<String>, extra: Vec<String>) {
    for e in extra {
        let e = e.trim().trim_start_matches("./").trim_end_matches('/').to_string();
        if !e.is_empty() && !base.contains(&e) {
            base.push(e);
        }
    }
}

/// The component directory a source file belongs to, under `cfg`. Resolution:
///
/// 1. **Scaffolding** — if a `scaffolding_prefix` appears as a contiguous run of
///    segments, the component is the path *before* it (multi-module: a module
///    dir that contains `src/main/java/…`); if the prefix is at the start, drop
///    it and continue with the remainder (single module → its top package).
/// 2. **Container descent** — otherwise walk the dir segments, stopping at (and
///    including) the first non-container segment.
///
/// Returns `None` for a root-level file (no directory to attribute it to).
pub fn component_dir_for(rel: &Path, cfg: &ScanConfig) -> Option<String> {
    let segs: Vec<String> = rel
        .parent()?
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    if segs.is_empty() {
        return None;
    }

    // 1. Scaffolding: find the earliest prefix match among the segments.
    for prefix in &cfg.scaffolding_prefixes {
        let parts: Vec<&str> = prefix.split('/').filter(|p| !p.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }
        if let Some(start) = window_pos(&segs, &parts) {
            if start > 0 {
                // Module dir before the scaffolding owns it.
                return Some(segs[..start].join("/"));
            }
            // Scaffolding at the root: descend into the package below it.
            let rest = &segs[parts.len()..];
            return Some(descend(rest, cfg).unwrap_or_else(|| segs[..parts.len()].join("/")));
        }
    }

    // 2. Plain container descent.
    descend(&segs, cfg)
}

/// Walk `segs`, taking segments until (and including) the first non-container.
fn descend(segs: &[String], cfg: &ScanConfig) -> Option<String> {
    if segs.is_empty() {
        return None;
    }
    let mut taken: Vec<&str> = Vec::new();
    for seg in segs {
        taken.push(seg);
        if !cfg.is_container(seg) {
            break;
        }
    }
    Some(taken.join("/"))
}

/// Index where `needle` first appears as a contiguous run inside `hay`.
fn window_pos(hay: &[String], needle: &[&str]) -> Option<usize> {
    if needle.is_empty() || needle.len() > hay.len() {
        return None;
    }
    (0..=hay.len() - needle.len())
        .find(|&i| hay[i..i + needle.len()].iter().zip(needle).all(|(a, b)| a == b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_more_languages_and_recognize_extensions() {
        let cfg = ScanConfig::default();
        for ext in ["rs", "go", "ts", "rb", "ex", "scala", "dart", "kt"] {
            assert!(cfg.is_source_ext(ext), "expected {ext} recognized");
        }
        assert!(!cfg.is_source_ext("txt"));
    }

    #[test]
    fn config_extends_additively_without_losing_defaults() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
        std::fs::write(
            tmp.path().join(".straymark/config.yml"),
            "language: en\narchitecture:\n  source_extensions: [erl]\n  container_dirs: [services]\n",
        )
        .unwrap();
        let cfg = resolve_scan_config(tmp.path());
        assert!(cfg.is_source_ext("erl"), "config extension added");
        assert!(cfg.is_source_ext("rs"), "default extension kept");
        assert!(cfg.container_dirs.iter().any(|d| d == "services"));
        assert!(cfg.container_dirs.iter().any(|d| d == "internal"));
    }

    #[test]
    fn missing_config_yields_defaults() {
        let tmp = tempfile::TempDir::new().unwrap();
        let cfg = resolve_scan_config(tmp.path());
        assert_eq!(cfg.source_extensions, ScanConfig::default().source_extensions);
    }

    #[test]
    fn container_descent_unchanged_for_go_rust_js() {
        let cfg = ScanConfig::default();
        let cases = [
            ("internal/modules/agents/runner.go", Some("internal/modules/agents")),
            ("internal/core/bus.go", Some("internal/core")),
            ("cmd/server/main.go", Some("cmd")),
            ("db/seeds/seeds.go", Some("db")),
            ("main.go", None),
        ];
        for (p, want) in cases {
            assert_eq!(component_dir_for(Path::new(p), &cfg).as_deref(), want, "{p}");
        }
    }

    #[test]
    fn maven_multimodule_attributes_to_the_module_dir() {
        // The collapse-to-`main` case (#279): a module dir holding src/main/java
        // becomes the component, so a multi-module build paints many boxes.
        let cfg = ScanConfig::default();
        assert_eq!(
            component_dir_for(Path::new("billing/src/main/java/com/acme/Svc.java"), &cfg).as_deref(),
            Some("billing"),
        );
        assert_eq!(
            component_dir_for(Path::new("users/src/main/kotlin/com/acme/User.kt"), &cfg).as_deref(),
            Some("users"),
        );
    }

    #[test]
    fn maven_singlemodule_descends_into_the_top_package_not_main() {
        // Single module at the root: no module dir, so seed the top package
        // segment instead of a meaningless `src/main` / `main` box.
        let cfg = ScanConfig::default();
        assert_eq!(
            component_dir_for(Path::new("src/main/java/com/acme/billing/Svc.java"), &cfg).as_deref(),
            Some("com"),
        );
    }
}
