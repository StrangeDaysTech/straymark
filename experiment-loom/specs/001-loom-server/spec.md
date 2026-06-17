# Feature Spec 001 — Loom: knowledge-graph visualization server

> **SpecKit artifact — the WHAT.** This is the feature constitution: what Loom must do,
> for whom, and how we know it is correct. The HOW lives in `plan.md`; the ordered work
> lives in `tasks.md`. Status: **draft / experimental (v0, N=1)**.
> Originates the dogfood Charter: `../../CHARTER-01-loom-server.md`
> (`originating_spec: experiment-loom/specs/001-loom-server/spec.md`).

## 1. Problem & intent

StrayMark documents form a directed, typed graph through their frontmatter cross-links.
That graph is legible to the AI and to the CLI (`straymark audit` builds it internally),
but to a human it is only ever visible one document at a time (`straymark explore`). As a
corpus grows, a human loses the ability to see structure: supersession chains, orphaned
requirements, Charter origins, topical clusters, dangling references.

**Loom makes the graph visible and navigable in a browser, in real time.** It is an
opt-in, experimental complement to the TUI — not a replacement, not a new source of truth.

## 2. Users & primary stories

**U1 — Adopter engineer** keeping a StrayMark project healthy.
**U2 — Adopter lead / auditor** assessing the shape of the governance corpus.
**U3 — Evaluator** deciding whether to adopt StrayMark at all.

- **S1 (U1).** *As an engineer, I open the graph and immediately see the document set as a
  force-directed map, nodes colored by type, so I grasp the corpus shape at a glance.*
- **S2 (U1).** *As an engineer, I select a node and its entire thread — everything it
  links to and everything that links to it, transitively — lights up while the rest dims,
  so I can follow a line of reasoning across documents.*
- **S3 (U1).** *As an engineer, I edit a `.md` file (e.g. add an entry to `related:`) and
  see the graph update within ~1 second without reloading the page.*
- **S4 (U2).** *As a lead, I open a stats panel and see counts by type and risk, the list
  of orphaned documents, and the list of dangling references (links to ids that don't
  exist), so I know where the corpus is incomplete.*
- **S5 (U2).** *As an auditor, I select a document and read its metadata and a body
  excerpt in a side panel, with clickable in/out links, without leaving the graph.*
- **S6 (U2).** *As a lead, I filter the graph by type / status / risk / tag / date range
  to isolate, say, all high-risk ADRs created this quarter.*
- **S7 (U3).** *As an evaluator, I run one command and a compelling, self-explanatory
  picture of "what StrayMark gives me" appears in my browser.*

## 3. The graph model (contract)

Loom builds **one directed multigraph** from all StrayMark documents discovered in the
target directory (excluding `templates/` and `i18n/` mirrors, matching the CLI's
`discover_documents`).

### 3.1 Node

One node per document. A node's `id` is its frontmatter `id` (fallback: filename stem).

| Field | Source | Use |
|---|---|---|
| `id` | frontmatter `id` (or filename stem) | identity, edge endpoints |
| `doc_type` | `DocType::prefix()` (ADR, AILOG, REQ, …) | primary color |
| `title` | frontmatter `title` | label |
| `status` | frontmatter `status` | badge / filter |
| `risk_level` | frontmatter `risk_level` (default `unset`) | size / filter |
| `created` | frontmatter `created` | date filter / timeline |
| `agent` | frontmatter `agent` | filter |
| `tags` | frontmatter `tags[]` | filter |
| `path` | file path | "open in editor" |
| `community` | computed (Louvain) | cluster color |
| `degree_in` / `degree_out` | computed | sizing, orphan detection |
| `is_orphan` | computed (`degree_in+degree_out == 0`) | orphan list |

### 3.2 Edge — **typed**

