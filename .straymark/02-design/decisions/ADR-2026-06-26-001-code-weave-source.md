---
id: ADR-2026-06-26-001
title: Loom code weave — source of the code graph (native parser vs. external indexer)
status: draft
created: 2026-06-26
updated: 2026-06-26
agent: claude-opus-4-8-1m
confidence: high
review_required: true
# --- Approval workflow (optional, fill at review time) ---
# reviewed_by: <reviewer-id>
# reviewed_at: YYYY-MM-DD
# review_outcome: approved
risk_level: medium
eu_ai_act_risk: not_applicable
iso_42001_clause: []
alternatives_documented: [codebase-memory-mcp-as-dependency, code-graph-in-loom-only, no-feature-gate]
api_changes: []
tags: [loom, experimental, code-weave, knowledge-graph, tree-sitter, architecture]
related: [CHARTER-02-code-weave, ADR-2026-06-02-001]
supersedes: []
---

# ADR: Loom code weave — source of the code graph (native parser vs. external indexer)

## Status

draft

**Note**: This document was created by an AI agent and requires human review.

> **Immutability Rule**: Once an ADR reaches `accepted` status, it MUST NOT be modified. If
> the decision changes, create a new ADR with `supersedes: ADR-2026-06-26-001`.

## Context

Loom today holds two disjoint graphs: the **document** knowledge graph (Spec 001) and the
**architecture plan** (Spec 002), whose only bridge to code is **file-level** glob matching —
a component is a bucket of globs, nothing inside it is a node. Spec 003 ("code weave") fuses
**code symbols** into the same fabric so a function sits next to the ADR that decided it, the
Charter modifying it, and the TDE flagging its debt — *"conocimiento extendido"*.

The motivating external reference is **`codebase-memory-mcp`** (DeusData, MIT): it parses a
repo with tree-sitter (≈158 grammars) into a symbol graph in SQLite and exposes it to agents
over MCP, reporting large token savings for structural queries (its own arXiv preprint
reports ~10× fewer tokens at a measured **−9-point answer-quality** trade-off, 83% vs 92% —
not a free lunch). It proves the *shape of the idea* (code as a navigable graph) but indexes
the half StrayMark does **not** care about in isolation; StrayMark's value is the **weave with
governance**.

This ADR records the one decision in Spec 003 with repo-wide consequences: **where the code
graph comes from** — built natively in our own parsing layer, or sourced from an external
indexer such as `codebase-memory-mcp`. It extends `ADR-2026-06-02-001`, whose central
rationale ("one parser → drift structurally impossible") now applies to code as well as
documents.

## Decision

We will:

1. **Build the code graph natively**, reusing the project's existing tree-sitter layer
   (`arborist-metrics`, already a CLI dependency for `straymark analyze`), not by consuming
   `codebase-memory-mcp` (or any external indexer) as a runtime dependency or sidecar process.
2. **Keep the code graph in `straymark-core` behind a default-off `codegraph` cargo feature**,
   with `arborist-metrics` enabled only by that feature. The default `core` build, the
   document knowledge graph, and `straymark audit` stay free of any code-parsing dependency.
3. **Compose the weave, never annotate.** The `document → symbol` cross-edge is the
   composition `document → file` (already extracted by `core::architecture::gather`) ∘
   `file → symbol` (the new code graph). This requires **no new frontmatter** (it preserves
   Spec 002 NFR5) and degrades to file-level when no line/parse evidence exists.
4. **Leave `core::graph` (the audit oracle) unmutated.** Code nodes live in a separate
   `core::codegraph`; the weave is an overlay merged only behind `/api/graph?include=code`, so
   the bare `straymark audit` / `/api/graph` output stays byte-for-byte unchanged.
5. **Defer `CALLS`/`IMPORTS`** (the call graph) to a later milestone (C3), shipped
   language-by-language; the file-level weave (C1) and function nodes (C2) come first.

## Alternatives Considered

### 1. Consume `codebase-memory-mcp` as a dependency / sidecar
- **Description**: Run the external indexer (or link its engine), read its SQLite symbol
  graph, and weave governance onto it.
- **Pros**: 158 languages out of the box; mature, fast, sub-ms structural queries; no symbol-
  extraction code to write; an arXiv-backed design.
- **Cons**: Adds a **second process + external SQLite store + MCP/IPC surface** to a tool
  whose entire posture is "one read-only loopback binary" (Spec 001 §9, NFR4; `ADR-2026-06-02-001`
  §6). Introduces a **second parser** with its own grammar set — exactly the drift
  `ADR-2026-06-02-001` was written to prevent, now between "what *it* calls a symbol" and what
  our analysis does. Contradicts "single static binary, zero deps" for Loom and drags a daemon
  lifecycle into `straymark loom serve`.
- **Why not**: The whole point of the weave is that it shows the *same* code truth the rest of
  StrayMark would compute. One parser family + one binary + loopback-only outweigh the
  convenience of a ready-made 158-language index. It remains valuable as **conceptual
  reference** and a *possible* optional post-graduation enrichment source — not the v0 engine.

### 2. Put the code graph in `straymark-loom` only (keep `core` parser-free forever)
- **Description**: Build the code graph and weave inside the Loom crate; `core` never gains a
  code-parsing dependency, not even optional.
- **Pros**: `core` stays minimal; the CLI's `cargo publish` is untouched.
- **Cons**: The CLI's textual companion (Spec 003 FR7, "what governs `path::symbol`?") and the
  consistency test (acceptance §5) both need the **pure** weave in `core`; splitting the pure
  function across crates duplicates it or couples `cli` → `loom`.
