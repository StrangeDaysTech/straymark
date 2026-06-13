//! Typed, bidirectional, orphan-preserving knowledge graph over StrayMark
//! documents.
//!
//! Generalizes the CLI's original `audit_engine::build_traceability` adjacency
//! (forward-only, `related`-only, orphan-dropping) into the graph model of
//! Loom Spec 001 §3: one node per document, typed edges derived from the
//! frontmatter field that produced them, a `resolved` flag for dangling
//! references, and bidirectional adjacency.
//!
//! Determinism: nodes keep the input document order; edges keep declaration
//! order (per document, fields in a fixed order; within a field, list order).

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

use crate::document::StrayMarkDocument;

/// The frontmatter field an edge was derived from (Spec 001 §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EdgeType {
    /// `related` — stored directed, semantically undirected.
    RelatedTo,
    /// `supersedes` — directed, semantic.
    Supersedes,
    /// `alternatives_documented` — directed.
    DocumentsAlternative,
    /// `api_changes` — node → external API string (usually unresolved).
    ChangesApi,
    /// `originating_ailogs` — document origin.
    OriginatesFrom,
}

impl EdgeType {
    /// Stable string form (matches the serialized representation).
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::RelatedTo => "RELATED_TO",
            EdgeType::Supersedes => "SUPERSEDES",
            EdgeType::DocumentsAlternative => "DOCUMENTS_ALTERNATIVE",
            EdgeType::ChangesApi => "CHANGES_API",
            EdgeType::OriginatesFrom => "ORIGINATES_FROM",
        }
    }
}

/// One node per discovered document, carrying the metadata the visualization
/// surfaces need (Spec 001 §3.1).
#[derive(Debug, Clone, Serialize)]
pub struct Node {
    /// Frontmatter `id`, falling back to the filename stem.
    pub id: String,
    /// True when `id` came from frontmatter (not the filename fallback).
    pub has_explicit_id: bool,
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub risk_level: String,
    pub created: Option<String>,
    pub agent: Option<String>,
    pub tags: Vec<String>,
    pub path: PathBuf,
    /// Incoming edges (resolved references from other documents).
    pub degree_in: usize,
    /// Outgoing edges (including unresolved/dangling references).
    pub degree_out: usize,
}

impl Node {
    /// A document nothing links to and that links to nothing.
    pub fn is_orphan(&self) -> bool {
        self.degree_in + self.degree_out == 0
    }
}

/// A directed, typed edge. `source`/`target` are node ids; an edge whose
/// target id is not present in the corpus is kept with `resolved: false`
/// (a dangling reference — a first-class signal, never silently dropped).
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub resolved: bool,
}

/// The typed, bidirectional document multigraph.
#[derive(Debug, Default)]
pub struct Graph {
    /// Nodes in input (document discovery) order.
    pub nodes: Vec<Node>,
    /// Edges in declaration order.
    pub edges: Vec<Edge>,
    node_index: HashMap<String, usize>,
    out_edges: Vec<Vec<usize>>,
    in_edges: Vec<Vec<usize>>,
}

/// (field accessor, edge type) pairs in fixed declaration order.
fn edge_sources(
    doc: &StrayMarkDocument,
) -> impl Iterator<Item = (EdgeType, &Vec<String>)> {
    let fm = &doc.frontmatter;
    [
        (EdgeType::RelatedTo, fm.related.as_ref()),
        (EdgeType::Supersedes, fm.supersedes.as_ref()),
        (EdgeType::DocumentsAlternative, fm.alternatives_documented.as_ref()),
        (EdgeType::ChangesApi, fm.api_changes.as_ref()),
        (EdgeType::OriginatesFrom, fm.originating_ailogs.as_ref()),
    ]
    .into_iter()
    .filter_map(|(et, list)| list.map(|l| (et, l)))
}

impl Graph {
    /// Build the graph from parsed documents. Never drops a node or an edge:
    /// orphan documents become isolated nodes; references to ids absent from
    /// the corpus become `resolved: false` edges.
    pub fn build(docs: &[&StrayMarkDocument]) -> Graph {
        let mut graph = Graph::default();

        // Pass 1 — nodes, in input order.
        for doc in docs {
            let (id, has_explicit_id) = match &doc.frontmatter.id {
                Some(id) => (id.clone(), true),
                None => (
                    doc.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&doc.filename)
                        .to_string(),
                    false,
                ),
            };
            let fm = &doc.frontmatter;
            let node = Node {
                id: id.clone(),
                has_explicit_id,
                doc_type: doc.doc_type.prefix().to_string(),
                title: fm.title.clone().unwrap_or_else(|| "Untitled".into()),
                status: fm.status.clone().unwrap_or_else(|| "unknown".into()),
                risk_level: fm.risk_level.clone().unwrap_or_else(|| "unset".into()),
                created: fm.created.clone(),
                agent: fm.agent.clone(),
                tags: fm.tags.clone().unwrap_or_default(),
                path: doc.path.clone(),
                degree_in: 0,
                degree_out: 0,
            };
            // First document wins on id collision (mirrors no strong guarantee
            // in the legacy code; collisions are a corpus defect surfaced by
            // `straymark validate`).
            graph.node_index.entry(id).or_insert(graph.nodes.len());
            graph.nodes.push(node);
            graph.out_edges.push(Vec::new());
            graph.in_edges.push(Vec::new());
        }

