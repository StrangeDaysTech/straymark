use std::collections::{HashMap, HashSet, VecDeque};

use chrono::NaiveDate;
use serde::Serialize;

use crate::compliance::{self, ComplianceReport};
use crate::document::StrayMarkDocument;

/// A single entry in the audit timeline
#[derive(Debug, Clone, Serialize)]
pub struct TimelineEntry {
    pub date: String,
    pub doc_type: String,
    pub id: String,
    pub title: String,
    pub agent: String,
    pub risk_level: String,
    pub status: String,
}

/// A node in a traceability chain
#[derive(Debug, Clone, Serialize)]
pub struct TraceabilityNode {
    pub id: String,
    pub doc_type: String,
    pub title: String,
}

/// A chain of related documents
#[derive(Debug, Clone, Serialize)]
pub struct TraceabilityChain {
    pub root: TraceabilityNode,
    pub chain: Vec<TraceabilityNode>,
}

/// Complete audit report
#[derive(Debug, Serialize)]
pub struct AuditReport {
    pub period_start: String,
    pub period_end: String,
    pub system_filter: Option<String>,
    pub timeline: Vec<TimelineEntry>,
    pub traceability_chains: Vec<TraceabilityChain>,
    pub risk_distribution: Vec<(String, usize)>,
    pub compliance_summary: Vec<ComplianceReport>,
    pub total_docs: usize,
}

/// Parse a date string in YYYY-MM-DD format
fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Filter documents by optional date range
fn filter_by_dates<'a>(
    docs: &'a [StrayMarkDocument],
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Vec<&'a StrayMarkDocument> {
    docs.iter()
        .filter(|d| {
            let date = d.frontmatter.created.as_deref().and_then(parse_date);
            match (date, from, to) {
                (Some(d), Some(f), Some(t)) => d >= f && d <= t,
                (Some(d), Some(f), None) => d >= f,
                (Some(d), None, Some(t)) => d <= t,
                (_, None, None) => true,
                (None, _, _) => true, // include docs without dates
            }
        })
        .collect()
}

/// Filter documents by system name (matches tags or title, case-insensitive)
fn filter_by_system<'a>(
    docs: Vec<&'a StrayMarkDocument>,
    system: &str,
) -> Vec<&'a StrayMarkDocument> {
    let needle = system.to_lowercase();
    docs.into_iter()
        .filter(|d| {
            let title_match = d
                .frontmatter
                .title
                .as_deref()
                .is_some_and(|t| t.to_lowercase().contains(&needle));
            let tag_match = d.frontmatter.tags.as_ref().is_some_and(|tags| {
                tags.iter()
                    .any(|tag| tag.to_lowercase().contains(&needle))
            });
            title_match || tag_match
        })
        .collect()
}

/// Build the chronological timeline from documents
fn build_timeline(docs: &[&StrayMarkDocument]) -> Vec<TimelineEntry> {
    let mut entries: Vec<TimelineEntry> = docs
        .iter()
        .map(|d| TimelineEntry {
            date: d
                .frontmatter
                .created
                .clone()
                .unwrap_or_else(|| "unknown".into()),
            doc_type: d.doc_type.prefix().to_string(),
            id: d
                .frontmatter
                .id
                .clone()
                .unwrap_or_else(|| d.filename.clone()),
            title: d
                .frontmatter
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".into()),
            agent: d
                .frontmatter
                .agent
                .clone()
                .unwrap_or_else(|| "unknown".into()),
            risk_level: d
                .frontmatter
                .risk_level
                .clone()
                .unwrap_or_else(|| "unset".into()),
            status: d
                .frontmatter
                .status
                .clone()
                .unwrap_or_else(|| "unknown".into()),
        })
        .collect();
    entries.sort_by(|a, b| a.date.cmp(&b.date));
    entries
}

