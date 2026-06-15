use serde::Deserialize;
use std::path::Path;

use straymark_core::charter::{is_charter_filename, parse_charter, CharterFrontmatter};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocStatus {
    Draft,
    Accepted,
    Deprecated,
    Superseded,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for DocStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "DRAFT"),
            Self::Accepted => write!(f, "ACCEPTED"),
            Self::Deprecated => write!(f, "DEPRECATED"),
            Self::Superseded => write!(f, "SUPERSEDED"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DocFrontMatter {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: Option<DocStatus>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub confidence: Option<ConfidenceLevel>,
    #[serde(default)]
    pub review_required: Option<bool>,
    #[serde(default)]
    pub risk_level: Option<RiskLevel>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
    #[serde(default)]
    pub supersedes: Vec<String>,
}

/// Typed frontmatter the TUI knows how to render. Governance documents use the
/// `Doc` variant (DocFrontMatter — status enum draft|accepted|…, plus tags /
/// related / confidence / risk). Charters use the `Charter` variant because
/// their schema (declared|in-progress|closed status, effort_estimate, trigger,
/// originating_ailogs/spec) is disjoint from governance docs — see
/// `straymark_core::charter::CharterFrontmatter`. Dispatch happens in `Document::load`
/// based on the file path so the panel can `match` instead of guessing.
#[derive(Debug, Clone)]
pub enum DocumentMetadata {
    Doc(DocFrontMatter),
    Charter(CharterFrontmatter),
}

#[derive(Debug, Clone)]
pub struct Document {
    pub frontmatter: Option<DocumentMetadata>,
    pub body: String,
    pub filename: String,
}

impl Document {
    pub fn load(path: &Path) -> Option<Self> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if is_charter_path(path) {
            // Charter path → try the charter parser. If the frontmatter is
            // malformed we still want to surface the document so the user can
            // open it and fix the issue from the TUI, so fall back to reading
            // the raw content with `frontmatter = None`.
            match parse_charter(path) {
                Ok(c) => {
                    return Some(Self {
                        frontmatter: Some(DocumentMetadata::Charter(c.frontmatter)),
                        body: c.body,
                        filename,
                    });
                }
                Err(_) => {
                    let content = std::fs::read_to_string(path).ok()?;
                    return Some(Self {
                        frontmatter: None,
                        body: content,
                        filename,
                    });
                }
            }
        }

        let content = std::fs::read_to_string(path).ok()?;
        let (fm, body) = parse_frontmatter(&content);
        Some(Self {
            frontmatter: fm.map(DocumentMetadata::Doc),
            body,
            filename,
        })
    }

    /// Tags surfaced for tag-chip rendering and tag-search navigation. Charters
    /// have no tags in their schema, so this returns an empty slice for them.
    pub fn tags(&self) -> &[String] {
        match &self.frontmatter {
            Some(DocumentMetadata::Doc(fm)) => &fm.tags,
            _ => &[],
        }
    }

    /// Related identifiers shown in the "Related" section and follow-on
    /// navigation. For governance docs this is the explicit `related:` list.
    /// For Charters there is no `related:` field — instead, origin links
    /// (`originating_ailogs` and `originating_spec`) are surfaced here so the
    /// TUI can follow them with the same `MetaSelection::Related` machinery.
    pub fn related(&self) -> Vec<String> {
        match &self.frontmatter {
            Some(DocumentMetadata::Doc(fm)) => fm.related.clone(),
            Some(DocumentMetadata::Charter(fm)) => {
                let mut out = Vec::new();
                if let Some(ailogs) = &fm.originating_ailogs {
                    out.extend(ailogs.iter().cloned());
                }
                if let Some(spec) = &fm.originating_spec {
                    out.push(spec.clone());
                }
                out
            }
            None => Vec::new(),
        }
    }
}

/// True when `path` points to a Charter file under `.straymark/charters/` whose
/// filename matches the `NN-slug.md` Charter pattern. Used to decide which
/// frontmatter schema to apply in `Document::load`. Anything under that
/// directory that isn't a numbered Charter (e.g., the status-board `README.md`
/// adopters maintain by hand) is treated as a regular markdown file with no
/// typed frontmatter.
fn is_charter_path(path: &Path) -> bool {
    if !is_charter_filename(path) {
        return false;
    }
    let mut comps = path.components().rev();
    // Skip the filename itself.
    if comps.next().is_none() {
        return false;
    }
    // Parent directory must be "charters" and grandparent must be ".straymark".
    let parent = comps
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.to_string());
    let grand = comps
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.to_string());
    matches!(parent.as_deref(), Some("charters"))
        && matches!(grand.as_deref(), Some(".straymark"))
}

