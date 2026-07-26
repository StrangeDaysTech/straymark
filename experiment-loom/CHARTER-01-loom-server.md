---
charter_id: CHARTER-01-loom-server
status: in-progress
effort_estimate: L
trigger: "Three independent framework reviewers flagged a graphical knowledge-graph view as a cognitive aid and adoption attractor; the TUI is outpaced as corpora grow (Sentinel)."
originating_spec: experiment-loom/specs/001-loom-server/spec.md
---

# Charter: Build Loom — experimental knowledge-graph visualization server

> **Status (mirrored from frontmatter — source of truth is above):** in-progress. Effort: L.
>
> **Origin:** SpecKit spec `experiment-loom/specs/001-loom-server/spec.md`. Architecture
> decision recorded in `.straymark/02-design/decisions/ADR-2026-06-02-001-loom-stack.md`
> (`ADR-2026-06-02-001`).

## Context

StrayMark documents cross-link through their frontmatter into a typed, directed graph that
the AI and the CLI already consume, but that a human can only read one document at a time
(`straymark explore`). As corpora grow, that ceases to convey structure. Loom is an
experimental third component — a **development dashboard** rendered live in the browser on
`localhost`, rebuilding from filesystem changes — with two complementary surfaces: a
**Knowledge Graph view** of documents (Spec 001) and an **Architecture Plan view** of the
system with a "you are here" status overlay (Spec 002). This Charter is the bounded,
multi-session work block that builds Loom from a zero-regression core extraction through both
MVP surfaces. It is **experimental (v0/N=1)** and ships behind an opt-in CLI download gate.

## Scope

**In scope:**

1. **M0 — `straymark-core` extraction.** Convert the repo root to a Cargo workspace; move
   the document model (`DocType`, `Frontmatter`, `StrayMarkDocument`, `parse_document`,
   `discover_documents`, `detect_doc_type`) from `cli/src/document.rs` into a new
   `straymark-core` crate; generalize `cli/src/audit_engine.rs::build_traceability` into a
   typed, bidirectional, metadata-carrying, orphan-preserving `core::graph` builder; CLI
   test suite passes byte-for-byte; publish `straymark-core`; ship as a `cli-` patch.
2. **M1 — `loom-0.1.0` walking skeleton.** `straymark-loom` (axum + tokio + notify) builds
   the graph via `straymark-core`, serves the spec §4 API + WS over loopback only, renders
   a Sigma.js force graph colored by type with node-thread highlighting and < 1s live
   updates; `straymark loom serve` downloads/launches it; `release-loom.yml` ships it.
3. **M2 — `loom-0.2.0` analytics + panels.** Louvain coloring, node summary panel, corpus
   stats panel (orphans, dangling refs), server-side filters.
4. **M3 — `loom-0.3.0` rich UI.** Incremental WS deltas, cycle/SCC reporting, centrality
   sizing, search/pin/open-in-editor, UI i18n.
5. **A1 — Architecture model + generator + projection** (Spec 002; ships as a `cli-`
   increment). `straymark-core` pure status projection (active-charter declared files via
   `charter_files`, drift, closed charters/AILOGs, TDE, declared-vs-wired → component state
   by file-glob match); `straymark architecture generate|sync|validate` CLI (hybrid seed
   from code dirs + ADR C4/Affected-Components); `/api/where` + optional `straymark status
   --where`.
6. **A2 — Architecture Plan view** (the second MVP surface, a `loom-0.x` release). maxGraph
   render of a human-authored `plan.drawio` with non-destructive "you are here" overlay,
   layer toggle (00–09 floors), component detail panel, "Where are we" panel, and cross-view
   linking with the Knowledge Graph.
