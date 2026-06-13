---
id: AILOG-2026-06-13-001
title: Loom M3 — rich UI (incremental deltas, cycles, centrality, search/pin/open, i18n)
status: accepted
created: 2026-06-13
agent: claude-fable-5
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 1650
files_modified: [CHANGELOG.md, Cargo.lock, README.md, cli/Cargo.toml, cli/src/config.rs, cli/src/utils.rs, core/Cargo.toml, core/src/config.rs, core/src/graph.rs, core/src/lib.rs, docs/adopters/CLI-REFERENCE.md, docs/i18n/es/README.md, docs/i18n/es/adopters/CLI-REFERENCE.md, docs/i18n/zh-CN/README.md, docs/i18n/zh-CN/adopters/CLI-REFERENCE.md, experimento/CHANGELOG.md, experimento/Cargo.toml, experimento/specs/001-loom-server/tasks.md, experimento/src/main.rs, experimento/src/server.rs, experimento/src/snapshot.rs, experimento/src/watcher.rs, experimento/web/index.html, experimento/web/package-lock.json, experimento/web/package.json, experimento/web/src/i18n.ts, experimento/web/src/main.ts]
observability_scope: none
tags: [loom, incremental, delta, cycles, scc, centrality, search, i18n, knowledge-graph]
related: [AILOG-2026-06-12-003, ADR-2026-06-02-001]
---

# AILOG: Loom M3 — rich UI

## Summary

Executed milestone **M3** of `CHARTER-01-loom-server` (T3.1–T3.5): Loom's analytical
dashboard becomes a rich exploration tool. Rebuilds are now incremental and pushed as
WebSocket `delta` events that the SPA patches in place (preserving layout); dependency
cycles are detected and surfaced; nodes can be sized by betweenness / PageRank / degree;
the UI gains search, subgraph pinning, and editor deep-links; and all interface strings
are internationalized (`en` / `es` / `zh-CN`) driven by the project's configured language.
The component and web package advanced to `0.3.0` and `straymark-core` to `0.3.0`. T3.6
remains open only for the post-merge release tag.

## Context

M2 delivered Louvain communities, panels, and server-side filters. M3 (Spec 001 §3.3,
NFR2, NFR5) turns the dashboard into an exploration tool without changing Loom's
loopback-only, read-only security posture or the shared `straymark-core` contract.

## Actions Performed

1. **T3.1 — Incremental rebuild + WS deltas.** Added an mtime-keyed parse cache
   (`snapshot::ParseCache`) and `Snapshot::build_cached`: only files whose modification
   time changed are re-parsed; the graph is still rebuilt in full (cheap in-memory). The
   watcher diffs the new graph against the previous one (`ApiGraph::diff`) and pushes
   `{event:"delta", added, removed, changed, edges, stats}`; the SPA's `applyDelta` patches
   graphology in place, preserving positions of unchanged nodes. The initial WS sync stays
   a full `rebuild`. The parse cache is warmed by the initial build and threaded into the
   watcher.
2. **T3.2 — Cycle/SCC reporting.** Added `straymark_core::graph::cycles_in` (iterative
   Tarjan SCC) over the resolved directed semantic edges (`SUPERSEDES`, `ORIGINATES_FROM`);
   `RELATED_TO` is symmetric and excluded, unresolved edges cannot form a cycle. Reports
   components of size > 1 plus semantic self-edges. Surfaced in `/api/stats` (`cycles`) and
   listed in the corpus stats panel.
3. **T3.3 — Centrality sizing.** Added `graphology-metrics`; a header selector sizes nodes
   by betweenness (default — bridge documents), PageRank, or degree, computed client-side
   and normalized to a fixed pixel range.
4. **T3.4 — Search / pin / open-in-editor.** A search box matches by id/title and animates
   the camera to the result; "Pin subgraph" isolates the selected document's thread as a
   working set (reducers dim everything else); the node panel offers `vscode://` and
   `cursor://` deep-links plus a copy-path button. All client-side — the server gains no new
   capability and stays strictly read-only.
5. **T3.5 — UI i18n.** Extracted all interface strings into `web/src/i18n.ts`
   (`en`/`es`/`zh-CN`) behind a `t()` helper. Moved language resolution
   (`resolve_language`, `detect_os_locale`, `parse_posix_locale`) into
   `straymark-core::config`; the CLI's `StrayMarkConfig::resolve_language` now delegates, so
   the CLI and Loom share one source of truth. Loom resolves the project's language and
   serves it at the new `GET /api/meta`; the SPA localizes itself at boot.
6. **Release preparation.** Bumped `straymark-loom` and the web package to `0.3.0` and
   `straymark-core` to `0.3.0` (additive API → minor bump; the CLI's core dependency
   requirement was bumped to match). Updated `Cargo.lock`, release notes, and the
   current-version tables. T3.6 remains pending until the post-merge `loom-0.3.0` tag.

## Modified Files

