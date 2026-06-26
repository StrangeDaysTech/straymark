# Feature Spec 003 — Loom Code Weave ("extended knowledge")

> **SpecKit artifact — the WHAT.** The third capability of the Loom dashboard: fuse the
> **code** of a project into the **governance** knowledge graph so that a code symbol and the
> decision that governs it live in **one woven graph**, not two neighbouring views. Companion
> to Spec 001 (the knowledge graph of documents) and Spec 002 (the architecture plan). The
> HOW is in `plan.md`; the source-of-the-code-graph decision (native vs. external indexer) is
> recorded in `docs/decisions/ADR-2026-06-26-001-code-weave-source.md`. The work block is
> `experiment-loom/CHARTER-02-code-weave.md`. Status: **draft / experimental (v0, N=1)**.

## 1. Problem & intent

Loom today holds **two** graphs that never touch each other:

- The **knowledge graph** (Spec 001) — documents (ADR, AILOG, TDE, CHARTER, REQ, …) linked by
  typed frontmatter edges (`SUPERSEDES`, `ORIGINATES_FROM`, `RELATED_TO`, …). It answers *"how
  do decisions relate?"*
- The **architecture plan** (Spec 002) — components (buckets of file globs) placed in layers,
  with a "you are here" status overlay. The only bridge to code is **file-level**: a Charter
  "touches" a component when its declared files fall inside the component's globs. A component
  is a *bucket of globs*; nothing inside it is a graph node.

What is missing is the thing operators actually reach for: **the code itself, woven into the
same fabric of knowledge.** A function should be a first-class node sitting next to the ADR
that decided it, the Charter that is modifying it, and the TDE that flags its debt — so the
answer to *"what governs this code?"* and *"what code does this decision actually touch?"* is
a **traversal of one graph**, not a manual cross-reference between two pictures.

The external reference for the *idea* (not the implementation) is **`codebase-memory-mcp`**
(DeusData): it indexes code with tree-sitter into a symbol graph and lets an agent traverse
it instead of reading files blindly. We borrow the *shape of the question* — code as a
navigable graph — and weave it into governance, which is the half nobody else indexes. We do
**not** adopt its engine (see `ADR-2026-06-26-001` and §13).

This is **"conocimiento extendido"**: governance + code as one entramado.

## 2. Users & primary stories

- **S1 (operator, daily).** *I select an in-progress Charter and the graph lights up not just
  the documents it relates to, but the actual functions and files it is changing — I see the
  reach of the work down to the symbol.*
- **S2 (operator).** *I click a function in the graph and a panel shows the ADR that decided
  its component, the Charter currently modifying it, and any open TDE whose debt sits on it.*
- **S3 (architect).** *I open a component and expand it: its files, then their functions —
  the architecture plan stops being an opaque box and becomes drill-downable to code.*
- **S4 (reviewer / auditor).** *For a closed Charter I can confirm exactly which symbols its
  AILOGs claim to have modified, attributed by line range, not just "some file in this dir".*
- **S5 (evaluator).** *The woven graph — decisions and the code they govern in one view — is
  the single most legible answer to "what does StrayMark give me that a code indexer alone
  does not".*

## 3. The woven graph (contract)

### 3.1 The weave is a composition — no new frontmatter

The cross-link from a governance document to a code symbol is the **composition of two
relations the project already owns**, never a new annotation an adopter must write:

```
document ──(governance file set, already extracted by core::architecture::gather)──▶ file
file     ──(code-graph builder: source walk + parser)─────────────────────────────▶ symbol
─────────────────────────────────────────────────────────────────────────────────────────
∴ document → file → symbol      (the woven cross-edge)
```

The `document → file` half is exactly what `core::architecture::project` already uses to
decide component state (glob/path matching via the one project-wide `core::drift::glob_match`).
The `file → symbol` half is new (the code graph). Composing them needs **zero** per-document
frontmatter — it extends the existing glob join down one hop, the same way Spec 002's
component is "bucket of globs" extended to "bucket whose files have symbols". This preserves
the framework's standing non-goal of requiring no new frontmatter (Spec 002 NFR5).