        // Pass 2 — typed edges, in declaration order.
        for (source_idx, doc) in docs.iter().enumerate() {
            let source_id = graph.nodes[source_idx].id.clone();
            for (edge_type, targets) in edge_sources(doc) {
                for target in targets {
                    let target_idx = graph.node_index.get(target.as_str()).copied();
                    let edge_idx = graph.edges.len();
                    graph.edges.push(Edge {
                        source: source_id.clone(),
                        target: target.clone(),
                        edge_type,
                        resolved: target_idx.is_some(),
                    });
                    graph.out_edges[source_idx].push(edge_idx);
                    graph.nodes[source_idx].degree_out += 1;
                    if let Some(t) = target_idx {
                        graph.in_edges[t].push(edge_idx);
                        graph.nodes[t].degree_in += 1;
                    }
                }
            }
        }

        graph
    }

    /// Node lookup by id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.node_index.get(id).map(|&i| &self.nodes[i])
    }

    /// Outgoing edges of a node, in declaration order.
    pub fn out_edges(&self, id: &str) -> impl Iterator<Item = &Edge> {
        self.node_index
            .get(id)
            .map(|&i| self.out_edges[i].as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|&e| &self.edges[e])
    }

    /// Incoming (resolved) edges of a node.
    pub fn in_edges(&self, id: &str) -> impl Iterator<Item = &Edge> {
        self.node_index
            .get(id)
            .map(|&i| self.in_edges[i].as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|&e| &self.edges[e])
    }

    /// Documents with no links in either direction, in input order.
    pub fn orphans(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().filter(|n| n.is_orphan())
    }

    /// Edges whose target id is absent from the corpus, in declaration order.
    pub fn dangling_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(|e| !e.resolved)
    }

    /// The thread of a node (Spec 001 §3.3, story S2): *"everything it links
    /// to and everything that links to it, transitively"* — its transitive
    /// **descendants** (following out-edges) plus its transitive **ancestors**
    /// (following in-edges), optionally bounded by `depth` per direction.
    ///
    /// Deliberately NOT the undirected connected component: that would also
    /// pull in "siblings" (documents merely co-citing a shared neighbor), and
    /// in tightly-linked corpora it degenerates into highlighting everything.
    /// The thread is the node's line of reasoning, not its neighborhood.
    ///
    /// Returns the node ids and edge indices (into [`Graph::edges`]) to
    /// highlight. The edge set is every edge whose endpoints both belong to
    /// the thread, plus the dangling out-edges of thread members.
    pub fn thread(&self, id: &str, depth: Option<usize>) -> Option<Thread> {
        let &start = self.node_index.get(id)?;

        let mut in_thread = vec![false; self.nodes.len()];
        let mut node_ids = Vec::new();
        in_thread[start] = true;
        node_ids.push(self.nodes[start].id.clone());

        // Two bounded BFS passes: forward over out-edges, backward over
        // in-edges. `Forward` follows what the document links to; `Backward`
        // follows what links to the document.
        for forward in [true, false] {
            let mut visited = vec![false; self.nodes.len()];
            visited[start] = true;
            let mut frontier = vec![start];
            let mut level = 0usize;
            while !frontier.is_empty() && depth.map_or(true, |d| level < d) {
                let mut next = Vec::new();
                for idx in frontier {
                    let edges = if forward { &self.out_edges[idx] } else { &self.in_edges[idx] };
                    for &e in edges {
                        let edge = &self.edges[e];
                        let neighbor = if forward { edge.target.as_str() } else { edge.source.as_str() };
                        if let Some(&n) = self.node_index.get(neighbor) {
                            if !visited[n] {
                                visited[n] = true;
                                if !in_thread[n] {
                                    in_thread[n] = true;
                                    node_ids.push(self.nodes[n].id.clone());
                                }
                                next.push(n);
                            }
                        }
                    }
                }
                frontier = next;
                level += 1;
            }
        }

        // Highlight every edge internal to the thread (including ones not
        // walked, e.g. an ancestor linking directly to a descendant), plus
        // dangling out-edges of thread members.
        let edge_ids: Vec<usize> = self
            .edges
            .iter()
            .enumerate()
            .filter(|(_, edge)| {
                let src_in = self.node_index.get(edge.source.as_str()).is_some_and(|&i| in_thread[i]);
                if !src_in {
                    return false;
                }
                match self.node_index.get(edge.target.as_str()) {
                    Some(&t) => in_thread[t],
                    None => !edge.resolved, // dangling out-edge of a member
                }
            })
            .map(|(i, _)| i)
            .collect();

        Some(Thread { node_ids, edge_ids })
    }
}

