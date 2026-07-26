# Implementation Plan 003 — Loom Code Weave

> **SpecKit artifact — the HOW.** Derives from `spec.md` (the WHAT) and feeds `tasks.md` (the
> ordered work). The source-of-the-code-graph decision is recorded as a dogfood ADR:
> `.straymark/02-design/decisions/ADR-2026-06-26-001-code-weave-source.md`. The work block is
> `../../CHARTER-02-code-weave.md`. Status: **draft / experimental**.

## 1. Architecture at a glance

```
        .straymark/ or docs/ (Markdown)        source tree (.rs/.ts/.py/…)
                  │ read-only                          │ read-only
   core::document │ parse                core::codegraph│ walk + parse (feature-gated)
                  ▼                                     ▼
        ┌───────────────────┐                 ┌───────────────────┐
        │ core::graph::Graph │                 │ core::codegraph    │
        │ (DOC nodes/edges,  │                 │ (FILE/FN nodes,    │
        │  the AUDIT ORACLE) │                 │  CONTAINS edges)   │
        └─────────┬─────────┘                 └─────────┬─────────┘
                  │                                     │
                  │   core::weave::weave(doc, code, links)  ← PURE, zero I/O
                  └──────────────────┬──────────────────┘
                                     ▼
                         ┌───────────────────────┐
                         │ WovenGraph (overlay):  │
                         │ doc + code by ref      │
                         │ + cross-edges w/ prov. │
                         └───────────┬───────────┘
                  serves             │  /api/graph?include=code   (default omits code)
   straymark-loom ───────────────────┘   straymark CLI: textual "what governs X?"
```

The split mirrors the proven Spec 002 shape: a **pure** core function (`weave`, like
`project`) fed by an **impure** gatherer (`weave_gather`, like `architecture::gather`).

## 2. Where the code lives (the load-bearing decision)

`straymark-core` today has **no** code-parsing dependency; `arborist-metrics` is an
**optional, CLI-only** dep behind the `analyze` feature (`cli/Cargo.toml`). The weave must not
force a parser into the default `core` build (NFR6). Plan:

```
core/
  src/
    codegraph/                # NEW — the code half (feature-gated: `codegraph`)
      mod.rs
      model.rs                #   CodeNode{kind,path,range,language}, CodeEdge, CodeGraph
      build.rs                #   PURE: CodeGraph::build(files: &[FileSymbols]) -> CodeGraph
      gather.rs               #   IMPURE: walk (reuse architecture::gather) + parse → FileSymbols
    weave.rs                  # NEW — PURE: weave(doc, code, WeaveLinks) -> WovenGraph
    graph.rs                  # UNCHANGED shape (audit oracle); maybe add NodeKind only if a
                              #   merged serialization needs to disambiguate doc vs code
    architecture/
      gather.rs               # MODIFIED — retain doc_id→(path, Option<line_range>, provenance)
                              #   instead of flattening into anonymous GovernanceState buckets
      projection.rs           # project() re-expressible over the weave (single source of truth)
```

