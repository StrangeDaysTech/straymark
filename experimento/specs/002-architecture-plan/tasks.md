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

## A1.2 — `straymark architecture generate` (hybrid seed) — FR7, spec §5

> Writes `model.yml` + `plan.drawio`. The server stays read-only (NFR4); generation is the
> CLI's job (spec §7).

- [ ] T2.1 — CLI wiring: `Architecture { #[command(subcommand)] }` in `main.rs` with
  `Generate { path, --force }`, `Sync { path }`, `Validate { path }`; handler module
  `cli/src/commands/architecture/{mod,generate,sync,validate}.rs`. (CLAUDE.md command table
  gets a new row.)
- [ ] T2.2 — Codebase-structure seed: reuse `analysis_engine::walk_source_files` inventory to
  propose components from top-level dirs/modules; map each proposed component's `globs` to its
  dir (`dir/**`). Seed `layers` from `DocType::directory()` stages 00–09 (user-renamable).
  (spec §5 generate, §3.1.)
- [ ] T2.3 — ADR enrichment: mine **C4 mermaid blocks** and **"Affected Components"** tables
  from existing ADRs (`DocType::Adr` bodies) to enrich/rename proposed components and add
  `links` edges. New parser `cli/src/commands/architecture/adr_mining.rs` (C4 container/
  component lines + the markdown "Affected Components" table the ADR template ships). (spec §5)
- [ ] T2.4 — Auto-layout `plan.drawio`: emit valid mxGraph XML with one cell per component,
  grouped by layer (layered layout — dagre-style row-per-layer is enough for the seed; no JS
  dependency, pure Rust emitter), each cell carrying `straymark_component_id`. Geometry is a
  first draft the human re-arranges in DrawIO. (spec §3.2, §6; NFR1 — Loom only restyles.)
- [ ] T2.5 — `--force` guard: refuse to overwrite an existing `model.yml`/`plan.drawio`
  without `--force` (protects human edits). Idempotent dry default prints what it *would*
  write.

## A1.3 — `straymark architecture sync` + `validate` — FR7, FR9, spec §5, §3.3

- [ ] T3.1 — `validate`: parse `model.yml`, parse `plan.drawio` cell ids
  (`straymark_component_id` attrs), walk disk for glob coverage → report the §3.3 integrity
  signals (undrawn / unmodeled / empty) via `core::architecture::validate_model`. Human +
  `--output json`. (FR9)
- [ ] T3.2 — `sync`: detect **new** code dirs (vs current `model.yml` globs) and **new** ADR
  components since last generation; **append** suggested components/links to `model.yml` and
  **report** them — never clobber human edits, never overwrite `plan.drawio` geometry
  (append-only new cells, auto-placed). (spec §5 sync, NFR1.)
- [ ] T3.3 — Tests: `architecture validate` on a fixture with one undrawn + one unmodeled +
  one empty component reports exactly those three; `sync` on a fixture with a new dir appends
  one component and leaves existing cells byte-identical.

## A1.4 — `straymark status --where` (textual "you are here") — spec §8, §14

- [ ] T4.1 — Build `GovernanceState` in the CLI from the existing extractors now in `core`
  (active/closed charters via `discover_and_parse`, declared files via `parse_files_to_modify`,
  drift via `core::drift::compute_drift` against git-modified, TDE docs via
  `discover_documents`+`DocType::Tde`+open status, wiring-gap via the declared-vs-wired
  profile, on-disk inventory via `walk_source_files`).
- [ ] T4.2 — `straymark status --where [path]`: load `architecture/model.yml` (if present),
  call `core::architecture::project`, print the per-layer/per-component state with the active
  ("you are here") components highlighted, plus the §8 "Where are we" summary (active charters
  + declared-vs-modified progress + recent AILOGs + open debt). Degrade gracefully with a
  helpful message if no `model.yml` exists (point to `architecture generate`).
- [ ] T4.3 — **Consistency gate (NFR3):** the components flagged `active`/`in-progress` match
  `straymark charter list --status in-progress` + `charter drift`; `implemented` matches
  `charter list --status closed`. Add an integration test asserting this equivalence on a
  fixture corpus. (spec §11.5 textual half.)

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
