# Tasks 002 — Loom Architecture Plan

> **SpecKit artifact — the ordered, checkable work.** Derived from `spec.md` (the WHAT) and
> the shared `../001-loom-server/plan.md` (the HOW). Each task is verifiable. Phases map to
> releasable increments. This is the ex-ante skeleton for **A1**; A2/A3 tasks get refined
> when those tracks start. FR/NFR ids reference `spec.md`. The format/projection/generator
> decisions are recorded in `docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md`.
> Status: **draft**.

## Track map (from `spec.md` §12)

- **A1 — Model + generator + projection.** Pure "you are here" projection in `straymark-core`
  + `straymark architecture generate|sync|validate` + `straymark status --where`. Ships as a
  `cli-` increment; immediate **textual** value before any new pixels. **← this file.**
- **A2 — Architecture Plan view.** maxGraph render of `plan.drawio` + live overlay + layer
  toggle + component panel + "Where are we" panel + cross-view linking. A `loom-0.x` release.
- **A3 — Axonometric/BIM exploded view.** North star, post-MVP.

## Scope decisions baked into A1 (settled 2026-06-14)

- **Governance-state extraction moves into `straymark-core`** (mirrors M0's `document.rs`
  move). Today charter parsing (`charter.rs`, `charter_files.rs`) and drift
  (`compute_drift`/`glob_match` in `commands/charter/drift.rs`) live in `cli/`. They move to
  `core` so the projection's inputs are core-computable and the future Loom server (A2),
  which depends on `core` not `cli`, reuses exactly one extractor. Resolves the §4 "pure
  function in core" requirement structurally and mitigates plan §8 R5 ("one parser").
- **`straymark status --where` ships in A1** (spec §14 open question → yes). The shared pure
  projection makes the textual companion nearly free and is the headline value of A1 as a
  `cli-` increment.
- **Dogfood home:** this source repo has no root `.straymark/`, so the architecture artifacts
  live at `experimento/architecture/{model.yml,plan.drawio}` (adopter convention is
  `.straymark/architecture/`). spec §3.1, §14.

---

## A1.0 — Governance primitives → `core` (own PR, no tag — DONE 2026-06-14)

> The M0-style de-risking step: a pure move + minimal generalization, gated on the unchanged
> CLI test suite. Isolated and bisectable before any architecture code. **No standalone
> release** (unlike M0, which had to publish core): zero user value and core 0.4.1 is still
> unpublished, so the version bump + crates.io publish + tag ride the **A1.5** `cli-` release.

- [x] T0.1 — **Moved** charter parsing into `core` via `git mv`: `core/src/charter.rs`
  (`CharterStatus`, `Charter`, `CharterFrontmatter`, `discover_charters`,
  `discover_and_parse`, `parse_charter`, `find_by_id`, `charters_dir`, `display_title`,
  `origin_kind`, …) and `core/src/charter_files.rs` (`DeclaredFile`, `parse_files_to_modify`,
  `is_wildcard`). Added to `core::lib`. `split_frontmatter` also moved → `core/src/utils.rs`
  (charter depended on it) and re-exported from `cli::utils` so `crate::followups` /
  `crate::commands::approve` keep their paths.
- [x] T0.2 — **Moved** the drift matcher primitives `compute_drift`, `glob_match`,
  `wildcard_satisfied_by` (+ their 3 unit tests) → `core/src/drift.rs` (`pub`). The CLI
  `drift` command keeps its git-range diff, AILOG suppression, and Batch-Ledger
  orchestration; it now `use straymark_core::drift::compute_drift`.
- [x] T0.3 — **DONE in A1.1.** The projection's AILOG "Modified Files" parser was authored
  fresh as a pure string parser `core::ailog::parse_modified_files(body) -> Vec<String>`
  (reusing the shared `charter_files` path-extraction helpers). `cli/src/ailog.rs` (the
  Batch-Ledger orchestration) stays in `cli` — the new parser is its own thing, not a move.
- [x] T0.4 — **DONE in A1.1 (resolved by reuse, not a new dep).** No `glob`/`regex` crate
  added: the projection matches component globs with the existing `core::drift::glob_match`,
  keeping one matcher project-wide (NFR3 — the projection's `active`/`in-progress` line up
  with `charter drift` by construction). `serde_yaml` already present.
