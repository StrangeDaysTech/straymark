# Changelog — Loom

All notable changes to the **Loom** component (StrayMark's experimental knowledge-graph
visualization server) are documented here. Loom is versioned independently from the
Framework (`fw-*`) and the CLI (`cli-*`) under the **`loom-X.Y.Z`** tag prefix.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/).

> ⚠️ **Loom is EXPERIMENTAL (v0 / N=1).** While the major version is `0`, anything may
> change between releases without a deprecation cycle.

## [Unreleased]

### Planned — Architecture Plan view (Spec 002)
- A1: `straymark-core` "you are here" status projection (component state by file-glob match
  over active/closed Charters, drift, TDE, declared-vs-wired) + `straymark architecture
  generate|sync|validate` + `/api/where`.
- A2: maxGraph rendering of a human-authored `plan.drawio` with a non-destructive status
  overlay, layer toggle, component panel, and cross-linking with the Knowledge Graph.
- A3 (north star): axonometric/BIM exploded-layers view.

## [0.4.0] — 2026-06-13 (connectivity: reference normalization + entity nodes)

Closes the M1 connectivity follow-ups (R1 + R2) surfaced by dogfooding against the Sentinel
corpus, where 330 of 395 references were dangling. All changes are in the shared
`straymark-core` graph builder, so `straymark audit` gains the same connectivity.

### Added

- **Charter / plan / audit nodes** (R2): a new `straymark-core::entities` module discovers
  `.straymark/charters/*.md` (by `charter_id`), `.straymark/plans/PLAN-*.telemetry.yaml`
  (by `plan_id`), and `.straymark/audits/*/review.md`, and `Graph::build_with_entities`
  injects them as `CHARTER` / `PLAN` / `AUDIT` nodes. References by `CHARTER-NN`, full
  `charter_id`, `PLAN-NN`, or audit path now resolve instead of dangling.

### Changed

- **Reference normalization** (R1): the graph builder resolves an edge target by exact id
  and, failing that, by unique file basename, unique relative-path suffix, `CHARTER-NN`
  prefix, or the leading dated id prefix — never resolving an ambiguous match. Resolved
  targets are canonicalized to the node id.

### Result (Sentinel, measured)

- Dangling references **330 → 87**; nodes **131 → 193** (+41 charters, +5 plans, +16 audits);
  orphans **2 → 0**. The remaining ~87 are references to files outside the governance corpus
  (`.specify/memory/…`, `constitution.md`), which correctly stay dangling.

## [0.3.0] — 2026-06-13 (M3, rich UI)

### Added

- **Incremental rebuild + WS `delta` events** (NFR2): a parse cache re-parses only
  files whose modification time changed; the watcher diffs the new graph against the
  previous one and pushes `{event:"delta", added, removed, changed, edges, stats}`. The
  SPA patches the graph in place, preserving the layout of unchanged documents instead of
  re-laying out on every edit. The initial WS sync remains a full `rebuild`.
- **Cycle / SCC reporting** (spec §3.3): `straymark-core` detects dependency cycles
  (strongly-connected components) over the resolved directed semantic edges
  (`SUPERSEDES`, `ORIGINATES_FROM`); `RELATED_TO` is symmetric and never reported.
  Surfaced in `/api/stats` as `cycles` and listed in the corpus stats panel.
- **Centrality-based node sizing**: a header selector sizes nodes by Betweenness
  (default — highlights bridge documents), PageRank, or Degree, computed client-side.
- **Search, pin, open-in-editor**: a search box centers the camera on a matched
  document; "Pin subgraph" isolates the selected document's thread as a working set;
  the node panel offers VS Code / Cursor deep-links and a copy-path button (client-side
  only — the server stays strictly read-only).
- **UI internationalization** (NFR5): all interface strings move behind a string table
  (`en` / `es` / `zh-CN`); the active language is the project's configured language,
  resolved by the shared `straymark-core` config logic (same as `straymark explore`) and
  served at the new `GET /api/meta` endpoint.

