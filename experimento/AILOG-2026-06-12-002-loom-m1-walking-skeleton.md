---
id: AILOG-2026-06-12-002
title: Loom M1 — walking skeleton (loom-0.1.0 server + UI + CLI launcher)
status: accepted
created: 2026-06-12
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 1400
files_modified: [core/src/graph.rs, core/Cargo.toml, experimento/Cargo.toml, experimento/src/main.rs, experimento/src/server.rs, experimento/src/snapshot.rs, experimento/src/watcher.rs, experimento/web/package.json, experimento/web/vite.config.ts, experimento/web/tsconfig.json, experimento/web/index.html, experimento/web/src/main.ts, cli/src/main.rs, cli/src/commands/loom/mod.rs, cli/src/commands/loom/serve.rs, cli/src/download.rs, cli/src/self_update.rs, cli/Cargo.toml, .github/workflows/release-loom.yml, .gitignore, CHANGELOG.md, CLAUDE.md]
observability_scope: none
tags: [loom, server, axum, sigma, websocket, walking-skeleton]
related: [AILOG-2026-06-12-001, ADR-2026-06-02-001]
---

# AILOG: Loom M1 — walking skeleton (`loom-0.1.0`)

## Summary

Executed milestone **M1** of `CHARTER-01-loom-server` (T1.1–T1.12): the first
runnable Loom — an axum server that watches a project, builds the typed
knowledge graph via `straymark-core`, serves the Spec 001 §4 API over loopback
only, and renders a live Sigma.js force-directed UI with thread highlighting;
plus the `straymark loom serve` download-on-demand launcher (CLI 3.24.0) and
the `release-loom.yml` pipeline. All 6 M1 acceptance criteria verified.

## Context