The edge type is determined by the frontmatter field that produced it (this is the upgrade
over the CLI's current untyped, forward-only `related` adjacency):

| Edge type | Source field | Notes |
|---|---|---|
| `RELATED_TO` | `related` | stored directed, rendered undirected |
| `SUPERSEDES` | `supersedes` | directed, semantic |
| `DOCUMENTS_ALTERNATIVE` | `alternatives_documented` | directed |
| `CHANGES_API` | `api_changes` | node→external API string (rendered as a leaf if surfaced) |
| `ORIGINATES_FROM` | `originating_ailogs`, `originating_spec` | Charter origin |

Each edge carries `resolved: bool`. An edge whose target `id` is **not** present in the
corpus is `resolved: false` → a **dangling reference**, surfaced as a first-class signal
(the CLI does not visualize these today).

### 3.3 Computed graph properties

- **Bidirectional adjacency.** Store both `out_edges[id]` and `in_edges[id]`. "What links
  *to* this node" = `in_edges[id]`. (Today's `build_traceability` discards this.)
- **Orphans.** `degree_in + degree_out == 0`.
- **Cycles / SCCs.** Over the *directed semantic* edges (`SUPERSEDES`, `ORIGINATES_FROM`);
  a `RELATED_TO`↔`RELATED_TO` pair is not reported as a cycle.
- **Communities.** Louvain over the undirected projection → `node.community`.
- **Thread of a node.** The connected neighborhood reachable from a node (bounded by an
  optional depth), returned as `{node_ids, edge_ids}` — the set the UI highlights for S2.

## 4. API surface (contract)

Read-only HTTP + a WebSocket for live updates. JSON bodies.

| Method | Path | Returns |
|---|---|---|
| GET | `/api/graph?type=&status=&risk=&tag=&from=&to=` | full filtered `{nodes, edges, stats}` |
| GET | `/api/node/:id` | node + `in_edges` + `out_edges` + body excerpt |
| GET | `/api/node/:id/thread?depth=N` | `{node_ids, edge_ids}` to highlight (depth optional → full component) |
| GET | `/api/stats` | counts by type/status/risk, orphans, dangling refs, cycles, top-degree |
| WS | `/api/stream` | push `{event:"rebuild", graph}` or `{event:"delta", added, removed, changed}` |
| GET | `/` | the embedded single-page app |
| GET | `/healthz` | liveness |

Filtering is **server-side** (query params) so large corpora are never shipped whole and
the client stays thin.

## 5. Real-time behavior

- The server watches the target directory with the `notify` crate, **debounced ~250ms** to
  coalesce editor save storms.
- On a settled change it re-parses (full rebuild in M1; incremental, only-changed-files in
  M3), diffs against the current graph, and pushes over `/api/stream`.
- **Acceptance:** an edit to a watched `.md` is reflected in an already-open browser within
  **1 second** without a manual reload.

## 6. Functional requirements

- **FR1.** Parse every StrayMark document under the target dir using the *same* code path
  as the CLI (`straymark-core`), excluding `templates/` and `i18n/`.
- **FR2.** Build the typed, bidirectional graph of §3, including unresolved/dangling edges
  and orphan nodes (never silently drop nodes or edges).
- **FR3.** Serve the API of §4 and the embedded SPA.
- **FR4.** Render a force-directed graph, nodes colored by `doc_type`, sized by degree.
- **FR5.** Node selection highlights its thread (§3.3) and dims the rest, with no relayout.
- **FR6.** Live-update on filesystem change per §5.
- **FR7.** Bind `127.0.0.1` only; reject non-loopback `Host` headers; never write to the
  watched directory.
- **FR8.** Surface corpus stats: counts by type/risk, orphan list, dangling-reference list.
- **FR9.** Filter by type/status/risk/tag/date (server-side).
- **FR10.** Community coloring (Louvain) and per-node summary panel.

(Requirements are grouped into milestones in `plan.md` §Phasing — FR1–FR7 are M1; FR8–FR10
are M2+.)

## 7. Non-functional requirements

- **NFR1 (consistency).** For any corpus, `GET /api/graph` node/edge sets are consistent
  with what `straymark audit` derives — same parser, no drift.
- **NFR2 (performance).** Cold build of a low-thousands-document corpus < ~1s; incremental
  rebuild after a single-file edit < ~250ms (M3). Frontend stays interactive at ≥ ~2–3k
  nodes (WebGL).
- **NFR3 (portability).** Single self-contained binary per platform (linux-gnu, darwin
  x64/arm64, windows-msvc); web assets embedded; adopter never runs npm.
- **NFR4 (security).** Loopback-only, read-only, anti-DNS-rebinding `Host` check; no auth
  needed because no non-loopback surface.
- **NFR5 (i18n-ready).** UI strings behind a string table from day one; actual `es`/`zh-CN`
  translation driven by the project's configured language (deferred to M3).

## 8. Acceptance criteria (definition of done for M1)

1. `straymark loom serve` in a project with a populated document set opens
   `http://127.0.0.1:7700` showing a force-directed graph, nodes colored by type.
2. `curl 127.0.0.1:7700/api/graph` returns `{nodes, edges, stats}` whose node/edge sets
   match `straymark audit` for the same corpus (NFR1).
3. Selecting a node lights up its thread and dims the rest (S2/FR5).
4. Editing a watched `.md` (e.g. changing a `related:` entry) updates the open browser in
   < 1s with no reload (S3/§5).
5. The server refuses to bind anything but `127.0.0.1` and rejects a forged non-loopback
   `Host` header (FR7/NFR4).
6. Orphan nodes and dangling references appear in the graph/stats rather than being dropped
   (FR2/FR8 — list panel may be M2, but the data must be present in `/api/stats` at M1).

## 9. Out of scope (for this spec)

- Editing documents from the UI (Loom is read-only).
- Authentication / multi-user / remote hosting (loopback-only).
- A graph database backend (in-memory only; see plan.md §Risks for the revisit trigger).
- AI-generated summaries / topic modeling à la Infranodus' LDA panel (possible *future*
  graduation work, explicitly not v0).
- Non-StrayMark Markdown graphs (Obsidian-style wiki-links).

## 10. Open questions

- `straymark-core` crates.io strategy (publish vs path-dep) — affects the workspace
  refactor; recommendation in plan.md is to publish.
- Default port (proposed **7700**) and whether to auto-open the browser by default.
- Whether the graduation criteria (when Loom stops being experimental) should be frozen
  now in the Charter or revisited after the first external adopter (N=2).
