//! Mine `.specify/memory/` for intended components.
//!
//! SpecKit's project memory holds the global plan as free-form markdown. The
//! per-module architecture/requirements docs follow a stable naming
//! convention (`Arquitectura - <X>.md`, `Requisitos - <X>.md`, and the English
//! equivalents) that we mine into `IntendedComponent`s. Everything else
//! (constitution, vision, navigation map, drawio) is ignored in Phase 1.
//!
//! Tolerant (R1): naming is the only signal trusted here; the body is
//! free-form, so memory-derived findings stay low-severity downstream.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

/// A component the design intends to exist, mined from project memory.
#[derive(Debug, Clone, Serialize)]
pub struct IntendedComponent {
    /// Slug join key, e.g. `policyengine`, `identity-module`.
    pub id: String,
    /// Human label as written in the filename, e.g. `PolicyEngine`.
    pub label: String,
    pub kind: MemoryKind,
    /// Source filenames (relative to `memory/`) that declared it.
    pub sources: Vec<String>,
    /// Explicit path globs from the memory file's frontmatter (#314). When
    /// present, C1 uses these instead of the slug heuristic and reports at
    /// High confidence instead of Low.
    pub paths: Vec<String>,
}

/// Which memory docs declared a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Architecture,
    Requirements,
    Both,
}

/// Recognized filename prefixes → the doc kind they denote.
const PREFIXES: &[(&str, DocKind)] = &[
    ("arquitectura -", DocKind::Arch),
    ("architecture -", DocKind::Arch),
    ("requisitos -", DocKind::Req),
    ("requirements -", DocKind::Req),
];

#[derive(Clone, Copy)]
enum DocKind {
    Arch,
    Req,
}

/// Mine `memory_dir` for intended components, sorted by id. Missing directory
/// yields an empty list.
pub fn mine(memory_dir: &Path) -> Vec<IntendedComponent> {
    // Accumulate per id so Arquitectura + Requisitos collapse into one `Both`.
    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();

    let mut files: Vec<_> = std::fs::read_dir(memory_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();

    for path in files {
        let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !fname.to_lowercase().ends_with(".md") {
            continue;
        }
        let Some((label, kind)) = classify(fname) else {
            continue;
        };
        let id = slug(&label);
        let paths = read_component_paths(&path);
        let entry = acc.entry(id.clone()).or_insert_with(|| Acc {
            label: label.clone(),
            has_arch: false,
            has_req: false,
            sources: Vec::new(),
            paths: Vec::new(),
        });
        match kind {
            DocKind::Arch => entry.has_arch = true,
            DocKind::Req => entry.has_req = true,
        }
        entry.sources.push(fname.to_string());
        entry.paths.extend(paths);
    }

    acc.into_iter()
        .map(|(id, a)| IntendedComponent {
            id,
            label: a.label,
            kind: match (a.has_arch, a.has_req) {
                (true, true) => MemoryKind::Both,
                (false, true) => MemoryKind::Requirements,
                _ => MemoryKind::Architecture,
            },
            sources: a.sources,
            paths: a.paths,
        })
        .collect()
}

struct Acc {
    label: String,
    has_arch: bool,
    has_req: bool,
    sources: Vec<String>,
    paths: Vec<String>,
}

/// Returns `(label, kind)` if the filename matches a recognized prefix.
fn classify(fname: &str) -> Option<(String, DocKind)> {
    let stem = fname.strip_suffix(".md").or_else(|| fname.strip_suffix(".MD"))?;
    let lower = stem.to_lowercase();
    for (prefix, kind) in PREFIXES {
        if let Some(rest) = lower.strip_prefix(prefix) {
            // recover the original-cased label using the matched length
            let label = stem[stem.len() - rest.len()..].trim().to_string();
            if label.is_empty() {
                return None;
            }
            return Some((label, *kind));
        }
    }
    None
}

/// `PolicyEngine` → `policyengine`; `Identity Module` → `identity-module`.
fn slug(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut prev_dash = false;
    for c in label.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

/// Read the optional `paths:` list from a memory file's YAML frontmatter (#314).
/// Returns an empty vec when absent or unparseable (the heuristic stays the default).
fn read_component_paths(path: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    // Extract frontmatter between --- delimiters.
    let trimmed = content.trim_start();
    let Some(fm_start) = trimmed.strip_prefix("---") else {
        return Vec::new();
    };
    let Some(fm_end) = fm_start.find("\n---") else {
        return Vec::new();
    };
    let fm = &fm_start[..fm_end];
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(fm) else {
        return Vec::new();
    };
    yaml.get("paths")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_matches_spanish_and_english() {
        assert!(matches!(
            classify("Arquitectura - PolicyEngine.md"),
            Some((l, DocKind::Arch)) if l == "PolicyEngine"
        ));
        assert!(matches!(
            classify("Requirements - Identity Module.md"),
            Some((l, DocKind::Req)) if l == "Identity Module"
        ));
        assert!(classify("INDEX.md").is_none());
        assert!(classify("constitution.md").is_none());
    }

    #[test]
    fn slug_normalizes() {
        assert_eq!(slug("PolicyEngine"), "policyengine");
        assert_eq!(slug("Identity Module"), "identity-module");
    }
}
