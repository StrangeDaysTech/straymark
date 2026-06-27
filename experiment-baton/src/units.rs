//! Routable-unit inventory (Baton Phase 2, B1).
//!
//! Reads the work StrayMark *already recorded* in a project — at the four
//! granularities that already exist, **inventing no new vocabulary** (concept
//! §4.3 / charter framing decision #1): Charter, Batch, Follow-up, Task. The
//! dry-run router (B2+) classifies these units and recommends a tier; this module
//! only *enumerates* them and harvests the signals a reader can read directly.
//!
//! Read-only. Charter reading reuses `straymark_core::charter`; the other three
//! are tolerant, line-oriented scans (the `scan.rs` philosophy: no regex,
//! char-boundary-safe over accented prose). A signal a reader cannot see stays
//! `None` — never fabricated (honest inputs → honest classification, B3).

use std::path::{Path, PathBuf};

use serde::Serialize;
use straymark_core::charter::{discover_and_parse, display_title, read_frontmatter_yaml};
use straymark_core::charter_files::parse_files_to_modify;

use crate::intent::SourceRef;

/// Directories never walked for governance artifacts.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    ".docusaurus",
];

/// The granularity of a routable unit — the existing artifact it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Granularity {
    Charter,
    Batch,
    Followup,
    Task,
}

impl Granularity {
    pub const ALL: [Granularity; 4] = [
        Granularity::Charter,
        Granularity::Batch,
        Granularity::Followup,
        Granularity::Task,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Granularity::Charter => "charter",
            Granularity::Batch => "batch",
            Granularity::Followup => "followup",
            Granularity::Task => "task",
        }
    }

    /// Parse a `--granularity` value (`all` → `None`).
    pub fn parse(s: &str) -> Option<Option<Granularity>> {
        match s.trim().to_lowercase().as_str() {
            "all" => Some(None),
            "charter" => Some(Some(Granularity::Charter)),
            "batch" => Some(Some(Granularity::Batch)),
            "followup" | "follow-up" => Some(Some(Granularity::Followup)),
            "task" => Some(Some(Granularity::Task)),
            _ => None,
        }
    }
}

/// One unit of recorded work, keyed and located, with the signals a reader could
/// read directly. The computed signals (complexity, arch state, coherence
/// findings) are folded in by B2; this is the inventory record.
#[derive(Debug, Clone, Serialize)]
pub struct RoutableUnit {
    pub id: String,
    pub granularity: Granularity,
    pub source: SourceRef,
    pub title: String,
    /// Charter human-time estimate (`XS`..`L`), when the unit carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_estimate: Option<String>,
    /// Follow-up bucket (`ready`, …) for `Followup` units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_bucket: Option<String>,
    /// Follow-up severity, when declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followup_severity: Option<String>,
    /// Authored work verb — the authoritative classification signal (#332):
    /// `design` | `implement` | `audit` | `operate`. `None` = undeclared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_verb: Option<String>,
    /// The residual-cognitive-load dimension: `new` | `upstream`. An `implement`
    /// unit that only instruments prior design (`upstream`) is mechanical.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub design_provenance: Option<String>,
    /// Declared file scope (charters: the `Files to modify` paths/globs).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scope_globs: Vec<String>,
}

/// Inventory routable units under `root`, optionally a single granularity
/// (`None` = all). Read-only; deterministic order.
pub fn inventory(root: &Path, only: Option<Granularity>) -> Vec<RoutableUnit> {
    let want = |g: Granularity| only.is_none_or(|o| o == g);
    let mut out = Vec::new();
    if want(Granularity::Charter) {
        out.extend(read_charters(root));
    }
    if want(Granularity::Batch) {
        out.extend(read_batches(root));
    }
    if want(Granularity::Followup) {
        out.extend(read_followups(root));
    }
    if want(Granularity::Task) {
        out.extend(read_tasks(root));
    }
    out.sort_by(|a, b| {
        (a.granularity.as_str(), a.id.as_str()).cmp(&(b.granularity.as_str(), b.id.as_str()))
    });
    out
}

// ---- Charter --------------------------------------------------------------

fn read_charters(root: &Path) -> Vec<RoutableUnit> {
    let (charters, _errs) = discover_and_parse(root);
    charters
        .iter()
        .map(|c| {
            let scope_globs = parse_files_to_modify(&c.body)
                .into_iter()
                .map(|d| d.path)
                .collect();
            // work_verb / design_provenance are not in the typed CharterFrontmatter;
            // read them from the raw frontmatter (absent → None).
            let (work_verb, design_provenance) = read_frontmatter_yaml(&c.path)
                .ok()
                .map(|y| (yaml_str(&y, "work_verb"), yaml_str(&y, "design_provenance")))
                .unwrap_or((None, None));
            RoutableUnit {
                id: c.frontmatter.charter_id.clone(),
                granularity: Granularity::Charter,
                source: SourceRef {
                    file: rel(root, &c.path),
                    symbol: Some(c.frontmatter.charter_id.clone()),
                },
                title: display_title(c),
                effort_estimate: Some(c.frontmatter.effort_estimate.as_str().to_string()),
                followup_bucket: None,
                followup_severity: None,
                work_verb,
                design_provenance,
                scope_globs,
            }
        })
        .collect()
}

