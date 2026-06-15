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

## A1.5 — Dogfood, acceptance, docs, release

- [ ] T5.1 — Run `straymark architecture generate` on this repo → commit a reviewed
  `experimento/architecture/model.yml` + `experimento/architecture/plan.drawio` (the dogfood
  artifacts; adopter home is `.straymark/architecture/`). Hand-refine to a legible first plan.
- [ ] T5.2 — **Acceptance (spec §11, A1 subset):** §11.1 (`generate` yields editable
  `model.yml`+`plan.drawio`) ✓; §11.5 textual half (`status --where` matches `charter list
  --status in-progress` + drift) ✓; Validation Criteria "Generator usefulness" — record the
  manual-edit count vs from-scratch. (Visual criteria §11.2/3/4/6 are **A2**.)
- [ ] T5.3 — Docs: add `architecture generate|sync|validate` and `status --where` rows to the
  CLI command table (CLAUDE.md is already updatable; also `docs/adopters/CLI-REFERENCE.md`
  EN + `docs/i18n/{es,zh-CN}/adopters/CLI-REFERENCE.md`). Note experimental status.
- [ ] T5.4 — Bump `core` minor (new `architecture` module surface) + `cli` minor; update
  `Cargo.lock`; root `CHANGELOG.md` `## CLI X.Y.Z` with `### Added (CLI)`
  (architecture command + `status --where`). PR → merge → tag `cli-X.Y.Z`. crates.io publish
  of `straymark-core` rides the `cli-` release per the M0 decision.
- [ ] T5.5 — Update `experimento/specs/002-architecture-plan/spec.md` §14 open questions:
  mark resolved (artifact home confirmed; `status --where` shipped in A1). Update the Loom
  memory note: A1 done, next = A2 (maxGraph render).

---

## A2 — Architecture Plan view (skeleton; refine when A2 starts) — `loom-0.x`

- [ ] A2.1 — `GET /api/architecture`, `/api/architecture/component/:id`,
  `/api/architecture/plan.drawio`, `/api/where` in the Loom server, reusing
  `core::architecture::project` (server builds `GovernanceState` from `core` extractors — the
  reason A1.0 moved them). (spec §7)
- [ ] A2.2 — Watcher extension: rebuild + push architecture-status deltas on `.straymark/` or
  `architecture/` changes over the shared `WS /api/stream`. (FR6)
- [ ] A2.3 — `web/`: maxGraph (`@maxgraph/core`) loads `plan.drawio`, preserves geometry,
  applies status as non-destructive cell-style overrides keyed on `straymark_component_id`.
  (FR3, NFR1)
- [ ] A2.4 — Layer toggle (FR4), component detail panel (FR5/S2), "Where are we" panel
  (`/api/where`), cross-view linking with the KG (FR8/§8).
- [ ] A2.5 — Acceptance: spec §11.2/3/4/6 (lit/shaded/dimmed overlay, DrawIO round-trip
  lossless, layer toggle, live overlay < ~1s). CHANGELOG + `loom-0.x` tag.

## A3 — Axonometric/BIM exploded view (north star, post-MVP)

- [ ] A3.1 — 2.5D stacked, explodable layers (the isometric "floors"). Pursued once the model
  is proven. (spec §12, §13.)
</content>
</invoke>
