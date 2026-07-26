# Implementation Plan 001 — Loom

> **SpecKit artifact — the HOW.** Derives from `spec.md` (the WHAT) and feeds `tasks.md`
> (the ordered work). The repo-wide architecture decision is recorded as a dogfood ADR:
> `.straymark/02-design/decisions/ADR-2026-06-02-001-loom-stack.md`. Status: **draft / experimental**.

## 1. Architecture at a glance

```
                      .straymark/ or docs/ (Markdown + frontmatter)
                                   │  (read-only)
                    notify watcher │  debounce ~250ms
                                   ▼
   ┌──────────────────────────────────────────────────────────────┐
   │  straymark-loom  (Rust, axum, tokio)   binds 127.0.0.1:7700    │
   │   ┌───────────────┐   builds   ┌────────────────────────────┐ │
   │   │ straymark-core│──────────▶ │ in-memory typed multigraph │ │
   │   │ (shared crate)│            │  + bidirectional adjacency │ │
   │   └───────────────┘            └────────────────────────────┘ │
   │        ▲  same parser                    │ serves              │
   │        │                       HTTP /api/* + WS /api/stream    │
   └────────┼──────────────────────────────────┼───────────────────┘
            │ depends on                        │  rust-embed
   ┌────────┴────────┐               ┌──────────▼───────────────────┐
   │  straymark-cli  │               │ web/ SPA  Sigma.js+graphology│
   │ `loom serve`    │ downloads &   │ force layout, Louvain,       │
   │  launches ──────┼──────────────▶│ reducers (thread highlight)  │
   └─────────────────┘  the binary   └──────────────────────────────┘
```

## 2. Workspace refactor (the load-bearing decision)

Today `cli/` is a standalone crate. Convert the repo root into a **virtual Cargo
workspace** and extract the document/graph model into a new `straymark-core` crate so that
Loom and the CLI share *exactly one* parser:

```
/Cargo.toml          # [workspace] members = ["core", "cli", "experiment-loom"]; [profile.release] opt-level=z, lto, strip
/core/               # straymark-core  (new)
/cli/                # straymark-cli   (now depends on straymark-core)
/experiment-loom/        # straymark-loom  (new; depends on straymark-core)
```

`straymark-core` = a **pure move** (no behavior change) of, from `cli/src/`:
- `document.rs` — `DocType`, `Frontmatter`, `StrayMarkDocument`, `parse_document`,
  `discover_documents`, `detect_doc_type`.
- the graph-building half of `audit_engine.rs::build_traceability()`, **generalized** into a
  `core::graph` module that keeps edges **bidirectional + typed + metadata-carrying +
  orphan-preserving** (today it keeps forward-only `related`, drops orphans, and discards
  all metadata beyond id/type/title).

**Regression oracle:** the existing CLI test suite (`document` tests, `audit_engine` tests,
the `charter_*` tests) must pass byte-for-byte after the move. Do the extraction as its own
reviewable step (M0) before any server code so the blast radius is isolated and bisectable.

**Import churn:** `cli/` modules switch `crate::document` → `straymark_core::document`
(~15 files: `audit_engine`, `validation`, `compliance`, `metrics_engine`, `tui/*`, several
`commands/*`). Mechanical.

**crates.io:** `straymark-cli`'s publish step now needs `straymark-core` resolvable.
**Decision (recommended):** publish `straymark-core` to crates.io alongside the next `cli-`
release; bump cli's dep from path to versioned. Alternative (path-dep + skip-on-publish)
documented in the ADR if we prefer to keep core unpublished while Loom is experimental.

## 3. Backend (`straymark-loom`)

- **Runtime:** `tokio` + `axum`. Routes per spec §4. Graph held behind an
  `Arc<RwLock<Graph>>` (read-mostly; writes only on rebuild).
- **Watcher:** `notify` (recommended `notify-debouncer-full`) on the resolved target dir.
  On settled events → rebuild (M1 full; M3 incremental keyed on changed paths) → compute a
  delta → broadcast over a `tokio::sync::broadcast` channel feeding all `/api/stream`
  sockets.
- **Graph build:** call `straymark_core::discover_documents` + parse, then
  `core::graph::build` → typed bidirectional graph; run Louvain server-side *or* defer to
  the client (M2 decision; recommend client-side first to keep the server thin).
- **Security middleware:** bind `127.0.0.1` explicitly; an axum layer rejects requests
  whose `Host` is not loopback (anti DNS-rebinding). No write paths exist.
- **Assets:** `rust-embed` embeds `web/dist/`; axum serves `/`. A `--assets-dir` flag
  overrides with a live dir for frontend dev (Vite dev server).

## 4. Frontend (`experiment-loom/web/`)

- **Stack:** TypeScript + Vite; **graphology** (graph data + analytics: degree, components,
  `graphology-communities-louvain`) + **Sigma.js** (WebGL render).
- **Layout:** `graphology-layout-forceatlas2` in a web worker.
- **Thread highlight (S2/FR5):** on node select, call `/api/node/:id/thread`, feed the
  returned `{node_ids, edge_ids}` into Sigma **reducers** that keep that set opaque and dim
  the rest — pure render-time, no relayout.
- **Live updates:** a WS client applies `rebuild` (replace) / `delta` (patch) messages.
- **Panels (M2):** node summary (from `/api/node/:id`) and corpus stats (from
  `/api/stats`); filter controls drive `/api/graph` query params.
