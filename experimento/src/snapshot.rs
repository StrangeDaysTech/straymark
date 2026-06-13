//! The in-memory state Loom serves: the core graph plus the precomputed,
//! serializable API view (Spec 001 §3–§4). Rebuilt on every settled
//! filesystem change.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use straymark_core::document::{discover_documents, parse_document, StrayMarkDocument};
use straymark_core::graph::{cycles_in, Cycle, Edge, Graph, Node};

/// How much of a document body `/api/node/:id` returns.
const EXCERPT_CHARS: usize = 600;

/// Parsed-document cache keyed by path, used to make rebuilds incremental
/// (Spec 001 NFR2 / T3.1): on a filesystem change only changed files are
/// re-parsed; unchanged files reuse their cached parse. The graph itself is
/// still rebuilt in full from the (mostly cached) document set — that is
/// in-memory and cheap; file IO + frontmatter parsing is the real cost
/// (plan.md §8 R2).
pub type ParseCache = BTreeMap<PathBuf, CachedDoc>;

/// A parsed document plus the modification time it was parsed at.
pub struct CachedDoc {
    mtime: Option<SystemTime>,
    doc: StrayMarkDocument,
}

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
    /// Dependency cycles over the resolved semantic edges (T3.2, spec §3.3).
    pub cycles: Vec<Cycle>,
}

/// What `/api/graph` returns (and what `rebuild` events push over the WS).
#[derive(Debug, Clone, Serialize)]
pub struct ApiGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<ApiEdge>,
    pub stats: Stats,
}

/// An incremental update pushed over the WS (`delta` event, T3.1). Nodes are
/// patched in place so the client preserves layout positions for unchanged
/// documents; edges are sent in full (cheap to redraw, and edge ids are
/// positional so a wholesale replace is the robust choice).
#[derive(Debug, Clone, Serialize)]
pub struct GraphDelta {
    /// Documents new since the previous snapshot.
    pub added: Vec<Node>,
    /// Ids of documents removed since the previous snapshot.
    pub removed: Vec<String>,
    /// Documents whose metadata changed (same id, different fields).
    pub changed: Vec<Node>,
    /// The complete new edge set.
    pub edges: Vec<ApiEdge>,
    /// The recomputed corpus stats.
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

