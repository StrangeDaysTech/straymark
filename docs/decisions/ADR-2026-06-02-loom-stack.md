---
id: ADR-2026-06-02-001
title: Loom experimental component — stack, workspace extraction, and distribution
status: draft
created: 2026-06-02
updated: 2026-06-02
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
alternatives_documented: []
api_changes: []
tags: [loom, experimental, knowledge-graph, workspace, architecture]
related: [CHARTER-01-loom-server]
supersedes: []
---

# ADR: Loom experimental component — stack, workspace extraction, and distribution

## Status

draft

**Note**: This document was created by an AI agent and requires human review.

> **Immutability Rule**: Once an ADR reaches `accepted` status, it MUST NOT be modified. If
> the decision changes, create a new ADR with `supersedes: ADR-2026-06-02-001`.

## Context

StrayMark documents form a typed, directed graph through their frontmatter
(`related`, `supersedes`, `alternatives_documented`, `api_changes`, `originating_ailogs`,
`originating_spec`). That graph is consumed by the AI and built internally by the CLI for
`straymark audit`, but it is only ever visible to a human one document at a time via the
`straymark explore` TUI. As corpora grow (reference case: Sentinel), the TUI is outpaced as
a tool for *seeing the corpus shape*. Three reviewers independently flagged a graphical
knowledge-graph view as both a cognitive aid and a likely adoption attractor.

We are adding **Loom**, an **experimental** third component (alongside the Framework `dist/`
and CLI `cli/`) that serves a live, browser-based, force-directed view of the document
graph on `localhost`, rebuilding in real time from filesystem changes. It lives in
`experimento/`, carries its own release history (`loom-*` tags), and stays marked
experimental (v0/N=1 in docs, not in tags) until graduated. The feature spec is
`experimento/specs/001-loom-server/spec.md`; the work block is
`experimento/CHARTER-01-loom-server.md`.

This ADR records the **architecture decisions** that have repo-wide consequences — the
backend stack, the workspace/`straymark-core` extraction, the frontend library, the asset
bundling, and the distribution model — because they touch the existing CLI and the repo
root, not just the new folder.

## Decision

We will:

1. **Build the server in Rust + axum + tokio**, watching the filesystem with `notify` and
   pushing updates to the browser over WebSocket.
2. **Refactor the repo root into a Cargo workspace** (`members = ["core", "cli",
   "experimento"]`) and **extract a new `straymark-core` crate** holding the document model
   and a generalized graph builder, moved out of `cli/src/document.rs` and
   `cli/src/audit_engine.rs`. Both the CLI and Loom depend on `straymark-core`, so they
   parse frontmatter with one identical code path and the graph cannot drift from the CLI's
   truth. We will **publish `straymark-core` to crates.io** so `straymark-cli`'s existing
   `cargo publish` resolves cleanly.
3. **Render the frontend with Sigma.js + graphology** (TypeScript/Vite), using WebGL
   rendering, `graphology-communities-louvain` for clusters, force-atlas2 for layout, and
   Sigma reducers for the "select a node → highlight its thread, dim the rest" interaction.
4. **Embed the built web assets into the binary with `rust-embed`**, producing a single
   self-contained `straymark-loom` executable per platform; the npm build runs only in CI.
5. **Distribute via `straymark loom serve`** — a thin CLI subcommand that downloads the
   latest `loom-*` release asset on demand (reusing `download.rs`/`platform.rs`), caches
   it, prints an EXPERIMENTAL banner, and launches it. The CLI does **not** take a
   dependency on axum/tokio; the download-on-demand gate is the opt-in boundary.
6. **Bind `127.0.0.1` only**, reject non-loopback `Host` headers, and keep the server
   strictly read-only.

## Alternatives Considered

### 1. Duplicate the parser inside `experimento/` (no workspace refactor)
- **Description**: Copy the document/graph model into the new crate; leave `cli/` untouched.
- **Pros**: Zero blast radius on the CLI now; no crates.io coordination.
- **Cons**: Guarantees drift the moment a new `DocType` or relationship field is added — and
  the China regulatory profile shows such fields *are* added regularly. Two parsers means
  the graph can silently disagree with `straymark audit`.
- **Why not**: The entire value of Loom is that it shows the *same* truth the CLI enforces.
  Structural drift-prevention (one parser) outweighs the one-time refactor cost.

### 2. Node/TypeScript full-stack server
- **Description**: Implement server and frontend in JS/TS for the richer visualization
  ecosystem out of the box.
- **Pros**: Largest viz/library ecosystem; one language across server and UI.
- **Cons**: Introduces a second first-class backend language into a Rust-first repo and
  **reimplements the frontmatter parser from scratch** (drift risk per Alternative 1), plus
  a heavier runtime to distribute.
- **Why not**: Reusing `straymark-core` is the dominant consideration; we accept a confined
  JS toolchain for the *frontend only* (mirroring how `website/` is isolated).