- [x] T0.5 — Updated `cli/` imports `crate::charter`/`crate::charter_files` →
  `straymark_core::…` (15 charter sites + 2 charter_files sites, word-boundary sed left
  `charter_schema` untouched); removed `mod charter;`/`mod charter_files;` from `main.rs`.
- [x] T0.6 — **Regression gate green:** full `cargo test --workspace` passes (core 81, cli
  unit 314, all integration suites incl. `charter_*`/`audit`/`validate`/`charter_drift` —
  0 failed); moved unit tests relocated with their code (none lost). `cargo clippy` on
  `core`+`loom` clean (fixed one surfaced `ptr_arg` `&PathBuf`→`&Path` in the moved charter).
- [x] T0.7 — **No version bump / no tag** (see header). A1.0 merges as its own PR; the
  `core` minor bump + `cli` minor bump + `Cargo.lock` + CHANGELOG + crates.io publish + tag
  all happen at **A1.5**. Dogfood AILOG: `experimento/AILOG-2026-06-14-002`.

## A1.1 — Architecture model + status projection in `core` (library only, test-gated — DONE)

> The pure heart of A1. No user-facing command yet; lands as `core` lib code + unit tests
> against hand-authored fixtures. FR1, FR2, FR9, NFR2. **No tag/bump** (rides A1.5, per A1.0).

- [x] T1.1 — `core/src/architecture/model.rs`: `serde` structs for `model.yml` (spec §3.1) —
  `ArchModel { version, layers: Vec<Layer>, components: Vec<Component> }`,
  `Layer { id, label, order }`, `Component { id, label, layer, globs, links, docs, external }`
  (`links`/`docs`/`external` `#[serde(default)]`). `parse_model(path)` + I/O-free
  `parse_model_str(content)` (mirrors `charter::parse_charter_str`) with `validate_structure`:
  unknown layer ref + duplicate id are hard errors, empty-globs is a `ModelIssue` warning. (FR1)
- [x] T1.2 — `core/src/architecture/projection.rs`: the **pure** projection inputs as a plain
  data struct (zero IO) — `GovernanceState { active_charter_files, in_progress_files,
  closed_charter_files, tde_files, wiring_gap_files, on_disk_files }` (each a `Vec<String>` of
  paths; `#[derive(Default)]` so callers fill only what they have). (spec §4.)
- [x] T1.3 — `ComponentState` enum + `project(model, state) -> Projection` mapping each
  component to one or more states per spec §4 table: `active`, `in-progress` (refinement of
  `active`), `implemented`, `has-debt`, `wiring-gap`, `uncharted` (on-disk match, no other
  set — suppressed by any documented state). Glob match reuses `core::drift::glob_match`.
  Deterministic (test asserts equal inputs → equal `Projection`). (FR2, NFR3)
- [x] T1.4 — Layer-level rollup: `LayerRollup { layer_id, label, order, counts:
  BTreeMap<ComponentState, usize> }` per layer, preserving model layer order. (spec §4 last ¶)
- [x] T1.5 — Integrity signals (spec §3.3): `Undrawn` / `Unmodeled` / `Empty` via
  `validate_model(model, drawio_cell_ids, on_disk) -> Vec<IntegritySignal>` (`drawio_cell_ids`
  is a stub input until A1.3 parses `plan.drawio`). (FR9)
- [x] T1.6 — Unit tests in `core` (21 new, inline `#[cfg(test)]`, no on-disk fixtures): each
  state derivation, each integrity signal, glob edge cases (`**`/`*` span dirs; bare
  trailing-slash literal matches nothing — documented gotcha), AILOG `## Modified Files`
  parsing. `cargo test --workspace` green (core 102, cli 314, all integration suites);
  `cargo clippy -p straymark-core` clean. (NFR2 — globs only, no AST.)

## A1.2 — `straymark architecture generate` (hybrid seed) — FR7, spec §5 — DONE

> Writes `model.yml` + `plan.drawio`. The server stays read-only (NFR4); generation is the
> CLI's job (spec §7). **No tag/bump** (rides A1.5). All in `cli/` — the command is not
> feature-gated and builds with `--no-default-features`.