/// Build traceability chains from document relationships
fn build_traceability(docs: &[&StrayMarkDocument]) -> Vec<TraceabilityChain> {
    // Build lookup by ID and by filename stem
    let mut doc_by_id: HashMap<String, &StrayMarkDocument> = HashMap::new();
    let mut referenced_ids: HashSet<String> = HashSet::new();

    for doc in docs {
        if let Some(id) = &doc.frontmatter.id {
            doc_by_id.insert(id.clone(), doc);
        }
    }

    // Build adjacency: id -> list of ids it references
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for doc in docs {
        if let Some(id) = &doc.frontmatter.id {
            if let Some(related) = &doc.frontmatter.related {
                let refs: Vec<String> = related
                    .iter()
                    .filter(|r| doc_by_id.contains_key(r.as_str()))
                    .cloned()
                    .collect();
                for r in &refs {
                    referenced_ids.insert(r.clone());
                }
                if !refs.is_empty() {
                    adjacency.insert(id.clone(), refs);
                }
            }
        }
    }

    // If no relationships exist, return empty
    if adjacency.is_empty() {
        return vec![];
    }

    // Find root nodes: documents that are not referenced by others
    let root_ids: Vec<String> = docs
        .iter()
        .filter_map(|d| d.frontmatter.id.clone())
        .filter(|id| !referenced_ids.contains(id))
        .filter(|id| adjacency.contains_key(id))
        .collect();

    // BFS from each root to build chains
    let mut chains = Vec::new();
    let mut globally_visited: HashSet<String> = HashSet::new();

    for root_id in &root_ids {
        if globally_visited.contains(root_id) {
            continue;
        }

        let root_doc = match doc_by_id.get(root_id.as_str()) {
            Some(d) => d,
            None => continue,
        };

        let root_node = TraceabilityNode {
            id: root_id.clone(),
            doc_type: root_doc.doc_type.prefix().to_string(),
            title: root_doc
                .frontmatter
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".into()),
        };

        let mut chain_nodes = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        visited.insert(root_id.clone());
        globally_visited.insert(root_id.clone());

        if let Some(refs) = adjacency.get(root_id) {
            for r in refs {
                if !visited.contains(r) {
                    queue.push_back(r.clone());
                }
            }
        }

        while let Some(current_id) = queue.pop_front() {
            if visited.contains(&current_id) {
                continue;
            }
            visited.insert(current_id.clone());
            globally_visited.insert(current_id.clone());

            if let Some(doc) = doc_by_id.get(current_id.as_str()) {
                chain_nodes.push(TraceabilityNode {
                    id: current_id.clone(),
                    doc_type: doc.doc_type.prefix().to_string(),
                    title: doc
                        .frontmatter
                        .title
                        .clone()
                        .unwrap_or_else(|| "Untitled".into()),
                });

                if let Some(refs) = adjacency.get(&current_id) {
                    for r in refs {
                        if !visited.contains(r) {
                            queue.push_back(r.clone());
                        }
                    }
                }
            }
        }

        if !chain_nodes.is_empty() {
            chains.push(TraceabilityChain {
                root: root_node,
                chain: chain_nodes,
            });
        }
    }

    chains
}

/// Calculate risk distribution
fn risk_distribution(docs: &[&StrayMarkDocument]) -> Vec<(String, usize)> {
    let levels = ["low", "medium", "high", "critical"];
    levels
        .iter()
        .map(|level| {
            let count = docs
                .iter()
                .filter(|d| d.frontmatter.risk_level.as_deref() == Some(level))
                .count();
            (level.to_string(), count)
        })
        .collect()
}