### 3.2 Code nodes & edges

New **code** node kinds (kept separate from document nodes — see §3.4):

| Kind | Meaning | Phase |
|---|---|---|
| `File` | a source file in the on-disk inventory | 1 |
| `Function` / `Method` | a top-level function or method, with its line span | 2 |
| `Module` / `Class` | container scopes (where the language has them) | 2+ |

New **code** edges:

| Edge | Meaning | Phase |
|---|---|---|
| `CONTAINS` | file → function/method (and module/class → member) | 2 |
| `CALLS` | function → function | 3 (deferred) |
| `IMPORTS` | file/module → file/module | 3 (deferred) |

### 3.3 Woven cross-edges (document → code)

The heart of the spec. All point from a governance node to a code node, carry a typed
relation and a **provenance** that records how precise the attribution is:

| Cross-edge | Derived from | Mirrors Spec 002 state |
|---|---|---|
| `MODIFIES` | in-progress Charter declared files / git-modified ∩ code node | `active` / `in-progress` |
| `IMPLEMENTS` | closed Charter + its AILOGs' files ∩ code node | `implemented` |
| `INCURS_DEBT` | open TDE `affects`/related files ∩ code node | `has-debt` |
| `DECIDES` | ADR `api_changes` / "Affected Components" reach ∩ code node | (new) |

