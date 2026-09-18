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
use straymark_core::charter::{
    discover_and_parse, discover_charters, display_title, read_frontmatter_yaml,
};
use straymark_core::charter_files::parse_files_to_modify;

use crate::intent::SourceRef;
use crate::scan::is_nested_checkout;

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

// ---- Parent declarations (#332 inheritance) --------------------------------

/// `(work_verb, design_provenance)` as declared in a frontmatter or ledger line.
type Declaration = (Option<String>, Option<String>);

/// A Charter as a candidate parent: where its `originating_spec` resolves
/// (canonical, `None` if absent or unresolvable), which AILOG holds its batch
/// ledger, and what it declares.
struct CharterOrigin {
    spec: Option<PathBuf>,
    /// Key of the AILOG `straymark charter batch-complete` writes this Charter's
    /// ledger into: `originating_ailogs[0]`, else `execution_ailogs[0]`.
    ledger_ailog: Option<String>,
    declaration: Declaration,
}

/// Every Charter's origin, read from the raw frontmatter so a typed-schema
/// problem (a bad enum, a missing required field) does not hide a parent.
/// `Err` lists the Charters whose parent links cannot be read at all: any of
/// them could be a competing parent, so the parent inventory is incomplete.
fn charter_origins(root: &Path) -> Result<Vec<CharterOrigin>, Vec<String>> {
    let mut origins = Vec::new();
    let mut unreadable = Vec::new();
    for path in discover_charters(root) {
        let Ok(yaml) = read_frontmatter_yaml(&path) else {
            unreadable.push(rel(root, &path));
            continue;
        };
        let spec = match yaml.get("originating_spec") {
            None => None,
            Some(v) => match v.as_str() {
                Some(origin) => root.join(origin).canonicalize().ok(),
                None => {
                    unreadable.push(rel(root, &path));
                    continue;
                }
            },
        };
        let Ok(ledger_ailog) = ledger_ailog(&yaml) else {
            unreadable.push(rel(root, &path));
            continue;
        };
        origins.push(CharterOrigin {
            spec,
            ledger_ailog,
            declaration: (
                yaml_str(&yaml, "work_verb"),
                yaml_str(&yaml, "design_provenance"),
            ),
        });
    }
    if unreadable.is_empty() {
        Ok(origins)
    } else {
        Err(unreadable)
    }
}

/// The Charter's ledger AILOG, resolved exactly as `charter batch-complete`
/// does. `Err` = a link field present but not a list of ids.
fn ledger_ailog(yaml: &serde_yaml::Value) -> Result<Option<String>, ()> {
    for key in ["originating_ailogs", "execution_ailogs"] {
        let Some(value) = yaml.get(key) else {
            continue;
        };
        let ids = value
            .as_sequence()
            .ok_or(())?
            .iter()
            .map(|id| id.as_str().ok_or(()))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(first) = ids.first() {
            return Ok(ailog_key(first.trim()));
        }
    }
    Ok(None)
}

/// `AILOG-YYYY-MM-DD-NNN[x]`: the first five `-` segments, which is how
/// `straymark` resolves an AILOG id to its file. A slugged id and a filename
/// stem therefore compare equal, while `…-028` and `…-028b` do not.
fn ailog_key(s: &str) -> Option<String> {
    s.starts_with("AILOG-")
        .then(|| s.split('-').take(5).collect::<Vec<_>>().join("-"))
}

/// Charters whose unreadable frontmatter disables inheritance from Charters
/// for the whole inventory (empty = inheritance active). Lets the CLI say *why*
/// tasks and batches stay undeclared instead of only nudging to declare a verb.
pub fn inheritance_blockers(root: &Path) -> Vec<String> {
    charter_origins(root).err().unwrap_or_default()
}

/// The declaration every candidate parent agrees on. No artifact says which
/// units each parent owns, so several parents must all declare the same thing:
/// one that differs, or declares nothing, makes the unit ambiguous.
fn agreed<'a>(mut parents: impl Iterator<Item = &'a Declaration>) -> Option<Declaration> {
    let first = parents.next()?;
    if parents.any(|d| d != first) {
        return None;
    }
    Some(first.clone())
}

// ---- Batch (from AILOG `## Batch Ledger`) ---------------------------------

/// The ratified placement (#332 §3, #428): a batch declares on its own ledger
/// line only when it differs from its parent — the AILOG's frontmatter, else the
/// Charter(s) whose ledger this AILOG is.
fn read_batches(root: &Path) -> Vec<RoutableUnit> {
    let mut out = Vec::new();
    // An unreadable Charter could be a competing parent (see `read_tasks`).
    let origins = charter_origins(root).unwrap_or_default();
    for path in find_files(root, |p| {
        ext_is(p, "md") && file_name(p).starts_with("AILOG-")
    }) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let stem = file_stem(&path);
        let rel_path = rel(root, &path);
        let parent = batch_parent(&path, &stem, &origins);
        let first = out.len();
        // Per-batch `Work verb` / `Design provenance` lines, in batch order.
        let mut own: Vec<Declaration> = Vec::new();
        let mut in_comment = false;
        let mut in_fence = false;
        let mut in_batch = false;
        for line in content.lines() {
            let visible = outside_html_comments(line, &mut in_comment);
            let t = visible.trim();
            if t.starts_with("```") || t.starts_with("~~~") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            if is_heading(t) {
                in_batch = false;
                // `### Batch 1 — B1: crate scaffold + SpecKit adapter (T1.1–T1.5)`
                let Some(rest) = t.strip_prefix("### Batch ") else {
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
                own.push((None, None));
                in_batch = true;
                continue;
            }
            if !in_batch {
                continue;
            }
            let Some(declared) = own.last_mut() else {
                continue;
            };
            if let Some(v) = field_value(t, "**Work verb**") {
                declared.0.get_or_insert(v);
            } else if let Some(v) = field_value(t, "**Design provenance**") {
                declared.1.get_or_insert(v);
            }
        }
        // A batch's own `Work verb` line overrides its parent as a whole
        // declaration; a provenance line alone qualifies nothing.
        for (unit, declared) in out[first..].iter_mut().zip(own) {
            let (verb, provenance) = if declared.0.is_some() {
                declared
            } else {
                parent.clone()
            };
            unit.work_verb = verb;
            unit.design_provenance = provenance;
        }
    }
    out
}

