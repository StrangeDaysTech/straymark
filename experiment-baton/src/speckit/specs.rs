//! Parse the `specs/**` tree: one `ParsedSpec` per `specs/<id>/` directory.
//!
//! Tolerant (FR1): each artifact is optional; absence yields empty fields, not
//! errors. Deterministic (NFR3): directory listings are sorted before parsing.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::scan::{scan_endpoints, scan_ids};

/// A single SpecKit feature spec, as far as Phase 1 mines it.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ParsedSpec {
    /// Directory name, e.g. `005-frontend-dashboard`.
    pub id: String,
    pub title: Option<String>,
    /// `FR-NNN` / `FRNN` requirements found in `spec.md`.
    pub requirements: Vec<Requirement>,
    /// `PM-NNN` decisions found in `post-mvp-backlog.md`.
    pub decisions: Vec<BacklogDecision>,
    /// API endpoints this spec references (a conservative consume hint).
    pub consumes: Vec<ConsumesHint>,
    /// Governance decision ids (`PM-`/`AILOG-`/`AIDEC-`/`ADR-`) cited anywhere in
    /// the spec's own files — used to tell whether a consumer spec acknowledges
    /// the decision that defined a contract it depends on (finding class C4).
    pub referenced_decisions: Vec<String>,
    /// Files under `contracts/`.
    pub contract_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Requirement {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacklogDecision {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    /// Governance documents the decision references (e.g. the AILOG that
    /// recorded it). Phase 2 turns these into provenance edges.
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsumesHint {
    pub endpoint: String,
    /// File the hint was found in, relative to the spec directory.
    pub location: String,
}

/// Parse every `specs/<id>/` directory under `specs_dir`, sorted by id.
pub fn parse_all(specs_dir: &Path) -> Vec<ParsedSpec> {
    let mut dirs = sorted_dirs(specs_dir);
    dirs.retain(|p| p.is_dir());
    dirs.iter().map(|d| parse_spec_dir(d)).collect()
}

fn parse_spec_dir(dir: &Path) -> ParsedSpec {
    let id = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    let spec_md = read(dir.join("spec.md"));
    let backlog_md = read(dir.join("post-mvp-backlog.md"));

    let title = spec_md.as_deref().and_then(first_heading);
    let requirements = spec_md.as_deref().map(parse_requirements).unwrap_or_default();

    let decisions = backlog_md
        .as_deref()
        .map(parse_backlog_decisions)
        .unwrap_or_default();

    let (contract_files, mut consumes) = parse_contracts(&dir.join("contracts"));
    // consume hints also come from the spec body itself.
    if let Some(body) = &spec_md {
        for ep in scan_endpoints(body) {
            consumes.push(ConsumesHint {
                endpoint: ep,
                location: "spec.md".to_string(),
            });
        }
    }
    dedup_consumes(&mut consumes);

    // Decision ids cited anywhere in the spec's own authored files (spec.md +
    // plan.md + tasks.md), so a consumer that acknowledges the defining decision
    // is exempt from C4.
    let mut referenced_decisions = Vec::new();
    let mut seen_dec = HashSet::new();
    for fname in ["spec.md", "plan.md", "tasks.md"] {
        let Some(body) = read(dir.join(fname)) else {
            continue;
        };
        for prefix in ["PM-", "AILOG-", "AIDEC-", "ADR-"] {
            for id in scan_ids(&body, prefix) {
                if seen_dec.insert(id.clone()) {
                    referenced_decisions.push(id);
                }
            }
        }
    }

    ParsedSpec {
        id,
        title,
        requirements,
        decisions,
        consumes,
        referenced_decisions,
        contract_files,
    }
}

/// `FR-NNN` / `FRNN` requirements: first line each id appears on becomes its text.
fn parse_requirements(body: &str) -> Vec<Requirement> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for line in body.lines() {
        for id in scan_ids(line, "FR") {
            if seen.insert(id.clone()) {
                out.push(Requirement {
                    id,
                    text: clean_line(line),
                });
            }
        }
    }
    out
}

/// Parse post-MVP backlog `## ` sections into decisions.
fn parse_backlog_decisions(body: &str) -> Vec<BacklogDecision> {
    let mut out = Vec::new();
    for section in split_sections(body) {
        let ids = scan_ids(&section.body, "PM");
        let Some(id) = ids.into_iter().next() else {
            continue;
        };
        let status = section
            .body
            .lines()
            .find(|l| {
                let low = l.to_lowercase();
                low.contains("estado") || low.contains("status")
            })
            .map(clean_line);
        let references = scan_ids(&section.body, "AILOG-");
        out.push(BacklogDecision {
            id,
            title: section.heading,
            status,
            references,
        });
    }
    out
}

fn parse_contracts(contracts_dir: &Path) -> (Vec<PathBuf>, Vec<ConsumesHint>) {
    let mut files = Vec::new();
    let mut consumes = Vec::new();
    if !contracts_dir.is_dir() {
        return (files, consumes);
    }
    for path in list_files_recursive(contracts_dir) {
        let rel = path
            .strip_prefix(contracts_dir.parent().unwrap_or(contracts_dir))
            .unwrap_or(&path)
            .to_path_buf();
        if let Some(body) = read(path.clone()) {
            let loc = rel.to_string_lossy().to_string();
            for ep in scan_endpoints(&body) {
                consumes.push(ConsumesHint {
                    endpoint: ep,
                    location: loc.clone(),
                });
            }
        }
        files.push(rel);
    }
    files.sort();
    (files, consumes)
}

// ---- small tolerant helpers ----------------------------------------------

struct Section {
    heading: String,
    body: String,
}

/// Split markdown into `## ` (level-2) sections. Text before the first heading
/// is dropped.
fn split_sections(body: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(s) = current.take() {
                sections.push(s);
            }
            current = Some(Section {
                heading: clean_inline(rest),
                body: String::new(),
            });
        } else if let Some(s) = current.as_mut() {
            s.body.push_str(line);
            s.body.push('\n');
        }
    }
    if let Some(s) = current.take() {
        sections.push(s);
    }
    sections
}