| File | Change Description |
|------|--------------------|
| `core/src/graph.rs` | `cycles_in` + `Cycle` (Tarjan SCC), `Node: PartialEq`, 4 cycle tests |
| `core/src/config.rs` | New module: `resolve_language`/`detect_os_locale`/`parse_posix_locale` + tests |
| `core/src/lib.rs`, `core/Cargo.toml` | `pub mod config`; version 0.3.0; tempfile dev-dep |
| `cli/src/config.rs`, `cli/src/utils.rs`, `cli/Cargo.toml` | Delegate locale to core; drop moved fns; core dep 0.3.0 |
| `experimento/src/snapshot.rs` | ParseCache, `build_cached`, `diff`/`GraphDelta`, `delta_event`, `Stats.cycles`, 3 tests |
| `experimento/src/watcher.rs`, `main.rs` | Warm cache + delta emission; resolve locale; `/api/meta` state |
| `experimento/src/server.rs` | `/api/meta` endpoint + `AppState.locale` |
| `experimento/web/src/i18n.ts` | String table (en/es/zh-CN) + `t()` |
| `experimento/web/src/main.ts`, `index.html` | Delta apply, sizing selector, search, pin, open-in-editor, cycles panel, i18n |
| changelogs, adopter docs, tasks | M3 release notes and current Loom version |

## Decisions Made

- **Incremental = parse cache, full graph rebuild.** The bottleneck is file IO + frontmatter
  parsing, not the in-memory graph build (plan §8 R2); caching parses by mtime makes the
  expensive part incremental while keeping graph correctness trivial.
- **Deltas patch nodes, replace edges wholesale.** Node positions are the layout-stability
  cost worth preserving; edge ids are positional indices, so a full edge replace is the
  robust choice and is cheap to redraw.
- **Cycles only over resolved `SUPERSEDES`/`ORIGINATES_FROM`.** `RELATED_TO` is symmetric by
  intent; reporting a related pair as a cycle would be noise (spec §3.3).
- **Centrality is client-side**, like Louvain, keeping the server thin.
- **Open-in-editor is client-side deep-links**, not a server endpoint — Loom never spawns a
  process and stays strictly read-only (the chosen security posture).
- **Language resolution moved to `straymark-core`, CLI delegates.** Same single-source
  principle as the M0 parser extraction; `straymark-core` bumped to 0.3.0 and the CLI's
  versioned dependency updated so the workspace and the next crates.io publish stay
  consistent.
- **T3.5 reuses the CLI's existing resolution semantics** (config `language` → OS locale →
  `en`), verified by the unchanged CLI `resolve_language` tests.

## Impact

- **Functionality:** additive. Existing endpoints, filters, panels, thread highlight, and
  community coloring remain; deltas replace the full-snapshot push on changes (initial sync
  unchanged).
- **Performance:** edits re-parse only changed files; the SPA patches instead of
  re-laying-out. The production JS bundle is ~236 KB / 61 KB gzip (was ~207/53; the increase
  is `graphology-metrics`).
- **Security:** unchanged — loopback-only, read-only, anti-DNS-rebinding `Host` check; no new
  server capability (open-in-editor is client-side).
- **Dependencies:** added `graphology-metrics`; production dependency audit clean.

## Verification

- [x] `cargo test` — full workspace passes: `straymark-core` 33/33 (incl. 4 cycle + 8 config
      tests), `straymark-loom` 9/9 (incl. cache, diff, cycle tests), CLI suites unchanged.
- [x] `cargo clippy -p straymark-core -p straymark-loom --no-deps -- -D warnings` — clean
      (also fixed the pre-existing `unnecessary_map_or` in `core/src/graph.rs`).
- [x] `npm run build` — TypeScript + Vite production build pass.
- [x] `npm audit --omit=dev` — 0 vulnerabilities (the dev-only esbuild/vite advisories remain
      outside the shipped static bundle, as in M1/M2).
- [x] Real-server smoke test on Sentinel (131 docs / 395 links): `/api/meta` returns the
      configured locale; `/api/stats` includes `cycles`; the SPA serves with search + sizing
      controls; forged `Host` returns HTTP 403.
- [x] End-to-end delta test on a temp project: WS initial sync is `rebuild`; editing a `.md`
      (change a status, add a new doc) produces `delta` events with the correct
      `added`/`changed`/`removed` sets and recomputed stats; `/api/meta` returns `es` from a
      temp `config.yml` (`language: es`), confirming project-driven i18n.

## Follow-ups

- **T3.6:** merge, then create and push tag `loom-0.3.0`.
- **Interactive UI acceptance:** the centrality selector, search camera focus, subgraph pin,
  and editor deep-links are visual; final acceptance is a browser pass against Sentinel.
- **Pre-existing CLI clippy debt:** a newer clippy toolchain flags ~19 `manual_checked_ops`
  and style lints across untouched CLI files (`audit_engine`, `followups`, `tui/*`, …);
  outside M3 scope (the Loom gate and `release-loom.yml` cover `core` + `loom` only).
- **Graduation track:** M3 ships the last Knowledge-Graph milestone; remaining for graduation
  are the Architecture-Plan track (A1/A2, Spec 002) and N=2.

## Additional Notes

- `straymark-core` 0.3.0 is not published to crates.io by this (Loom) release; it ships to
  crates.io on the next CLI release, where `release-cli.yml` publishes core before the CLI.

---

<!-- Template: StrayMark | https://strangedays.tech -->