/// What a ledger's batches inherit: the AILOG's own frontmatter declaration,
/// else the Charter(s) whose ledger this AILOG is, when they all agree.
fn batch_parent(path: &Path, stem: &str, origins: &[CharterOrigin]) -> Declaration {
    match read_frontmatter_yaml(path) {
        Ok(yaml) => {
            let own = (
                yaml_str(&yaml, "work_verb"),
                yaml_str(&yaml, "design_provenance"),
            );
            if own.0.is_some() {
                return own;
            }
        }
        // A frontmatter we cannot read may declare something we cannot see;
        // inheriting past it could route cheaper than the AILOG says.
        Err(_) => return (None, None),
    }
    let Some(key) = ailog_key(stem) else {
        return (None, None);
    };
    agreed(
        origins
            .iter()
            .filter(|o| o.ledger_ailog.as_deref() == Some(key.as_str()))
            .map(|o| &o.declaration),
    )
    .unwrap_or((None, None))
}

/// A markdown ATX heading (`#`…`######` followed by a space).
fn is_heading(line: &str) -> bool {
    line.starts_with('#') && line.trim_start_matches('#').starts_with(' ')
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
    // Only live entries count (#431): the shipped registry documents the entry
    // shape as a commented `### FU-NNN` example, and adopters keep examples in
    // fenced blocks. Metadata lines belong to the entry heading right above
    // them, never to an earlier entry across another heading.
    let mut in_comment = false;
    let mut in_fence = false;
    let mut in_entry = false;

    for line in content.lines() {
        let visible = outside_html_comments(line, &mut in_comment);
        let t = visible.trim();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || t.is_empty() {
            continue;
        }
        if let Some(h) = t.strip_prefix("## ") {
            // `## Bucket: ready` → `ready`; any other `## …` is a non-bucket section.
            bucket = Some(
                h.strip_prefix("Bucket:")
                    .unwrap_or(h)
                    .trim()
                    .to_string(),
            );
            in_entry = false;
            continue;
        }
        // `### FU-NNN — <description>`
        if let Some(rest) = t.strip_prefix("### ") {
            let (head, desc) = split_on_dash(rest);
            let id = head.split_whitespace().next().and_then(followup_id);
            in_entry = id.is_some();
            let Some(id) = id else {
                continue;
            };
            out.push(RoutableUnit {
                id: id.clone(),
                granularity: Granularity::Followup,
                source: SourceRef {
                    file: rel_path.clone(),
                    symbol: Some(id),
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
        if !in_entry {
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

/// Canonical follow-up id of a heading token: `FU-` + digits (`FU-012`, also
/// `FU-012:`), as `straymark followups` reads it. `FU-NNN` and other
/// placeholders are not ids.
fn followup_id(token: &str) -> Option<String> {
    let digits: String = token
        .strip_prefix("FU-")?
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    (!digits.is_empty()).then(|| format!("FU-{digits}"))
}

/// The part of `line` outside HTML comments. `in_comment` carries an open
/// `<!--` across lines, since a comment may span many.
fn outside_html_comments(line: &str, in_comment: &mut bool) -> String {
    let mut out = String::new();
    let mut rest = line;
    loop {
        if *in_comment {
            let Some(end) = rest.find("-->") else {
                return out;
            };
            rest = &rest[end + 3..];
            *in_comment = false;
        } else {
            let Some(start) = rest.find("<!--") else {
                out.push_str(rest);
                return out;
            };
            out.push_str(&rest[..start]);
            rest = &rest[start + 4..];
            *in_comment = true;
        }
    }
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

/// A task has no declaration slot (#332). Inherit only through explicit
/// Charter origins resolving to its sibling spec, never from titles or nearest
/// directories. Several Charters can cover one spec and none says which tasks it
/// owns, so they must all declare the same thing: a spec gaining a Charter that
/// agrees keeps its tasks classified, one that disagrees (or declares nothing)
/// makes them ambiguous.
fn task_declaration(root: &Path, tasks: &Path, origins: &[CharterOrigin]) -> Declaration {
    let resolve = || {
        let root = root.canonicalize().ok()?;
        let spec = tasks.parent()?.join("spec.md").canonicalize().ok()?;
        if !spec.is_file() || !spec.starts_with(&root) {
            return None;
        }
        agreed(
            origins
                .iter()
                .filter(|o| o.spec.as_ref() == Some(&spec))
                .map(|o| &o.declaration),
        )
    };
    resolve().unwrap_or((None, None))
}

fn read_tasks(root: &Path) -> Vec<RoutableUnit> {
    let mut out = Vec::new();
    // An unreadable Charter could be a competing parent. Do not silently turn
    // an incomplete inventory into permission to recommend a cheaper tier.
    let origins = charter_origins(root).unwrap_or_default();
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
        let (work_verb, design_provenance) = task_declaration(root, &path, &origins);
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
                work_verb: work_verb.clone(),
                design_provenance: design_provenance.clone(),
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
                if !SKIP_DIRS.contains(&file_name(&p)) && !is_nested_checkout(&p) {
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

        // This legacy fixture has no sibling spec.md to resolve as a parent.
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
