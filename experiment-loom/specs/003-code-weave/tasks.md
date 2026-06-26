# Tasks 003 — Loom Code Weave

> **SpecKit artifact — the ordered, checkable work.** Derived from `spec.md` (the WHAT) and
> `plan.md` (the HOW). Each task is verifiable; phases map to releasable increments. This is
> the **ex-ante skeleton for C1** (the file-level weave); C2/C3/C4 tasks get refined when
> those tracks start (the same way Spec 002's A2/A3 were stubs until they began). FR/NFR ids
> reference `spec.md`. The source decision is `docs/decisions/ADR-2026-06-26-001-code-weave-source.md`.
> Status: **draft**.

## Track map (from `spec.md` §9)

- **C1 — File-level weave.** `core::codegraph` (files only) + `core::weave` (file-level
  cross-edges) + the `WeaveLinks` refactor + CLI textual companion. No tree-sitter, no new
  frontmatter. Ships as a `core-`/`cli-` increment, then a `loom-0.x` file layer. **← this file.**
- **C2 — Symbol nodes.** `Function`/`Method` + `CONTAINS` via `arborist-metrics`; symbol-level
  cross-edges by line-range intersection; per-file incremental cache. A `loom-0.x` release.
- **C3 — `CALLS`/`IMPORTS`.** A true code dependency graph fused with governance; per-language.
- **C4 — Viz scaling.** Collapse-by-component, lazy expand, level-of-detail.

## Scope decisions baked into C1 (settled at spec time)

- **The weave is by composition, not annotation.** `document → file → symbol`; the
  `document → file` half already exists in `core::architecture::gather`. C1 needs **no new
  frontmatter** and **no code parser** — `File` nodes come from the existing on-disk inventory
  (`spec.md` §3.1, §4).
- **The document graph (`core::graph`) is the audit oracle and is not mutated.** Code lives in
  a separate `core::codegraph`; the weave is an overlay merged only behind `include=code`
  (`spec.md` §3.4, NFR1).
- **`codegraph` is a default-off cargo feature.** The default `core` build and `straymark
  audit` stay free of any code-parsing dependency (`spec.md` NFR6; `plan.md` §2). C1 adds the
  feature even though it pulls no parser yet, so C2 only flips a dep, not the architecture.
- **Native code graph, not `codebase-memory-mcp`.** Decision + trade-offs in
  `ADR-2026-06-26-001` (`spec.md` §5).
- **Dogfood home:** as with Specs 001/002, this source repo has no root `.straymark/`; weave
  artifacts/queries dogfood against `docs/` + `experiment-loom/architecture/`.

---

## C1.0 — `WeaveLinks` refactor of `architecture::gather` (own PR, no tag — de-risk)

> The M0/A1.0-style de-risking step: a structure-preserving refactor gated on the unchanged
> CLI + `status --where` test suites. Today `build_governance_state` flattens doc→file
> provenance into anonymous `GovernanceState` `Vec<String>` buckets; C1.0 makes it **also**
> emit `doc_id → Vec<(path, Option<line_range>, kind)>` without changing existing behaviour.

- [ ] T0.1 — In `core/src/architecture/gather.rs`, add a `WeaveLinks` producer beside
  `build_governance_state` that retains, per source document, the `(path, line_range, kind)`
  it contributed — reusing the same extractors (active/closed charter declared files via
  `charter_files::parse_files_to_modify`, closed-AILOG + `core::ailog::parse_modified_files`,
  open-TDE files, git-modified). `kind ∈ {Modifies, Implements, IncursDebt, Decides}`.
- [ ] T0.2 — `core::ailog::parse_modified_files` currently tokenizes the `## Modified Files`
  rows; surface the **line column** (today discarded) as `Option<(u32,u32)>` so C2 has it.
  C1 ignores the line range (file-level only); the data just stops being dropped.
- [ ] T0.3 — **Regression gate green:** `build_governance_state` output and the existing
  `status --where` consistency tests are byte-for-byte unchanged (the new producer is
  additive). `cargo test --workspace` + `cargo clippy -p straymark-core` clean.

## C1.1 — `core::codegraph` (files only) + `core::weave` (file-level) — library, test-gated

> The pure heart of C1. No user-facing command yet; lands as `core` lib code behind the
> default-off `codegraph` feature + unit tests against hand-authored fixtures. FR1, FR2, FR3.

- [ ] T1.1 — Add the `codegraph` cargo feature to `core/Cargo.toml` (default-off; no deps yet).
  `core/src/codegraph/model.rs`: `CodeNode { id, kind: CodeNodeKind (File|Function|Method|
  Module|Class), path, range: Option<(u32,u32)>, language }`, `CodeEdge { kind: Contains|
  Calls|Imports, from, to }`, `CodeGraph` with deterministic ids (`path` for files,
  `path#start-name` for symbols) and input-order stability (mirrors `graph.rs`). (FR1)
- [ ] T1.2 — `core/src/codegraph/build.rs`: **pure** `CodeGraph::build(files: &[FileSymbols])
  -> CodeGraph`; for C1, `FileSymbols` carries only `{ path, language }` → one `File` node
  each, no symbols. Deterministic (test: equal inputs → equal graph). (FR1)
- [ ] T1.3 — `core/src/codegraph/gather.rs`: **impure** `collect(root, &ScanConfig)` reusing
  `core::architecture::gather::collect_source_files` (one walker, shared
  `EXCLUDED_DIRS`/`SOURCE_EXTENSIONS`, #279 `ScanConfig`) → `Vec<FileSymbols>` (files only). (FR1)
- [ ] T1.4 — `core/src/weave.rs`: **pure** `weave(doc: &Graph, code: &CodeGraph, links:
  &WeaveLinks) -> WovenGraph`. For each `(doc_id, path, _line, kind)`: match the `File` node by
  path; emit a cross-edge `kind` with `provenance: file-glob`. (C1 emits only file-level; the
  line-range branch is C2.) `WovenGraph` embeds `&Graph` + `&CodeGraph` by reference and adds
  only the cross-edge list (`graph.rs` untouched — oracle intact). Deterministic, zero I/O. (FR2, FR3, NFR1)
- [ ] T1.5 — Re-express `architecture::projection::project` over the weave (a component's state
  = aggregate of its owned files' cross-edges) so the weave and the plan view share one source
  of truth; assert the re-expressed `project` equals the current one on the existing fixtures. (NFR7)
- [ ] T1.6 — Unit tests in `core` (inline `#[cfg(test)]`, no on-disk fixtures): each cross-edge
  kind, file-path matching, `provenance: file-glob` stamping, determinism, and an **oracle
  test** asserting the bare `Graph` serialization is unchanged by the presence of the weave
  (NFR1). `cargo test --workspace` + `cargo test -p straymark-core --features codegraph` +
  `cargo build -p straymark-core --no-default-features` all green; clippy clean. (NFR1, NFR6)

## C1.2 — CLI textual companion ("what governs X?") — FR7, acceptance §5

> The headline value of C1 as a `cli-` increment: the weave answers governance↔code questions
> textually before any new pixels (the `status --where` precedent). Enables `codegraph` only
> for this path; `audit`/`validate` stay parser-free.

- [ ] T2.1 — A `straymark` path (subcommand/flag, named at implementation time) that builds the
  document graph + file-level code graph + weave and answers: "what governs `<path>`?" (the
  documents reaching it, with provenance) and "what code does `<doc-id>` touch?" (its files).
  Reuses `where_view`/`common` root resolution. (FR7)
- [ ] T2.2 — **Consistency gate (NFR7):** a test analogous to
  `where_is_consistent_with_charter_list` asserting, on one fixture corpus, that a document's
  woven file reach equals the files Spec 002's `project` attributes to the component(s) holding
  them — the visual and textual weave cannot disagree. (acceptance §5)
- [ ] T2.3 — Docs: new row(s) in `CLAUDE.md` command table + `docs/adopters/CLI-REFERENCE.md`
  EN/es/zh-CN (marked EXPERIMENTAL), versioning tables bumped. `cargo test --workspace` green.

## C1.3 — Loom file layer + cross-highlight — `loom-0.x` (FR4, FR5, FR6)

> The visual half of C1: a `loom-0.x` release. The document graph stays the **default** view;
> code is an opt-in `File` layer. (Symbol drill-down is C2; collapse-by-component hardens in C4.)

- [ ] T3.1 — Server: enable `codegraph`; build `CodeGraph` + `weave` on the rebuild cycle;
  serve `/api/graph?include=code` (default omits code → byte-identical to today, NFR1) and the
  extended `/api/node/:id` (a doc's reached files; a file's governing docs + provenance).
  Read-only; local `serde::Serialize` view types (A2.1 pattern). (FR4)
- [ ] T3.2 — Watcher: widen relevance to source extensions for the code graph (the A2.2
  pattern); the document graph still rebuilds only on `.md`; broadcast a weave signal on
  settled source changes. (FR6)
- [ ] T3.3 — Frontend: a "show code (files)" toggle adds `File` nodes + cross-edges; selecting
  a Charter/ADR/TDE highlights the files it reaches; selecting a file highlights its governing
  documents. Default view unchanged (documents only). Localized (en/es/zh-CN). (FR5)
- [ ] T3.4 — Verify on the dogfood (this repo's `docs/` + source tree) and Sentinel; acceptance
  §1 (`/api/graph` byte-identical without `include=code`), §2 file half, §3 file-level
  attribution. `tsc` + `vite build` clean.

## C1.4 — Acceptance + release

- [ ] T4.1 — Acceptance (spec §11, C1 subset): §11.1 ✓, §11.2 file half ✓, §11.3 file-level
  attribution ✓, §11.5 consistency gate ✓, §11.6 live update ✓ (symbol halves of §11.2/3/4
  are C2; §11.4 drill-down hardens in C4).
- [ ] T4.2 — **Reverse the published non-goals (spec §12):** update `experiment-loom/README.md`
  Non-goals — the "no AST/graph extractor" item is overturned (the "no new frontmatter" half
  preserved) — **in the same PR** that lands the visual layer, so the shipped doc never lies.
- [ ] T4.3 — Bump `core` + (if touched) `cli`; `cargo check --workspace` refreshes
  `Cargo.lock`; root `CHANGELOG.md` entry; `experiment-loom/CHANGELOG.md` for the `loom-0.x`.
  PR → merge → tag. Dogfood AILOG per increment (`risk_level`, `review_required`).
- [ ] T4.4 — Update the Loom memory note (C1 shipped; next = C2 symbol nodes).

---

## C2 — Symbol nodes (preview — refined when the track starts)

- [ ] C2.0 — Enable `arborist-metrics` under the `codegraph` feature in `core`; resolve
  `spec.md` §14 (does the public surface expose a function's **end line**? if not, consume the
  richer internal type or approximate the span). Keep `--no-default-features` green.
- [ ] C2.1 — `codegraph::gather` parses each file → `Function`/`Method` nodes (+ `Module`/
  `Class` where available) + `CONTAINS` edges; per-file `(path, content-hash)` cache.
- [ ] C2.2 — `weave`: when a cross-edge has a line range, intersect with function spans → emit
  symbol-level cross-edges (`provenance: line-range`); keep the file-level edge when no span
  intersects (NFR2).
- [ ] C2.3 — Loom: component → files → symbols drill-down; symbol detail panel (governing docs
  + provenance). Symbol layer never default-loaded (depends on C4 collapse).

## C3 — `CALLS` / `IMPORTS` (preview)

- [ ] C3.x — Extend the parser layer to emit call/import references; ship language-by-language
  (Rust/Go/TS first); "unsupported language → file-level only" degradation.

## C4 — Viz scaling (preview)

- [ ] C4.x — Collapse-by-component as the aggregation key; lazy expand; level-of-detail by
  zoom; server-side filtering. Mandatory before any real-repo symbol view.