fn parse_frontmatter(content: &str) -> (Option<DocFrontMatter>, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content.to_string());
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    let closing = after_first.find("\n---");
    match closing {
        Some(pos) => {
            let yaml_str = &after_first[..pos];
            let body_start = pos + 4; // skip \n---
            let body = after_first[body_start..].trim_start_matches('\n').to_string();
            let fm: Option<DocFrontMatter> = serde_yaml::from_str(yaml_str).ok();
            (fm, body)
        }
        None => (None, content.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
id: ADR-2025-06-15-001
title: Test Document
status: accepted
created: 2025-06-15
risk_level: low
tags: [rust, tui]
related: [REQ-2025-06-10-003]
---

# Test Document

Some content here.
"#;
        let (fm, body) = parse_frontmatter(content);
        let fm = fm.unwrap();
        assert_eq!(fm.id, "ADR-2025-06-15-001");
        assert_eq!(fm.status, Some(DocStatus::Accepted));
        assert!(body.starts_with("# Test Document"));
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn load_charter_uses_charter_variant() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join(".straymark").join("charters").join("01-foo.md");
        write_file(
            &p,
            "---\ncharter_id: CHARTER-01-foo\nstatus: in-progress\neffort_estimate: M\ntrigger: \"x\"\n---\n\n# Charter: Foo\n",
        );
        let doc = Document::load(&p).unwrap();
        match doc.frontmatter {
            Some(DocumentMetadata::Charter(fm)) => {
                assert_eq!(fm.charter_id, "CHARTER-01-foo");
            }
            other => panic!("expected Charter variant, got {:?}", other),
        }
    }

    #[test]
    fn load_non_charter_uses_doc_variant() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("00-governance").join("ADR-2025-01-01-001.md");
        write_file(
            &p,
            "---\nid: ADR-2025-01-01-001\ntitle: A Decision\nstatus: accepted\n---\n\n# Body\n",
        );
        let doc = Document::load(&p).unwrap();
        match doc.frontmatter {
            Some(DocumentMetadata::Doc(fm)) => {
                assert_eq!(fm.id, "ADR-2025-01-01-001");
                assert_eq!(fm.status, Some(DocStatus::Accepted));
            }
            other => panic!("expected Doc variant, got {:?}", other),
        }
    }

    #[test]
    fn load_charter_with_broken_frontmatter_falls_back_to_no_frontmatter() {
        // Path looks like a charter but frontmatter is missing required fields.
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join(".straymark").join("charters").join("02-broken.md");
        write_file(&p, "no frontmatter at all\nbody\n");
        let doc = Document::load(&p).unwrap();
        assert!(doc.frontmatter.is_none());
        assert!(doc.body.contains("no frontmatter"));
    }

    #[test]
    fn readme_in_charters_dir_is_not_treated_as_charter() {
        // README.md doesn't match NN-slug.md so it should NOT use the charter
        // parser even though it lives under .straymark/charters/.
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join(".straymark").join("charters").join("README.md");
        write_file(&p, "# Charters status board\n");
        let doc = Document::load(&p).unwrap();
        assert!(doc.frontmatter.is_none());
    }

    #[test]
    fn related_helper_materializes_charter_origin_links() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join(".straymark").join("charters").join("03-origins.md");
        write_file(
            &p,
            "---\ncharter_id: CHARTER-03-origins\nstatus: declared\neffort_estimate: S\ntrigger: \"x\"\noriginating_ailogs:\n  - AILOG-2026-04-28-021\n  - AILOG-2026-04-28-022\noriginating_spec: specs/001/spec.md\n---\n\nBody.\n",
        );
        let doc = Document::load(&p).unwrap();
        let related = doc.related();
        assert_eq!(related.len(), 3);
        assert_eq!(related[0], "AILOG-2026-04-28-021");
        assert_eq!(related[2], "specs/001/spec.md");
        assert!(doc.tags().is_empty());
    }
}
