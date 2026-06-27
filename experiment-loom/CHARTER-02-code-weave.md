---
charter_id: CHARTER-02-code-weave
status: draft
effort_estimate: L
trigger: "An external code-indexing MCP (codebase-memory-mcp) surfaced the idea of code-as-a-navigable-graph; Loom already builds a document knowledge graph and a glob-only architecture plan, but code and governance never touch. The opportunity is to fuse them into one woven graph ('conocimiento extendido') — the half no code indexer covers."
originating_spec: experiment-loom/specs/003-code-weave/spec.md
related: [CHARTER-01-loom-server, ADR-2026-06-26-001]
---

# Charter: Build Loom Code Weave — fuse code symbols into the governance graph

> **Status (mirrored from frontmatter — source of truth is above):** draft. Effort: L.
>
> **Origin:** SpecKit spec `experiment-loom/specs/003-code-weave/spec.md`. Source-of-the-code-
> graph decision recorded in `docs/decisions/ADR-2026-06-26-001-code-weave-source.md`.
> Successor to `CHARTER-01-loom-server` (kept separate — see Context).

## Context

Loom has two graphs that never touch: the **document** knowledge graph (Spec 001) and the
**architecture plan** (Spec 002), whose only bridge to code is **file-level** glob matching
(a component is a bucket of globs; nothing inside it is a node). This Charter is the bounded,
multi-session work block that builds the **code weave** (Spec 003): code symbols become
first-class nodes in the same fabric as the governance documents, so a function sits next to
the ADR that decided it, the Charter modifying it, and the TDE flagging its debt —
*"conocimiento extendido"*.