// ---- Batch (from AILOG `## Batch Ledger`) ---------------------------------

fn read_batches(root: &Path) -> Vec<RoutableUnit> {
    let mut out = Vec::new();
    for path in find_files(root, |p| {
        ext_is(p, "md") && file_name(p).starts_with("AILOG-")
    }) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let stem = file_stem(&path);
        let rel_path = rel(root, &path);
        for line in content.lines() {
            // `### Batch 1 — B1: crate scaffold + SpecKit adapter (T1.1–T1.5)`
            let Some(rest) = line.trim().strip_prefix("### Batch ") else {
                continue;
            };
            let (num, title) = split_on_dash(rest);
            let num = num.split_whitespace().next().unwrap_or("").trim();
            if num.is_empty() {
                continue;
            }
            out.push(RoutableUnit {
                id: format!("{stem}#batch-{num}"),
                granularity: Granularity::Batch,
                source: SourceRef {
                    file: rel_path.clone(),
                    symbol: Some(format!("Batch {num}")),
                },
                title: if title.is_empty() { rest.trim().to_string() } else { title },
                effort_estimate: None,
                followup_bucket: None,
                followup_severity: None,
                work_verb: None,
                design_provenance: None,
                scope_globs: Vec::new(),
            });
        }
    }
    out
}

// ---- Follow-up (the registry) ---------------------------------------------

fn read_followups(root: &Path) -> Vec<RoutableUnit> {
    let registry = root.join(".straymark").join("follow-ups-backlog.md");
    let Ok(content) = std::fs::read_to_string(&registry) else {
        return Vec::new();
    };
    let rel_path = rel(root, &registry);
    let mut out = Vec::new();
    let mut bucket: Option<String> = None;

    for line in content.lines() {
        let t = line.trim();
        if let Some(h) = t.strip_prefix("## ") {
            // `## Bucket: ready` → `ready`; any other `## …` is a non-bucket section.
            bucket = Some(
                h.strip_prefix("Bucket:")
                    .unwrap_or(h)
                    .trim()
                    .to_string(),
            );
            continue;
        }
        // `### FU-NNN — <description>`
        if let Some(rest) = t.strip_prefix("### ") {
            let (head, desc) = split_on_dash(rest);
            let Some(id) = head.split_whitespace().next() else {
                continue;
            };
            if !id.starts_with("FU-") {
                continue;
            }
            out.push(RoutableUnit {
                id: id.to_string(),
                granularity: Granularity::Followup,
                source: SourceRef {
                    file: rel_path.clone(),
                    symbol: Some(id.to_string()),
                },
                title: if desc.is_empty() { rest.trim().to_string() } else { desc },
                effort_estimate: None,
                followup_bucket: bucket.clone(),
                followup_severity: None,
                work_verb: None,
                design_provenance: None,
                scope_globs: Vec::new(),
            });
            continue;
        }
        // `- **Label**: value` metadata lines within the current entry.
        if let Some(last) = out.last_mut() {
            if last.granularity == Granularity::Followup {
                if let Some(v) = field_value(t, "**Severity**") {
                    last.followup_severity.get_or_insert(v);
                } else if let Some(v) = field_value(t, "**Work verb**") {
                    last.work_verb.get_or_insert(v);
                } else if let Some(v) = field_value(t, "**Design provenance**") {
                    last.design_provenance.get_or_insert(v);
                }
            }
        }
    }
    out
}

/// Value of a `- **Label**: value` metadata line (trimmed, backtick-stripped),
/// or `None` if the line doesn't carry that label.
fn field_value(line: &str, label: &str) -> Option<String> {
    if !line.contains(label) {
        return None;
    }
    let (_, v) = line.rsplit_once(':')?;
    let v = v.trim().trim_matches('`').trim();
    (!v.is_empty()).then(|| v.to_string())
}

/// Extract a string field from parsed YAML frontmatter.
fn yaml_str(y: &serde_yaml::Value, key: &str) -> Option<String> {
    y.get(key).and_then(|v| v.as_str()).map(str::to_string)
}

// ---- Task (from `specs/**/tasks.md`) --------------------------------------

