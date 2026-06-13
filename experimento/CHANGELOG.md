# Changelog — Loom

All notable changes to the **Loom** component (StrayMark's experimental knowledge-graph
visualization server) are documented here. Loom is versioned independently from the
Framework (`fw-*`) and the CLI (`cli-*`) under the **`loom-X.Y.Z`** tag prefix.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/).

> ⚠️ **Loom is EXPERIMENTAL (v0 / N=1).** While the major version is `0`, anything may
> change between releases without a deprecation cycle.

## [Unreleased]

### Added
- Intention documents seeding the component: `README.md` (Loom reframed as a multi-view
  development dashboard), the SpecKit set `specs/001-loom-server/{spec,plan,tasks}.md`
  (Knowledge Graph view) and `specs/002-architecture-plan/spec.md` (Architecture Plan view —
  "you are here" status overlay), the dogfood work-block Charter `CHARTER-01-loom-server.md`,
  and this changelog. Architecture decisions recorded in
  `docs/decisions/ADR-2026-06-02-loom-stack.md` (stack) and
  `docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md` (plan model/format).

### Planned — 0.1.0 (M1, walking skeleton — Knowledge Graph view)
- `straymark loom serve` launches a loopback-only axum server that watches `.straymark/`
  (or `docs/`), builds the typed knowledge graph via the shared `straymark-core` crate, and
  serves a Sigma.js + graphology force-directed web UI with live filesystem updates over
  WebSocket and node-thread highlighting.

### Planned — Architecture Plan view (Spec 002)
- A1: `straymark-core` "you are here" status projection (component state by file-glob match
  over active/closed Charters, drift, TDE, declared-vs-wired) + `straymark architecture
  generate|sync|validate` + `/api/where`.
- A2: maxGraph rendering of a human-authored `plan.drawio` with a non-destructive status
  overlay, layer toggle, component panel, and cross-linking with the Knowledge Graph.
- A3 (north star): axonometric/BIM exploded-layers view.

[Unreleased]: https://github.com/StrangeDaysTech/straymark/compare/HEAD...HEAD
