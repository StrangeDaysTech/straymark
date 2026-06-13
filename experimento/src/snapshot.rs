//! The in-memory state Loom serves: the core graph plus the precomputed,
//! serializable API view (Spec 001 §3–§4). Rebuilt on every settled
//! filesystem change.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use straymark_core::document::{discover_documents, parse_document, StrayMarkDocument};
use straymark_core::graph::{Edge, Graph, Node};

/// How much of a document body `/api/node/:id` returns.
const EXCERPT_CHARS: usize = 600;

/// An edge as served by the API: the core edge plus its stable index id
/// (thread highlighting addresses edges by this id).
#[derive(Debug, Clone, Serialize)]
pub struct ApiEdge {
    pub id: usize,
    #[serde(flatten)]
    pub edge: Edge,
}

/// Corpus statistics (Spec 001 §4 `/api/stats`; data required at M1 by
/// acceptance criterion 6 even though the UI panel ships in M2).
#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub total_docs: usize,
    pub total_edges: usize,
    pub by_type: BTreeMap<String, usize>,
    pub by_status: BTreeMap<String, usize>,
    pub by_risk: BTreeMap<String, usize>,
    pub orphans: Vec<String>,
    pub dangling_references: Vec<ApiEdge>,
}

/// What `/api/graph` returns (and what `rebuild` events push over the WS).
#[derive(Debug, Clone, Serialize)]
pub struct ApiGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<ApiEdge>,
    pub stats: Stats,
}

/// Server-side `/api/graph` filters (Spec 001 FR9/S6). All populated fields
/// are combined with AND; date bounds are inclusive.
#[derive(Debug, Default, Deserialize)]
pub struct GraphFilters {
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    pub status: Option<String>,
    pub risk: Option<String>,
    pub tag: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

impl GraphFilters {
    pub fn is_empty(&self) -> bool {
        self.doc_type.is_none()
            && self.status.is_none()
            && self.risk.is_none()
            && self.tag.is_none()
            && self.from.is_none()
            && self.to.is_none()
    }

    fn matches(&self, node: &Node) -> bool {
        matches_value(&self.doc_type, &node.doc_type)
            && matches_value(&self.status, &node.status)
            && matches_value(&self.risk, &node.risk_level)
            && self.tag.as_ref().is_none_or(|tag| {
                node.tags
                    .iter()
                    .any(|candidate| candidate.eq_ignore_ascii_case(tag))
            })
            && self
                .from
                .as_ref()
                .is_none_or(|from| node.created.as_ref().is_some_and(|created| created >= from))
            && self
                .to
                .as_ref()
                .is_none_or(|to| node.created.as_ref().is_some_and(|created| created <= to))
    }
}

fn matches_value(filter: &Option<String>, value: &str) -> bool {
    filter
        .as_ref()
        .is_none_or(|wanted| value.eq_ignore_ascii_case(wanted))
}

impl ApiGraph {
    /// Return the induced graph for matching nodes. Dangling references from
    /// matching source nodes remain visible; resolved edges require both
    /// endpoints to survive the filter.
    pub fn filtered(&self, filters: &GraphFilters) -> ApiGraph {
        if filters.is_empty() {
            return self.clone();
        }

        let nodes: Vec<Node> = self
            .nodes
            .iter()
            .filter(|node| filters.matches(node))
            .cloned()
            .collect();
        let ids: std::collections::HashSet<&str> =
            nodes.iter().map(|node| node.id.as_str()).collect();
        let edges: Vec<ApiEdge> = self
            .edges
            .iter()
            .filter(|edge| {
                ids.contains(edge.edge.source.as_str())
                    && (!edge.edge.resolved || ids.contains(edge.edge.target.as_str()))
            })
            .cloned()
            .collect();
        let stats = build_stats(&nodes, &edges);

        ApiGraph {
            nodes,
            edges,
            stats,
        }
    }
}

/// One rebuilt view of the corpus.
pub struct Snapshot {
    /// The core graph (kept for thread queries).
    pub graph: Graph,
    /// Precomputed API view.
    pub api: ApiGraph,
    /// id → body excerpt, for the node detail endpoint.
    pub excerpts: BTreeMap<String, String>,
    /// id → whether the body continues beyond the returned excerpt.
    pub excerpt_truncated: BTreeMap<String, bool>,
}

impl Snapshot {
    /// Discover, parse, and build — the same code path as the CLI (FR1/NFR1).
    pub fn build(watch_dir: &Path) -> Result<Snapshot> {
        let paths = discover_documents(watch_dir);
        // Unparseable files are skipped, never fatal: a half-saved document
        // must not take the dashboard down mid-edit.
        let docs: Vec<StrayMarkDocument> = paths
            .iter()
            .filter_map(|p| parse_document(p).ok())
            .collect();
        let doc_refs: Vec<&StrayMarkDocument> = docs.iter().collect();
        let graph = Graph::build(&doc_refs);

        let mut excerpts = BTreeMap::new();
        let mut excerpt_truncated = BTreeMap::new();
        for (node, doc) in graph.nodes.iter().zip(docs.iter()) {
            let body = doc.body.trim();
            let excerpt: String = body.chars().take(EXCERPT_CHARS).collect();
            excerpts.insert(node.id.clone(), excerpt);
            excerpt_truncated.insert(node.id.clone(), body.chars().count() > EXCERPT_CHARS);
        }

        let api = build_api_view(&graph);
        Ok(Snapshot {
            graph,
            api,
            excerpts,
            excerpt_truncated,
        })
    }

