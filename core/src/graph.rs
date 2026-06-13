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
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::document::StrayMarkDocument;
use crate::entities::Entity;

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
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
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

/// Resolve an edge target to a node index (R1 — reference normalization).
/// Tries, in order: the exact `id`; the document's file **basename** (with or
/// without a trailing `.md`); and the leading StrayMark dated **id prefix**
/// extracted from the basename. The filename fallbacks resolve only when the
/// match is **unique** (`by_filename` stores `None` for shared basenames), so a
/// reference is never wired to the wrong document. Returns `None` (→ a dangling
/// reference) when nothing matches.
fn resolve_target(
    raw: &str,
    node_index: &HashMap<String, usize>,
    by_filename: &HashMap<String, Option<usize>>,
    by_path_suffix: &HashMap<String, Option<usize>>,
    by_charter_prefix: &HashMap<String, Option<usize>>,
) -> Option<usize> {
    // a) Exact id (the common, canonical case).
    if let Some(&i) = node_index.get(raw) {
        return Some(i);
    }
    // b) By file basename, tolerating a missing `.md` extension.
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw);
    if let Some(Some(i)) = by_filename.get(base) {
        return Some(*i);
    }
    if let Some(Some(i)) = by_filename.get(&format!("{base}.md")) {
        return Some(*i);
    }
    // c) By relative path suffix (R2): a path-style reference like
    //    `.straymark/audits/CHARTER-13/review.md`, where the basename alone
    //    (`review.md`) is shared across many documents.
    let norm = raw.replace('\\', "/");
    let norm = norm.trim_start_matches("./").trim_start_matches('/');
    if let Some(Some(i)) = by_path_suffix.get(norm) {
        return Some(*i);
    }
    // d) By `CHARTER-NN` prefix (R2): `CHARTER-15` → the charter whose
    //    `charter_id` starts with `CHARTER-15-` (boundary-safe).
    if let Some(prefix) = charter_prefix_of(raw) {
        if let Some(Some(i)) = by_charter_prefix.get(prefix) {
            return Some(*i);
        }
    }
    // e) By the leading dated id prefix (`TYPE-YYYY-MM-DD-NNN`) of the basename,
    //    e.g. `AILOG-2026-05-07-041-some-slug.md` → id `AILOG-2026-05-07-041`.
    if let Some(id) = leading_id(base) {
        if let Some(&i) = node_index.get(id) {
            return Some(i);
        }
    }
    None
}

/// Insert `idx` under `key` unless the key is already taken — a shared key maps
/// to `None` and never resolves, so a reference is never wired to the wrong node.
fn insert_unique(map: &mut HashMap<String, Option<usize>>, key: String, idx: usize) {
    map.entry(key).and_modify(|slot| *slot = None).or_insert(Some(idx));
}

/// Component-wise suffixes of `path` (deepest first capped at 6 levels), joined
/// with `/`, for resolving path-style references against node paths.
fn path_suffixes(path: &Path) -> Vec<String> {
    let comps: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let start = comps.len().saturating_sub(6);
    (start..comps.len()).map(|s| comps[s..].join("/")).collect()
}

/// Extract the `CHARTER-NN` prefix from a charter id or reference
/// (`CHARTER-15` and `CHARTER-15-auth-…` both → `CHARTER-15`). Boundary-safe:
/// the digit run must end at the string end or a `-`, so `CHARTER-15` never
/// matches `CHARTER-150`.
fn charter_prefix_of(raw: &str) -> Option<&str> {
    let rest = raw.strip_prefix("CHARTER-")?;
    let digits_end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    if digits_end == 0 {
        return None;
    }
    if digits_end < rest.len() && rest.as_bytes()[digits_end] != b'-' {
        return None;
    }
    Some(&raw[.."CHARTER-".len() + digits_end])
}