### Changed

- Language resolution (`resolve_language`, `detect_os_locale`, `parse_posix_locale`) moved
  from the CLI into `straymark-core` (`core::config`); the CLI now delegates, so the CLI
  and Loom share one source of truth (`straymark-core` 0.2.0 → 0.3.0).

## [0.2.0] — 2026-06-13 (M2, analytics + panels)

### Added

- **Louvain community coloring** over the graph's undirected projection, with cluster
  colors and a compact interactive legend of the largest communities. Labels use a
  representative document title; clicking a community focuses its subgraph.
- **Corpus stats panel** with counts by type/status/risk plus navigable orphan and
  dangling-reference lists.
- **Node summary panel** with metadata, body excerpt, and clickable incoming/outgoing
  relationship endpoints. The panel shows the source path and explicitly identifies
  truncated excerpts instead of implying that they are complete documents.
- **Server-side graph filters** for `type`, `status`, `risk`, `tag`, and inclusive
  `from`/`to` created-date bounds. Filtered responses retain dangling references from
  matching sources and recalculate their stats.
- Two filter-behavior tests plus excerpt-truncation coverage, bringing the Loom suite to
  6 tests.

### Changed

- The web UI now refetches the active filtered view after live rebuild events while
  preserving the existing thread-highlight and no-reload workflow.

## [0.1.0] — 2026-06-12 (M1, walking skeleton — Knowledge Graph view)

### Added

- **Server** (`straymark-loom`, axum + tokio): builds the typed knowledge graph via the
  shared `straymark-core` crate (FR1/FR2 — same parser as the CLI, NFR1 verified against
  `straymark audit`) and serves the Spec 001 §4 API: `GET /api/graph`, `/api/node/:id`,
  `/api/node/:id/thread?depth=N`, `/api/stats` (counts, orphans, dangling references),
  `/healthz`, and `WS /api/stream`.
- **Live updates** (FR6/§5): `notify` watcher with 250ms debounce; settled `.md` changes
  rebuild the snapshot and push a `rebuild` event — measured ~255ms from save to an open
  browser, well under the 1s acceptance bound.
- **Security** (FR7/NFR4): binds `127.0.0.1` exclusively (refuses anything else), rejects
  non-loopback `Host` headers (anti DNS-rebinding), read-only by construction; unparseable
  (mid-save) documents are skipped, never fatal.
- **Web UI** (FR4/FR5, Vite + TypeScript + graphology + Sigma.js, embedded via rust-embed):
  force-directed graph (ForceAtlas2) colored by document type, sized by degree;
  selecting a node lights its full thread and dims the rest (no relayout); node detail
  panel (metadata + body excerpt); type legend; corpus counters; WS auto-reconnect;
  positions preserved across rebuilds. `--assets-dir` overrides the embedded bundle for
  frontend development.
- **CLI launcher**: `straymark loom serve` (cli-3.24.0) downloads the platform binary from
  the latest `loom-*` release on first use, caches it in `~/.straymark/bin/`, prints the
  EXPERIMENTAL banner, and spawns it (download-on-demand = the opt-in gate).
- **CI**: `.github/workflows/release-loom.yml` — frontend built in CI and embedded;
  4-platform matrix; GitHub-release-only (`--latest=false`, no crates.io while experimental).

### Milestone trail

- M0 (the `straymark-core` extraction this release builds on) shipped as `cli-3.23.1`
  (PR #239) together with the component's intention docs (README, SpecKit sets 001/002,
  `CHARTER-01-loom-server`, ADR-2026-06-02-001/-002).

[Unreleased]: https://github.com/StrangeDaysTech/straymark/compare/loom-0.4.0...HEAD
[0.4.0]: https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.0
[0.3.0]: https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.3.0
[0.2.0]: https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.2.0
[0.1.0]: https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.1.0