    /// The serialized WS rebuild event.
    pub fn rebuild_event(&self) -> String {
        serde_json::json!({ "event": "rebuild", "graph": &self.api }).to_string()
    }
}

fn build_api_view(graph: &Graph) -> ApiGraph {
    let edges: Vec<ApiEdge> = graph
        .edges
        .iter()
        .enumerate()
        .map(|(id, edge)| ApiEdge {
            id,
            edge: edge.clone(),
        })
        .collect();

    let stats = build_stats(&graph.nodes, &edges);

    ApiGraph {
        nodes: graph.nodes.clone(),
        edges,
        stats,
    }
}

fn build_stats(nodes: &[Node], edges: &[ApiEdge]) -> Stats {
    let mut by_type = BTreeMap::new();
    let mut by_status = BTreeMap::new();
    let mut by_risk = BTreeMap::new();
    for node in nodes {
        *by_type.entry(node.doc_type.clone()).or_insert(0) += 1;
        *by_status.entry(node.status.clone()).or_insert(0) += 1;
        *by_risk.entry(node.risk_level.clone()).or_insert(0) += 1;
    }

    Stats {
        total_docs: nodes.len(),
        total_edges: edges.len(),
        by_type,
        by_status,
        by_risk,
        orphans: nodes
            .iter()
            .filter(|node| node.is_orphan())
            .map(|node| node.id.clone())
            .collect(),
        dangling_references: edges.iter().filter(|e| !e.edge.resolved).cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn test_snapshot_from_fixture_corpus() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("01-requirements/REQ-2026-03-01-001-login.md"),
            "---\nid: REQ-2026-03-01-001\ntitle: Login\nstatus: approved\ncreated: 2026-03-01\nrisk_level: medium\nrelated: [ADR-2026-03-02-001, MISSING-1]\n---\n# Login body\n",
        );
        write(
            &root.join("02-design/decisions/ADR-2026-03-02-001-jwt.md"),
            "---\nid: ADR-2026-03-02-001\ntitle: JWT\nstatus: accepted\ncreated: 2026-03-02\nrisk_level: high\n---\n# JWT body\n",
        );
        write(
            &root.join("06-evolution/technical-debt/TDE-2026-04-01-001-orphan.md"),
            "---\nid: TDE-2026-04-01-001\ntitle: Orphan\nstatus: open\ncreated: 2026-04-01\n---\n# Debt\n",
        );

        let snap = Snapshot::build(root).unwrap();
        assert_eq!(snap.api.stats.total_docs, 3);
        assert_eq!(snap.api.stats.total_edges, 2);
        assert_eq!(snap.api.stats.orphans, vec!["TDE-2026-04-01-001"]);
        assert_eq!(snap.api.stats.dangling_references.len(), 1);
        assert_eq!(snap.api.stats.by_type.get("REQ"), Some(&1));
        assert!(snap.excerpts["ADR-2026-03-02-001"].contains("JWT body"));
        assert!(!snap.excerpt_truncated["ADR-2026-03-02-001"]);

        // Edge ids are stable indices into graph.edges.
        assert_eq!(snap.api.edges[0].id, 0);
        assert_eq!(snap.api.edges[1].id, 1);

        // Rebuild event is valid JSON with the expected envelope.
        let event: serde_json::Value = serde_json::from_str(&snap.rebuild_event()).unwrap();
        assert_eq!(event["event"], "rebuild");
        assert_eq!(event["graph"]["stats"]["total_docs"], 3);
    }