- **Why not**: A **default-off cargo feature** gives the same "zero default cost" as keeping it
  out of `core`, without splitting the pure function. `core` stays parser-free unless a
  consumer asks for `codegraph`. Revisit only if the feature graph proves awkward.

### 3. Add `arborist-metrics` to `core` unconditionally (no feature gate)
- **Description**: Make the code graph always available in `core`.
- **Pros**: Simplest dependency story; no `--features` juggling.
- **Cons**: Forces a tree-sitter stack into **every** `core` consumer, including the lean
  document-only CLI paths and `straymark audit`; inflates build time and the `opt-level=z` CLI
  footprint for users who never weave.
- **Why not**: Violates Spec 003 NFR6 ("default cost unchanged"). The feature gate is cheap
  insurance; CI keeps a `--no-default-features` build green to enforce it.

## Consequences

### Positive
- One parser family → the code graph cannot silently disagree with the rest of StrayMark
  (the `ADR-2026-06-02-001` drift argument, extended to code).
- Loom stays a single read-only loopback binary — no daemon, no external store, no IPC.
- No new frontmatter for adopters (the weave is by composition); the file-level rung works
  with zero annotation.
- The audit oracle is provably untouched (code merged only behind `include=code`).
- Default builds pay nothing (feature-gated); `--no-default-features` stays viable.

### Negative
- We write **symbol-extraction code ourselves** rather than getting 158 languages free; the
  native parser covers ~12 languages, and `CALLS`/`IMPORTS` must be authored per-language (C3).
- `arborist-metrics`' public surface may expose only a function's start line + complexity, not
  a full span; the C2 line-range intersection may need its richer internal types or an
  approximated span (Spec 003 §14).
- A new (optional) code-parsing dependency enters `core`, gated but present in `Cargo.toml`.

### Neutral
- `codebase-memory-mcp` is not adopted now but explicitly left open as a *possible* optional
  enrichment source after graduation (not a v0 dependency).

### Quality Impact Assessment

| Quality Characteristic (ISO 25010:2023) | Impact | Description |
|-----------------------------------------|--------|-------------|
| Functional Suitability | + | Governance and code become one navigable graph (the headline capability) |
| Performance Efficiency | ~ | Parsing per FS event is the watch-item; mitigated by per-file content-hash cache (C2) |
| Compatibility | + | One parser family; the audit oracle and document graph stay byte-identical |
| Maintainability | + | Pure `weave` mirrors the proven `project` split; no second indexer to track |
| Security | + | No new process/store/network surface; Loom stays loopback-only read-only |
| Flexibility | ~ | Native means per-language work for CALLS/IMPORTS, but full control of the model |

## Affected Components

| Component | Type of Change | Impact |
|-----------|----------------|--------|
| `straymark-core` (`core/`) | New `codegraph` + `weave` modules (feature-gated); `architecture::gather` refactor | High |
| `straymark-cli` (`cli/`) | New weave-facing textual path; `audit`/`validate` unchanged | Low |
| `straymark-loom` (`experiment-loom/`) | Code layer + weave endpoints + drill-down | Medium |
| `experiment-loom/README.md` | Non-goals reversed (AST extractor allowed; no-new-frontmatter kept) | Low |

## Implementation Plan

1. C1 — file-level weave (no parser): `codegraph` (files only) + `weave` + the `WeaveLinks`
   refactor + CLI textual companion; then a `loom-0.x` file layer.
2. C2 — symbol nodes via `arborist-metrics` + line-range intersection + per-file cache.
3. C3 — `CALLS`/`IMPORTS` (deferred), language-by-language.
4. C4 — viz scaling (collapse-by-component).

(Detail in `experiment-loom/specs/003-code-weave/plan.md` and `tasks.md`.)

## Success Metrics

- After C1, `/api/graph` (no `include=code`) and `straymark audit` are byte-for-byte unchanged
  (oracle intact), while `/api/graph?include=code` adds file nodes + woven cross-edges.
- A `--no-default-features` build of `straymark-core` compiles with **no** code-parsing
  dependency.
- The CLI textual weave and the Spec 002 `project` agree on a fixture corpus (consistency
  test).

## Validation Criteria

| Metric | Target Value | Measurement Method | Timeline |
|--------|-------------|-------------------|----------|
| Audit-oracle regression | 0 | diff `/api/graph` & `straymark audit` vs pre-weave | C1 |
| Default-build parser deps | 0 | `cargo tree --no-default-features` shows no tree-sitter | C1 |
| Weave↔projection consistency | exact match | fixture test (analogous to `where_is_consistent_with_charter_list`) | C1 |
| Symbol attribution correctness | file-level fallback on no line info | unit test on degraded provenance | C2 |

## References

- `experiment-loom/specs/003-code-weave/spec.md`, `plan.md`, `tasks.md`
- `experiment-loom/CHARTER-02-code-weave.md`
- `ADR-2026-06-02-001` (Loom stack — the "one parser, no drift" principle this extends)
- Reused in-repo infra: `core/src/architecture/{projection,gather}.rs`, `core/src/graph.rs`,
  `core/src/ailog.rs`, `cli/src/analysis_engine.rs` (existing `arborist-metrics` usage)
- Inspiration (idea only, not a dependency): `codebase-memory-mcp` (DeusData) — code as a
  navigable graph + MCP traversal

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-26 | claude-opus-4-8-1m | Initial creation (draft, pending human review) |

<!-- Template: StrayMark | https://strangedays.tech -->