`provenance ∈ { file-glob, line-range, explicit }` — `file-glob` means "this doc governs this
file (and, coarsely, all its symbols)"; `line-range` means "this doc's modified-line interval
intersects this function's span" (exact symbol attribution); `explicit` means a doc literally
named `path::symbol`. This mirrors the knowledge graph's `RefKind` instinct of distinguishing
a precise reference from a coarse one (#262).

### 3.4 The document graph stays the audit oracle

The existing `core::graph::Graph` is the regression oracle for `straymark audit` and the
`/api/graph` ≡ `straymark audit` invariant (Spec 001 NFR1). **It is not mutated.** Code nodes
live in a separate code graph; the weave is an **overlay** that references both by id and adds
only the cross-edge list. The bare audit/`/api/graph` path serializes byte-for-byte as today;
the woven view is an opt-in superset (`/api/graph?include=code`).

## 4. Granularity ladder (cheap → precise)

The weave degrades gracefully; each rung is a phase:

| Rung | `document → X` derived from | Cost | Precision |
|---|---|---|---|
| **File-level** | governance file set ∩ `File` nodes (reuse glob match) | ~free (data exists) | "ADR-x governs `auth/login.rs`" |
| **Symbol-level** | modified-line interval ∩ function span | one code-parse pass | "AILOG-x modified `login()`" |
| **Explicit** | a doc names `file.rs::symbol` in `related`/`affects` | free | exact, opt-in |

Where line information is absent (an AILOG with no line column, an unparsed language), the
weave **falls back to file-level for that document** — it never asserts a wrong symbol. This
graceful degradation is the design's safety valve and a hard requirement (§10 NFR2).

## 5. Source of the code graph (contract)

The code graph is built **natively**, reusing the project's own parsing layer, **not** by
consuming an external indexer (`codebase-memory-mcp`) as a dependency or sidecar process. The
decision and its trade-offs are recorded in `ADR-2026-06-26-001`. Consequences carried by
this spec:

- One parser family for the whole project (the `ADR-2026-06-02-001` "no drift" principle
  extended from documents to code).
- No second process, no external SQLite store, no network/IPC surface — Loom stays a single
  read-only loopback binary (Spec 001 §9, NFR4).
- The code-graph builder is **feature-gated** so the default `straymark-core` (and the
  document-only audit path) stays free of any code-parsing dependency.

## 6. Rendering & scale (contract)

The woven graph can be enormous (the motivating kernel example is ~4.8M symbol nodes — it does
not render). The default view is therefore **unchanged**: documents only. Code is an **opt-in,
expandable layer**:

- **Collapse by component.** The Spec 002 architecture model's component globs are exactly the
  aggregation key: render a component node, expand-on-click to its files, then to their
  symbols. The plan view and the woven graph share this drill-down.
- **Level of detail by zoom**; server-side filtering on `/api/graph` (already exists).
- **Never** render "all symbols" as a default. The symbol layer is reached by drilling into a
  component or by following a woven cross-edge from a document.

## 7. API surface (additions to Spec 001 §4 / Spec 002 §7)

| Method | Path | Returns |
|---|---|---|
| GET | `/api/graph?include=code` | the document graph **plus** code nodes + `CONTAINS`/code edges + woven cross-edges (default `include` omits code → byte-identical to today) |
| GET | `/api/code/component/:id` | a component's code subgraph: its files → symbols, collapsed/expandable |
| GET | `/api/node/:id` (extended) | for a code node: the documents that govern it (woven cross-edges, with provenance); for a doc node: the code it reaches |

No new write paths — the server stays read-only (NFR4); the code graph is built in-memory on
the same rebuild cycle as the document graph (FR6 of Spec 001).

## 8. Functional requirements

- **FR1.** Build a code graph (`File` nodes in Phase 1; `Function`/`Method` + `CONTAINS` in
  Phase 2) from the on-disk source inventory, reusing `core::architecture::gather`'s walker
  (`collect_source_files`, the shared `EXCLUDED_DIRS`/`SOURCE_EXTENSIONS`).
- **FR2.** Compute woven cross-edges (`MODIFIES`/`IMPLEMENTS`/`INCURS_DEBT`/`DECIDES`) as a
  **pure function** of (document graph + code graph + per-document file/line provenance), so
  the CLI and the Loom server compute the same weave (the Spec 002 §4 `project` discipline).
- **FR3.** Carry `provenance` on every cross-edge; degrade to file-level when no line info.
- **FR4.** Serve the woven graph behind `/api/graph?include=code`, leaving the default
  (document-only) response byte-identical to today.
- **FR5.** Drill-down: component → files → symbols, collapsed by default (§6).
- **FR6.** Live-update the code graph + weave on the shared watcher cycle (Spec 001 FR6),
  using an incremental per-file cache so unchanged files are not re-parsed.
- **FR7.** A textual companion: `straymark` can answer "what governs this file/symbol?" and
  "what code does this doc touch?" from the same pure weave (the `status --where` precedent).

## 9. Phasing (slots into the shared Loom phasing in `../001-loom-server/plan.md`)

- **C1 — File-level weave, no code parser.** Code `File` nodes from the existing inventory +
  woven cross-edges by reusing globs. Zero new parsing, zero frontmatter. Proves the thesis.
  Ships as a `cli-` increment (pure `core` library + textual companion) then a `loom-0.x`
  release for the visual layer.
- **C2 — Symbol nodes.** `Function`/`Method` + `CONTAINS` via the native parser; symbol-level
  cross-edges by line-range intersection; per-file incremental cache.
- **C3 — `CALLS` / `IMPORTS` (deferred).** A true code dependency graph fused with governance.
  Multi-language, shipped language-by-language; its own milestone.
- **C4 — Viz scaling.** Collapse-by-component, lazy expand, level-of-detail (needed the moment
  C2 lands on any real repo).

## 10. Non-functional requirements

- **NFR1 (oracle intact).** The default `/api/graph` and `straymark audit` outputs are
  byte-for-byte unchanged. Code lives in a separate graph, merged only behind `include=code`.
- **NFR2 (graceful degradation).** No cross-edge ever asserts a symbol it cannot prove; absent
  line/parse info, it degrades **loudly** to file-level (provenance records which).
- **NFR3 (one parser, no drift).** The code graph uses the project's own parsing layer; no
  second indexer with its own grammar set (`ADR-2026-06-26-001`).
- **NFR4 (read-only, loopback, single binary).** No external process, store, or network
  surface beyond the existing loopback server.
- **NFR5 (no new frontmatter).** The weave is derived by composition; adopters add nothing to
  their documents (the file-level rung works with zero annotation).
- **NFR6 (default cost unchanged).** The code-graph dependency is feature-gated; a build/run
  that does not ask for the weave pays nothing for it.
- **NFR7 (consistency).** A document's woven reach is consistent with what Spec 002's
  `project` reports for the component containing those files (the weave is a refinement of the
  same governance state, not a parallel computation).

## 11. Acceptance criteria (definition of done for the Code Weave MVP — C1+C2)

1. `/api/graph?include=code` returns the document graph plus code nodes and woven cross-edges;
   `/api/graph` (no param) is byte-identical to the pre-weave response (NFR1).
2. Selecting an in-progress Charter highlights the **files** (C1) and then the **functions**
   (C2) it modifies, reached through `MODIFIES` cross-edges.
3. Clicking a function shows the documents governing it with provenance (file-glob vs.
   line-range), and a function with no line evidence degrades to a file-level attribution
   rather than a wrong symbol (NFR2).
4. A component in the plan view expands to its files and their functions (drill-down, §6);
   the default graph view still shows documents only (no symbol explosion).
5. A textual query ("what governs `path::symbol`?") and the visual weave give the same answer
   (FR7/NFR7), asserted by a consistency test analogous to
   `where_is_consistent_with_charter_list`.
6. A change to a watched file updates the weave live (< ~1s) without re-parsing unchanged
   files (FR6).

## 12. What this spec changes about Loom's published non-goals

This capability **deliberately overturns** two non-goals shipped in `experiment-loom/README.md`:

- "Use a graph database" — still avoided for the document graph; only revisited for the code
  graph if the §10 NFR6 performance trigger fires (see `plan.md` and CHARTER-01 R2's
  pre-declared SQLite escape hatch). The README's caution stays; this spec narrows it.
- "Require a graph/AST extractor or new frontmatter" — **reversed for AST**: the code graph
  *is* an AST/symbol extractor. The "no new frontmatter" half is **preserved** (the weave is
  by composition, §3.1). The README must be updated in the same PR that lands this spec.

## 13. Out of scope (this spec)

- Consuming `codebase-memory-mcp` (or any external indexer) as a runtime dependency or sidecar
  (`ADR-2026-06-26-001` records why; an *optional* post-graduation enrichment source is not
  excluded but is not this MVP).
- `CALLS`/`IMPORTS` and a full call graph (deferred to C3).
- Editing code or governance from the Loom UI (server stays read-only).
- A new `component:`/`symbol:` frontmatter field (the weave needs none; an optional explicit
  `path::symbol` reference in existing fields is the `explicit` provenance rung, not a new
  field).
- Cross-repository / multi-project weaving.

## 14. Open questions

- Does `arborist-metrics`' public surface expose a function's **end line** (a full span), or
  only its start line + complexity? The C2 line-range intersection needs spans; if only start
  lines are exposed, C2 either consumes arborist's richer internal types or computes spans —
  resolve at C2 start (`plan.md` §3).
- Where does the per-file parse cache live? Not `.straymark/` (Loom never writes there); a
  scratch dir under `~/.straymark/` or `target/`-style state — confirm at C2.
- Should `straymark-core` gain the code-graph behind a default-off cargo feature, or should
  the code-graph builder live in `straymark-loom` (and a thin CLI path) so `core` keeps zero
  code-parsing deps entirely? `plan.md` §2 recommends the feature-gate; confirm at C1.
- Which languages get C3 `CALLS`/`IMPORTS` first — confirm the dogfood + adopter stack order
  (Rust / Go / TypeScript) when C3 starts.