- [x] T2.1 — CLI wiring: `Architecture { #[command(subcommand)] command: ArchitectureCommands }`
  in `main.rs` with `Generate { path, --force, --out }`, `Sync { path }`, `Validate { path }`;
  handler module `cli/src/commands/architecture/{mod,generate,sync,validate,adr_mining,drawio}.rs`.
  `sync`/`validate` are stubs that print "not yet implemented (A1.3)" so main.rs isn't re-touched
  in A1.3. CLAUDE.md command table got two new rows. (User-confirmed: `--out` + lenient root.)
- [x] T2.2 — Codebase-structure seed: a self-contained walker (NOT
  `analysis_engine::walk_source_files`, which is `#[cfg(feature = "analyze")]` and would couple
  the command to that feature) derives top-level source dirs → one component each, globs
  `dir/**`. Layers seeded from `DocType::ALL` → `directory()` stage prefixes (01-requirements …
  09-ai-models) plus an `unassigned` placeholder layer (order 0) where code components land for
  the human to reassign. (spec §5, §3.1.)
- [x] T2.3 — ADR enrichment: `cli/src/commands/architecture/adr_mining.rs` — hand-written
  line parsers (no `pulldown-cmark`, which is `tui`-gated) for the `## Affected Components`
  table and ```mermaid `C4Context|C4Container|C4Component` blocks (element `(id,"label",…)` +
  `Rel(from,to,…)` lines, quote-aware arg split). Enriches matched components' labels and adds
  `links` from C4 rels; unmatched ADR components are **reported, not appended** (keeps the seed
  clean — refinement choice over the plan's append, avoids empty-glob noise). ADRs discovered
  under `.straymark/` or, absent an install, `docs/decisions/` (so the A1.5 self-dogfood enriches
  too). (spec §5)
- [x] T2.4 — Auto-layout `plan.drawio`: `drawio.rs` pure-Rust mxGraph emitter — `<mxfile>` /
  `<mxGraphModel>` with one `<object straymark_component_id="…">`-wrapped vertex per component,
  laid out row-per-layer (by `Layer::order`), XML-escaped, deterministic geometry. Orphan-layer
  components still emitted in a trailing row. (spec §3.2, §6; NFR1.)
- [x] T2.5 — `--force` / `--out` guard: refuses to overwrite an existing `model.yml`/`plan.drawio`
  without `--force`; the built model is round-tripped through
  `straymark_core::architecture::parse_model_str` before writing so an invalid model is never
  emitted. Verified e2e on this repo (`--out /tmp/...`: 4 components, 9 layers, 4 ADRs mined,
  2 labels improved; re-run refused, `--force` rewrote). Tests: CLI 327 (13 new — adr_mining 6,
  drawio 3, generate 4), core 102, `--no-default-features` build clean, clippy clean (only
  pre-existing `assert_cmd` test deprecations).

## A1.3 — `straymark architecture sync` + `validate` — FR7, FR9, spec §5, §3.3 — DONE

> No tag/bump (rides A1.5). First a refactor: the shared scan/mining/render machinery moved
> from `generate.rs` to `cli/src/commands/architecture/common.rs` so all three handlers agree.

- [x] T3.1 — `validate.rs`: `parse_model` + new `drawio::parse_component_ids` (scans
  `straymark_component_id="…"`, XML-unescaped) + on-disk glob coverage via `common` → the §3.3
  signals through `core::architecture::validate_model`. `--output text|json|markdown` (local
  `#[derive(Serialize)]` report, so the core `IntegritySignal` surface stays untouched until
  A1.5). **Exits 1** on any signal / invalid / missing model (user-confirmed; CI-gateable).
  Degrades to glob-coverage-only when `plan.drawio` is absent. (FR9)
- [x] T3.2 — `sync.rs`: detect top-level source dirs not covered by an existing component's
  globs (`dir_covered` via `core::drift::glob_match`; honors human-narrowed globs and renamed
  ids), enrich the new ones from ADRs, then **append-only**. **Dry-run by default**
  (user-confirmed); `--apply` text-appends component blocks to `model.yml` (EOF, preserving
  every existing byte — validates with `parse_model_str` before writing) and inserts new cells
  into `plan.drawio` via `drawio::append_cells` (below the existing geometry, existing cells
  byte-identical). Never clobbers human edits/geometry (NFR1). (spec §5, NFR1.)