7. **A3 — Axonometric/BIM view (north star, post-MVP).** 2.5D stacked, explodable layers.
8. **Graduation gate (declared ex-ante).** Loom remains EXPERIMENTAL until **all** hold:
   (a) M1–M3 and A1–A2 shipped (both MVP surfaces); (b) **N=2** — a second independent adopter (beyond this project's own
   dogfooding) exercises Loom and files feedback via the adopter intake flow; (c) the
   loopback security posture (bind, `Host` rejection, read-only) is verified; (d) the
   `/api/graph`≡`straymark audit` consistency invariant holds across the adopter corpora.
   Only then is graduation (drop the EXPERIMENTAL banner, promote the brand, formalize the
   support contract) considered.

**Out of scope:**

- In-UI document editing, authentication, remote/multi-user hosting — out of scope because
  Loom is loopback-only and read-only by design (spec §9).
- A graph-database backend — deferred unless the performance trigger in `plan.md` §8 R2
  fires (corpus > ~10k docs or sub-second incremental rebuild becomes impossible).
- AI summaries / topic modeling (Infranodus' LDA-style panels) — deferred to possible
  post-graduation work; explicitly not v0.

## Files to modify

<!-- This Charter is authored ex-ante for a component that does not yet exist; most rows
     are "New". The reused CLI infra rows reference paths confirmed to exist in the tree. -->

| File | Change |
|---|---|
| `Cargo.toml` (repo root) | New — `[workspace] members = ["core","cli","experiment-loom"]`; move `[profile.release]` here |
| `core/Cargo.toml`, `core/src/document.rs` | New — `straymark-core`; moved from `cli/src/document.rs` |
| `core/src/graph.rs` | New — generalized typed/bidirectional graph (from `audit_engine`) |
| `cli/src/document.rs` | Removed — moved to `straymark-core` |
| `cli/src/audit_engine.rs` | Modified — consume `straymark_core::document` + `core::graph` |
| `cli/src/{validation,compliance,metrics_engine}.rs`, `cli/src/tui/*`, `cli/src/commands/*` | Modified — `crate::document` → `straymark_core::document` |
| `cli/Cargo.toml` | Modified — depend on `straymark-core`; patch version bump |
| `cli/src/main.rs` | Modified — add `Loom { Serve { --port, --no-open } }` subcommand + dispatch |
| `cli/src/commands/loom/serve.rs`, `cli/src/commands/loom/mod.rs` | New — download-on-demand launcher (reuses `download.rs`/`platform.rs`) |
| `experiment-loom/Cargo.toml`, `experiment-loom/src/**` | New — `straymark-loom` (axum, notify, rust-embed) |
| `experiment-loom/web/**` | New — Sigma.js + graphology (graph) + maxGraph (plan) frontend (Vite/TS) |
| `core/src/architecture.rs` | New — architecture model + pure "you are here" status projection (A1) |
| `cli/src/commands/architecture/*.rs` | New — `straymark architecture generate/sync/validate`; `status --where` (A1) |
| `experiment-loom/architecture/{model.yml,plan.drawio}` | New — dogfood architecture model + DrawIO layout |
| `.github/workflows/release-loom.yml` | New — clone of `release-cli.yml`, `loom-*` trigger, npm build step |
| `experiment-loom/CHANGELOG.md` | Modified — release entries per milestone |
| `CHANGELOG.md` (root) | Modified — record the `cli-` patch (M0) and component introduction |

## Verification

### Local checks

```bash
# Workspace build & test (M0 regression oracle — must pass byte-for-byte vs pre-refactor)
cargo build
cargo test

# Graph/CLI consistency (M1): the server graph must equal the CLI's derivation
straymark audit --json > /tmp/audit.json
curl -s http://127.0.0.1:7700/api/graph > /tmp/graph.json
# (compare node/edge id sets — see tasks.md T1.11)

# Frontend build (CI also runs this; embedded via rust-embed)
cd experiment-loom/web && npm ci && npm run build
```

### Production smoke (after deploy)

Not applicable — Loom is a localhost, single-binary tool with no deployed environment.
External auditors should skip this section.

## Risks

- **R1 — Workspace refactor blast radius.** med/med.
  Mitigation: M0 is a standalone pure-move PR gated on the unchanged CLI test suite
  (byte-for-byte). crates.io coordination resolved by publishing `straymark-core`. If the
  extraction regresses any test, the PR does not merge.
- **R2 — Rebuild cost at scale.** low/med.
  Mitigation: debounce ~250ms + incremental re-parse in M3. Trigger to revisit a graph DB
  (SQLite recursive CTE / sled, never Neo4j): corpus > ~10k docs OR incremental rebuild
  can't stay sub-second. Until then in-memory adjacency is correct.
- **R3 — JS toolchain coherence in a Rust-first repo.** low/low.
  Mitigation: confine all JS to `experiment-loom/web/`, build only in CI, embed via rust-embed
  (mirror `website/`).
- **R4 — Localhost network exposure.** low/med.
  Mitigation: bind `127.0.0.1` only; reject non-loopback `Host` (anti DNS-rebinding);
  read-only server. If the bind or `Host` check fails, the server refuses to start.
- **R5 — Model drift between Loom and the CLI.** low/high if it occurred.
  Mitigation: the shared `straymark-core` crate makes drift structurally impossible (one
  parser). This is the central rationale of `ADR-2026-06-02-001`.
- **R6 — DrawIO/maxGraph round-trip fidelity (A2).** med/med.
  Mitigation: overlay status as non-destructive cell-style overrides keyed on
  `straymark_component_id`; never rewrite geometry; round-trip test (move/reroute in real
  DrawIO → reload → diff geometry) is an A2 acceptance gate (`ADR-2026-06-02-002`). If
  fidelity can't be guaranteed, fall back to a read-only render without write-back.
- **R7 — Glob mapping coarseness (A1).** low/med.
  Mitigation: components may pin explicit `docs:`/paths in `model.yml` for purely conceptual
  units; `straymark architecture validate` reports empty/uncharted components so gaps are
  visible rather than silent.

## Tasks

1. Sync main, branch `feat/loom-m0-core-extraction`.
2. Execute M0 per `experiment-loom/specs/001-loom-server/tasks.md` (T0.1–T0.8).
3. AILOG for M0 (`risk_level: medium`, `review_required: true`).
4. Local verification passes clean (workspace tests + audit diff).
5. PR, merge, tag `cli-X.Y.Z`. Then repeat the branch→implement→AILOG→verify→PR→tag loop
   per milestone (M1 `loom-0.1.0`, M2 `loom-0.2.0`, M3 `loom-0.3.0`).
6. **Multi-batch execution** (this Charter spans 3+ batches / >1 day): maintain a
   `## Batch Ledger` in each milestone's AILOG; run `straymark charter batch-complete
   CHARTER-01-loom-server <N>` after each batch commit.
7. Run `straymark charter drift CHARTER-01-loom-server <range>` before each commit; document
   any omission/expansion drift in the AILOG.
8. Commit + push + open PR per milestone.

## Charter Closure

When closing this Charter (after M3 ships and the graduation gate is evaluated):

1. **Atomic update (format v4)**: if drift was detected, edit `## Files to modify` and/or
   add `## Closing notes` in the same PR — do not defer.
2. **Post-merge drift check**: `straymark charter drift CHARTER-01-loom-server
   origin/main..HEAD`; validate clean or all drifts documented in the AILOGs.
3. **Move the row** in the charters index (if/when this repo adopts one) to `## Closed` and
   reference the PRs.
4. **Status frontmatter** moves `in-progress` → `closed` (optionally `closed_at:`).
5. **Do not delete** this file — the planning history matters as much as the AILOGs.

> Note: this repo (StrayMark source) does not currently install a root `.straymark/`; Loom's
> Charter therefore lives self-contained under `experiment-loom/`. If the repo later adopts a
> charters index, relocate accordingly.