- **i18n:** all UI strings behind a string table from day one (NFR5); translation in M3.

All JS is confined to `experiment-loom/web/` and built **only in CI** — mirrors how
`website/`'s Docusaurus toolchain is isolated in a Rust-first repo. Adopters never run npm.

## 5. CLI integration

- New subcommand in `cli/src/main.rs`: `Loom { #[command(subcommand)] }` with
  `Serve { #[arg(long, default_value="7700")] port: u16, #[arg(long)] no_open: bool }`.
- Handler `cli/src/commands/loom/serve.rs`:
  1. Resolve project root (reuse the `explore`/`status` resolution).
  2. Look for cached `~/.straymark/bin/straymark-loom`; if missing/stale, use
     `download::get_latest_release_by_prefix("loom-")` + `platform::current_target()` +
     `download::download_file` + archive extraction (same zip/tar.gz path as the framework
     install).
  3. Print a loud **EXPERIMENTAL** banner (opt-in, unstable, loopback-only).
  4. Spawn the binary pointed at the project; optionally open the browser.
- The CLI does **not** depend on axum/tokio — it only downloads and launches. Keeps the
  `opt-level=z` CLI footprint intact.

## 6. Release & CI

- **Tag prefix:** `loom-X.Y.Z` (independent history).
- **`.github/workflows/release-loom.yml`** = clone of `release-cli.yml` with:
  - trigger `on: push: tags: ['loom-*']`; `VERSION=${TAG#loom-}` verified against
    `experiment-loom/Cargo.toml`.
  - extra pre-build: `npm ci && npm run build` in `experiment-loom/web/` so `rust-embed` picks
    up `web/dist/`.
  - same 4-target matrix; asset name `straymark-loom-v{ver}-{target}.{ext}`.
  - **no** `cargo publish` for `straymark-loom` while experimental (GitHub-release-only).
  - keep the "delete previous releases of this prefix" cleanup.
- The `straymark-core` extraction (M0) ships under the existing `release-cli.yml` as a
  normal `cli-` patch (and, per §2, a `straymark-core` crates.io publish).

## 7. Phasing (each milestone = a releasable increment)

- **M0 — `straymark-core` extraction.** Ships as a `cli-` patch (no `loom-` tag). Pure
  move + graph generalization. CLI tests are the oracle. De-risks the refactor first.
- **M1 — Walking skeleton (`loom-0.1.0`).** FR1–FR7: watch → parse → full graph over WS →
  Sigma force graph colored by type → thread highlight on select → live update < 1s →
  loopback-only. Delivers the core "wow"; no panels yet.
- **M2 — Analytics + panels (`loom-0.2.0`).** FR8–FR10: Louvain coloring, node summary
  panel, corpus stats panel (orphans, dangling refs), server-side filters.
- **M3 — Rich, Infranodus-like (`loom-0.3.0`).** Incremental WS deltas, cycle/SCC reporting,
  centrality-based sizing, search, "pin subgraph", "open in editor" (uses `path`), UI i18n.

> **Second surface — Architecture Plan view (Spec `../002-architecture-plan/spec.md`).** Loom
> is a dashboard, not a single graph. The Architecture Plan view adds tracks **A1** (a pure
> "you are here" status projection in `straymark-core` + a `straymark architecture
> generate|sync|validate` CLI, shipping as a `cli-` increment) and **A2** (the maxGraph
> DrawIO render + live overlay + layer toggle, a `loom-0.x` release alongside/after M2);
> **A3** (axonometric/BIM) is the north star. A1 reuses `charter_files`, `charter drift`,
> `metrics_engine`, TDE docs, and `analyze declared-vs-wired` — all already in the CLI. See
> `.straymark/02-design/decisions/ADR-2026-06-02-002-architecture-plan-format.md`.

## 8. Risks

- **R1 — Workspace refactor blast radius.** Mitigation: M0 as a standalone pure-move PR
  gated on the unchanged CLI test suite; crates.io snag resolved by publishing
  `straymark-core` (§2).
- **R2 — Rebuild cost at scale.** The bottleneck is full re-parse per FS event, not graph
  size. Mitigation: debounce + incremental re-parse (M3). **Revisit a graph DB (SQLite
  recursive CTE / sled, never Neo4j) only if** corpus > ~10k docs **or** incremental
  rebuild can't stay sub-second. Until then, in-memory adjacency is correct (spec NFR2).
- **R3 — JS toolchain coherence in a Rust-first repo.** Mitigation: confine all JS to
  `experiment-loom/web/`, build only in CI, embed via rust-embed (mirror `website/`).
- **R4 — Localhost exposure.** Mitigation: bind `127.0.0.1`, reject non-loopback `Host`,
  read-only server (FR7/NFR4).
- **R5 — Model drift between Loom and the CLI.** Mitigation: the shared `straymark-core`
  crate makes drift structurally impossible (one parser) — this is the whole point of §2.

## 9. References

- `spec.md` (this feature's WHAT) and `tasks.md` (ordered work).
- `.straymark/02-design/decisions/ADR-2026-06-02-001-loom-stack.md` (the architecture decision record).
- `../../CHARTER-01-loom-server.md` (the work-block Charter; `originating_spec` → this spec).
- Reused CLI infra: `cli/src/document.rs`, `cli/src/audit_engine.rs`, `cli/src/download.rs`,
  `cli/src/platform.rs`, `cli/src/self_update.rs`, `.github/workflows/release-cli.yml`.