/// If `base` starts with a StrayMark dated id (`TYPE-YYYY-MM-DD-NNN`: ASCII
/// uppercase type, then 4-2-2-3 digit groups separated by `-`), return that
/// prefix slice; otherwise `None`. Hand-rolled (core has no regex dependency).
fn leading_id(base: &str) -> Option<&str> {
    let bytes = base.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_uppercase() {
        i += 1;
    }
    if i == 0 {
        return None; // no TYPE segment
    }
    for &len in &[4usize, 2, 2, 3] {
        if i >= bytes.len() || bytes[i] != b'-' {
            return None;
        }
        i += 1;
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i - start != len {
            return None;
        }
    }
    Some(&base[..i])
}

impl Graph {
    /// Build the graph from parsed documents. Never drops a node or an edge:
    /// orphan documents become isolated nodes; references to ids absent from
    /// the corpus become `resolved: false` edges.
    pub fn build(docs: &[&StrayMarkDocument]) -> Graph {
        Graph::build_with_entities(docs, &[])
    }

    /// Like [`Graph::build`] but also injects non-document [`Entity`] nodes
    /// (charters, plans, audit reviews — R2), appended after the documents, so
    /// references to them resolve instead of dangling. Entity links become
    /// edges resolved with the same reference-normalization fallbacks.
    pub fn build_with_entities(docs: &[&StrayMarkDocument], entities: &[Entity]) -> Graph {
        let mut graph = Graph::default();

        // Pass 1a — document nodes, in input order.
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

        // Pass 1b — entity nodes (charters/plans/audits), after the documents.
        for ent in entities {
            let node = Node {
                id: ent.id.clone(),
                has_explicit_id: true,
                doc_type: ent.doc_type.clone(),
                title: ent.title.clone(),
                status: ent.status.clone(),
                risk_level: "unset".into(),
                created: None,
                agent: None,
                tags: Vec::new(),
                path: ent.path.clone(),
                degree_in: 0,
                degree_out: 0,
            };
            graph.node_index.entry(ent.id.clone()).or_insert(graph.nodes.len());
            graph.nodes.push(node);
            graph.out_edges.push(Vec::new());
            graph.in_edges.push(Vec::new());
        }

        // Reference-resolution indices over ALL nodes (R1 basename + R2 path
        // suffix + R2 `CHARTER-NN` prefix). A shared key maps to `None` and is
        // never used, so a reference is never wired to the wrong node.
        let mut by_filename: HashMap<String, Option<usize>> = HashMap::new();
        let mut by_path_suffix: HashMap<String, Option<usize>> = HashMap::new();
        let mut by_charter_prefix: HashMap<String, Option<usize>> = HashMap::new();
        for (idx, node) in graph.nodes.iter().enumerate() {
            if let Some(base) = node.path.file_name().and_then(|s| s.to_str()) {
                insert_unique(&mut by_filename, base.to_string(), idx);
            }
            for suffix in path_suffixes(&node.path) {
                insert_unique(&mut by_path_suffix, suffix, idx);
            }
            if node.doc_type == "CHARTER" {
                if let Some(prefix) = charter_prefix_of(&node.id) {
                    insert_unique(&mut by_charter_prefix, prefix.to_string(), idx);
                }
            }
        }

        // Edge specs in declaration order: documents first, then entities.
        let mut specs: Vec<(usize, EdgeType, String)> = Vec::new();
        for (source_idx, doc) in docs.iter().enumerate() {
            for (edge_type, targets) in edge_sources(doc) {
                for target in targets {
                    specs.push((source_idx, edge_type, target.clone()));
                }
            }
        }
        for (ei, ent) in entities.iter().enumerate() {
            let source_idx = docs.len() + ei;
            for (edge_type, target) in &ent.links {
                specs.push((source_idx, *edge_type, target.clone()));
            }
        }

        // Pass 2 — create typed edges, resolving each target.
        for (source_idx, edge_type, target) in specs {
            let source_id = graph.nodes[source_idx].id.clone();
            let target_idx = resolve_target(
                &target,
                &graph.node_index,
                &by_filename,
                &by_path_suffix,
                &by_charter_prefix,
            );
            // Canonicalize the stored target to the resolved node's id so a
            // reference written by filename/path/prefix points at the real node
            // (the UI, audit, threads and cycles all key on id).
            let target_str = match target_idx {
                Some(t) => graph.nodes[t].id.clone(),
                None => target.clone(),
            };
            let edge_idx = graph.edges.len();
            graph.edges.push(Edge {
                source: source_id,
                target: target_str,
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
            while !frontier.is_empty() && depth.is_none_or(|d| level < d) {
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

    /// Dependency cycles among documents (Spec 001 §3.3): strongly-connected
    /// components over the **resolved directed semantic** edges
    /// (`SUPERSEDES`, `ORIGINATES_FROM`). `RELATED_TO` is symmetric by intent
    /// and never reported; unresolved (dangling/external) edges cannot form a
    /// cycle. A component is a cycle when it has more than one member, or a
    /// single document with a semantic self-edge (e.g. a doc superseding
    /// itself). Such a cycle is a corpus defect (e.g. A supersedes B supersedes
    /// A). Deterministic: components and their members keep graph node order.
    pub fn cycles(&self) -> Vec<Cycle> {
        let node_ids: Vec<String> = self.nodes.iter().map(|n| n.id.clone()).collect();
        cycles_in(&node_ids, &self.edges)
    }
}

/// Edge types that express a *directed dependency*; a cycle among them is a
/// corpus defect. `RELATED_TO` (symmetric) and the unresolved/external kinds
/// are excluded (Spec 001 §3.3).
fn is_semantic(edge_type: EdgeType) -> bool {
    matches!(edge_type, EdgeType::Supersedes | EdgeType::OriginatesFrom)
}

/// Dependency cycles among a set of nodes (by id) connected by `edges`, over
/// the resolved directed semantic edges only. Reusable by both the full graph
/// ([`Graph::cycles`]) and a filtered/induced view, which only has the node
/// ids and the edge list. Deterministic: members keep `node_ids` order and
/// components are returned by smallest member index.
pub fn cycles_in(node_ids: &[String], edges: &[Edge]) -> Vec<Cycle> {
    let n = node_ids.len();
    let index: HashMap<&str, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    // Semantic adjacency (node idx → target node idxs), resolved only.
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for edge in edges {
        if edge.resolved && is_semantic(edge.edge_type) {
            if let (Some(&s), Some(&t)) = (
                index.get(edge.source.as_str()),
                index.get(edge.target.as_str()),
            ) {
                adj[s].push(t);
            }
        }
    }

    let mut cycles: Vec<Cycle> = tarjan_scc(&adj)
        .into_iter()
        .filter_map(|mut comp| {
            let is_cycle = comp.len() > 1 || (comp.len() == 1 && adj[comp[0]].contains(&comp[0]));
            if !is_cycle {
                return None;
            }
            comp.sort_unstable(); // node_ids order within the component
            let members: std::collections::HashSet<usize> = comp.iter().copied().collect();

            // Distinct semantic edge types internal to the component, in edge
            // declaration order (deterministic).
            let mut edge_types: Vec<EdgeType> = Vec::new();
            for edge in edges {
                if edge.resolved
                    && is_semantic(edge.edge_type)
                    && index
                        .get(edge.source.as_str())
                        .is_some_and(|s| members.contains(s))
                    && index
                        .get(edge.target.as_str())
                        .is_some_and(|t| members.contains(t))
                    && !edge_types.contains(&edge.edge_type)
                {
                    edge_types.push(edge.edge_type);
                }
            }

            Some(Cycle {
                node_ids: comp.iter().map(|&i| node_ids[i].clone()).collect(),
                edge_types,
            })
        })
        .collect();

    cycles.sort_by_key(|c| {
        c.node_ids
            .iter()
            .filter_map(|id| index.get(id.as_str()).copied())
            .min()
            .unwrap_or(usize::MAX)
    });
    cycles
}

/// Iterative Tarjan's strongly-connected-components over an adjacency list.
/// Iterative (explicit stack) so deep semantic chains can't overflow the call
/// stack. Components are returned in discovery order.
fn tarjan_scc(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut index_of: Vec<Option<usize>> = vec![None; n];
    let mut lowlink: Vec<usize> = vec![0; n];
    let mut on_stack: Vec<bool> = vec![false; n];
    let mut scc_stack: Vec<usize> = Vec::new();
    let mut next_index = 0usize;
    let mut sccs: Vec<Vec<usize>> = Vec::new();

    for root in 0..n {
        if index_of[root].is_some() {
            continue;
        }
        // DFS frame: (node, next child cursor).
        let mut call_stack: Vec<(usize, usize)> = vec![(root, 0)];
        while let Some(&(node, cursor)) = call_stack.last() {
            if cursor == 0 {
                index_of[node] = Some(next_index);
                lowlink[node] = next_index;
                next_index += 1;
                scc_stack.push(node);
                on_stack[node] = true;
            }
            if cursor < adj[node].len() {
                call_stack.last_mut().unwrap().1 += 1;
                let w = adj[node][cursor];
                match index_of[w] {
                    None => call_stack.push((w, 0)),
                    Some(wi) if on_stack[w] => lowlink[node] = lowlink[node].min(wi),
                    _ => {}
                }
            } else {
                if lowlink[node] == index_of[node].unwrap() {
                    let mut comp = Vec::new();
                    loop {
                        let w = scc_stack.pop().unwrap();
                        on_stack[w] = false;
                        comp.push(w);
                        if w == node {
                            break;
                        }
                    }
                    sccs.push(comp);
                }
                call_stack.pop();
                if let Some(&(parent, _)) = call_stack.last() {
                    lowlink[parent] = lowlink[parent].min(lowlink[node]);
                }
            }
        }
    }
    sccs
}

/// The highlight set for a node's thread (Spec 001 §3.3): node ids plus edge
/// indices into [`Graph::edges`].
#[derive(Debug, Clone, Serialize)]
pub struct Thread {
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<usize>,
}

/// A dependency cycle (strongly-connected component) over the directed
/// semantic edges — see [`Graph::cycles`].
#[derive(Debug, Clone, Serialize)]
pub struct Cycle {
    /// Member document ids, in graph node order.
    pub node_ids: Vec<String>,
    /// The distinct semantic edge types participating in the cycle.
    pub edge_types: Vec<EdgeType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocType, Frontmatter};
    use crate::entities::Entity;
    use std::path::PathBuf;

    fn charter_entity(id: &str, path: &str) -> Entity {
        Entity {
            id: id.into(),
            doc_type: "CHARTER".into(),
            title: id.into(),
            status: "closed".into(),
            path: PathBuf::from(path),
            links: Vec::new(),
        }
    }

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

    #[test]
    fn test_cycles_supersession_loop_reported() {
        // A supersedes B, B supersedes A — a real corpus defect.
        let a = make_doc(
            "ADR-A-a.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-A".into()),
                supersedes: Some(vec!["ADR-B".into()]),
                ..Default::default()
            },
        );
        let b = make_doc(
            "ADR-B-b.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-B".into()),
                supersedes: Some(vec!["ADR-A".into()]),
                ..Default::default()
            },
        );
        let docs = [&a, &b];
        let g = Graph::build(&docs);

        let cycles = g.cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].node_ids, vec!["ADR-A", "ADR-B"]); // graph order
        assert_eq!(cycles[0].edge_types, vec![EdgeType::Supersedes]);
    }

    #[test]
    fn test_cycles_related_pair_not_reported() {
        // A ↔ B via `related` is symmetric by intent, not a dependency cycle.
        let a = make_doc(
            "REQ-A-a.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-A".into()),
                related: Some(vec!["REQ-B".into()]),
                ..Default::default()
            },
        );
        let b = make_doc(
            "REQ-B-b.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-B".into()),
                related: Some(vec!["REQ-A".into()]),
                ..Default::default()
            },
        );
        let docs = [&a, &b];
        let g = Graph::build(&docs);
        assert!(g.cycles().is_empty());
    }

    #[test]
    fn test_cycles_self_supersession_reported() {
        let a = make_doc(
            "ADR-A-a.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-A".into()),
                supersedes: Some(vec!["ADR-A".into()]),
                ..Default::default()
            },
        );
        let docs = [&a];
        let g = Graph::build(&docs);
        let cycles = g.cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].node_ids, vec!["ADR-A"]);
    }

    #[test]
    fn test_cycles_dangling_semantic_edge_is_not_a_cycle() {
        // A supersedes a missing id — unresolved, so no cycle.
        let a = make_doc(
            "ADR-A-a.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-A".into()),
                supersedes: Some(vec!["ADR-GHOST".into()]),
                ..Default::default()
            },
        );
        let docs = [&a];
        let g = Graph::build(&docs);
        assert!(g.cycles().is_empty());
    }

    #[test]
    fn test_resolve_reference_by_filename() {
        // REQ references the ADR by its FILE NAME, not its id (R1).
        let req = make_doc(
            "REQ-2026-03-01-001-login.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-2026-03-01-001".into()),
                related: Some(vec!["ADR-2026-03-02-001-jwt.md".into()]),
                ..Default::default()
            },
        );
        let adr = make_doc(
            "ADR-2026-03-02-001-jwt.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-03-02-001".into()),
                ..Default::default()
            },
        );
        let docs = [&req, &adr];
        let g = Graph::build(&docs);

        assert_eq!(g.edges.len(), 1);
        assert!(g.edges[0].resolved);
        // The stored target is canonicalized to the node's id.
        assert_eq!(g.edges[0].target, "ADR-2026-03-02-001");
        assert_eq!(g.dangling_edges().count(), 0);
        // Bidirectional adjacency is wired as for any resolved edge.
        let incoming: Vec<&str> = g
            .in_edges("ADR-2026-03-02-001")
            .map(|e| e.source.as_str())
            .collect();
        assert_eq!(incoming, vec!["REQ-2026-03-01-001"]);
    }

    #[test]
    fn test_resolve_reference_by_id_prefix() {
        // The reference uses a different descriptive slug than the actual file,
        // but the leading dated id matches → resolves by id prefix (R1).
        let ailog = make_doc(
            "AILOG-2026-05-07-041-real-slug.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("AILOG-2026-05-07-041".into()),
                ..Default::default()
            },
        );
        let adr = make_doc(
            "ADR-2026-05-08-001-x.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-05-08-001".into()),
                related: Some(vec!["AILOG-2026-05-07-041-some-other-slug.md".into()]),
                ..Default::default()
            },
        );
        let docs = [&ailog, &adr];
        let g = Graph::build(&docs);
        let e = g.edges.iter().find(|e| e.source == "ADR-2026-05-08-001").unwrap();
        assert!(e.resolved);
        assert_eq!(e.target, "AILOG-2026-05-07-041");
    }

    #[test]
    fn test_ambiguous_filename_stays_dangling() {
        // Two documents share the same basename → a reference by that basename
        // must NOT be resolved (could be either; never guess).
        let a = make_doc(
            "review.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("A-1".into()),
                ..Default::default()
            },
        );
        let b = make_doc(
            "review.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("B-1".into()),
                ..Default::default()
            },
        );
        let c = make_doc(
            "REQ-2026-03-01-001-x.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-2026-03-01-001".into()),
                related: Some(vec!["review.md".into()]),
                ..Default::default()
            },
        );
        let docs = [&a, &b, &c];
        let g = Graph::build(&docs);
        let e = g.edges.iter().find(|e| e.source == "REQ-2026-03-01-001").unwrap();
        assert!(!e.resolved);
    }

    #[test]
    fn test_entity_charter_resolves_by_prefix_and_full_id() {
        let req = make_doc(
            "REQ-2026-03-01-001-x.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-2026-03-01-001".into()),
                // CHARTER-15 (short prefix) and the full charter_id of another.
                related: Some(vec!["CHARTER-15".into(), "CHARTER-16-other".into()]),
                ..Default::default()
            },
        );
        let c15 = charter_entity("CHARTER-15-auth", ".straymark/charters/15-auth.md");
        let c16 = charter_entity("CHARTER-16-other", ".straymark/charters/16-other.md");
        let docs = [&req];
        let g = Graph::build_with_entities(&docs, &[c15, c16]);

        let targets: Vec<&str> = g
            .out_edges("REQ-2026-03-01-001")
            .map(|e| e.target.as_str())
            .collect();
        assert_eq!(targets, vec!["CHARTER-15-auth", "CHARTER-16-other"]);
        assert!(g.out_edges("REQ-2026-03-01-001").all(|e| e.resolved));
        assert_eq!(g.node("CHARTER-15-auth").unwrap().doc_type, "CHARTER");
    }

    #[test]
    fn test_entity_charter_prefix_is_boundary_safe() {
        // CHARTER-1 must not resolve to CHARTER-15.
        let req = make_doc(
            "REQ-2026-03-01-001-x.md",
            DocType::Req,
            Frontmatter {
                id: Some("REQ-2026-03-01-001".into()),
                related: Some(vec!["CHARTER-1".into()]),
                ..Default::default()
            },
        );
        let c15 = charter_entity("CHARTER-15-auth", ".straymark/charters/15-auth.md");
        let docs = [&req];
        let g = Graph::build_with_entities(&docs, &[c15]);
        assert!(!g.edges.iter().any(|e| e.resolved));
    }

    #[test]
    fn test_entity_audit_resolves_by_path_suffix_when_basename_ambiguous() {
        let adr = make_doc(
            "ADR-2026-03-02-001-x.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-03-02-001".into()),
                related: Some(vec![".straymark/audits/CHARTER-13/review.md".into()]),
                ..Default::default()
            },
        );
        // Two audits share the `review.md` basename → basename is ambiguous, so
        // resolution must use the relative path suffix.
        let a13 = Entity {
            id: "audits/CHARTER-13/review.md".into(),
            doc_type: "AUDIT".into(),
            title: "Audit — CHARTER-13".into(),
            status: "reviewed".into(),
            path: PathBuf::from("/x/.straymark/audits/CHARTER-13/review.md"),
            links: Vec::new(),
        };
        let a14 = Entity {
            id: "audits/CHARTER-14/review.md".into(),
            doc_type: "AUDIT".into(),
            title: "Audit — CHARTER-14".into(),
            status: "reviewed".into(),
            path: PathBuf::from("/x/.straymark/audits/CHARTER-14/review.md"),
            links: Vec::new(),
        };
        let docs = [&adr];
        let g = Graph::build_with_entities(&docs, &[a13, a14]);
        let e = g.edges.iter().find(|e| e.source == "ADR-2026-03-02-001").unwrap();
        assert!(e.resolved);
        assert_eq!(e.target, "audits/CHARTER-13/review.md");
    }

    #[test]
    fn test_entity_plan_resolves_by_id_and_links_wire() {
        let adr = make_doc(
            "ADR-2026-03-02-001-x.md",
            DocType::Adr,
            Frontmatter {
                id: Some("ADR-2026-03-02-001".into()),
                related: Some(vec!["PLAN-01".into()]),
                ..Default::default()
            },
        );
        let plan = Entity {
            id: "PLAN-01".into(),
            doc_type: "PLAN".into(),
            title: "Deploy".into(),
            status: "closed".into(),
            path: PathBuf::from(".straymark/plans/PLAN-01.telemetry.yaml"),
            links: Vec::new(),
        };
        // A charter whose originating_ailogs link wires to a real AILOG node.
        let ailog = make_doc(
            "AILOG-2026-05-11-043-x.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("AILOG-2026-05-11-043".into()),
                ..Default::default()
            },
        );
        let charter = Entity {
            id: "CHARTER-15-auth".into(),
            doc_type: "CHARTER".into(),
            title: "Auth".into(),
            status: "closed".into(),
            path: PathBuf::from(".straymark/charters/15-auth.md"),
            links: vec![(EdgeType::OriginatesFrom, "AILOG-2026-05-11-043".into())],
        };
        let docs = [&adr, &ailog];
        let g = Graph::build_with_entities(&docs, &[plan, charter]);

        let plan_edge = g.edges.iter().find(|e| e.target == "PLAN-01").unwrap();
        assert!(plan_edge.resolved);
        // The charter's originating_ailogs link resolved to the AILOG node.
        let incoming: Vec<&str> = g
            .in_edges("AILOG-2026-05-11-043")
            .map(|e| e.source.as_str())
            .collect();
        assert_eq!(incoming, vec!["CHARTER-15-auth"]);
    }
}