/// Generate a complete audit report.
/// `from` and `to` are optional date bounds.
/// `system` filters by system/component name.
pub fn generate_audit(
    docs: &[StrayMarkDocument],
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    system: Option<&str>,
    straymark_dir: &std::path::Path,
) -> AuditReport {
    // Apply filters
    let mut filtered = filter_by_dates(docs, from, to);
    if let Some(sys) = system {
        filtered = filter_by_system(filtered, sys);
    }

    let total_docs = filtered.len();
    let timeline = build_timeline(&filtered);
    let traceability_chains = build_traceability(&filtered);
    let risk_dist = risk_distribution(&filtered);

    // Compliance summary (run all checkers on full doc set)
    let compliance_summary = vec![
        compliance::check_eu_ai_act(docs, straymark_dir),
        compliance::check_iso_42001(docs, straymark_dir),
        compliance::check_nist_ai_rmf(docs, straymark_dir),
    ];

    // Determine period labels
    let dates: Vec<NaiveDate> = filtered
        .iter()
        .filter_map(|d| d.frontmatter.created.as_deref())
        .filter_map(parse_date)
        .collect();

    let period_start = from
        .or_else(|| dates.iter().min().copied())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "N/A".into());

    let period_end = to
        .or_else(|| dates.iter().max().copied())
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "N/A".into());

    AuditReport {
        period_start,
        period_end,
        system_filter: system.map(String::from),
        timeline,
        traceability_chains,
        risk_distribution: risk_dist,
        compliance_summary,
        total_docs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocType, Frontmatter};
    use std::path::PathBuf;

    fn make_doc(filename: &str, doc_type: DocType, fm: Frontmatter) -> StrayMarkDocument {
        StrayMarkDocument {
            path: PathBuf::from(format!(".straymark/test/{}", filename)),
            filename: filename.to_string(),
            doc_type,
            frontmatter: fm,
            body: String::new(),
        }
    }

    #[test]
    fn test_empty_docs() {
        let dir = PathBuf::from("/tmp/test");
        let report = generate_audit(&[], None, None, None, &dir);
        assert_eq!(report.total_docs, 0);
        assert!(report.timeline.is_empty());
        assert!(report.traceability_chains.is_empty());
    }

    #[test]
    fn test_date_filtering() {
        let dir = PathBuf::from("/tmp/test");
        let docs = vec![
            make_doc(
                "AILOG-2026-01-15-001-old.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-01-15-001".into()),
                    created: Some("2026-01-15".into()),
                    ..Default::default()
                },
            ),
            make_doc(
                "AILOG-2026-03-15-001-mid.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-15-001".into()),
                    created: Some("2026-03-15".into()),
                    ..Default::default()
                },
            ),
            make_doc(
                "AILOG-2026-06-01-001-new.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-06-01-001".into()),
                    created: Some("2026-06-01".into()),
                    ..Default::default()
                },
            ),
        ];

        let from = NaiveDate::from_ymd_opt(2026, 3, 1);
        let to = NaiveDate::from_ymd_opt(2026, 3, 31);
        let report = generate_audit(&docs, from, to, None, &dir);
        assert_eq!(report.total_docs, 1);
        assert_eq!(report.timeline[0].id, "AILOG-2026-03-15-001");
    }

    #[test]
    fn test_system_filtering() {
        let dir = PathBuf::from("/tmp/test");
        let docs = vec![
            make_doc(
                "AILOG-2026-03-20-001-auth.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-20-001".into()),
                    created: Some("2026-03-20".into()),
                    tags: Some(vec!["auth-service".into(), "security".into()]),
                    ..Default::default()
                },
            ),
            make_doc(
                "AILOG-2026-03-21-001-pay.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-21-001".into()),
                    created: Some("2026-03-21".into()),
                    tags: Some(vec!["payment".into()]),
                    ..Default::default()
                },
            ),
        ];

        let report = generate_audit(&docs, None, None, Some("auth"), &dir);
        assert_eq!(report.total_docs, 1);
        assert_eq!(report.timeline[0].id, "AILOG-2026-03-20-001");
    }

    #[test]
    fn test_traceability_chain() {
        let dir = PathBuf::from("/tmp/test");
        let docs = vec![
            make_doc(
                "REQ-2026-03-01-001-login.md",
                DocType::Req,
                Frontmatter {
                    id: Some("REQ-2026-03-01-001".into()),
                    created: Some("2026-03-01".into()),
                    title: Some("Login Requirement".into()),
                    related: Some(vec!["ADR-2026-03-02-001".into()]),
                    ..Default::default()
                },
            ),
            make_doc(
                "ADR-2026-03-02-001-jwt.md",
                DocType::Adr,
                Frontmatter {
                    id: Some("ADR-2026-03-02-001".into()),
                    created: Some("2026-03-02".into()),
                    title: Some("Use JWT".into()),
                    related: Some(vec!["AILOG-2026-03-03-001".into()]),
                    ..Default::default()
                },
            ),
            make_doc(
                "AILOG-2026-03-03-001-impl.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-03-001".into()),
                    created: Some("2026-03-03".into()),
                    title: Some("Implement JWT".into()),
                    ..Default::default()
                },
            ),
        ];

        let report = generate_audit(&docs, None, None, None, &dir);
        assert_eq!(report.traceability_chains.len(), 1);
        let chain = &report.traceability_chains[0];
        assert_eq!(chain.root.id, "REQ-2026-03-01-001");
        assert_eq!(chain.chain.len(), 2);
        assert_eq!(chain.chain[0].id, "ADR-2026-03-02-001");
        assert_eq!(chain.chain[1].id, "AILOG-2026-03-03-001");
    }

    #[test]
    fn test_risk_distribution() {
        let dir = PathBuf::from("/tmp/test");
        let docs = vec![
            make_doc(
                "AILOG-2026-03-20-001-a.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-20-001".into()),
                    created: Some("2026-03-20".into()),
                    risk_level: Some("low".into()),
                    ..Default::default()
                },
            ),
            make_doc(
                "ETH-2026-03-21-001-b.md",
                DocType::Eth,
                Frontmatter {
                    id: Some("ETH-2026-03-21-001".into()),
                    created: Some("2026-03-21".into()),
                    risk_level: Some("high".into()),
                    ..Default::default()
                },
            ),
        ];

        let report = generate_audit(&docs, None, None, None, &dir);
        let low = report
            .risk_distribution
            .iter()
            .find(|(l, _)| l == "low")
            .unwrap()
            .1;
        let high = report
            .risk_distribution
            .iter()
            .find(|(l, _)| l == "high")
            .unwrap()
            .1;
        assert_eq!(low, 1);
        assert_eq!(high, 1);
    }

    #[test]
    fn test_timeline_sorted_by_date() {
        let dir = PathBuf::from("/tmp/test");
        let docs = vec![
            make_doc(
                "AILOG-2026-03-25-001-later.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-25-001".into()),
                    created: Some("2026-03-25".into()),
                    ..Default::default()
                },
            ),
            make_doc(
                "AILOG-2026-03-10-001-earlier.md",
                DocType::Ailog,
                Frontmatter {
                    id: Some("AILOG-2026-03-10-001".into()),
                    created: Some("2026-03-10".into()),
                    ..Default::default()
                },
            ),
        ];

        let report = generate_audit(&docs, None, None, None, &dir);
        assert_eq!(report.timeline[0].id, "AILOG-2026-03-10-001");
        assert_eq!(report.timeline[1].id, "AILOG-2026-03-25-001");
    }
}