    #[test]
    fn test_unparseable_file_skipped_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("01-requirements/REQ-2026-03-01-001-ok.md"),
            "---\nid: REQ-2026-03-01-001\ntitle: Ok\n---\nbody\n",
        );
        // Dated, typed filename but broken frontmatter (mid-save state).
        write(
            &root.join("01-requirements/REQ-2026-03-02-001-broken.md"),
            "---\nid: [unclosed\n",
        );
        let snap = Snapshot::build(root).unwrap();
        assert_eq!(snap.api.stats.total_docs, 1);
    }

    #[test]
    fn test_long_body_marks_excerpt_as_truncated() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("ADR-2026-03-02-001-long.md"),
            &format!(
                "---\nid: ADR-LONG\ntitle: Long body\n---\n{}",
                "a".repeat(EXCERPT_CHARS + 1)
            ),
        );

        let snap = Snapshot::build(root).unwrap();
        assert_eq!(snap.excerpts["ADR-LONG"].chars().count(), EXCERPT_CHARS);
        assert!(snap.excerpt_truncated["ADR-LONG"]);
    }

    #[test]
    fn test_filtered_graph_combines_metadata_and_date_filters() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("REQ-2026-03-01-001-login.md"),
            "---\nid: REQ-1\ntitle: Login\nstatus: approved\ncreated: 2026-03-01\nrisk_level: high\ntags: [auth]\nrelated: [ADR-1, MISSING]\n---\nbody\n",
        );
        write(
            &root.join("ADR-2026-03-02-001-jwt.md"),
            "---\nid: ADR-1\ntitle: JWT\nstatus: accepted\ncreated: 2026-03-02\nrisk_level: high\ntags: [auth]\n---\nbody\n",
        );
        write(
            &root.join("ADR-2026-04-02-002-other.md"),
            "---\nid: ADR-2\ntitle: Other\nstatus: draft\ncreated: 2026-04-02\nrisk_level: low\n---\nbody\n",
        );
        let snap = Snapshot::build(root).unwrap();

        let filtered = snap.api.filtered(&GraphFilters {
            doc_type: Some("adr".into()),
            risk: Some("HIGH".into()),
            tag: Some("auth".into()),
            from: Some("2026-03-01".into()),
            to: Some("2026-03-31".into()),
            ..GraphFilters::default()
        });

        assert_eq!(
            filtered
                .nodes
                .iter()
                .map(|n| n.id.as_str())
                .collect::<Vec<_>>(),
            vec!["ADR-1"]
        );
        assert!(filtered.edges.is_empty());
        assert_eq!(filtered.stats.by_type.get("ADR"), Some(&1));
        assert_eq!(filtered.stats.total_docs, 1);
    }

    #[test]
    fn test_filtered_graph_keeps_dangling_edges_from_matching_sources() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("REQ-2026-03-01-001-login.md"),
            "---\nid: REQ-1\ntitle: Login\nrelated: [ADR-1, MISSING]\n---\nbody\n",
        );
        write(
            &root.join("ADR-2026-03-02-001-jwt.md"),
            "---\nid: ADR-1\ntitle: JWT\n---\nbody\n",
        );
        let snap = Snapshot::build(root).unwrap();

        let filtered = snap.api.filtered(&GraphFilters {
            doc_type: Some("REQ".into()),
            ..GraphFilters::default()
        });

        assert_eq!(filtered.nodes.len(), 1);
        assert_eq!(filtered.edges.len(), 1);
        assert!(!filtered.edges[0].edge.resolved);
        assert_eq!(filtered.stats.dangling_references.len(), 1);
    }
}