### 3. Bundle the server into the CLI as a cargo feature (`--features loom`)
- **Description**: Compile the server into the CLI binary behind a feature flag.
- **Pros**: One binary; no separate download step.
- **Cons**: Drags tokio/axum/notify into the CLI's dependency tree and its crates.io
  publish, inflating the `opt-level=z` CLI footprint and coupling release cadences.
- **Why not**: Fights the "isolated, experimental, own release history" requirement. The
  download-on-demand model keeps the CLI lean and Loom independently shippable.

### 4. Cytoscape.js / D3-force instead of Sigma.js
- **Description**: Alternative frontend graph libraries.
- **Pros**: Cytoscape — clean API; D3 — maximal control.
- **Cons**: Cytoscape's canvas/SVG rendering hits a scaling ceiling around 1–2k nodes and
  has weaker built-in analytics; D3-force is maximal effort for everything ("scarce MVP"
  risk).
- **Why not**: Sigma+graphology gives WebGL scale, first-class Louvain/centrality, and a
  clean data/render split that mirrors our backend core/server split — best fit for a
  "base meant to grow".

## Consequences

### Positive
- One parser → the graph is provably consistent with `straymark audit` (no drift).
- A reusable, published `straymark-core` invites future components and external reuse.
- Lean CLI preserved; Loom ships and breaks independently behind a clear experimental gate.
- WebGL frontend scales to thousands of nodes; the data/render split keeps room to grow.

### Negative
- One-time **workspace refactor touching ~15 CLI files** (mechanical import changes) and a
  new crates.io publish coordination for `straymark-core`.
- A confined **JS/npm toolchain** enters the repo (frontend only, CI-built).
- A new **localhost network surface** to secure (mitigated: loopback-only, read-only,
  anti-rebinding).

### Neutral
- A third release workflow (`release-loom.yml`) and tag prefix join `fw-`/`cli-`.

### Quality Impact Assessment

| Quality Characteristic (ISO 25010:2023) | Impact | Description |
|-----------------------------------------|--------|-------------|
| Functional Suitability | + | Makes the document graph directly legible to humans for the first time |
| Performance Efficiency | ~ | In-memory build is fast for low-thousands of docs; full re-parse per FS event is the watch-item (incremental in M3) |
| Compatibility | + | Shared `straymark-core` guarantees interoperability between CLI and Loom |
| Maintainability | + | One parser instead of two; clear crate boundaries |
| Security | ~ | New loopback surface, mitigated by bind/Host/read-only constraints |
| Flexibility | + | Sigma/graphology + crate split give a base designed to grow |

## Affected Components

| Component | Type of Change | Impact |
|-----------|----------------|--------|
| repo root | New (`/Cargo.toml` workspace) | Medium |
| `straymark-core` (`core/`) | New crate | High |
| `straymark-cli` (`cli/`) | Modified (imports, dep on core, `loom` subcommand) | Medium |
| `straymark-loom` (`experimento/`) | New crate + `web/` | High |
| `.github/workflows/release-loom.yml` | New | Low |

## Implementation Plan

1. M0 — extract `straymark-core` (pure move + graph generalization); CLI test suite is the
   regression oracle; publish core; ship as a `cli-` patch.
2. M1 — walking skeleton `loom-0.1.0` (watch → graph → Sigma → live → thread highlight).
3. M2 — analytics + panels `loom-0.2.0`.
4. M3 — rich, Infranodus-like `loom-0.3.0`.

(Detail in `experimento/specs/001-loom-server/plan.md` and `tasks.md`.)

## Success Metrics

- After M0, the full workspace test suite and `straymark audit` output are byte-for-byte
  unchanged (zero-regression extraction).
- After M1, `/api/graph` node/edge sets equal those derived by `straymark audit` for the
  same corpus; a `.md` edit reflects in the browser in < 1s.

## Validation Criteria

| Metric | Target Value | Measurement Method | Timeline |
|--------|-------------|-------------------|----------|
| Extraction regressions | 0 | `cargo test` workspace + `audit` diff vs pre-refactor | M0 |
| Graph/CLI consistency | exact match | diff `/api/graph` vs `straymark audit` | M1 |
| Live-update latency | < 1s | edit a watched `.md`, observe browser | M1 |
| Frontend interactivity | ≥ 2–3k nodes | manual profiling | M1–M2 |

## References

- `experimento/specs/001-loom-server/spec.md`, `plan.md`, `tasks.md`
- `experimento/CHARTER-01-loom-server.md`
- Reused CLI infra: `cli/src/document.rs`, `cli/src/audit_engine.rs`, `cli/src/download.rs`,
  `cli/src/platform.rs`, `.github/workflows/release-cli.yml`
- Inspiration (idea only): Infranodus (entity map + side panels)

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-02 | claude-opus-4-8-1m | Initial creation (draft, pending human review) |

<!-- Template: StrayMark | https://strangedays.tech -->
