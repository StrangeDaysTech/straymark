# Tasks 001 — Loom

> **SpecKit artifact — the ordered, checkable work.** Derived from `plan.md`. Each task is
> verifiable. Milestones map to releasable increments (see `plan.md` §7). This is the
> ex-ante skeleton; tasks get refined as each milestone starts. Status: **draft**.
> FR/NFR ids reference `spec.md`.

## M0 — `straymark-core` extraction (ships as a `cli-` patch, no `loom-` tag)

- [ ] T0.1 — Add root `/Cargo.toml` `[workspace]` (`members = ["core", "cli"]`; add
  `experimento` later) and move `[profile.release]` (opt-level=z, lto, strip) to the root.
- [ ] T0.2 — Create `core/` crate `straymark-core`; **move** `document.rs` (`DocType`,
  `Frontmatter`, `StrayMarkDocument`, `parse_document`, `discover_documents`,
  `detect_doc_type`) from `cli/src/`.
- [ ] T0.3 — Update `cli/` imports `crate::document` → `straymark_core::document`
  (~15 files: `audit_engine`, `validation`, `compliance`, `metrics_engine`, `tui/*`,
  `commands/*`).
- [ ] T0.4 — Add `core::graph`: generalize `audit_engine::build_traceability` into a typed,
  **bidirectional**, metadata-carrying, **orphan-preserving** graph builder (node + typed
  edge structs per spec §3; `resolved` flag for dangling refs).
- [ ] T0.5 — Point `audit_engine` at `core::graph` for its existing traceability output
  (behavior unchanged for `straymark audit`).
- [ ] T0.6 — **Regression gate:** full `cargo test` workspace passes byte-for-byte;
  `straymark audit` output on a sample corpus is identical to pre-refactor.
- [ ] T0.7 — crates.io: publish `straymark-core`; bump `straymark-cli` dep to versioned
  (per ADR decision); bump `cli/Cargo.toml` patch version; update CHANGELOG.
- [ ] T0.8 — PR, merge, tag `cli-X.Y.Z`.

## M1 — Walking skeleton (`loom-0.1.0`) — FR1–FR7

- [ ] T1.1 — Add `experimento/` to workspace members; scaffold `experimento/Cargo.toml`
  (`straymark-loom`, deps: tokio, axum, notify/notify-debouncer-full, rust-embed, serde).
- [ ] T1.2 — Build pipeline: `core::discover_documents` + parse → `core::graph::build` →
  `Arc<RwLock<Graph>>`. (FR1, FR2)
- [ ] T1.3 — axum server binds `127.0.0.1:7700`; `GET /api/graph`, `GET /api/node/:id`,
  `GET /api/node/:id/thread`, `GET /healthz`. (FR3)
- [ ] T1.4 — Security layer: explicit loopback bind + non-loopback `Host` rejection. (FR7)
- [ ] T1.5 — `notify` watcher (debounce ~250ms) → rebuild → broadcast over `WS /api/stream`
  (`rebuild` event, full snapshot at M1). (FR6, §5)
- [ ] T1.6 — `web/`: Vite + TS + graphology + Sigma scaffold; render force graph, nodes
  colored by `doc_type`, sized by degree. (FR4)
- [ ] T1.7 — Thread highlight: WS client + `/api/node/:id/thread` + Sigma reducers (dim
  non-thread). (FR5, S2)
- [ ] T1.8 — `rust-embed` of `web/dist`; `--assets-dir` dev override. (NFR3)
- [ ] T1.9 — CLI: `Loom { Serve { --port, --no-open } }` in `main.rs` +
  `commands/loom/serve.rs` (download-on-demand via `get_latest_release_by_prefix("loom-")`,
  cache, EXPERIMENTAL banner, spawn). (plan §5)
- [ ] T1.10 — `.github/workflows/release-loom.yml` (clone of release-cli.yml; npm build
  step; `loom-*` trigger; no crates.io publish). (plan §6)
- [ ] T1.11 — **Acceptance (spec §8):** all 6 M1 criteria pass, incl. NFR1 consistency
  (`/api/graph` ≡ `straymark audit`) and live-update < 1s.
- [ ] T1.12 — `experimento/CHANGELOG.md` → `0.1.0`; PR; merge; tag `loom-0.1.0`.

## M2 — Analytics + panels (`loom-0.2.0`) — FR8–FR10

- [ ] T2.1 — Louvain communities → `node.community` → cluster coloring.
- [ ] T2.2 — Node summary panel (metadata + body excerpt + clickable in/out links). (S5)
- [ ] T2.3 — Corpus stats panel: counts by type/risk, orphan list, dangling-reference list.
  (FR8, S4)
- [ ] T2.4 — Server-side filters `type/status/risk/tag/from/to` on `/api/graph`; UI
  controls. (FR9, S6)
- [ ] T2.5 — Acceptance; CHANGELOG → `0.2.0`; tag `loom-0.2.0`.

## M3 — Rich, Infranodus-like (`loom-0.3.0`)

- [ ] T3.1 — Incremental rebuild (only changed files) + WS `delta` events. (NFR2)
- [ ] T3.2 — Cycle/SCC reporting over semantic edges. (spec §3.3)
- [ ] T3.3 — Centrality-based node sizing.
- [ ] T3.4 — Search, "pin subgraph", "open in editor" (uses node `path`).
- [ ] T3.5 — UI i18n driven by the project's configured language. (NFR5)
- [ ] T3.6 — Acceptance; CHANGELOG → `0.3.0`; tag `loom-0.3.0`.

## Graduation (post-M3, evaluated against the Charter's criteria)

- [ ] G.1 — Confirm N=2 (a second independent adopter exercises Loom) per the Charter.
- [ ] G.2 — Decide whether to graduate Loom out of `experimento/` and promote the brand
  (rename folder, drop the EXPERIMENTAL banner, formalize support contract).