- **`codegraph` is a default-off cargo feature** in `straymark-core`. `arborist-metrics`
  becomes an optional core dep enabled only by that feature. A build that does not ask for the
  weave (the CLI's document-only paths, `straymark audit`) pulls in **no** parser (NFR6).
- **`straymark-loom` enables `codegraph`**; the CLI enables it only for the weave-facing
  command(s). `straymark audit` / `validate` / the document graph stay parser-free.
- **Alternative considered:** put the whole code graph in `straymark-loom` and keep `core`
  entirely parser-free. Rejected for C1+ because the CLI's textual companion (FR7) and the
  consistency test (acceptance §5) both need the weave in `core`; a feature gate gives the
  same "zero default cost" without splitting the pure function across crates. Revisit only if
  the feature graph proves awkward (`spec.md` §14).

## 3. The code graph (`core::codegraph`)

- **Model.** `CodeNode { id, kind: File|Function|Method|Module|Class, path: String, range:
  Option<(u32,u32)>, language: String }`. Stable ids: `path` for files, `path#start-name` for
  symbols (deterministic, declaration order — the same determinism contract as `graph.rs`).
  `CodeEdge { kind: Contains|Calls|Imports, from, to }`.
- **Build (pure).** `CodeGraph::build(files: &[FileSymbols]) -> CodeGraph` — deterministic,
  no I/O. `FileSymbols { path, language, symbols: Vec<SymbolSpan> }`.
- **Gather (impure).** Reuse `core::architecture::gather::collect_source_files` (the one
  walker, the shared `EXCLUDED_DIRS`/`SOURCE_EXTENSIONS`, the `ScanConfig` from #279) to find
  files; for C1 that is the whole code graph (files only). For C2, parse each file:
  - **Parser:** `arborist-metrics` (already the project's tree-sitter layer; 17 extensions /
    12 languages). It yields per-function `name` + start `line` + complexity. **Open
    question (spec §14, resolve at C2):** whether its public surface exposes a function's
    **end line**. If not, either (a) consume arborist's richer internal `FunctionMetrics` if
    it carries a span, or (b) derive an approximate span (next-symbol-start − 1) — good enough
    for line-range intersection, with file-level fallback when ambiguous (NFR2).
  - **Incremental cache (C2):** key parsed `FileSymbols` by `(path, content-hash)`; on a
    `notify` event only changed files are re-parsed. The cache is plain state, **not** a graph
    DB, and lives outside `.straymark/` (scratch under `~/.straymark/` or `target/`-style).

## 4. The weave (`core::weave` — the heart)

- **`WeaveLinks` (impure input).** A **refactor of `core::architecture::gather`**: instead of
  flattening into the anonymous `GovernanceState { active_charter_files: Vec<String>, … }`
  buckets, retain provenance: `doc_id → Vec<(path, Option<(u32,u32)> line_range, kind)>` where
  `kind ∈ {Modifies, Implements, IncursDebt, Decides}`. The existing extractors already
  compute the paths (active/closed charter declared files, AILOG `## Modified Files`, open-TDE
  files, git-modified); the change is to **keep which document owns which path** (and its line
  column, already tokenized by `core::ailog::parse_modified_files` but currently discarded).
- **`weave(doc: &Graph, code: &CodeGraph, links: &WeaveLinks) -> WovenGraph` (pure).** For
  each `(doc_id, path, line_range, kind)`: find the matching `File` node (path match); emit a
  file-level cross-edge `kind` with `provenance: file-glob`. If `line_range` is present and the
  code graph has `Function` nodes for that file, intersect the interval with each function's
  span and emit symbol-level cross-edges with `provenance: line-range`; if none intersect,
  keep the file-level edge (NFR2). Deterministic; zero I/O — the `project()` contract, so the
  CLI and Loom compute identical weaves.
- **`WovenGraph` shape.** Embed-and-overlay: hold `&Graph` and `&CodeGraph` by reference (or
  their snapshots) and add only the cross-edge list. This keeps `graph.rs` untouched (oracle
  intact) and makes the cross-edges a separately-toggleable layer. Serialization for
  `include=code` merges the three at the API boundary; without it, the bare `Graph`
  serializes exactly as today (NFR1).
- **`project()` over the weave.** Re-express a component's Spec 002 state as an aggregate of
  its files' cross-edges (a component is `active` iff some owned file has a `Modifies` edge,
  etc.). One source of truth instead of two parallel matchers; guarantees `spec.md` NFR7.

## 5. Loom server & frontend

- **Server (`experiment-loom/src/`).** Enable the `codegraph` feature. Extend the snapshot to
  build `CodeGraph` + `weave` on the rebuild cycle (Spec 001 watcher). New/extended endpoints
  per `spec.md` §7 (`/api/graph?include=code`, `/api/code/component/:id`, extended
  `/api/node/:id`). All read-only; local `serde::Serialize` view types keep the `core` surface
  stable (the A2.1 pattern). The watcher's relevance widens to source extensions for the code
  graph; the document graph still rebuilds only on `.md` (the A2.2 pattern).
- **Frontend (`experiment-loom/web/`).** The document graph stays the default. Code is an
  opt-in layer: a toggle adds `File`/symbol nodes; component drill-down (reusing the Spec 002
  plan view's component cells) expands to files → symbols. **Collapse-by-component is
  mandatory before any symbol layer ships** (`spec.md` §6 / C4) — never load all symbols.

## 6. CLI integration

- A thin weave-facing path (C1): `straymark` answers "what governs this file/symbol?" /
  "what code does this doc touch?" from `core::weave`, reusing the `where_view`/`common`
  resolution. This is the FR7 textual companion and the home of the acceptance §5 consistency
  test. It enables the `codegraph` feature only for that path; `audit`/`validate` stay
  parser-free.

## 7. Phasing (each milestone = a releasable increment)

- **C1 — File-level weave (cheapest valuable slice).** `core::codegraph` (files only) +
  `core::weave` (file-level cross-edges) + the `WeaveLinks` refactor of `architecture::gather`
  + CLI textual companion. **No tree-sitter, no new frontmatter.** Ships as a `cli-`/`core-`
  increment; then a `loom-0.x` release adds the file layer + cross-highlight to the graph.
- **C2 — Symbol nodes.** Parse via `arborist-metrics`; `Function`/`Method` + `CONTAINS`;
  symbol-level cross-edges by line-range intersection; per-file incremental cache. `loom-0.x`.
- **C3 — `CALLS`/`IMPORTS` (deferred).** Extend the parser layer to emit references; the
  multi-language tax is paid language-by-language (Rust/Go/TS first). Its own milestone.
- **C4 — Viz scaling.** Collapse-by-component, lazy expand, LOD. Lands with/just after C2.

## 8. Risks

- **R1 — Default-build cost / dependency creep.** Pulling a tree-sitter stack into `core`
  would bloat every consumer. Mitigation: default-off `codegraph` feature (§2); CI must keep a
  `--no-default-features` build green so the parser-free path can't regress.
- **R2 — Rebuild cost at scale.** Walking + parsing a large tree per FS event blows the
  sub-second budget. Mitigation: per-file content-hash cache + incremental re-parse (C2);
  CHARTER-01 R2's **pre-declared** trigger to revisit SQLite (recursive CTE) — reuse that
  escape hatch, don't invent a new one.
- **R3 — Node explosion in the viz.** ~4.8M symbol nodes do not render. Mitigation
  (mandatory, C4): collapse-by-component as the aggregation key; default view is documents
  only; symbols reached only by drill-down or cross-edge.
- **R4 — Symbol-attribution false precision.** Line numbers drift when a file is reformatted
  after an AILOG was written. Mitigation: stamp `provenance` and **degrade loudly** to
  file-level rather than assert a wrong symbol (NFR2); surface coarse-vs-exact in the UI.
- **R5 — Oracle regression.** Any change to `graph.rs` Node/Edge serialization risks the
  `/api/graph ≡ straymark audit` invariant. Mitigation: code nodes stay in `CodeGraph`, merged
  only behind `include=code`; the bare audit path stays byte-identical; an oracle test guards
  it.
- **R6 — Multi-language gaps.** The parser covers ~12 languages; `CALLS`/`IMPORTS` extraction
  is per-language. Mitigation: file-level weave (C1) is language-agnostic and works
  everywhere immediately; symbol/call layers ship per-language with explicit
  "unsupported language → file-level only" degradation.
- **R7 — Drift between code graph and document graph.** Two graphs rebuilt at different
  cadences can disagree. Mitigation: build both from the **same** `notify` rebuild cycle and
  the same `ScanConfig`; `weave` is a pure function of both snapshots, so it cannot observe a
  torn state.

## 9. References

- `spec.md` (this feature's WHAT) and `tasks.md` (ordered work).
- `.straymark/02-design/decisions/ADR-2026-06-26-001-code-weave-source.md` (native vs. external code graph).
- `../../CHARTER-02-code-weave.md` (the work-block Charter; `originating_spec` → this spec).
- Prior art in-repo: `core/src/architecture/{projection,gather}.rs` (the pure-function +
  impure-gatherer split the weave mirrors), `core/src/graph.rs` (the audit oracle),
  `core/src/ailog.rs` (`parse_modified_files` — file+line provenance source),
  `cli/src/analysis_engine.rs` (the existing `arborist-metrics` usage).
- Inspiration (idea only, not a dependency): `codebase-memory-mcp` (DeusData) — code as a
  navigable graph; see `ADR-2026-06-26-001` §Alternatives for why we do not consume it.
