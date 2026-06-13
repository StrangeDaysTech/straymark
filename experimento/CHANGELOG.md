# Changelog — Loom

All notable changes to the **Loom** component (StrayMark's experimental knowledge-graph
visualization server) are documented here. Loom is versioned independently from the
Framework (`fw-*`) and the CLI (`cli-*`) under the **`loom-X.Y.Z`** tag prefix.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/).

> ⚠️ **Loom is EXPERIMENTAL (v0 / N=1).** While the major version is `0`, anything may
> change between releases without a deprecation cycle.

## [Unreleased]

### Planned — 0.2.0 (M2, analytics + panels)
- Louvain community coloring, corpus stats panel (orphans, dangling references),
  server-side filters (`type/status/risk/tag/from/to`) on `/api/graph`.

### Planned — Architecture Plan view (Spec 002)
- A1: `straymark-core` "you are here" status projection (component state by file-glob match
  over active/closed Charters, drift, TDE, declared-vs-wired) + `straymark architecture
  generate|sync|validate` + `/api/where`.
- A2: maxGraph rendering of a human-authored `plan.drawio` with a non-destructive status
  overlay, layer toggle, component panel, and cross-linking with the Knowledge Graph.
- A3 (north star): axonometric/BIM exploded-layers view.

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

[Unreleased]: https://github.com/StrangeDaysTech/straymark/compare/loom-0.1.0...HEAD
[0.1.0]: https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.1.0