- [x] T3.3 — Tests + e2e: `drawio` unit tests (`parse_component_ids` round-trip/unescape;
  `append_cells` preserves the existing prefix byte-for-byte, adds one cell below max-y;
  rejects non-DrawIO input); `sync::dir_covered` (covered dir not proposed; human-narrowed glob
  still covers). E2E on this repo: `validate` on a model hand-edited to hold one undrawn + one
  unmodeled + one empty reported exactly those three and exited 1 (text + json); `sync` on a
  sandbox with a new dir reported it (dry-run), then `--apply` appended one component +
  preserved the existing `model.yml`/`plan.drawio` prefix verbatim, and a follow-up `validate`
  was consistent (exit 0). CLI 333 tests, core 102, `--no-default-features` build clean, clippy
  clean (only pre-existing `assert_cmd`/CLI-debt warnings). CLAUDE.md rows refreshed.

## A1.4 — `straymark status --where` (textual "you are here") — spec §8, §14 — DONE

> No tag/bump (rides A1.5). Lives in `cli/src/commands/architecture/where_view.rs` (reuses
> `common::{resolve_root, artifact_paths, collect_source_files}`) so the projection consumer
> sits beside `generate`/`sync`/`validate`. `--where` + `--out` are flags on `status`; without
> `--where`, `status` is byte-for-byte unchanged. Loom A2's `/api/where` will build its own
> `GovernanceState` and call the same pure `core::architecture::project`.

- [x] T4.1 — `build_governance_state(root)`: active charter files via `discover_and_parse`
  filtered to `in-progress` → `parse_files_to_modify`; closed charters likewise + folding in
  `core::ailog::parse_modified_files` of their `originating_ailogs`; `in-progress` = declared ∩
  git-modified (`git diff --name-only HEAD`, intersected via `core::drift::compute_drift` so it
  matches `charter drift`); open TDEs via `discover_documents`+`DocType::Tde`+`related`;
  on-disk via `common::collect_source_files`. **wiring-gap left empty** (needs an explicit
  `declared-vs-wired` profile; not part of the §11.5 gate — documented in the module header to
  avoid silent noise).