/// The highlight set for a node's thread (Spec 001 §3.3): node ids plus edge
/// indices into [`Graph::edges`].
#[derive(Debug, Clone, Serialize)]
pub struct Thread {
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<usize>,
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
    fn test_empty_graph() {
        let g = Graph::build(&[]);
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
    }

    #[test]
    fn test_typed_edges_and_bidirectional_adjacency() {
        let req = make_doc(
            "REQ-2026-03-01-001-login.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-2026-03-01-001".into()),
                related: Some(vec!["ADR-2026-03-02-001".into()]),
                ..Default::default()
            },
        );
        let adr = make_doc(
            "ADR-2026-03-02-001-jwt.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-03-02-001".into()),
                supersedes: Some(vec!["ADR-2026-01-01-001".into()]),
                api_changes: Some(vec!["POST /login".into()]),
                ..Default::default()
            },
        );
        let old_adr = make_doc(
            "ADR-2026-01-01-001-old.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-01-01-001".into()),
                ..Default::default()
            },
        );
        let docs = [&req, &adr, &old_adr];
        let g = Graph::build(&docs);

        assert_eq!(g.nodes.len(), 3);
        assert_eq!(g.edges.len(), 3);

        let types: Vec<EdgeType> = g.edges.iter().map(|e| e.edge_type).collect();
        assert_eq!(
            types,
            vec![EdgeType::RelatedTo, EdgeType::Supersedes, EdgeType::ChangesApi]
        );

        // Bidirectional: "what links TO the old ADR" answers in one lookup.
        let incoming: Vec<&str> = g
            .in_edges("ADR-2026-01-01-001")
            .map(|e| e.source.as_str())
            .collect();
        assert_eq!(incoming, vec!["ADR-2026-03-02-001"]);

        // api_changes points at an external string → unresolved, but kept.
        let api_edge = g.edges.iter().find(|e| e.edge_type == EdgeType::ChangesApi).unwrap();
        assert!(!api_edge.resolved);
        assert_eq!(api_edge.target, "POST /login");
    }

    #[test]
    fn test_orphans_preserved() {
        let orphan = make_doc(
            "TDE-2026-04-01-001-orphan.md",
            DocType::Tde,
            Frontmatter {
                id: Some("TDE-2026-04-01-001".into()),
                ..Default::default()
            },
        );
        let docs = [&orphan];
        let g = Graph::build(&docs);
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.orphans().count(), 1);
        assert!(g.node("TDE-2026-04-01-001").unwrap().is_orphan());
    }

    #[test]
    fn test_dangling_reference_surfaced() {
        let doc = make_doc(
            "ADR-2026-03-02-001-jwt.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-03-02-001".into()),
                related: Some(vec!["MISSING-2026-01-01-001".into()]),
                ..Default::default()
            },
        );
        let docs = [&doc];
        let g = Graph::build(&docs);
        assert_eq!(g.edges.len(), 1);
        assert!(!g.edges[0].resolved);
        assert_eq!(g.dangling_edges().count(), 1);
        // Degree out counts the attempt; the node is not an orphan.
        assert!(!g.node("ADR-2026-03-02-001").unwrap().is_orphan());
    }

    #[test]
    fn test_filename_stem_fallback_id() {
        let doc = make_doc(
            "AILOG-2026-03-03-001-impl.md",
            DocType::Ailog,
            Frontmatter::default(),
        );
        let docs = [&doc];
        let g = Graph::build(&docs);
        let node = &g.nodes[0];
        assert_eq!(node.id, "AILOG-2026-03-03-001-impl");
        assert!(!node.has_explicit_id);
    }

    #[test]
    fn test_originating_ailogs_edge() {
        let ailog = make_doc(
            "AILOG-2026-03-03-001-impl.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("AILOG-2026-03-03-001".into()),
                ..Default::default()
            },
        );
        let adr = make_doc(
            "ADR-2026-03-05-001-followup.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-03-05-001".into()),
                originating_ailogs: Some(vec!["AILOG-2026-03-03-001".into()]),
                ..Default::default()
            },
        );
        let docs = [&ailog, &adr];
        let g = Graph::build(&docs);
        let e = &g.edges[0];
        assert_eq!(e.edge_type, EdgeType::OriginatesFrom);
        assert!(e.resolved);
        assert_eq!(e.source, "ADR-2026-03-05-001");
        assert_eq!(e.target, "AILOG-2026-03-03-001");
    }

    #[test]
    fn test_thread_full_component_and_depth() {
        // REQ → ADR → AILOG chain; TDE isolated.
        let req = make_doc(
            "REQ-1-a.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-1".into()),
                related: Some(vec!["ADR-1".into()]),
                ..Default::default()
            },
        );
        let adr = make_doc(
            "ADR-1-b.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-1".into()),
                related: Some(vec!["AILOG-1".into()]),
                ..Default::default()
            },
        );
        let ailog = make_doc(
            "AILOG-1-c.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("AILOG-1".into()),
                ..Default::default()
            },
        );
        let tde = make_doc(
            "TDE-1-d.md",
            DocType::Tde,
            Frontmatter {
                id: Some("TDE-1".into()),
                ..Default::default()
            },
        );
        let docs = [&req, &adr, &ailog, &tde];
        let g = Graph::build(&docs);

        // Full component from the middle node reaches both ends, not the orphan.
        let t = g.thread("ADR-1", None).unwrap();
        let mut nodes = t.node_ids.clone();
        nodes.sort();
        assert_eq!(nodes, vec!["ADR-1", "AILOG-1", "REQ-1"]);
        assert_eq!(t.edge_ids.len(), 2);

        // Depth 1 from REQ: only the direct neighbor.
        let t1 = g.thread("REQ-1", Some(1)).unwrap();
        let mut nodes1 = t1.node_ids.clone();
        nodes1.sort();
        assert_eq!(nodes1, vec!["ADR-1", "REQ-1"]);

        // Unknown id → None; isolated node → itself only.
        assert!(g.thread("NOPE", None).is_none());
        let t_orphan = g.thread("TDE-1", None).unwrap();
        assert_eq!(t_orphan.node_ids, vec!["TDE-1"]);
        assert!(t_orphan.edge_ids.is_empty());
    }

    #[test]
    fn test_thread_excludes_siblings() {
        // A → C ← B: B is a "sibling" of A (co-cites C) — not part of A's
        // line of reasoning, so it must NOT light up (S2 semantics).
        let a = make_doc(
            "REQ-A-a.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-A".into()),
                related: Some(vec!["ADR-C".into()]),
                ..Default::default()
            },
        );
        let b = make_doc(
            "REQ-B-b.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-B".into()),
                related: Some(vec!["ADR-C".into()]),
                ..Default::default()
            },
        );
        let c = make_doc(
            "ADR-C-c.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-C".into()),
                ..Default::default()
            },
        );
        let docs = [&a, &b, &c];
        let g = Graph::build(&docs);

        let t = g.thread("REQ-A", None).unwrap();
        let mut nodes = t.node_ids.clone();
        nodes.sort();
        assert_eq!(nodes, vec!["ADR-C", "REQ-A"]); // B excluded
        assert_eq!(t.edge_ids, vec![0]); // only A→C, not B→C

        // From C, both citers are ancestors — they DO light up.
        let tc = g.thread("ADR-C", None).unwrap();
        let mut nodes_c = tc.node_ids.clone();
        nodes_c.sort();
        assert_eq!(nodes_c, vec!["ADR-C", "REQ-A", "REQ-B"]);
    }

    #[test]
    fn test_thread_includes_dangling_edges() {
        let doc = make_doc(
            "ADR-1-a.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-1".into()),
                related: Some(vec!["MISSING-1".into()]),
                ..Default::default()
            },
        );
        let docs = [&doc];
        let g = Graph::build(&docs);
        let t = g.thread("ADR-1", None).unwrap();
        assert_eq!(t.node_ids, vec!["ADR-1"]);
        assert_eq!(t.edge_ids, vec![0]); // the dangling edge belongs to the thread
    }

    #[test]
    fn test_deterministic_order() {
        let a = make_doc(
            "REQ-2026-03-01-001-a.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-2026-03-01-001".into()),
                related: Some(vec!["B-2".into(), "B-1".into()]),
                ..Default::default()
            },
        );
        let docs = [&a];
        let g = Graph::build(&docs);
        let targets: Vec<&str> = g.edges.iter().map(|e| e.target.as_str()).collect();
        // Declaration order of the `related` list is preserved.
        assert_eq!(targets, vec!["B-2", "B-1"]);
    }
}
