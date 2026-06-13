//! The in-memory state Loom serves: the core graph plus the precomputed,
//! serializable API view (Spec 001 §3–§4). Rebuilt on every settled
//! filesystem change.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;
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

/// One rebuilt view of the corpus.
pub struct Snapshot {
    /// The core graph (kept for thread queries).
    pub graph: Graph,
    /// Precomputed API view.
    pub api: ApiGraph,
    /// id → body excerpt, for the node detail endpoint.
    pub excerpts: BTreeMap<String, String>,
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
        for (node, doc) in graph.nodes.iter().zip(docs.iter()) {
            let body = doc.body.trim();
            let excerpt: String = body.chars().take(EXCERPT_CHARS).collect();
            excerpts.insert(node.id.clone(), excerpt);
        }

        let api = build_api_view(&graph);
        Ok(Snapshot { graph, api, excerpts })
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
        .map(|(id, edge)| ApiEdge { id, edge: edge.clone() })
        .collect();

    let mut by_type = BTreeMap::new();
    let mut by_status = BTreeMap::new();
    let mut by_risk = BTreeMap::new();
    for node in &graph.nodes {
        *by_type.entry(node.doc_type.clone()).or_insert(0) += 1;
        *by_status.entry(node.status.clone()).or_insert(0) += 1;
        *by_risk.entry(node.risk_level.clone()).or_insert(0) += 1;
    }

    let stats = Stats {
        total_docs: graph.nodes.len(),
        total_edges: graph.edges.len(),
        by_type,
        by_status,
        by_risk,
        orphans: graph.orphans().map(|n| n.id.clone()).collect(),
        dangling_references: edges.iter().filter(|e| !e.edge.resolved).cloned().collect(),
    };

    ApiGraph { nodes: graph.nodes.clone(), edges, stats }
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

        // Edge ids are stable indices into graph.edges.
        assert_eq!(snap.api.edges[0].id, 0);
        assert_eq!(snap.api.edges[1].id, 1);

        // Rebuild event is valid JSON with the expected envelope.
        let event: serde_json::Value =
            serde_json::from_str(&snap.rebuild_event()).unwrap();
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
}