fn read_tasks(root: &Path) -> Vec<RoutableUnit> {
    let mut out = Vec::new();
    for path in find_files(root, |p| file_name(p) == "tasks.md") {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let spec_id = path
            .parent()
            .and_then(|d| d.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("spec")
            .to_string();
        let rel_path = rel(root, &path);
        for line in content.lines() {
            let t = line.trim();
            let body = t
                .strip_prefix("- [ ] ")
                .or_else(|| t.strip_prefix("- [x] "))
                .or_else(|| t.strip_prefix("- [X] "));
            let Some(body) = body else { continue };
            let (head, text) = split_on_dash(body);
            let Some(tid) = head.split_whitespace().next() else {
                continue;
            };
            // Task ids look like `T1.1` / `T3.5`.
            if !(tid.starts_with('T') && tid[1..].chars().next().is_some_and(|c| c.is_ascii_digit()))
            {
                continue;
            }
            out.push(RoutableUnit {
                id: format!("{spec_id}:{tid}"),
                granularity: Granularity::Task,
                source: SourceRef {
                    file: rel_path.clone(),
                    symbol: Some(tid.to_string()),
                },
                title: if text.is_empty() { body.trim().to_string() } else { text },
                effort_estimate: None,
                followup_bucket: None,
                followup_severity: None,
                work_verb: None,
                design_provenance: None,
                scope_globs: Vec::new(),
            });
        }
    }
    out
}

// ---- shared helpers -------------------------------------------------------

/// Split on the first em-dash (`—`), trimming both sides. Falls back to
/// `(whole, "")` when there is no dash.
fn split_on_dash(s: &str) -> (String, String) {
    match s.split_once('—') {
        Some((l, r)) => (l.trim().to_string(), r.trim().to_string()),
        None => (s.trim().to_string(), String::new()),
    }
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn file_name(p: &Path) -> &str {
    p.file_name().and_then(|n| n.to_str()).unwrap_or_default()
}

fn file_stem(p: &Path) -> String {
    p.file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string()
}

fn ext_is(p: &Path, want: &str) -> bool {
    p.extension().and_then(|e| e.to_str()) == Some(want)
}

/// Recursively collect files under `root` matching `pred`, skipping vendor dirs.
fn find_files(root: &Path, pred: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if !SKIP_DIRS.contains(&file_name(&p)) {
                    stack.push(p);
                }
            } else if pred(&p) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn corpus() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/governance-corpus")
    }

    fn ids(units: &[RoutableUnit], g: Granularity) -> Vec<&str> {
        units
            .iter()
            .filter(|u| u.granularity == g)
            .map(|u| u.id.as_str())
            .collect()
    }

    #[test]
    fn inventory_finds_all_four_granularities() {
        let u = inventory(&corpus(), None);
        assert!(!ids(&u, Granularity::Charter).is_empty(), "expected a charter");
        assert!(!ids(&u, Granularity::Batch).is_empty(), "expected a batch");
        assert!(!ids(&u, Granularity::Followup).is_empty(), "expected a follow-up");
        assert!(!ids(&u, Granularity::Task).is_empty(), "expected a task");
    }

    #[test]
    fn granularity_filter_is_respected() {
        let only = inventory(&corpus(), Some(Granularity::Followup));
        assert!(!only.is_empty());
        assert!(only.iter().all(|u| u.granularity == Granularity::Followup));
    }

    #[test]
    fn charter_unit_carries_effort_and_scope() {
        let u = inventory(&corpus(), Some(Granularity::Charter));
        let c = u.iter().find(|u| u.id == "CHARTER-01-example").expect("charter");
        assert_eq!(c.effort_estimate.as_deref(), Some("L"));
        assert!(
            c.scope_globs.iter().any(|g| g.contains("statuscenter")),
            "scope globs from Files-to-modify: {:?}",
            c.scope_globs
        );
    }

    #[test]
    fn followup_unit_carries_bucket_and_severity() {
        let u = inventory(&corpus(), Some(Granularity::Followup));
        let fu = u.iter().find(|u| u.id == "FU-201").expect("FU-201");
        assert_eq!(fu.followup_bucket.as_deref(), Some("ready"));
        assert_eq!(fu.followup_severity.as_deref(), Some("high"));
    }

    #[test]
    fn declared_work_verb_is_harvested() {
        let u = inventory(&corpus(), None);
        let c = u.iter().find(|u| u.id == "CHARTER-01-example").expect("charter");
        assert_eq!(c.work_verb.as_deref(), Some("implement"));
        assert_eq!(c.design_provenance.as_deref(), Some("new"));

        let fu = u.iter().find(|u| u.id == "FU-201").expect("FU-201");
        assert_eq!(fu.work_verb.as_deref(), Some("implement"));
        assert_eq!(fu.design_provenance.as_deref(), Some("upstream"));

        // Batches/tasks have no declaration slot in the prototype → undeclared.
        assert!(u.iter().filter(|u| u.granularity == Granularity::Task).all(|u| u.work_verb.is_none()));
    }

    #[test]
    fn batch_and_task_ids_are_keyed() {
        let u = inventory(&corpus(), None);
        assert!(
            ids(&u, Granularity::Batch).iter().any(|i| i.contains("#batch-1")),
            "batch id keyed by ailog + number"
        );
        assert!(
            ids(&u, Granularity::Task).iter().any(|i| i.ends_with(":T1.1")),
            "task id keyed by spec dir + task token"
        );
    }
}