The weave is built by **composition** (`document → file → symbol`), reusing the
`document → file` provenance `core::architecture::gather` already computes, so it needs **no
new frontmatter**. The code graph is built **natively** (the project's own tree-sitter layer),
not by consuming the external `codebase-memory-mcp`, per `ADR-2026-06-26-001`. It is
**experimental (v0/N=1)** and continues Loom's opt-in, loopback-only, read-only posture.

**Why a new Charter, not an amendment to CHARTER-01.** CHARTER-01 is `in-progress` toward a
graduation gate declared *ex-ante* (M1–M3 + A1–A2, N=2). This work **reverses two non-goals**
CHARTER-01 published ("no AST/graph extractor", graph-DB caution) — folding that into
CHARTER-01 would muddy its gate. CHARTER-02 owns the reversal explicitly and carries its own
scope and graduation criteria.

## Scope

**In scope:**

1. **C1 — File-level weave (no code parser).** `core::codegraph` (`File` nodes from the
   existing on-disk inventory) + `core::weave` (pure `weave(doc, code, links)` emitting
   file-level `MODIFIES`/`IMPLEMENTS`/`INCURS_DEBT`/`DECIDES` cross-edges with `provenance`) +
   the `WeaveLinks` refactor of `core::architecture::gather` (retain `doc_id → files`) + a CLI
   textual companion ("what governs `<path>`?" / "what code does `<doc>` touch?"). `codegraph`
   is a **default-off cargo feature**; `core`'s default build and `straymark audit` stay
   parser-free. Ships as a `core-`/`cli-` increment, then a `loom-0.x` file layer + cross-
   highlight. **No new frontmatter, no tree-sitter yet.**
2. **C2 — Symbol nodes.** `Function`/`Method` (+ `Module`/`Class` where available) +
   `CONTAINS` via `arborist-metrics` (enabled under `codegraph`); symbol-level cross-edges by
   line-range intersection (modified-line interval ∩ function span), degrading to file-level
   when no span (NFR2); per-file `(path, content-hash)` incremental cache. A `loom-0.x` release.
3. **C3 — `CALLS`/`IMPORTS` (deferred).** Extend the parser layer to emit references → a true
   code dependency graph fused with governance; shipped **language-by-language** (Rust/Go/TS
   first), with "unsupported language → file-level only" degradation. Its own milestone.
4. **C4 — Viz scaling.** Collapse-by-component (the architecture model's globs are the
   aggregation key), lazy expand, level-of-detail. Mandatory before any real-repo symbol view.
5. **Graduation gate (declared ex-ante).** The code weave remains EXPERIMENTAL until **all**
   hold: (a) C1 + C2 shipped (file + symbol weave) with C4 collapse so a real repo renders;
   (b) **N=2** — a second independent adopter exercises the weave and files feedback via the
   adopter intake flow; (c) the audit-oracle invariant holds (`/api/graph` without
   `include=code` ≡ `straymark audit`, byte-for-byte, across the adopter corpora); (d) the
   `--no-default-features` parser-free build stays green in CI. Only then is graduation
   considered (jointly with CHARTER-01's Loom graduation).

**Out of scope:**

- Consuming `codebase-memory-mcp` (or any external indexer) as a runtime dependency or sidecar
  — `ADR-2026-06-26-001` records why; an *optional* post-graduation enrichment source is not
  excluded but is not this work.
- `CALLS`/`IMPORTS` beyond C3's staged per-language rollout; a complete call graph for all ~12
  languages at once.
- New `component:`/`symbol:` frontmatter; editing code or governance from the Loom UI (server
  stays read-only).
- A graph-database backend — the document graph stays in-memory; the code graph is revisited
  for SQLite only if CHARTER-01 R2's pre-declared performance trigger fires.

## Files to modify

<!-- Authored ex-ante for a capability that does not yet exist; most rows are "New". Paths
     referencing existing files are confirmed present in the tree. -->

| File | Change |
|---|---|
| `core/Cargo.toml` | Modified — add default-off `codegraph` feature; `arborist-metrics` optional dep under it |
| `core/src/codegraph/{mod,model,build,gather}.rs` | New — code graph (files in C1; symbols in C2) |
| `core/src/weave.rs` | New — pure `weave(doc, code, links) -> WovenGraph` + `WovenGraph`/cross-edge types |
| `core/src/architecture/gather.rs` | Modified — emit `WeaveLinks` (`doc_id → (path, line_range, kind)`) beside `build_governance_state` |
| `core/src/ailog.rs` | Modified — surface the `## Modified Files` line column (today discarded) |
| `core/src/architecture/projection.rs` | Modified — re-express `project()` over the weave (one source of truth) |
| `core/src/graph.rs` | Minimal/none — code merged only behind `include=code`; oracle serialization unchanged |
| `core/src/lib.rs` | Modified — `pub mod codegraph;` (feature-gated) + `pub mod weave;` |
| `cli/src/commands/**` | New — weave-facing textual path ("what governs X?"); `audit`/`validate` untouched |
| `experiment-loom/src/**` | Modified — build code graph + weave on rebuild; `/api/graph?include=code`, `/api/code/component/:id`, extended `/api/node/:id` |
| `experiment-loom/web/**` | Modified — opt-in code (file) layer + cross-highlight (C1); component→file→symbol drill-down (C2/C4) |
| `experiment-loom/README.md` | Modified — reverse the "no AST/graph extractor" non-goal (keep "no new frontmatter") |
| `docs/adopters/CLI-REFERENCE.md` (EN + es + zh-CN) | Modified — document the weave-facing command (EXPERIMENTAL) |
| `CLAUDE.md` | Modified — command-table row(s) for the weave path |
| `CHANGELOG.md` (root) + `experiment-loom/CHANGELOG.md` | Modified — `core-`/`cli-` increment + `loom-0.x` entries |

## Verification

### Local checks

```bash
# Default build stays parser-free (NFR6) — no tree-sitter in the default core dep tree
cargo build
cargo tree -p straymark-core --no-default-features | grep -i tree-sitter   # expect: no match

# Weave library + feature build
cargo test --workspace
cargo test -p straymark-core --features codegraph

# Audit-oracle invariant (NFR1): default /api/graph must equal straymark audit, unchanged
straymark audit --json > /tmp/audit.json
curl -s 'http://127.0.0.1:7700/api/graph' > /tmp/graph.json            # no include=code
# (compare node/edge id sets vs audit — must match the pre-weave baseline)
curl -s 'http://127.0.0.1:7700/api/graph?include=code' > /tmp/woven.json # superset w/ code

# Frontend build (CI also runs this; embedded via rust-embed)
cd experiment-loom/web && npm ci && npm run build
```

### Production smoke (after deploy)

Not applicable — Loom is a localhost, single-binary tool with no deployed environment.
External auditors should skip this section.

## Risks

- **R1 — Default-build dependency creep.** med/high if it occurred.
  Mitigation: `codegraph` is a default-off cargo feature; CI keeps a `--no-default-features`
  build green so the parser-free path cannot silently regress. If a code-parsing dep leaks
  into the default tree, the build gate fails.
- **R2 — Rebuild cost at scale.** low/med.
  Mitigation: per-file `(path, content-hash)` cache + incremental re-parse (C2). Trigger to
  revisit a graph DB (SQLite recursive CTE) is **inherited** from CHARTER-01 R2 (corpus too
  large / sub-second rebuild impossible) — reuse it, do not invent a new one.
- **R3 — Audit-oracle regression.** low/high if it occurred.
  Mitigation: code nodes stay in `core::codegraph`; the weave merges only behind
  `include=code`; the bare `Graph` serialization is unchanged and guarded by an oracle test.
- **R4 — Symbol-attribution false precision.** med/med.
  Mitigation: every cross-edge carries `provenance`; absent line/parse evidence it degrades
  **loudly** to file-level rather than assert a wrong symbol (NFR2); the UI shows coarse vs.
  exact. If line numbers drift (post-AILOG reformat), the fallback is file-level, never wrong.
- **R5 — Code↔document graph drift.** low/high if it occurred.
  Mitigation: both graphs build from the same `notify` cycle and the same `ScanConfig`; `weave`
  is a pure function of both snapshots, so it cannot observe a torn state — the same
  structural argument as `ADR-2026-06-02-001`, extended to code.
- **R6 — Multi-language extraction gaps (C3).** med/med.
  Mitigation: the file-level weave (C1) is language-agnostic and works everywhere immediately;
  symbol/call layers ship per-language with explicit degradation for unsupported languages.
- **R7 — Viz node explosion.** med/high if shipped naively.
  Mitigation (C4, mandatory): collapse-by-component; default view is documents only; symbols
  reached only by drill-down or following a cross-edge — never "all symbols" by default.

## Tasks

1. Sync main, branch `feat/loom-code-weave-c1.0` (this spec PR is `feat/loom-code-weave-spec`).
2. Execute C1 per `experiment-loom/specs/003-code-weave/tasks.md` (C1.0 → C1.4), starting with
   the `WeaveLinks` refactor gated on the unchanged CLI + `status --where` suites.
3. AILOG per increment (`risk_level: medium`, `review_required: true`).
4. Local verification passes clean (workspace tests + `--no-default-features` build + audit
   oracle diff).
5. PR, merge, tag (`core-`/`cli-` for the library + textual companion; `loom-0.x` for the
   visual layer). Then repeat the branch→implement→AILOG→verify→PR→tag loop per milestone
   (C2, C3, C4).
6. **Multi-batch execution** (this Charter spans 3+ batches / >1 day): maintain a
   `## Batch Ledger` in each milestone's AILOG; run `straymark charter batch-complete
   CHARTER-02-code-weave <N>` after each batch commit.
7. Run `straymark charter drift CHARTER-02-code-weave <range>` before each commit; document any
   omission/expansion drift in the AILOG.
8. Commit + push + open PR per milestone.

## Charter Closure

When closing this Charter (after C2 ships and the graduation gate is evaluated):

1. **Atomic update (format v4)**: if drift was detected, edit `## Files to modify` and/or add
   `## Closing notes` in the same PR — do not defer.
2. **Post-merge drift check**: `straymark charter drift CHARTER-02-code-weave
   origin/main..HEAD`; validate clean or all drifts documented in the AILOGs.
3. **Status frontmatter** moves `draft` → `in-progress` when C1.0 starts, → `closed` at
   closure (optionally `closed_at:`).
4. **Do not delete** this file — the planning history matters as much as the AILOGs.

> Note: this repo (StrayMark source) does not currently install a root `.straymark/`; this
> Charter therefore lives self-contained under `experiment-loom/`, beside CHARTER-01. If the
> repo later adopts a charters index, relocate accordingly.