    /// Diff this (new) graph against a previous one, producing the incremental
    /// `delta` payload (T3.1). Nodes are compared by id; a node present in both
    /// but with different fields is `changed`. Edges are carried in full.
    pub fn diff(&self, prev: &ApiGraph) -> GraphDelta {
        let prev_by_id: HashMap<&str, &Node> =
            prev.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let cur_ids: HashSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();

        let mut added = Vec::new();
        let mut changed = Vec::new();
        for node in &self.nodes {
            match prev_by_id.get(node.id.as_str()) {
                None => added.push(node.clone()),
                Some(prev_node) if *prev_node != node => changed.push(node.clone()),
                Some(_) => {}
            }
        }
        let removed: Vec<String> = prev
            .nodes
            .iter()
            .filter(|n| !cur_ids.contains(n.id.as_str()))
            .map(|n| n.id.clone())
            .collect();

        GraphDelta {
            added,
            removed,
            changed,
            edges: self.edges.clone(),
            stats: self.stats.clone(),
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
    /// Cold build (test helper): parses every document. Production always goes
    /// through [`Snapshot::build_cached`] with a persistent cache.
    #[cfg(test)]
    pub fn build(watch_dir: &Path) -> Result<Snapshot> {
        let mut cache = ParseCache::new();
        Snapshot::build_cached(watch_dir, &mut cache)
    }

    /// Incremental build (T3.1/NFR2): re-parse only files whose modification
    /// time changed since they were last cached; reuse the cached parse for
    /// the rest. The graph is still rebuilt in full from the resulting
    /// document set (cheap, in-memory). Unparseable files are skipped, never
    /// fatal — a half-saved document must not take the dashboard down mid-edit.
    pub fn build_cached(watch_dir: &Path, cache: &mut ParseCache) -> Result<Snapshot> {
        let paths = discover_documents(watch_dir);
        let present: HashSet<&Path> = paths.iter().map(|p| p.as_path()).collect();
        // Forget files that no longer exist.
        cache.retain(|p, _| present.contains(p.as_path()));

        for path in &paths {
            let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
            let needs_parse = match cache.get(path) {
                Some(cached) => mtime.is_none() || cached.mtime != mtime,
                None => true,
            };
            if needs_parse {
                match parse_document(path) {
                    Ok(doc) => {
                        cache.insert(path.clone(), CachedDoc { mtime, doc });
                    }
                    // Drop any stale entry so a later valid save is picked up.
                    Err(_) => {
                        cache.remove(path);
                    }
                }
            }
        }

        // Build from the cached docs, in discovery order.
        let doc_refs: Vec<&StrayMarkDocument> =
            paths.iter().filter_map(|p| cache.get(p).map(|c| &c.doc)).collect();
        let graph = Graph::build(&doc_refs);

        let mut excerpts = BTreeMap::new();
        let mut excerpt_truncated = BTreeMap::new();
        for (node, doc) in graph.nodes.iter().zip(doc_refs.iter()) {
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

    /// The serialized WS `rebuild` event (full snapshot — sent on initial
    /// connect).
    pub fn rebuild_event(&self) -> String {
        serde_json::json!({ "event": "rebuild", "graph": &self.api }).to_string()
    }

    /// The serialized WS `delta` event: this snapshot's graph diffed against
    /// `prev` (T3.1). Sent on settled filesystem changes so open clients patch
    /// rather than re-layout.
    pub fn delta_event(&self, prev: &ApiGraph) -> String {
        let mut value = serde_json::to_value(self.api.diff(prev)).unwrap_or_default();
        if let Some(obj) = value.as_object_mut() {
            obj.insert("event".into(), serde_json::json!("delta"));
        }
        value.to_string()
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

    let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let raw_edges: Vec<Edge> = edges.iter().map(|e| e.edge.clone()).collect();

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
        cycles: cycles_in(&node_ids, &raw_edges),
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
    fn test_build_cached_matches_cold_build_and_reuses_parses() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("REQ-2026-03-01-001-login.md"),
            "---\nid: REQ-1\ntitle: Login\nrelated: [ADR-1]\n---\nbody\n",
        );
        write(
            &root.join("ADR-2026-03-02-001-jwt.md"),
            "---\nid: ADR-1\ntitle: JWT\n---\nbody\n",
        );

        let cold = Snapshot::build(root).unwrap();
        let mut cache = ParseCache::new();
        let warm = Snapshot::build_cached(root, &mut cache).unwrap();
        assert_eq!(cache.len(), 2);
        // Same nodes/edges as the cold build.
        assert_eq!(warm.api.nodes.len(), cold.api.nodes.len());
        assert_eq!(warm.api.edges.len(), cold.api.edges.len());

        // A second build with the unchanged cache yields the same graph and a
        // cache of the same size (reuse, no growth).
        let again = Snapshot::build_cached(root, &mut cache).unwrap();
        assert_eq!(cache.len(), 2);
        assert_eq!(again.api.stats.total_docs, 2);

        // Removing a file drops it from the cache on the next build.
        std::fs::remove_file(root.join("ADR-2026-03-02-001-jwt.md")).unwrap();
        let shrunk = Snapshot::build_cached(root, &mut cache).unwrap();
        assert_eq!(cache.len(), 1);
        assert_eq!(shrunk.api.stats.total_docs, 1);
    }

    #[test]
    fn test_diff_detects_added_removed_changed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("REQ-2026-03-01-001-login.md"),
            "---\nid: REQ-1\ntitle: Login\nstatus: draft\n---\nbody\n",
        );
        write(
            &root.join("ADR-2026-03-02-001-jwt.md"),
            "---\nid: ADR-1\ntitle: JWT\n---\nbody\n",
        );
        let prev = Snapshot::build(root).unwrap();

        // Change REQ-1's status, remove ADR-1, add a new TDE.
        write(
            &root.join("REQ-2026-03-01-001-login.md"),
            "---\nid: REQ-1\ntitle: Login\nstatus: approved\n---\nbody\n",
        );
        std::fs::remove_file(root.join("ADR-2026-03-02-001-jwt.md")).unwrap();
        write(
            &root.join("TDE-2026-04-01-001-debt.md"),
            "---\nid: TDE-1\ntitle: Debt\n---\nbody\n",
        );
        let next = Snapshot::build(root).unwrap();

        let delta = next.api.diff(&prev.api);
        assert_eq!(delta.added.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["TDE-1"]);
        assert_eq!(delta.removed, vec!["ADR-1"]);
        assert_eq!(delta.changed.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["REQ-1"]);

        // The delta event is valid JSON tagged `delta`.
        let event: serde_json::Value = serde_json::from_str(&next.delta_event(&prev.api)).unwrap();
        assert_eq!(event["event"], "delta");
        assert_eq!(event["removed"][0], "ADR-1");
    }

    #[test]
    fn test_stats_report_supersession_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join("ADR-2026-03-01-001-a.md"),
            "---\nid: ADR-A\ntitle: A\nsupersedes: [ADR-B]\n---\nbody\n",
        );
        write(
            &root.join("ADR-2026-03-02-001-b.md"),
            "---\nid: ADR-B\ntitle: B\nsupersedes: [ADR-A]\n---\nbody\n",
        );
        let snap = Snapshot::build(root).unwrap();
        assert_eq!(snap.api.stats.cycles.len(), 1);
        assert_eq!(snap.api.stats.cycles[0].node_ids, vec!["ADR-A", "ADR-B"]);
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