- [x] T4.2 — `straymark status --where [path] [--out DIR]`: `parse_model` → `project` → render
  per-layer/per-component badges with active components marked "← you are here", then the §8
  summary (active Charter title + declared-vs-modified progress % + recent AILOGs + debt/uncharted
  counts). Degrades to an `architecture generate` hint when no `model.yml` (success, not exit 1 —
  it's a status view). `--out` mirrors the other `architecture` commands so a non-self-adopter
  repo (model under `experimento/architecture/`) can dogfood.
- [x] T4.3 — **Consistency gate (NFR3):** `cli/tests/architecture_where_test.rs` —
  `where_is_consistent_with_charter_list` asserts on one fixture corpus that the `active`
  component is exactly the one `charter list --status in-progress` reports and `implemented` is
  exactly the `--status closed` one (with an uncommitted edit inside the active component → the
  declared∩git-modified `in-progress` badge). Plus `where_marks_active_in_progress_and_implemented`
  and `where_degrades_without_model`. CLI 335 unit + 3 integration tests, core 102, clippy clean
  (only pre-existing `assert_cmd`/CLI-debt warnings). CLAUDE.md status row added.

## A1.5 — Dogfood, acceptance, docs, release — DONE (tag `cli-3.25.0` rides this PR)

> The track's one release: bumps `core` 0.4.1→**0.5.0** (new `architecture` surface) + `cli`
> 3.24.0→**3.25.0** (architecture command + `status --where`), covering A1.0–A1.4 which all
> merged without a tag. crates.io publish of `straymark-core` rides the `cli-3.25.0` release
> per the M0 decision. **Branched fresh from `main` after A1.4 (#254) merged** (no
> stacked-PR-on-squash hazard).

- [x] T5.1 — Ran `straymark architecture generate --out experimento/architecture` on this repo
  (no root `.straymark/`, so the dogfood home is `experimento/architecture/`; the adopter home
  stays `.straymark/architecture/`). Seed = 4 components (cli/core/experimento/website, globs
  `dir/**`) in the `unassigned` layer + 9 stage layers; ADR mining improved 2 labels.
  Hand-refined to a legible first plan: the 9 stage placeholders → **3 real layers**
  (tooling / visualization / web), components reassigned, 2 dependency links added
  (`cli→core`, `experimento→core`), labels sharpened; `plan.drawio` cells regrouped by layer +
  2 dependency edges. Component **ids kept stable** (the BIM join key) so model↔plan stays
  consistent — `architecture validate` is **exit 0** (4 components, no signals).
- [x] T5.2 — **Acceptance (spec §11, A1 subset):** §11.1 ✓ (`generate` yields a `model.yml` +
  `plan.drawio` that `validate` accepts and a human refined). §11.5 textual half ✓ (the
  `status --where` consistency gate `where_is_consistent_with_charter_list` asserts `active` =
  `charter list --status in-progress` and `implemented` = `--status closed`; the dogfood repo
  has no in-progress Charter so every component reads `uncharted`, the correct degenerate
  answer). **Generator usefulness:** the seed supplied every component id, glob, the valid YAML,
  and the entire `plan.drawio` (cells + ids + geometry) for free; the human work was ~8 semantic
  edits (layer model + dependencies + labels) — from-scratch would have meant hand-writing ~46
  lines of `model.yml` + ~30 of mxGraph XML. Visual criteria §11.2/3/4/6 are **A2**.
- [x] T5.3 — Docs: `architecture generate|sync|validate` + `status --where` rows added to
  `CLAUDE.md` (in the A1.x increments) and to `docs/adopters/CLI-REFERENCE.md` EN +
  `docs/i18n/{es,zh-CN}/adopters/CLI-REFERENCE.md` (a `### architecture` section + a
  `#### status --where` subsection, marked EXPERIMENTAL `cli-3.25.0+`). Versioning tables bumped
  to `cli-3.25.0` across the 3 CLI-REFERENCE + 3 README files.
- [x] T5.4 — Bumped `core` 0.4.1→0.5.0 + `cli` 3.24.0→3.25.0 (+ the `straymark-core` dep
  references in `cli/Cargo.toml` and `experimento/Cargo.toml`); `cargo check --workspace`
  refreshed `Cargo.lock` (workspace compiles, loom rebuilds against core 0.5.0). Root
  `CHANGELOG.md` gained `## CLI 3.25.0 / Core 0.5.0` (Added CLI / Added Core / Changed Core).
  PR → merge → tag `cli-3.25.0` (crates.io publish of `straymark-core` rides it).
- [x] T5.5 — `spec.md` §14 open questions struck through + resolved (artifact home confirmed;
  layer-seed confirmed; `status --where` shipped A1.4; auto-layout engine deferred to A2). Loom
  memory note updated: A1 done, next = **A2** (maxGraph render).

---

## A2 — Architecture Plan view (the visual "you are here") — `loom-0.5.0`

> **Release discipline (user-confirmed 2026-06-16):** single release. A2.0–A2.4 each merge
> **tagless**, branched fresh from `main` (no stacked-PR-on-squash hazard); **A2.5** carries the
> one `loom-0.5.0` tag covering all of A2 (the analog of A1.5 for the CLI track). Loom is dev-
> tested with a local branch binary + `--assets-dir experimento/web/dist` (not `loom serve`,
> which downloads the published release). The CLI/core are unaffected (`loom-*` is independent).

- [x] **A2.0 — Refactor: `build_governance_state` → `core::architecture` (de-risk, CLI as
  oracle).** New `core::architecture::gather` holds `build_governance_state(root)` + its helpers
  (`git_modified_files`, the open-TDE/closed-AILOG/declared extractors) and the source-file
  walker (`collect_source_files` + `EXCLUDED_DIRS`/`SOURCE_EXTENSIONS`). `agent_logs_dir` +
  `find_ailog_file` moved from `cli/src/ailog.rs` to `core::ailog`. `cli`'s `where_view.rs` calls
  `core::architecture::build_governance_state`; `common.rs` + `cli::ailog` re-export the moved
  `core` fns so the call sites are unchanged. Regression oracle green: the 3 `status --where`
  integration tests + the architecture-command suite byte-for-byte unchanged (710 tests). **core
  bumped 0.5.0 → 0.6.0** (0.5.0 is already published, so the additive `gather`/`ailog` surface
  can't ride the same version); `cli` + `experimento` core dep pins bumped. core 0.6.0 stays
  **unpublished** until the next `cli-*` release publishes it (loom uses the local path).
- [x] **A2.1 — Server architecture endpoints (spec §7).** New `experimento/src/architecture.rs`
  holds **axum-free, fixture-testable** builders (`build_architecture`, `component_detail`,
  `build_where`, `read_plan_drawio`) reusing `core::architecture::{parse_model,
  build_governance_state, project}` + `collect_source_files`/`glob_match`/`charter`/`ailog`; the
  thin handlers in `server.rs` wrap them. Routes: `GET /api/architecture`
  (`{model_present, layers (+per-state counts), components (+states), edges}`),
  `/api/architecture/component/{id}` (meta + states + owned on-disk files), `/api/where`
  (active charters + declared/touched progress + recent AILOGs + open-debt files),
  `/api/architecture/plan.drawio` (raw XML, `application/xml`, geometry preserved — status is a
  client overlay, NFR1). Read-only (NFR4); local `serde::Serialize` types keep the `core`
  surface stable. **Added `--arch-dir`** to the loom binary + `AppState` (splits `project_root`
  for governance/globs from the model dir) so the dogfood (`experimento/architecture/`, repo-root
  globs) resolves — analogous to the CLI's `--out`. **Integrity signals (FR9) deferred** —
  `validate_model` needs the `plan.drawio` cell ids; the DrawIO parser lives in the CLI and rides
  the A2.3 frontend (move to `core` then). 5 fixture tests + e2e curl on the dogfood (4
  components project `uncharted`, `component/cli` owns 104 files). 715 tests; clippy clean.
- [x] **A2.2 — Watcher: architecture deltas (FR6).** The shared `notify` watcher now also
  recomputes the architecture overlay on a settled change and broadcasts a small
  `{"event":"architecture"}` **signal** over the existing `WS /api/stream` (the client refetches
  `/api/architecture` + `/api/where` — a signal, not the payload, keeps the watcher decoupled
  from the response shape; the projection is small so a refetch is cheap). **No-op suppression**
  reuses the KG pattern: `architecture::projection_signature` (the `build_architecture` JSON) is
  cached in the closure and compared, so a bare mtime touch that doesn't move any component's
  state broadcasts nothing. Relevance widened from `.md`-only to `.md` ∪ `model.yml` ∪
  `.drawio` (`is_architecture_relevant`); the KG still only rebuilds on `.md`. When `--arch-dir`
  points outside the watch dir, that dir is watched too. e2e (python `websockets`): flipping a
  Charter `in-progress`→`declared` delivers the `architecture` event; 717 tests; clippy clean.
- [ ] **A2.3 — Frontend: maxGraph plan render (FR3, NFR1).** Add `@maxgraph/core` to
  `web/package.json`. Introduce a top-level **view switch** (KG | Plan tabs) without rewriting
  the Sigma monolith (`main.ts` stays; add `plan.ts` + a thin switcher). Load
  `/api/architecture/plan.drawio` into maxGraph, **preserve the human geometry**, and apply the
  §4 status as **non-destructive cell-style overrides** (fillColor/opacity/stroke/badge) keyed
  on `straymark_component_id`. i18n strings via `i18n.ts`.
- [ ] **A2.4 — Frontend: toggles, panels, cross-view (FR4/FR5/FR8/§8).** Layer toggle
  (show/hide cells by component `layer`), component detail panel (S2 — click a component →
  Charters/docs/debt/owned files), shared "Where are we" panel (`/api/where`), and cross-view
  linking (component → filter the KG to its docs; KG doc select → highlight components). Follow
  the event-delegation-on-stable-container pattern (avoid the per-element-listener storm bug).
- [ ] **A2.5 — Acceptance + release.** Spec §11.2 (lit/shaded/dimmed overlay), §11.3 (DrawIO
  round-trip lossless — edit `plan.drawio` in real DrawIO, reload, geometry + overlay intact),
  §11.4 (layer toggle), §11.6 (live overlay < ~1s). Dogfood on `experimento/architecture/`.
  `experimento/CHANGELOG.md` + bump `experimento/Cargo.toml`; PR → merge → tag **`loom-0.5.0`**.
  Update spec §11 acceptance + the Loom memory note (A2 done, next = A3 axonometric north star).

## A3 — Axonometric/BIM exploded view (north star, post-MVP)

- [ ] A3.1 — 2.5D stacked, explodable layers (the isometric "floors"). Pursued once the model
  is proven. (spec §12, §13.)