M0 (AILOG-2026-06-12-001, PR #239, `cli-3.23.1`) put the shared parser and
graph in place. M1 is the "walking skeleton": the thinnest end-to-end slice
that delivers the core wow — *see* the corpus as a live map (FR1–FR7).

## Actions Performed

1. **core 0.2.0** — `Graph::thread(id, depth)` (Spec §3.3): undirected BFS
   over incident edges, optional depth bound, dangling out-edges included in
   the highlight set; 2 new unit tests (20 total in core).
2. **T1.1** — `experimento/` joined the workspace; `straymark-loom` crate
   (axum 0.8, tokio, notify-debouncer-full, rust-embed; `publish = false`).
3. **T1.2/T1.3** — `snapshot.rs`: discover→parse→build via `straymark-core`
   (unparseable mid-save files skipped, never fatal); precomputed API view
   with edge index ids; `/api/graph`, `/api/node/:id` (+excerpt, in/out
   edges), `/api/node/:id/thread?depth`, `/api/stats` (orphans + dangling
   refs, M1-required data per acceptance 6), `/healthz`.
4. **T1.4** — security: explicit `127.0.0.1` bind (refuses otherwise; never
   falls back to a wider interface) + middleware rejecting non-loopback
   `Host` (allows `localhost`/`127.0.0.0/8`/`[::1]`, with ports); unit-tested
   against DNS-rebinding shapes (`localhost.evil.com`, etc.).
5. **T1.5** — watcher: 250ms debounce, `.md`-only relevance filter, rebuild →
   `Arc` swap → broadcast `{event:"rebuild", graph}` to all WS clients;
   initial full sync on connect; lagged receivers skip, never crash.
6. **T1.6/T1.7/T1.8** — `web/`: Vite + TS strict + graphology + Sigma v3.
   Type-color palette (16 doc types), degree sizing, ForceAtlas2 (300 it
   first load / 30 on rebuild with positions preserved), thread highlight via
   node/edge reducers (no relayout), node detail panel, legend, counters, WS
   auto-reconnect. Bundle 46KB gzip, embedded via rust-embed; `--assets-dir`
   dev override; SPA fallback.
7. **T1.9** — CLI 3.24.0: `Loom { Serve { path, --port, --no-open } }`;
   `commands/loom/serve.rs` reuses `download.rs`/`platform.rs`/
   `self_update.rs` extractors (made `pub(crate)`); version-marker cache in
   `~/.straymark/bin/`; offline → cached binary with a warning; EXPERIMENTAL
   banner; browser auto-open (xdg-open/open/cmd). `strip_tag_prefix` learns
   `loom-`.
8. **T1.10** — `release-loom.yml`: version-vs-tag gate on
   `experimento/Cargo.toml`, `npm ci && npm run build` before cargo (assets
   embedded), 4-target matrix, workspace `target/` paths, `--latest=false`,
   prefix-scoped release cleanup. `experimento/web/package-lock.json`
   committed for `npm ci` (gitignore exception, mirrors `website/`).
9. **Docs** — versioning tables gain a Loom row and CLI 3.24.0 (README ×3,
   CLI-REFERENCE ×3); new `loom serve` command section (EN/ES/zh-CN);
   CLAUDE.md (commands table, structure tree, Loom release workflow); root
   CHANGELOG and `experimento/CHANGELOG.md` → 0.1.0.

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `core/src/graph.rs` | +151/-0 | `thread()` + `Thread` + 2 tests |
| `experimento/src/{main,server,snapshot,watcher}.rs` | +800/-0 | The Loom server (4 modules, 4 tests) |
| `experimento/web/**` | +260/-0 (+lockfile) | Vite/TS/Sigma frontend |
| `experimento/Cargo.toml` | +32/-0 | straymark-loom crate |
| `cli/src/commands/loom/{mod,serve}.rs` | +200/-0 | Download-on-demand launcher |
| `cli/src/main.rs` | +30/-0 | `Loom`/`LoomCommands` wiring |
| `cli/src/{download,self_update}.rs` | +4/-3 | `loom-` prefix; extractors `pub(crate)` |
| `.github/workflows/release-loom.yml` | +180/-0 | Loom release pipeline |
| docs (README/CLI-REFERENCE ×6, CLAUDE.md, CHANGELOGs) | ~+150 | Versioning, command docs, release notes |

## Decisions Made

- **core 0.2.0 (not 0.1.1)** for the additive `thread()` API; the CLI's dep
  bumped to `0.2.0` so the publish-core-before-cli CI step stays coherent.
- **Edge identity = index** into `graph.edges` (stable per snapshot): gives
  the UI addressable edges without changing the core `Edge` shape.
- **Sync ForceAtlas2** (not the worker) at M1 — corpora are small; the worker
  supervisor is an M3 refinement if needed.
- **`npm audit`**: 2 high findings in the dev-only esbuild/vite dev-server
  chain (GHSA esbuild CORS). Not shipped: the production bundle is static JS;
  builds run only in CI. Accepted for M1; revisit on vite major bumps.
- **Dangling edges are API-only at M1**: Sigma needs both endpoints, so the
  UI renders resolved edges; dangling refs surface via `/api/stats` (and the
  M2 stats panel). The data is never dropped (FR2).

## Impact

- **Functionality**: new, additive. No existing CLI command changes behavior.
- **Performance**: fixture rebuild ~5ms; live update save→browser ~255ms
  (debounce-dominated). NFR2 headroom is large at current corpus sizes.
- **Security**: new localhost listener — mitigated per FR7/NFR4 (loopback
  bind, `Host` rejection, read-only; refuses to start otherwise). R4 closed
  for M1.
- **Privacy**: N/A (serves the user's own local files to their own browser).
- **Environmental**: N/A.

## Verification

- [x] Code compiles without errors (workspace: core + cli + loom)
- [x] Tests pass — core 20, loom 4 (snapshot ×2, host-check, unparseable),
      CLI suite intact (see PR run: full `cargo test` green)
- [x] **Acceptance 1** — server serves the force-graph UI at `127.0.0.1:7700`
      (embedded assets; verified by HTTP + visual check in browser)
- [x] **Acceptance 2** — `/api/graph` ≡ `straymark audit` on the fixture
      corpus: node id sets and doc types identical; audit chains covered by
      resolved `RELATED_TO` edges (NFR1)
- [x] **Acceptance 3** — node selection lights the thread, dims the rest
      (Sigma reducers; visual check)
- [x] **Acceptance 4** — live update measured **~255ms** save→browser (<1s)
      via a WS client driving a real file edit
- [x] **Acceptance 5** — refuses non-loopback: bind is hardcoded loopback;
      forged `Host: evil.example.com` → 403 (curl-verified + unit tests)
- [x] **Acceptance 6** — orphans + dangling references present in
      `/api/stats` (curl-verified on a corpus containing both)
- [x] Manual review performed (human review pending at PR — `review_required: true`)

## Follow-ups

Surfaced by smoke-testing M1 against the reference adopter corpus (Sentinel,
130 discovered docs / 389 edges — **326 dangling**, which is signal, not noise):

- **R1 (new) — reference normalization.** Real `related:` entries reference
  documents by *filename* (`AILOG-2026-05-07-041-charter-13-….md`, 64 cases)
  and by *relative path* (`.straymark/audits/…`, `specs/…`; ~90 cases), not
  only by frontmatter id. The resolver (core::graph; shared with `audit`)
  could normalize: strip `.md`, match filename stem, resolve paths. Any change
  must move `audit` and Loom together (NFR1). Candidate: M2.
- **R2 (new) — Charters as graph nodes.** 47+ dangling refs target
  `CHARTER-*` ids/paths: Charters are among the most-referenced documents in
  a real corpus but are not `DocType`s, so they are invisible to the graph.
  Promoting them to first-class nodes (the CLI already parses them in
  `charter.rs`) would close the biggest legibility gap. Candidate: M2/M3.
- **R3 (new) — visual density at 100+ nodes.** Operator verdict on the
  Sentinel test: improved (short labels, dark hover, threshold 8) but still
  saturated. Candidates for M2/M3: label fade by zoom level, cluster-aware
  label budgets, edge bundling/opacity scaling, Louvain coloring (already
  planned) doubling as visual grouping.

## Additional Notes

- `loom-0.1.0` must be tagged **after** this PR merges so
  `release-loom.yml` finds matching versions; until the release exists,
  `straymark loom serve` reports "no release could be fetched" (verified
  message path).
- Suggested tag order at merge: `cli-3.24.0` and `loom-0.1.0` can be pushed
  together (`git push origin cli-3.24.0 loom-0.1.0`); the CLI release will
  also publish `straymark-core 0.2.0` to crates.io (idempotent step).

---

<!-- Template: StrayMark | https://strangedays.tech -->