fn first_heading(body: &str) -> Option<String> {
    body.lines()
        .find_map(|l| l.strip_prefix("# ").map(clean_inline))
}

fn dedup_consumes(v: &mut Vec<ConsumesHint>) {
    let mut seen = HashSet::new();
    v.retain(|c| seen.insert((c.endpoint.clone(), c.location.clone())));
}

/// Strip markdown bullet/emphasis/heading noise from a single line.
fn clean_line(line: &str) -> String {
    let t = line.trim();
    let t = t
        .trim_start_matches(['-', '*', '#', '>', ' ', '\t'])
        .trim();
    clean_inline(t)
}

fn clean_inline(s: &str) -> String {
    s.replace("**", "").replace('~', "").trim().to_string()
}

fn read(path: PathBuf) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn sorted_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    out.sort();
    out
}

fn list_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&d)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                stack.push(p);
            } else {
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

    #[test]
    fn requirements_capture_id_and_text() {
        let body = "## 6. FR\n- **FR-010** MUST show component health\n- **FR-011** other";
        let reqs = parse_requirements(body);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].id, "FR-010");
        assert!(reqs[0].text.contains("component health"));
    }

    #[test]
    fn backlog_decision_extracts_id_status_and_refs() {
        let body = "## 2. Health per component\n- **ID**: PM-002\n- **Estado**: CERRADO (2026-04-24)\n- Recorded in AILOG-2026-04-24-006";
        let decisions = parse_backlog_decisions(body);
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].id, "PM-002");
        assert!(decisions[0].status.as_deref().unwrap().contains("CERRADO"));
        assert_eq!(decisions[0].references, vec!["AILOG-2026-04-24-006"]);
    }
}
