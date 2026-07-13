# Changelog

All notable changes to StrayMark (formerly DevTrail; rebranded 2026-05-08, see [`ADR-2026-05-08-001`](docs/decisions/ADR-2026-05-08-rebranding-straymark.md)) will be documented in this file. Historical entries below preserve the "DevTrail" name where present — that was the project's name at the time of those releases.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [independent versioning](README.md#versioning) for Framework (`fw-`) and CLI (`cli-`).

---

## CLI 3.35.0 — 2026-07-13

### Added (CLI)

- **Auto-adoption safeguard S2 — provenance sentinel.** `straymark init` now writes
  `.straymark/.provenance.yml` recording `role: installed-project`, the pinned `framework_version`,
  `installed_at`, and `source_release`. `resolve_project_root` refuses to resolve a `.straymark/`
  whose provenance records a non-install role (e.g. a test fixture or a hand-marked distribution
  copy) — while **tolerating an absent sentinel** so existing adopters (with no provenance file)
  keep working. Complements the S1 distribution-source guard (they now share one
  `non_project_reason` check); the `framework_version` field is the pinned-release record the
  lagged self-adoption is built on. `straymark remove` cleans the file. Decision + plan:
  `docs/decisions/AIDEC-2026-07-13-001-*`.

  Deferred (tracked in the implementation plan): `straymark update-framework` refreshing the
  provenance version fields — harmless until safeguard S3 reads them.

---

## CLI 3.34.0 — 2026-07-13

### Added (CLI)

- **Auto-adoption safeguard: refuse to operate on the framework distribution source.**
  `resolve_project_root` now skips a `.straymark/` that sits next to a `dist-manifest.yml`
  (i.e. StrayMark's own `dist/`), so a command run with cwd or `--path` inside `dist/` can no
  longer resolve the shipped template as an installed project — it falls through to the real
  git-root install or reports "not installed", printing a `note:` explaining the skip. This
  is the mechanical guard (safeguard **S1**) that must exist before StrayMark self-adopts:
  without it, `straymark new`/`ailog`/`validate` pointed at `dist/` would read and write into
  the distribution. Pure detection helper `utils::is_distribution_source`.
- **CI hygiene backstop (safeguard S6):** a `dist-hygiene` workflow fails a PR/push if any
  dated governance artifact (`AILOG-2*`, `AIDEC-2*`, `ADR-2*`, `*.telemetry.yaml`,
  `CHARTER-<n>*`) appears under `dist/.straymark/` — the pollution S1 prevents, caught in CI
  if it ever slips through.

---

## Framework 4.35.0 / CLI 3.33.0 — 2026-07-13

### Added (Framework)

- **`AUDIT-ROUNDS-PATTERN.md` governance doc (EN/es/zh-CN) ([#341]).** Documents the per-round
  audit layout (sibling of `FOLLOW-UPS-BACKLOG-PATTERN.md`): the `--round <label>` namespacing, the
  resulting subfolder layout, telemetry `round:` tagging, and back-compat. Listed in
  `QUICK-REFERENCE.md`.
- **`AGENT-RULES.md` §12 (Audit Checkpoint) — two new bullets (EN/es/zh-CN).** "Audit a stable
  state" (#345 — PR → CI green → `--prepare` → launch auditors) and "Multi-round audits" (#341).

### Changed (Framework)

- **Audit skills thread `--round <label>` ([#341]).** `straymark-audit-prompt`,
  `straymark-audit-execute`, and `straymark-audit-review` (all four shipped copies each —
  `.agent`/`.claude`/`.gemini`/`.codex`) now document the per-round subfolder paths and pass the
  round label through the triad. `straymark-audit-review` also corrects the stale "CLI rejects
  re-audit" note to describe the new append-a-new-round behavior. `straymark-audit-prompt` gained
  the #345 "audit a stable state" note.

### Added (CLI)

- **`straymark charter audit --round <label>` ([#341]).** Optional per-round namespacing for
  multi-phase Charters that need more than one external-audit round. The whole triad (prompt,
  `report-*.md`, `review.md`) is written under `.straymark/audits/<CHARTER-ID>/<label>/`, so rounds
  never overwrite each other and the non-recursive `report-*.md` glob scopes to exactly the current
  round — fixing the cross-round report pollution into `review.md`/telemetry. The label is validated
  as a filesystem-safe slug. Each merged `external_audit` entry carries a `round:` field, and
  `--merge-into` **appends** a new round to an already-populated telemetry block (instead of bailing)
  as long as the round label is new; re-merging the same round, or merging into a populated block
  without a round label, is still rejected. Omit `--round` for single-round Charters (flat layout,
  fully back-compatible).

---

## CLI 3.32.0 — 2026-07-12

### Added (CLI)

- **`straymark charter audit --prepare`: stable-state guard ([#345]).** Adopter field report
  (greenfield .NET 10 / Rust CRDT): the audit prompt embeds the git diff at `--prepare` time,
  so generating it while the PR's CI was still in flight produced a stale prompt when CI forced
  a fix — the running auditors reviewed dead code. `--prepare` now emits a non-blocking warning
  when the working tree is dirty (`git status --porcelain`) or has unpushed commits
  (`git rev-list --count @{u}..HEAD`), reminding the operator of the stable-state sequence:
  PR → CI green → `--prepare` → launch auditors. Best-effort (git errors are silent); never
  blocks a prepare.

  Note: the `--round <label>` per-round namespacing for multi-phase audits ([#341]) is scoped
  to a dedicated follow-up round.

[#345]: https://github.com/StrangeDaysTech/straymark/issues/345
[#341]: https://github.com/StrangeDaysTech/straymark/issues/341

---

## CLI 3.31.0 — 2026-07-12

### Added (CLI)

- **`straymark charter close`: review checkpoint for unsigned `review_required` AILOGs
  ([#350]).** Adopter field report (weft): the close flow and the AILOG review/sign flow
  (`straymark approve`) were disconnected — a `review_required: true` AILOG only got signed
  if the operator happened to remember. Close now resolves the Charter's just-written AILOGs
  (the same git-range ∪ working-tree scope the follow-ups reconciliation uses) and prints a
  non-blocking `── Review checkpoint ──` block listing any that are `review_required: true`
  with no `review_outcome`, with the exact `straymark approve … --outcome approved` command
  to run. Warning, not a gate (honest gaps > forced signatures); runs in every close mode.

### Fixed (CLI)

- **`straymark followups drift`: extraction fidelity for `## Follow-ups` sections and
  emergent-risk blocks ([#346]).** Adopter field report (greenfield .NET 10 / Rust CRDT):
  `drift --apply` populated the backlog with noise and missed the real items. Three heuristic
  failures fixed:
  - **Under-capture** — an explicit `## Follow-ups (auditoría externa)` heading was skipped
    because heading matching required exact equality. Matching is now prefix-tolerant and
    collects **all** follow-ups sections, so suffixed/localized headings are extracted.
  - **Over-capture** — a prose summary line that merely mentioned `R<N> (new, not in Charter)`
    was extracted as a follow-up. Extraction now requires a *structural* risk declaration
    (a heading or list item that begins with the `R<N>` token); narrative paragraphs are
    skipped.
  - **Resolved-as-open** — a `## Risk: R<N>` heading whose remediation is documented inline
    in the section body (`Corregido a…`, a `Mitigaciones aplicadas` sub-block, an AIDEC
    reference) was extracted as `open`. Closure is now judged over the whole risk section
    (with ES/EN remediation participles), so resolved risks land as `suspected-closed`
    instead of polluting the open count.

  Not changed: dedup is still by content hash, so two rewordings of the same risk across an
  AILOG edit can still produce two entries (substance-level dedup deferred — changing the
  hash would re-drift already-extracted entries in existing registries).

[#346]: https://github.com/StrangeDaysTech/straymark/issues/346
[#350]: https://github.com/StrangeDaysTech/straymark/issues/350

---

## Framework 4.34.0 — 2026-07-08

### Fixed (Framework)

- **`straymark-audit-review`: the calibrator identity is now operator-provided, never
  self-detected.** Same class of bug fixed for `straymark-audit-execute` in 4.33.0, but for
  the consolidation step: when the review runs inside a router CLI (Qwen Code, Gemini CLI,
  …), the `calibrator:` / `**Reviewer:**` fields would carry the CLI product name instead of
  the operator-selected backend model. Step 1 now accepts an optional second argument
  `/straymark-audit-review <CHARTER-ID> <CALIBRATOR-SLUG>`, a calibrator-identity note makes
  the operator-provided identity authoritative and forbids the CLI product name, and a guard
  verifies the written `calibrator:` and `**Reviewer:**` fields match the slug. Applied to
  all four shipped copies (`.agent` workflow + `.claude`/`.gemini`/`.codex` skills).

---

## Framework 4.33.0 — 2026-07-08

### Fixed (Framework)

- **`straymark-audit-execute`: the auditor identity is now operator-provided, never
  self-detected.** Router CLIs (Qwen Code, Gemini CLI, …) inject a product identity by
  system prompt, so auditors would write `auditor: qwen-code` even after the operator
  confirmed a different backend model was selected via `/model` — corrupting attribution
  and faking cross-family agreement in the review step. Step 2 was rewritten to make the
  operator-provided identity authoritative (via an optional second argument
  `/straymark-audit-execute <CHARTER-ID> <AUDITOR-SLUG>` or an in-chat statement), to
  forbid substituting the CLI product name, and to add a mandatory post-write guard that
  verifies the `auditor:` frontmatter and the report header match the provided slug.
  Applied across all four shipped copies (`.agent` workflow + `.claude`/`.gemini`/`.codex`
  skills). Self-detection remains only as a fallback when the operator provides nothing.

---

## Framework 4.32.0 — 2026-06-28

Audit-prompt **v1.1**: a verification-fidelity pass on the unified audit prompt
(`.straymark/audit-prompts/audit-prompt.md`, EN + es), resynced with what the
framework learned since the v1 hardening (#261). The schema and the four finding
categories are unchanged; the changes are additive guidance.

### Changed (Framework)

- **Audit object vs. truth oracle + cross-boundary contracts** (closes #303). The
  `## STRICT scope` section now distinguishes *where you report defects* (the
  audit object) from *what you may read to validate them* (a truth oracle), and
  requires a client-auditing run to cross-check API/IPC/contract calls against
  the **real server-side definition** even when it is outside the `git_range`. A
  client↔server contract mismatch is an auditable defect of the client; green
  client-side tests do not absolve it (mocks encode the client's assumption).
- **Verification fidelity in Step 2** (addresses #306). For each
  "verified/resolved/done" claim, the auditor now asks *against which reality* it
  was checked — the condition that matters vs. a convenient proxy (local test,
  mock, the doc's own assertion) — opens the artifact rather than trusting a
  downstream summary, and flags a contract consumer that does not reference the
  decision defining its contract.
- **Follow-ups registry as the canonical `real_debt` destination**. The
  `real_debt` finding category now points to the follow-ups backlog registry
  (first-class since fw-4.21.0) with TDE promotion (`straymark followups
  promote`), instead of a loose "post-audit TDE doc".

## Framework 4.31.0 / CLI 3.30.0 — 2026-06-28

Graduate the Baton experiment's declared work-classification fields to the
framework (#332 step 2). After the schema was ratified
(`experiment-baton/06-work-verb-schema-ratification.md`), `work_verb` and
`design_provenance` become optional, additive, first-class Charter / follow-up
fields — declarable at authoring (cost ≈ 0) and surfaced by the validator as an
advisory nudge. No existing document changes shape; the 100%-undeclared legacy
corpus stays quiet.

### Added (Framework)

- **`work_verb` and `design_provenance` Charter frontmatter fields** (#332).
  Added to `charter.schema.v0.json` as optional `string` properties (documented
  vocabulary, intentionally **not** an `enum` so an out-of-vocabulary value is an
  advisory warning, never a blocking schema error) and to the Charter template
  (EN/es/zh-CN) as commented guidance. `work_verb`: design | implement | audit |
  operate. `design_provenance`: new | upstream (only meaningful for implement —
  upstream degrades the routing tier to operator).
- **Follow-up entry schema gains `Work verb` / `Design provenance` lines**
  (`FOLLOW-UPS-BACKLOG-PATTERN.md`, EN/es/zh-CN). Optional; documents the same
  controlled vocabulary at the follow-up grain.

### Added (CLI)

- **`straymark validate` advisory for declared classification** (#332). New
  warning-only rules `CHARTER-WORK-VERB` and `CHARTER-DESIGN-PROVENANCE` fire
  only when a Charter declares the field with a value outside the controlled
  vocabulary. An absent field emits nothing (anti-noise for legacy corpora); the
  check never affects the exit code.

## Framework 4.30.0 / CLI 3.29.0 / Core 0.9.0 — 2026-06-20

Adopter-feedback consolidation: four validator gaps from a 150-doc housekeeping
pass (#215) plus two Charter-workflow friction fixes (#207, #208), reported by
the Sentinel and lnxdrive adopters.

### Added (CLI)

- **META-003 now suggests the nearest canonical status** (#215 Gap 4). Invalid
  `status:` values get a `Did you mean '<x>'?` hint via a semantic alias map
  (`done`/`completed`/`closed` → `accepted`, …) with a Levenshtein fallback for
  typos.
- **`charter audit --prepare` warns on multi-batch under-coverage** (#208). When
  the Charter (or its referenced AILOGs) has completed batches in the Batch
  Ledger and no explicit `--range` was given, the CLI warns that the default
  `origin/main..HEAD` excludes already-merged phases and recommends an explicit
  range — turning a silent under-coverage into a visible one.

### Changed (CLI)

- **OBS-001 vocabulary widened** (#215 Gap 1). The observability content check is
  now case-insensitive over a broad term set (`otel`, `metric`/`métrica`, `span`,
  `trace`, `dashboard`, `collector`, `alert`, `slog`, `histogram`, `telemetr`, …),
  eliminating the false positives the narrow literal set produced.
- **`charter status` no longer advertises shipped commands as unavailable**
  (#207). Removed the stale "Phase 2 features (not yet available)" block that
  listed `charter drift` / `charter close` as `planned cli-3.7.0`.

### Added (Core)

- **Charter `execution_ailogs` + `context_spec` fields** (#215 Gap 2) — bless the
  documented close-time AILOG aggregation and area-context spec for dual-origin
  Charters, keeping `originating_ailogs`/`originating_spec` mutually exclusive.
  Wired into the architecture `implemented` overlay and the knowledge-graph edges;
  `CHARTER-AILOG-REF`/`CHARTER-SPEC-REF` cover the new fields.
- **`CHARTER-FILES-EXIST` exemption markers** (#215 Gap 3) — `(external)`,
  `(removed)`, `(relocated: <path>)` alongside `(new)`, so a closed Charter's
  historical table can carry cross-repo / relocated / removed paths. Unmarked
  missing paths (e.g. unsubstituted placeholders) are still flagged.

### Changed (Framework)

- **Charter schema** gains `execution_ailogs` + `context_spec` (mutual-exclusion
  unchanged); **charter + AILOG templates** (EN/es/zh-CN) document the new fields,
  the `status:` lifecycle vocabulary inline, and the `charter drift --range`
  syntax (dropping stale "Phase 2" language).
- **Skills**: `straymark-charter-new` documents the dual-origin fields;
  `straymark-audit-prompt` documents the multi-batch `--range` pitfall.
- **Docs**: CLI-REFERENCE (EN/es/zh-CN) documents the `charter audit --range`
  multi-batch pitfall.

---

## Framework 4.29.0 — 2026-06-20

Agent-native skills for the architecture model and the Loom lifecycle (#281) —
the terminal-free counterpart to the manual DrawIO refinement path. The
`architecture generate|sync|validate` commands and `loom serve` already shipped;
this release wraps them in skills so an adopter can drive the whole
`generate → refine → loom up` arc from the agent window, never editing YAML/DrawIO
or opening a shell.

### Added (Framework)

- **`/straymark-architecture` skill** *(EXPERIMENTAL, Loom A1.x)* — drives the
  `generate → refine → validate` arc in one guided pass: seeds the model with
  `straymark architecture generate`, then reassigns components out of the
  placeholder `unassigned` layer into real layers, infers dependency `links` from
  the import graph / directory structure / ADR "Affected Components" tables, syncs
  the `plan.drawio` (vertices *and* edges) so the 2D view shows arrows, and
  iterates `validate` to green. Pre-encodes the Sentinel refinement gotchas
  (`component.id` must not equal a `layer.id`; `links` is a list of string ids,
  not objects; 3D edges come from `model.yml` `links` while 2D edges come from
  `plan.drawio`; never delete a layer a component still points at) so no future
  agent re-hits them.
- **`/straymark-architecture-sync` skill** *(EXPERIMENTAL, Loom A1.3)* — wraps
  `straymark architecture sync` (append-only) to keep a curated model current as
  code grows: dry-run, surface new dirs/components, confirm, `--apply`,
  re-validate. Never re-refines from scratch.
- **`/straymark-loom` skill** *(EXPERIMENTAL, Loom v0)* — owns the Loom server
  lifecycle (up / down / status) from the agent window. Launches
  `straymark loom serve --no-open` in the background and reports
  `http://127.0.0.1:7700`, so the terminal-free operator just gets a link to
  click. The agent owns the process; the user stays in the chat window.
- All three skills ship across `.claude/`, `.gemini/`, `.codex/`, and
  `.agent/workflows/` like the existing skills, and are documented in the README,
  WORKFLOWS, QUICK-REFERENCE, and LOOM adopter guide (EN + es + zh-CN).

---

## CLI 3.28.2 — 2026-06-18

### Fixed (CLI)

- **`straymark explore` TUI is now legible on light terminals.** The explore
  TUI shipped a hard-coded dark (Catppuccin-inspired) palette built from fixed
  `Color::Rgb(...)` values and painted its own dark background over the whole
  screen, so it ignored the terminal's theme — on a light-background terminal
  the body text rendered nearly invisible (dark-on-dark). The TUI now defers to
  the terminal: backgrounds use `Color::Reset` (no forced fill), body text uses
  the terminal's default foreground, accents use the 16 standard ANSI colors
  (which the terminal remaps per its active theme), and code chips / selected
  rows use reverse video instead of a fixed background. Result: `explore` is
  readable on both light and dark terminals with zero configuration — the same
  principle the rest of the CLI already follows.

---

## CLI 3.28.1 — 2026-06-17

### Fixed (CLI)

- **`cargo install straymark-cli` no longer installs the internal `gen_codex_skills` build tool.**
  That binary lives in `cli/src/bin/` and was auto-discovered by Cargo as a second target, so the
  cargo-install path (used by `straymark update-cli`) leaked it into users' `~/.cargo/bin/`
  alongside `straymark`. It is now gated behind a non-default `dev-tools` feature
  (`required-features`), so `cargo install` ships only the `straymark` binary. CI and local
  regeneration invoke it with `--features dev-tools`. The GitHub-release tarballs were never
  affected (they only ever packaged `straymark`). Users who already have the leaked binary can
  remove it with `rm ~/.cargo/bin/gen_codex_skills`.

---

## Framework 4.28.0 / CLI 3.28.0 / Core 0.8.0 — 2026-06-16

Scoped `has-debt` attribution via a TDE `affects` field (#276). The framework ships the field in
`TEMPLATE-TDE.md` under `fw-4.28.0`; the CLI + `straymark-core` under `cli-3.28.0` (core 0.8.0 to
crates.io). Loom's overlay picks up the same projection in `loom-0.6.2` (no Loom code change — it
rebuilds against core 0.8.0 so the visual overlay and `status --where` stay consistent, NFR3).

### Added (architecture `has-debt` precision — #276, EXPERIMENTAL)

- **TDEs can scope their debt with an `affects` field.** Until now the `has-debt` overlay
  attributed an open TDE's debt to every component its `related` AILOGs touched — wider than the
  debt itself (a TDE about AuditTrail also lit `cmd`/`db`/`integration` because the same AILOG
  wired them). A TDE may now declare `affects: [<file globs>]` (e.g.
  `["internal/modules/audittrail/**"]`); when present, the projection attributes the debt to
  **exactly** those paths (expanded against the on-disk source files), ignoring the broader AILOG
  footprint. Absent `affects`, the AILOG-footprint attribution (#273) remains the fallback.
- **Framework:** the `affects` field is documented in `TEMPLATE-TDE.md` (EN / es / zh-CN).
- **Core:** `straymark-core::document::Frontmatter` gains `affects` (additive) — `straymark-core`
  bumped to **0.8.0** and published with `cli-3.28.0`.

---

## CLI 3.27.0 / Core 0.7.0 — 2026-06-16

A language- and structure-agnostic, configurable seed for `architecture generate` (#279), plus a
build-consistency fix (Loom's `straymark-core` dependency bound was left at `^0.6.0` after core
bumped to 0.7.0).

### Changed (architecture scanner — language/structure-agnostic, #279, EXPERIMENTAL)

- **`architecture generate` is now configurable and ecosystem-aware.** The source-file scan and
  the component-dir shaping moved to a single `straymark-core::architecture::scan` module driven
  by a `ScanConfig`, resolved from an optional **`architecture:` section in
  `.straymark/config.yml`** (additive over built-in defaults):
  - `source_extensions` — the seed now recognizes ~30 languages out of the box (added Ruby,
    Elixir, Scala, Dart, Clojure, Vue, Svelte, Lua, Julia, Haskell, Erlang, OCaml, …); a project
    in a non-default language is no longer an empty map, and adopters can add more.
  - `scaffolding_prefixes` — build scaffolding like `src/main/java` / `src/main/kotlin` is skipped,
    so a **Maven/Gradle** module is attributed to its module directory instead of collapsing the
    whole project into a single `main` box (multi-module → one component per module).
  - `container_dirs` / `excluded_dirs` — extend the descent/skip sets per project.
- Defaults are unchanged for existing Go/Rust/JS layouts (Sentinel still seeds the same 14
  components); the status overlay was already language-agnostic — this only shapes the seed.
- `straymark-core` → **0.7.0** (additive: new public `scan` API). Publishes with the next `cli-`
  release.

---

## Framework 4.27.0 / CLI 3.26.0 / Core 0.6.0 — 2026-06-16

Audit-prompt hardening (#261), architecture-model DX + a meaningful `has-debt` overlay (#273),
i18n-robust extractors (#263), and a follow-ups counter-integrity fix (#253). The framework ships
under `fw-4.27.0`; the CLI + `straymark-core` under `cli-3.26.0` (core 0.6.0 published to
crates.io). The Loom-side companion of #273's overlay ships in `loom-0.6.1`.

### Changed (architecture model DX — #273, EXPERIMENTAL)

- **`architecture generate` mines a real component structure.** The seed now descends through
  conventional *container* directories (`internal/`, `src/`, `pkg/`, `lib/`, `app/`, `modules/`, …)
  instead of mapping each top-level dir to one component. A Go `internal/` tree that used to
  collapse into a single `internal` blob now breaks out into `internal/core`,
  `internal/integration`, `internal/modules/<each>`, etc. — matching Loom's own live derivation,
  no manual refinement needed. A non-container leaf (`cmd/`, `db/`) still stays whole.
- **`has-debt` now maps onto components.** An open TDE's `related` frontmatter lists governance
  docs (AILOGs, audit reviews, Charters), not source paths, so the debt never matched a component
  glob and the overlay was always empty. The projection now resolves each `related` AILOG to the
  files it recorded as modified and feeds those source paths in, so debt lands on the components
  whose code the referenced AILOGs touched. An AILOG's modified files are read as the **union** of
  its `files_modified` frontmatter list and its `## Modified Files` table — older / etapa logs
  often carry only the frontmatter list, so reading just the table silently dropped them (it lit
  Identity/Core/Database but missed AuditTrail/CommsHub on Sentinel). The same union now also feeds
  the `implemented` signal from closed-Charter AILOGs. (The frontend half — painting `has-debt`
  over `implemented` — ships with Loom.)

### Fixed (architecture model DX — #273, EXPERIMENTAL)

- **Opaque `error: parsing model.yml`.** Two distinct violations (a component id colliding with a
  layer id; a component pointing at an unknown layer) used to fail with the same bare message
  because the CLI printed only the outermost error context. It now prints anyhow's full chain
  (`error: parsing …/model.yml: component id \`core\` collides with layer id \`core\` — …`), so
  the reason, not just the file, is visible — across every command.
- **Component↔layer id collisions get a specific message** (`component id \`core\` collides with
  layer id \`core\` — rename one`) distinct from a plain duplicate id, and the model invariants
  (ids unique across layers + components; no required `unassigned`-at-order-0 rule) are now
  documented in `validate_structure`.

### Fixed (CLI)

- `straymark followups` — the registry parser now detects well-formed `### FU-NNN` entry
  headings that are **invisible to the counters** and surfaces them as a warning instead of
  silently under-counting (#253). An entry goes invisible when its heading is glued to the
  previous line (no blank line before `### `) or sits before the first `## Bucket:` section;
  previously such an entry was dropped, so `recount` could report a clean "already in sync"
  while the backlog actually held one more open entry than the counter said. The warning now
  shows in `followups recount`, `status`, `list`, and `drift`.

### Fixed (i18n robustness — #263)

- **AILOG `## Modified Files` extractor** (`straymark-core`) now recognizes the heading in all
  three shipped locales (`Modified Files` / `Archivos modificados` / `修改的文件`), matching the
  Charter `Files to modify` extractor. A Spanish/Chinese-first AILOG no longer silently yields
  an empty file set (which under-reported `implemented` state in `status --where` / Loom).
- **Follow-ups `## Follow-ups` extractor** (`straymark followups drift`) now recognizes the
  section heading in the shipped locales (`Follow-ups` / `Seguimientos` / `后续工作` / `后续`), so
  drift extracts follow-ups from translated AILOGs instead of skipping the section.
- **Slug generation** (`straymark new`, `charter new`, `followups promote`) is now Unicode-aware:
  CJK and accented-Latin titles keep their characters instead of being vaporized to whatever
  ASCII fragments they contained. Truncation counts characters, not bytes.

Reference resolution was already slug-language-independent (it normalizes to the
`TYPE-YYYY-MM-DD-NNN` core id), and no extractor uses `\b`/whitespace ID regexes — so those two
risks from #263 needed no change.

### Changed (Framework — audit prompt hardening, #261)

- **Auditor independence is now enforced in the audit prompt** (`audit-prompts/audit-prompt.md`,
  EN + ES). The ABSOLUTE RULE forbids reading, grepping, or referencing any other auditor's
  `report-*.md` under `.straymark/audits/` — for this Charter or any other — because cross-model
  convergence is signal only when each auditor reached it independently. The "Your role" and
  "What you must NOT do" sections were reworded to match (a sibling report may already be on
  disk; do not open it).
- **The output contract is surfaced near the top** of the prompt (new "Output contract (read
  this first)" section, right after the ABSOLUTE RULE) instead of only at the end of a long
  prompt: required report frontmatter, the four finding categories, and an explicit warning that
  the report frontmatter is **deliberately different** from the embedded AILOG/AIDEC frontmatter
  the auditor reads (the mimicry that drifted real reports off-schema). A "Frontmatter note" was
  also added beside the embedded AILOGs.
- **`straymark-audit-review`** (skill, all runtimes) gained a **contamination guard**: it now
  scans each report for signs it read its siblings (references to another `report-*.md`, a
  cross-auditor comparison table, "I verified all N findings from the prior <model> audit") and
  excludes contaminated reports from the convergence/dedup math and the auditor rating.

---

## CLI 3.25.0 / Core 0.5.0 — Architecture Plan track A1 (EXPERIMENTAL)

The textual + authoring half of Loom's Architecture Plan view (Spec 002) — the operator's "where are we?" answered from the terminal, ahead of the visual overlay (A2). All surfaces are **EXPERIMENTAL (Loom v0)** and may change without a deprecation cycle.

### Added (CLI)

- `straymark architecture generate [path] [--force] [--out <dir>]` — write a first-draft
  `architecture/model.yml` + `plan.drawio` by mining codebase structure (one component per
  top-level source directory) enriched with ADR signal (C4 Mermaid diagrams + "Affected
  Components" tables improve labels and add links).
- `straymark architecture sync [path] [--out <dir>] [--apply]` — append-only reconciliation:
  detect new source dirs / ADR components not yet in the model and append them to `model.yml`
  + `plan.drawio` without clobbering human edits or DrawIO geometry. Dry-run by default.
- `straymark architecture validate [path] [--out <dir>] [--output <text|json|markdown>]` —
  report model↔plan integrity signals (`undrawn` / `unmodeled` / `empty`); exits 1 on any
  signal (CI-gateable). Degrades to glob-coverage-only when `plan.drawio` is absent.
- `straymark status --where [path] [--out <dir>]` — textual "you are here": projects each
  component's state (`active` / `in-progress` / `implemented` / `has-debt` / `uncharted`) from
  live governance signals (Charters + drift + open TDEs + on-disk inventory) and prints the
  "Where are we" summary. The `active`/`implemented` flags line up with
  `charter list --status in-progress`/`--status closed` + `charter drift` by construction.

### Added (Core)

- New `straymark_core::architecture` module: the typed `ArchModel` (`model.yml` parser +
  structural validation) and the **pure** `(model + GovernanceState) -> Projection` status
  function (zero I/O), shared by the CLI's `status --where` and the future Loom server so both
  compute the same answer. Plus `validate_model` for the model↔plan integrity signals.
- New `straymark_core::ailog::parse_modified_files` (the AILOG `## Modified Files` extractor
  feeding the `implemented` state).

### Changed (Core)

- Governance primitives the projection depends on — Charter parsing (`charter`,
  `charter_files`), the drift matcher (`drift`), and `split_frontmatter` (`utils`) — moved from
  `straymark-cli` into `straymark-core` so the CLI and Loom share one extractor (no behavior
  change; verified against the full regression suite).

---

## Loom 0.4.2 — R3: legibility at 100+ nodes

### Added (Loom)

- "Hide isolated" header toggle: hides nodes with no resolved edges (singletons, orphans) so
  the connected graph stands out; shows the hidden count.
- "Labels" header toggle: turn node labels off for a pure-structure view.
- On-screen zoom / fit-to-view controls (bottom-right) for precise navigation; trackpad/wheel
  pinch zoom is also gentler (lower `zoomingRatio`).

### Changed (Loom)

- Node-label density is capped so 100+ node graphs stay legible — only the most prominent node
  per screen region is labeled (centrality sizing keeps these meaningful), zooming reveals
  more, and the hovered/selected node always shows its full label.

---

## Loom 0.4.1 — fix: panel click responsiveness under filesystem churn

### Fixed (Loom)

- The watcher no longer broadcasts a WebSocket update when a rebuild yields an identical graph
  (a file's mtime moved but its content did not — an editor save, formatter, `touch`, or
  cloud-sync rewrite). These no-op broadcasts re-rendered open clients' side panels
  continuously and swallowed clicks on dangling-reference links, community buttons and stats
  sections.
- The stats and legend panels use event delegation on their stable containers instead of
  per-button listeners, so clicks survive the panels' innerHTML rewrites under frequent updates.

---

## Loom 0.4.0 — connectivity: reference normalization + entity nodes

Closes the Loom connectivity follow-ups (R1 + R2). On the Sentinel corpus, 330 of 395
references were dangling. The fixes live in the shared `straymark-core` graph builder, so
`straymark audit` gains the same connectivity.

### Added (Loom)

- Charter / plan / audit nodes (R2): `straymark-core::entities` discovers
  `.straymark/charters/*.md`, `plans/PLAN-*.telemetry.yaml` and `audits/*/review.md`, and
  `Graph::build_with_entities` injects them as `CHARTER` / `PLAN` / `AUDIT` nodes so
  references to them resolve.

### Changed (Loom)

- Reference normalization (R1): the graph builder resolves edge targets by exact id and then
  by unique file basename, unique relative-path suffix, `CHARTER-NN` prefix, or leading dated
  id prefix (never an ambiguous match); resolved targets are canonicalized to the node id.

Result on Sentinel: dangling references **330 → 87**, nodes **131 → 193**, orphans **2 → 0**.
The remaining references are to files outside the governance corpus and correctly stay
dangling. `straymark-core` → 0.4.0 (CLI dependency bumped to match).

---

## Loom 0.3.0 — M3 rich UI

Completes Loom M3 (`CHARTER-01-loom-server`): the analytical dashboard becomes a rich
exploration tool while remaining loopback-only and read-only.

### Added (Loom)

- Incremental rebuild with WebSocket `delta` events: a parse cache re-parses only changed
  files and the SPA patches the graph in place, preserving layout for unchanged documents.
- Dependency-cycle (SCC) reporting over the resolved semantic edges (`SUPERSEDES`,
  `ORIGINATES_FROM`), surfaced in `/api/stats` and the stats panel.
- Centrality-based node sizing with a selector (Betweenness — default, PageRank, Degree).
- Search with camera focus, "Pin subgraph" to isolate a thread, and VS Code / Cursor
  deep-links plus copy-path in the node panel (client-side; the server stays read-only).
- UI internationalization (`en` / `es` / `zh-CN`) driven by the project's configured
  language, served at the new `GET /api/meta` endpoint.

### Changed (Loom)

- Project language resolution moved into `straymark-core` (`core::config`); the CLI now
  delegates to it, so the CLI and Loom share one source of truth (`straymark-core` → 0.3.0).

---

## Loom 0.2.0 — M2 analytics and panels

Completes Loom M2 (`CHARTER-01-loom-server`): the walking skeleton becomes an analytical
dashboard while remaining loopback-only, read-only, and independently versioned.

### Added (Loom)

- Louvain community detection and cluster coloring over the undirected document graph,
  with a compact interactive legend labeled by representative document titles.
- Corpus stats panel with counts by type/status/risk, navigable orphan documents, and
  dangling references.
- Expanded node summary panel with clickable incoming/outgoing links, source path, and
  explicit truncated-excerpt signaling.
- Server-side `/api/graph` filters for type, status, risk, tag, and inclusive date range,
  with UI controls and filtered-view live rebuilds.

---

## Loom 0.1.0 / CLI 3.24.0 — Loom M1: the walking skeleton ships

First release of **Loom**, StrayMark's EXPERIMENTAL third component (`CHARTER-01-loom-server` M1): a loopback-only, read-only web dashboard that renders the project's document graph live in the browser.

### Added (Loom)

- **`straymark-loom 0.1.0`** (`loom-0.1.0`, GitHub-release-only): axum + tokio server that discovers and parses StrayMark documents via the shared `straymark-core` crate, builds the typed knowledge graph, and serves `GET /api/graph`, `/api/node/:id`, `/api/node/:id/thread?depth=N`, `/api/stats`, `/healthz`, and `WS /api/stream` over `127.0.0.1` only (Spec 001 §4).
- **Live rebuilds**: a `notify` watcher (250ms debounce) re-parses on settled `.md` changes and pushes a `rebuild` event over the WebSocket — an open browser reflects an edit in well under 1 second (measured ~255ms).
- **Web UI** (Sigma.js + graphology, embedded via rust-embed — adopters never run npm): force-directed graph colored by document type and sized by degree; selecting a node lights up its full thread (transitive in/out relationships) and dims the rest; node detail panel with metadata and body excerpt; live type legend and corpus counters.
- **Security posture** (Spec 001 FR7/NFR4): binds `127.0.0.1` exclusively, rejects non-loopback `Host` headers (anti DNS-rebinding), read-only by construction.
- **`release-loom.yml`**: 4-platform build matrix with the frontend compiled in CI and embedded; releases marked `--latest=false`.

### Added (CLI)

- **`straymark loom serve [path] [--port] [--no-open]`**: download-on-demand launcher — fetches the latest `loom-*` release binary for the host platform on first use (the download gate *is* the experimental opt-in boundary), caches it under `~/.straymark/bin/`, prints a loud EXPERIMENTAL banner, and launches it pointed at the project. The CLI gains no axum/tokio dependency. Falls back to the cached binary when offline.

### Added (core)

- `straymark-core 0.2.0`: `Graph::thread(id, depth)` — the connected neighborhood of a node (Spec 001 §3.3), powering `/api/node/:id/thread` and the UI's thread highlighting.

---

## CLI 3.23.1 — `straymark-core` extraction (Loom M0)

First milestone of the experimental **Loom** component (`experimento/`, `CHARTER-01-loom-server`): the document model and traceability graph move into a shared crate so the CLI and the upcoming Loom visualization server parse StrayMark documents with exactly the same code (`ADR-2026-06-02-001`). **No user-facing behavior changes** — the full test suite and the `straymark audit` output are byte-for-byte identical pre/post refactor.

### Added (CLI)

- **`straymark-core` crate** (published to crates.io): `core::document` (the document model, moved verbatim from `cli/src/document.rs`) and `core::graph` — a typed, bidirectional, orphan-preserving knowledge graph over frontmatter cross-links (`related`, `supersedes`, `alternatives_documented`, `api_changes`, `originating_ailogs`), with dangling references kept as first-class `resolved: false` edges (Loom Spec 001 §3).
- New optional frontmatter fields parsed (additive): `supersedes`, `alternatives_documented`, `originating_ailogs`.

### Changed (CLI)

- The repository root is now a Cargo **workspace** (`core` + `cli`); `[profile.release]` and `Cargo.lock` live at the root. Release binaries are built from `target/` (workspace) instead of `cli/target/` — `release-cli.yml` adjusted accordingly, and it now publishes `straymark-core` before `straymark-cli`.
- `audit_engine::build_traceability` is now a projection over `straymark_core::graph` (same output, one graph builder for all consumers).

---

## Framework 4.26.0 / CLI 3.23.0 — `charter drift` is native Rust (Windows-native parity)

Closes the last functional Windows-native gap ([#237](https://github.com/StrangeDaysTech/straymark/issues/237)): `straymark charter drift` no longer delegates to a bash script, so it runs without WSL or Git Bash.

### Changed (CLI)

- **`charter drift` ported to native Rust**: the command previously shelled out to `.straymark/scripts/check-charter-drift.sh` and failed on Windows-native (no `bash` in PATH). It now computes the declared-vs-modified set-difference in-process — declared files via `charter_files.rs` (already a byte-for-byte port of the script's awk extraction), modified files via `git diff --name-only`, plus the ellipsis/glob wildcard matching and the report. The deleted intermediate (parsing the script's stdout) removes a bug class. **AILOG-aware suppression, the Batch Ledger gate, all flags, exit codes, and output are unchanged** — preserved by the integration suite, which now runs on every platform (no bash gate) and doubles as the script-equivalence guarantee. The zero-false-positives property (Sentinel PLAN-05/PLAN-06) is retained.

### Deprecated (Framework)

- **`.straymark/scripts/check-charter-drift.sh`** is deprecated and unmaintained as of fw-4.26.0 / cli-3.23.0. It is no longer invoked by the CLI; it remains as a reference prototype (it seeded `charter_files.rs` and the native drift logic) and will be removed in a future release. Microsoft Coreutils was evaluated and rejected as a Windows script-parity vehicle (no shell, no `sed`/`awk`, preview status) — see the proposal under `docs/decisions/proposals/`.

---

## CLI 3.22.0 — `charter close` reconciles follow-ups and offers TDE promotion (RFC #135 Tier 3)

Closes the loop between Charter close and the follow-ups registry — the last open tier of the follow-ups automation roadmap ([#135](https://github.com/StrangeDaysTech/straymark/issues/135)), unblocked now that drift detection is reliable (cli-3.21.0, #229/#231).

### Added (CLI)

- **`charter close` follow-ups integration**: after an **interactive** close, the command runs the default `followups drift` scan (committed git range ∪ working tree) over the just-written AILOGs, extracts any `§Follow-ups` / `R<N> (new)` content not yet in the registry into `## Bucket: ready`, and then offers **per-entry TDE promotion** against the four AGENT-RULES.md §3 criteria. Declining a prompt leaves the follow-up extracted (captured, not promoted); accepting runs the `followups promote` flow (creates the TDE with `promoted_from_followup` traceability). No-op when there is no registry or nothing is unextracted; skipped on the `--from-template` paths (no interactive prompt context). The scan reuses the stabilized default drift (not scoped to `originating_ailogs`, which is the ex-ante seed rather than the execution AILOGs where follow-ups live).

### Changed (CLI)

- Refactored `followups drift` into reusable, side-effect-free cores — `detect_drift_candidates()` (scan + per-follow-up hash dedup) and `apply_candidates()` (write + return the created `FU-NNN` ids) — shared by `followups drift` and the new `charter close` integration. `followups drift` output is unchanged.

---

## Framework 4.25.0 / CLI 3.21.0 — follow-ups drift correctness: see the working tree + catch appended follow-ups

Two silent-data-loss bugs in `straymark followups drift`, both surfaced by the reference adopter (Sentinel) and both prerequisites for wiring drift into the Charter-close flow (RFC [#135](https://github.com/StrangeDaysTech/straymark/issues/135) Tier 3).

### Fixed (CLI)

- **`followups drift` default scan now sees the working tree** ([#229](https://github.com/StrangeDaysTech/straymark/issues/229)): the default scan considered only the committed git range (`git diff origin/main..HEAD`), so an uncommitted/untracked AILOG — the normal state at pre-commit time — was invisible, and the documented `drift --apply` flow reported "in sync" while real follow-up content went unextracted. The scan now unions the git range with the working tree (`git status --porcelain`), mirroring the v0 reference script. `--scan-all` semantics unchanged.
- **`followups drift` catches follow-ups appended to an already-extracted AILOG** ([#231](https://github.com/StrangeDaysTech/straymark/issues/231)): `fully_extracted_ailogs` was a whole-AILOG idempotency gate — once an AILOG id was in it, the file was never re-scanned, so follow-ups added later (the multi-batch Charter pattern, where one AILOG's `§Follow-ups` grows across batches) were silently dropped. Drift now dedups **per follow-up by a stable content hash** (`Source-hash`, stored on each entry): already-extracted AILOGs are re-scanned and individual follow-ups deduped, so appended content is caught. The stored hash is captured at extraction time and immune to later triage rewording, preserving the zero-false-positive property that motivated the original per-AILOG choice; legacy entries (pre-cli-3.21.0, no `Source-hash`) fall back to recomputing the hash from `Origin` + `description`. `fully_extracted_ailogs` is retained as informational metadata, no longer the skip gate.

### Changed (Framework)

- **`FOLLOW-UPS-BACKLOG-PATTERN.md`** (EN/ES/zh-CN): documents the working-tree union, the per-follow-up content-hash dedup (replacing the "Per-AILOG vs per-bullet granularity" section that described the #231 bug as an intentional trade-off), the new auto-managed `Source-hash` entry field, and the revised role of `fully_extracted_ailogs`.

---

## Framework 4.24.0 — portable installed skills: runtime-agnostic identity + current-work git context

Five portability/correctness fixes to the installed skills, surfaced by a Codex (`gpt-5.5`) review of the StrayMark skills running under Codex CLI in the reference adopter Sentinel ([#232](https://github.com/StrangeDaysTech/straymark/issues/232)). The skills ship in four families under `dist/` (`.claude/`, `.gemini/`, `.agent/workflows/` hand-maintained; `.codex/` generated from `.claude/` by `gen_codex_skills`); the fixes were applied to the hand-maintained sources and the Codex tree regenerated.

### Fixed

- **Runtime-agnostic agent identity** (`straymark-new`, `-adr`, `-aidec`, `-ailog`, `-mcard`, `-sec`): the skills no longer hardcode a platform identity in the `agent:` frontmatter they write. Codex skills, generated verbatim from the Claude source, previously instructed Codex to write `agent: claude-code-v1.0` — distorting provenance and agent/model telemetry. Skills now instruct the runtime to resolve its own canonical identity from `AGENT-RULES.md §1` (`claude-code-v1.0`, `gemini-cli-v1.0`, `codex-cli-v1.0`, `cursor-v1.0`, …). (#232 Finding 1)
- **Same-day ID discovery no longer misses nested documents** (`straymark-new`): the sequence-counting glob `ls .straymark/*/[TYPE]-…` matched only one directory level and returned 0 for types nested two levels deep (AILOG/AIDEC/ETH under `.straymark/07-ai-audit/…`), risking duplicate IDs. Replaced with a recursive `find`, plus a note to take the highest existing `NNN` (not the file count) when sequence gaps exist. (#232 Finding 2)
- **Context gathering describes current work, not the previous commit** (`straymark-new`, `-adr`, `-aidec`, `-ailog`, `-sec`): `git diff --stat HEAD~1 2>/dev/null || git diff --stat` never reached its fallback (the first command almost always succeeds) and summarized the prior commit. Replaced with explicitly labeled staged (`--cached`), unstaged, and untracked (`git status --porcelain`) blocks. (#232 Finding 3)
- **`straymark-status` reports uncommitted StrayMark documents** (`straymark-status`): the recent-document lookup used only `git log --since="1 hour ago"`, missing newly created docs in the working tree — exactly the pre-commit case the skill targets. It now combines git history with staged/unstaged/untracked `.straymark/**/*.md`, and the modified-source-files step gets the same current-work fix as Finding 3. (#232 Finding 4)
- **Audit-review degrades cleanly without a subagent primitive** (`straymark-audit-review`): "Launch Explore agents in parallel" assumed a runtime-specific capability. Reworded as capability-dependent — use parallel read-only subagents when the runtime provides them, otherwise verify findings directly in bounded groups by file. (#232 Finding 5)

### Adopter guidance

Run `straymark update-framework` to fw-4.24.0, then re-install the skills for your runtime (e.g. `straymark install-skills --agent codex`). No document migration is needed; the fixes only change how new documents are created and how status is reported.

---

## Framework 4.23.1 — migration sweep warning from the reference adopter's update run

Docs-only patch driven by the Sentinel update report ([#225](https://github.com/StrangeDaysTech/straymark/issues/225), cli-3.16.0 → 3.19.x / fw-4.20.0 → 4.22.0): the deprecated v0 bash script produced **silent false-negatives on drift detection itself** — its format-sensitive extractor (required both a `## Risk` heading and the exact `- **R<N> (new` bullet shape) never registered 8 AILOGs whose risks were written as bare paragraphs, reporting "in sync" while 29 entries sat unextracted. The native lenient parser caught them all on the first post-migration sweep.

### Changed (Framework)

- **`FOLLOW-UPS-BACKLOG-PATTERN.md` migration paragraph** (EN/ES/zh-CN): the migration command is now `drift --scan-all --apply`, with an explicit warning to run that first sweep with `--scan-all` *even if the legacy script reported "in sync"* — including the #225 data point (8 AILOGs / 29 entries silently missed).
- **`CLI-REFERENCE.md` lenient-parsing note** (EN/ES/zh-CN): the v0 → v1 upgrade fires on *any* write command — `drift --apply` (even with nothing to extract, cli-3.20.0+), `recount`, or `promote`. #225 Finding 2 (upgrade not firing on a no-op `--apply`) was already resolved by cli-3.20.0; this aligns the prose.

### Adopter guidance

Nothing to re-run if you already migrated with `--scan-all` (Sentinel's case). If you migrated with a plain `drift --apply` and the legacy script was your only drift check, run `straymark followups drift --scan-all --apply` once — the v0 script may have been blind to format variants your AILOGs use.

---

## Framework 4.23.0 / CLI 3.20.0 — N=2 feedback: `followups recount` + born-resolved closure idioms

First fixes driven by **external adopter feedback**: the lnxdrive adoption run ([#222](https://github.com/StrangeDaysTech/straymark/issues/222), the first N=2 data point gating the v1 schema's hard stabilization per principle #12 / ADR-2026-06-03-001) surfaced two gaps in the follow-ups lane — both ergonomic/vocabulary, neither schema-level.

### Added (CLI)

- **`straymark followups recount`** (#222 Finding 1) — recompute the CLI-owned `total_*` counters from actual entry statuses and rewrite the frontmatter, without scanning AILOGs or touching entries. Closes the loop between the sanctioned manual-triage workflow (Triage/Consumption = manual status edits) and the CLI-owned counter invariant: until now the only recomputing commands (`drift --apply`, `promote`) were no-ops after a pure-triage session, stranding the file with knowingly stale counters and no §13-compliant fix. Idempotent; upgrades v0 → v1 in place like every other write command.
- **Born-resolved closure idioms** (#222 Finding 2) — the anti-noise vocabulary now recognizes a closure verb (`updated` / `corrected` / `remediated` / `resolved` / `fixed` / `closed`) followed by `in this PR` / `in this commit` (e.g. the lnxdrive phrasing `Charter row updated atomically in this PR`), extracting those bullets as `suspected-closed` instead of reintroducing the TBD noise that #214 Signal 1 removed.

### Changed (CLI)

- **`followups drift --apply` recomputes counters even with zero extractions** (#222 Finding 1) — a pre-commit `drift --apply` now also reconciles counters left stale by a manual-triage session, making the `status` warning's remediation claim actually true.
- `followups status` stale-counter warning now points at `straymark followups recount`.

### Added (Framework)

- **Canonical closure-marker idioms** documented in `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN/ES/zh-CN) — the fixed vocabulary the anti-noise refinement recognizes, so AILOG authors converge on recognizable phrasings at write time instead of discovering unrecognized idioms at extraction.

### Changed (Framework)

- **`AGENT-RULES.md §13` post-Charter-close directive** (EN/ES/zh-CN) gains the recount step: after flipping statuses by hand, run `straymark followups recount` so the CLI-owned counters ride the same commit as the triage. The `/straymark-followups` skill (4 surfaces), `STRAYMARK.md §16` lifecycle table, and `QUICK-REFERENCE.md` command line follow suit.

### Adopter guidance

Run `straymark update` (CLI → `cli-3.20.0`, framework → `fw-4.23.0`). If your registry carries stale counters from a manual triage (the lnxdrive case): `straymark followups recount` fixes it in one command.

---

## Framework 4.22.0 — `/straymark-followups` skill ships the §13 directives as an invocable wrapper

Closes the last gap in the follow-ups first-class lane (fw-4.21.0 / cli-3.19.0): the `AGENT-RULES.md §13` directives — session-start "what's pending?" answered from the canonical registry, pre-commit `followups drift --apply` riding the same commit as the AILOG, post-Charter-close triage and operator-gated `promote` — now ship as an invocable skill across all four agent surfaces, mirroring the `straymark-charter-new` lane. Driven by [#220](https://github.com/StrangeDaysTech/straymark/issues/220) (deferred from the cli-3.19.0 PR because skills ride framework releases and fw-4.21.0 was already tagged). The skill is a **thin wrapper**: parsing, schema validation, counter recomputation, and the FU → TDE elevation all stay in the CLI (`straymark followups list/status/drift/promote`, cli-3.19.0+); the skill only drives the discipline.

### Added (Framework)

- **`/straymark-followups` skill** in all four agent surfaces — `.claude/skills/` (source of truth, full frontmatter), `.gemini/skills/`, `.agent/workflows/`, and the generated `.codex/skills/` (via `gen_codex_skills`). Wraps the three `AGENT-RULES.md §13` sub-flows; `allowed-tools` deliberately omits `Write` — every registry mutation goes through the CLI, and the frontmatter counters stay CLI-owned.
- **Skills table rows** for `/straymark-followups` in `QUICK-REFERENCE.md` (EN/ES/zh-CN) and in the `CLI-REFERENCE.md` Skills section (EN/ES/zh-CN). The CLI-REFERENCE table also gains the previously missing `/straymark-charter-new` row (shipped in fw-4.12.0, never listed there).

### Adopter guidance

Run `straymark update-framework` (→ `fw-4.22.0`). Claude and Gemini pick the skill up directly from the project tree; for Codex, re-run `straymark install-skills --agent codex` once to refresh the user-level skills directory. `/straymark-followups` is then invocable in any adopter repo that maintains the registry.

---

## CLI 3.19.1 — registry status annotations parse leniently

Patch found by validating cli-3.19.0 against the Sentinel production registry (65 entries): operators annotate status values in place — `- **Status**: open — **OVERDUE** (…)`, `open — mitigation in place (…)` — and the exact-match parser demoted those to `unknown`, which would have **undercounted the CLI-owned `total_open`** (observed live: 58 vs 62) on the first v0 → v1 migration write.

### Fixed (CLI)

- **`FuStatus::from_str_loose` / `Severity::from_str_loose`**: when the full value doesn't match the vocabulary, retry on the first whitespace-delimited token — so the in-place annotation idiom parses as its real status. Genuinely unknown values (e.g. `reopened`) still map to `unknown` (no over-match). Verified against the Sentinel registry: 65/65 entries now parse to their intended status (62 open / 3 promoted, zero unknown), matching the operator-declared counters exactly.

---

## CLI 3.19.0 — `straymark followups` namespace (companion to fw-4.21.0)

Ships the native CLI surface for the follow-ups backlog registry crystallized in fw-4.21.0 ([`ADR-2026-06-03-001`](docs/decisions/ADR-2026-06-03-followups-first-class.md), driven by [#214](https://github.com/StrangeDaysTech/straymark/issues/214)). The registry stops being invisible to tooling: it gains a CLI namespace, a synthetic group in `explore`, and a block in `status`. Collapses Tiers 2 and 4 of [#135](https://github.com/StrangeDaysTech/straymark/issues/135) into one native implementation; Tier 3 (`charter close` soft-integration) remains gated.

### Added (CLI)

- **`straymark followups list [--bucket] [--status] [--severity] [--label]`** *(new subcommand group)* — filterable table of registry entries (FU id, status, severity, bucket, destination, description). Malformed `### FU-` headings warn without failing.
- **`straymark followups status [FU-NNN]`** — registry pulse with counters **recomputed on the fly** from actual entry statuses (trustworthy even when the file frontmatter is stale — divergence is flagged), per-bucket breakdown, blocking/suspected-closed alerts, and advisory schema validation against `follow-ups-backlog.schema.v1.json`. With an id: entry field detail.
- **`straymark followups drift [--apply] [--scan-all] [--range]`** — native replacement for the deprecated adopter-side `check-followups-drift.sh` (~296 lines of bash retired). Per-AILOG granularity via `fully_extracted_ailogs` (0 false positives across 76 AILOGs in the reference adopter). `--apply` extracts into `## Bucket: ready`, registers the AILOG, **recomputes the CLI-owned counters** (#214 Signal 2) and upgrades v0 registries to v1 in place — non-destructively (unknown frontmatter fields survive; writes are surgical text edits, never a re-serialization). **Anti-noise refinement** (#214 Signal 1): bullets carrying a closure marker (`closed in-Charter`, `fixed in batch N`, a backtick-wrapped commit hash) land as **`suspected-closed`** instead of `ready`/TBD noise — across both documented occurrences that noise was 20–75% per batch. Seeds the registry from the framework template on first `--apply`.
- **`straymark followups promote FU-NNN [--title]`** — automates the FU → TDE elevation: creates the TDE from the framework template with `promoted_from_followup: FU-NNN`, flips the entry to `promoted` with `Destination`/`Promoted to` → TDE id, recomputes counters. Non-interactive by design (agent-friendly); prioritization stays human per `AGENT-RULES.md §3`.
- **`explore` TUI: synthetic "Follow-ups" group** — the registry file plus one sub-node per non-empty bucket, one entry per FU (badge `FU`; labels surface as tags; `FU-NNN` ids resolve as references). Appears only when the registry exists, mirroring `_charters`.
- **`status`: Follow-ups block** — status breakdown (open / in-progress / suspected-closed / closed+superseded / promoted) recomputed from entry statuses, with a blocking-severity alert; one-line adoption hint when no registry exists.
- **`cli/src/followups.rs`** — lenient registry parser (pure functions, no CLI deps; doc-tagged as the straymark-core move target for Loom M0) + **42 new tests** (25 unit, 17 integration) covering v0 lenient parsing, the v1 dimensions, closure-marker detection, counter recompute, idempotent upgrade, and the promote round-trip.

### Changed (CLI)

- `split_frontmatter` moved from `charter.rs` to `utils.rs` — one shared definition for the Charter and registry parsers.

### Adopter guidance

Run `straymark update` (CLI → `cli-3.19.0`, framework → `fw-4.21.0`). If you maintain a v0 registry: `straymark followups drift --apply` migrates it in one command; then delete the local bash script and point any pre-commit hook at the CLI. The `/straymark-followups` skill ships in a follow-up framework release.

---

## Framework 4.21.0 — Follow-ups backlog becomes a first-class entity (schema v1)

Promotes the follow-ups backlog from documented convention (v0, adopter-side bash) to **first-class framework entity**, following the lane Charter used: canonical schema, shipped agent directives, onboarding-level visibility. Driven by [#214](https://github.com/StrangeDaysTech/straymark/issues/214) (Sentinel post-stage triage at N=91 FUs — extractor noise ×2, silent counter drift, ad-hoc severity) and recorded in [`ADR-2026-06-03-001`](docs/decisions/ADR-2026-06-03-followups-first-class.md), which documents the design-principle #12 reframe: the structural evidence (91 FUs, schema already iterated under empirical pressure, 0 extraction false positives across 76 AILOGs, stable bucket vocabulary, internal Loom roadmap demand) justifies crystallizing as **v1 experimental**; hard stabilization stays gated on a second adopter. The native CLI surface (`straymark followups list/status/drift/promote`) ships in the companion release cli-3.19.0.

### Added (Framework)

- **`follow-ups-backlog.schema.v1.json`** (`.straymark/schemas/`) — frontmatter schema for the registry, experimental v1. Canonicalizes four optional entry dimensions surfaced empirically: **`Severity`** (`normal | blocking` — Sentinel's ad-hoc `PROD-BLOCKER`, #214 Signal 3), **`Origin-class`** (`ex-ante-planning | testing | telemetry | staging | real-env-bug` — making the registry queryable as the ex-post counterpart of SpecKit for Charter planning), **`Labels`** (free tags for grouping entries into planned Charters/mini-charters), and a formal **`Destination`** vocabulary (`chore | mini-charter | charter-replanning | operations | <charter-id> | <TDE id>`). New entry status **`suspected-closed`** for auto-extracted entries whose source AILOG carries an in-Charter closure marker.
- **Registry template** at `.straymark/templates/follow-ups-backlog.md` — empty frontmatter v1 + the five `## Bucket:` headers; adoption no longer starts from a blank file.
- **`STRAYMARK.md` §16 — Follow-ups backlog** — onboarding-level section mirroring §15 (Charters): what the registry is, the lifecycle (extraction → triage → consumption → promotion), how it relates to AILOGs/Charters/TDEs, and the CLI quick surface.
- **`AGENT-RULES.md` §13 — Follow-ups Backlog (registry maintenance)** — the agent directives now **ship with the framework** instead of being a suggested copy-paste block: session-start glance (the registry is the canonical answer to "what's pending?"), pre-commit `followups drift --apply` in the same commit as the AILOG, post-Charter-close review/confirmation/promotion. Root-cause fix for agents bypassing the registry and re-scanning AILOGs.

### Changed (Framework)

- **`FOLLOW-UPS-BACKLOG-PATTERN.md` graduates v0 → v1** (EN + ES + zh-CN): maturation chronology table; **frontmatter counters become CLI-owned** (recomputed on every write — closes the silent counter-drift failure mode, #214 Signal 2: declared `total_open: 47` vs 65 real); entry schema gains the four v1 dimensions; drift detection section rewritten for the native CLI including the **anti-noise refinement** (closure markers → `suspected-closed`, #214 Signal 1: 20–75% noise per auto-append batch across both documented occurrences); v0 bash script deprecated with a one-command migration path; "The registry as planning input" section documents the ex-post planning loop; resolved open questions struck through.
- **`QUICK-REFERENCE.md`**: new "First-Class Registries — Follow-ups Backlog" subsection beside the Charter one, a "When to Document" row for the pre-commit drift trigger, and the registry in the folder tree.
- **`DOCUMENTATION-POLICY.md` §6**: the registry appears in the folder structure (explicitly *not* a doc type).

### Migration (adopters on the v0 convention)

No action needed until cli-3.19.0: v0 registries remain readable forever (lenient parsing). On the first `straymark followups drift --apply`, the registry upgrades to v1 in place — non-destructively and idempotently (all v1 fields optional; only the version marker and counters are rewritten). Then delete the local `check-followups-drift.sh` and point any pre-commit hook at the CLI.

### Not in this release

The CLI surface itself (`followups list/status/drift/promote`, `explore`/`status` integration) ships in cli-3.19.0. Tier 3 of [#135](https://github.com/StrangeDaysTech/straymark/issues/135) (soft integration with `charter close`) remains open and gated on a second-adopter friction signal.

---

## CLI 3.18.0 — `analyze declared-vs-wired` subcommand (LNXDrive #209, Release B)

Ships the mechanical check the N=2 crossing unlocked: a config-driven set-difference that catches the *surface-declaration-without-wiring* anti-pattern's sub-class 5 — a declared client-side IPC/RPC proxy method with no implemented server-interface counterpart (the LNXDrive D-Bus/GOA regression). Follows the framework crystallization shipped in fw-4.20.0.

### Added (CLI)

- **`straymark analyze declared-vs-wired [path]`** *(new subcommand)* — reports symbols **declared but not wired** (`D \ W`) and, with `--show-orphans`, **wired but not declared** (`W \ D`). Language/IPC-agnostic by construction: the operator supplies a *declared* and a *wired* side as `(glob, regex)` pairs (capture group 1 = symbol name), either inline (`--declared-glob`/`--wired-glob`/`--declared-pattern`/`--wired-pattern`) or via a named `--profile` committed to `.straymark/config.yml`. Output `text`/`json`/`markdown`. Exit `1` when a declared symbol has no wiring counterpart — usable as a CI gate.
- **`declared_vs_wired.profiles` config block** in `.straymark/config.yml` — named profiles so a stack's regexes are committed once.

### Changed (CLI)

- **`straymark analyze` becomes a subcommand group.** The bare `straymark analyze [path]` is unchanged (still runs complexity analysis); `declared-vs-wired` is the first sub-analysis. Backward-compatible — guarded by a regression test.

### Scope

v0 covers sub-class 5 only (IPC proxy vs interface), the mechanically-tractable cross-stack check. The AST-based variants of sub-classes 1–4 and the dynamic runtime checks remain project-local — see `POLISH-CHARTER-PATTERN.md` Open questions.

---

## Framework 4.20.0 / CLI 3.17.0 — "declared but not wired" reaches N=2 (LNXDrive findings #209/#210)

Crystallizes the *surface-declaration-without-wiring* pattern to **v1** on the strength of a second independent domain (LNXDrive, a Rust Linux daemon + GTK desktop) validating what Sentinel's Go backend first surfaced, and ships the cheap mechanical backstops the two findings asked for. Two axes are reported separately and deliberately: **2 independent domains / 3 occurrences** — the third being a qualitatively new sub-class, a cross-component regression of an already-shipped mitigation.

### Added (Framework)

- **`POLISH-CHARTER-PATTERN.md` graduates v0/N=1 → v1/N=2** (EN + ES + zh-CN). Adds **sub-class 5** — *client-side IPC/RPC proxy method declared vs server interface implemented*, with the named variant "shipped-mitigation regression via an un-updated downstream consumer" (the LNXDrive D-Bus/GOA case: a GTK client kept calling a daemon method the daemon had removed, hidden behind an undefined `#[cfg(feature)]` so it compiled out and evaded both CI and review). Resolves the `analyze declared-vs-wired` open question now that the N=2 automation gate is crossed.
- **Charter template reconnaissance + cross-component guidance** (`charter-template.md` EN + ES + zh-CN). A comment above `## Files to modify` instructs authors to READ each path before declaring it (or tag created files "New"), and to list **all** consumers when a Charter touches a cross-component API — so a producer-side change can't silently orphan a consumer (#209.c). New format convention added to the closing notes.
- **`ADOPTERS.md`** records the first N=2 crossing and how it seeded CLI automation.

### Added (CLI)

- **`CHARTER-FILES-EXIST` validate rule** *(`straymark validate --include-charters`)* — warns when a `## Files to modify` row names a path that does not exist on disk and is not tagged "New" (Change column starting with New/Nuevo/新建, or a `(new)` path tag). Catches Charters authored against assumed, un-read code (finding #210). **Warn-only** (never fails the exit code); pure-Rust so it works on Windows-native without bash. Deliberately separate from `straymark charter drift` (which compares declared vs git-modified files) — "Charter mis-declared" (authoring bug) and "implementation drifted" stay in different commands with different rule codes (#210.3).
- **`charter new` reconnaissance nudge** — the printed "Next steps" now lead with a reconnaissance step: read every file before listing it in `## Files to modify`. The `straymark-charter-new` skill (Claude/Gemini/Codex variants) gains the matching instruction.
- **`charter_files` module** — shared Rust parser for the `## Files to modify` table (col-1 backtick paths, EN/ES/zh-CN headings, `(new)` exemption, wildcard pass-through), ported from the drift script's awk so the validate rule and the drift check agree on what counts as a declared file.

### Adopter guidance

Run `straymark update` (CLI → `cli-3.17.0`, framework → `fw-4.20.0`). To exercise the new check on existing Charters:

```bash
straymark validate --include-charters
```

The `analyze declared-vs-wired` subcommand (the IPC proxy-vs-interface check) ships in a follow-up CLI release; this release lands the pattern crystallization, the path-existence backstop, and the authoring discipline.

---

## Framework 4.19.0 / CLI 3.16.0 — Codex CLI (OpenAI) skill support

Adds first-class distribution of StrayMark skills for the **Codex CLI** (OpenAI), motivated by an adopter using Codex for external Charter audits in the Sentinel project. Codex's skill loader rejects the Claude-only `allowed-tools` frontmatter key and discovers skills only at the **user level** (`~/.codex/skills/`), not from the project tree like Claude and Gemini do. The release ships a fourth parallel skill variant generated from the Claude source, plus a new CLI command to install them.

### Added (Framework)

- **`dist/.codex/skills/` tree (11 skills)** — fourth parallel skill distribution alongside `.claude/skills/`, `.gemini/skills/`, and `.agent/workflows/`. Each `SKILL.md` keeps only the Codex-compatible frontmatter (`name`, `description`); the body is byte-identical to the Claude variant. Generated by `cargo run --bin gen_codex_skills` from the Claude source — one source of truth.
- **`.codex/skills/` entry in `dist/dist-manifest.yml`** — `straymark init` and `straymark update` now materialize the tree in the adopter's project alongside the existing skill directories.

### Added (CLI)

- **`straymark install-skills --agent <codex|claude|gemini> [--path .] [--dry-run] [--symlink]`** *(new subcommand)* — installs StrayMark skills into an AI agent's user-level skills directory. Currently only `--agent codex` performs work: copies (or symlinks with `--symlink`) every `straymark-*` skill from `<path>/.codex/skills/` into `$CODEX_HOME/skills/` (or `$HOME/.codex/skills/`). `--dry-run` previews without writing. `--agent claude|gemini` exits with an explanatory error because those agents read skills from the project tree directly. Re-installs replace any existing `straymark-*` directories at the target; non-`straymark-*` skills (e.g. Codex's `.system/` bundle) are left untouched.
- **`straymark validate --agent codex`** — agent-targeted validation path. Inspects `~/.codex/skills/straymark-*` for: presence of `SKILL.md`, parseable YAML frontmatter, required `name`/`description`, and absence of Claude-only keys (`allowed-tools`, `argument-hint`, `model`) whose presence signals a misinstallation (someone copied from `.claude/skills/`).
- **`cli/src/bin/gen_codex_skills`** — generator binary that transforms `dist/.claude/skills/*/SKILL.md` → `dist/.codex/skills/*/SKILL.md` with minimal frontmatter. Idempotent; supports `--check` for CI to detect drift between the two trees.

### Adopter guidance

Adopters who use Codex CLI should run `straymark update` (or update CLI to `cli-3.16.0` + framework to `fw-4.19.0`), then once per machine:

```bash
straymark install-skills --agent codex
straymark validate --agent codex      # confirms the install
```

Re-run `install-skills` after every `straymark update` to refresh skill content (or use `--symlink` once to track project changes automatically; Unix-only). Adopters who do **not** use Codex are unaffected — the `.codex/skills/` tree adds files but no CLI command runs against it without `--agent codex`.

---

## Framework 4.18.0 — Polish Charter as debt-detection pattern + "surface declaration without wiring" anti-pattern

Names a recurring anti-pattern surfaced empirically across a polish Charter session in the Sentinel adopter (`StrangeDaysTech/sentinel` CHARTER-19 → CHARTER-27, May 2026): **"Surface declaration without wiring"** — an artifact (env var documented in a runbook, metric instrument declared in a metrics package, URL referenced from an embedded HTML template, route marked public-by-contract) gets declared in one place while the implementation wiring lives in another place, with neither tooling nor review process correlating the two. Integration tests with mock adapters (`humatest`, in-memory event buses) systematically bypass the composed-app boot path where the gap would surface. The polish Charter — the closing Charter of an Etapa / SpecKit `Polish` Phase — is the load-bearing discovery vehicle because it exercises the documented operator runbook end-to-end against the real binary. New pattern doc + small charter-template addition formalize the convention so adopters treat the polish Charter as a debt-detection gate, not as cosmetic cleanup. Originated in [issue #199](https://github.com/StrangeDaysTech/straymark/issues/199). No CLI bump.

### Added (Framework)

- **`dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md`** (EN, ES, zh-CN) — new canonical pattern doc naming the anti-pattern and the four generalized sub-classes (env var ↔ consumer; metric instrument ↔ record-call site; HTML body URL ↔ registered route; public-by-contract route ↔ public-prefix entry). v0 status (proven in N=1 domain — Sentinel). Sections: Status, When this pattern applies, Shape (named anti-pattern + four sub-classes + why integration tests miss them), Adoption walkthrough, Reference implementation (Sentinel CHARTER-19 → CHARTER-27 chain + AIDEC-2026-05-22-001), Open questions, Credits, Related. Structure mirrors `FOLLOW-UPS-BACKLOG-PATTERN.md` for cross-doc consistency. Same N=1 → N=2 graduation gate for CLI crystallization (`straymark analyze declared-vs-wired` is listed in Open questions but not implemented).
- **`dist/.straymark/templates/charter/charter-template.md` — seventh `Format conventions` bullet** — surgical addition to the comment block pointing Charter authors at `POLISH-CHARTER-PATTERN.md` when the Charter closes an Etapa or SpecKit `Polish` Phase. Calls out the L effort budget and the emergent follow-on Charter expectation surfaced by the reference implementation. No new frontmatter field; no schema change.
- **`dist/.straymark/00-governance/QUICK-REFERENCE.md` `## Patterns` table** (EN, ES, zh-CN) — new row referencing the pattern doc with the `*(fw-4.18.0+)*` badge.

### Adopter guidance

Adopters running `straymark update-framework` after `fw-4.18.0` lands get the new pattern doc, the updated charter template, and the QUICK-REFERENCE row. No structural migration is required. Adopters closing an Etapa with ≥3 mock-adapter integration tests, ≥1 cross-module declaration/wiring artifact, or a runbook that has never been exercised end-to-end against the binary should read the pattern doc before scoping the polish Charter (budget L, not XS/S/M) and plan for emergent follow-on Charters.

---

## CLI 3.15.0 — Idempotent injection + host marker-health validation

Fixes a long-standing bug in `cli/src/inject.rs::replace_between_markers` that left a duplicated `<!-- straymark:end -->` in host files such as `.cursorrules`, `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`, `.cursor/rules/straymark.md`, and `.github/copilot-instructions.md` after every `straymark update-framework`. The root cause was two-fold: (1) the old code called `content.find(MARKER_BEGIN)` and `content.find(MARKER_END)` independently and never verified `start < end`; (2) more critically, `.cursorrules` and `.cursor/rules/straymark.md` embed `STRAYMARK.md`, whose own documentation contains the literal strings `<!-- straymark:begin -->` and `<!-- straymark:end -->` inside a fenced code block that describes the marker convention. `find(MARKER_END)` stopped at the in-docs literal instead of the trailing real END, so every update truncated the canonical block mid-embed and dropped the real END outside it as an orphan — accumulating one extra `<!-- straymark:end -->` per update. The fix makes injection idempotent and auto-repairing, and adds a diagnostic signal to `straymark validate`.

### Fixed (CLI)

- **`inject::find_canonical_block` uses first-BEGIN/last-END semantics** (`cli/src/inject.rs`) — switched from `find(MARKER_END)` to `rfind(MARKER_END)`, so the canonical span runs from the first BEGIN in the file to the *last* END. This correctly skips marker literals embedded in the payload (the actual STRAYMARK.md describes the markers in its own docs) and survives the pathological "two complete blocks" corruption case by engulfing both pairs into a single replacement region. Trade-off documented in the function's docstring.
- **`inject::replace_between_markers` / `inject::remove_between_markers`** (`cli/src/inject.rs`) — both route through a new `sanitize_orphan_markers` helper that preserves the canonical block intact and strips orphans (lone BEGIN/END markers outside the canonical span, or any markers when there is no canonical block). Lines containing only a marker are removed entirely; markers embedded inside lines with other content lose only the marker substring; consecutive blank lines created by the cleanup collapse to a single blank line. `inject_directive` now treats any marker (BEGIN or END) as a signal to take the replace-and-repair path, so a file containing only an orphan `<!-- straymark:end -->` is still repaired on the next update.
- **Sentinel reproduction covered by tests** — six new tests in `cli/src/inject.rs::tests` exercise orphan-end-before-begin, two complete blocks, orphan-begin-no-end, idempotency across two consecutive passes, end-to-end `.cursorrules` repair via the public `inject_directive` API, and the orphan-end-only-file case. The existing `test_replace_between_markers` was hardened to assert the output contains exactly one BEGIN and one END.

### Added (CLI)

- **`inject::MarkerHealth` + `inject::inspect_marker_health`** (`cli/src/inject.rs`) — diagnostic API that reports a host file's marker structure without mutating it: `begin_count`, `end_count`, `has_canonical_block`, and `end_before_begin`. `is_malformed()` returns true on asymmetric counts (the Sentinel case shows up as `end_count` one greater than `begin_count`), markers-without-a-canonical-block, or end-before-begin inversions. Five new tests cover healthy / no-markers / healthy-with-marker-literals-in-embed (the `.cursorrules` reality) / end-before-begin / extra-orphan-end (Sentinel) / orphan-begin cases.
- **`straymark validate` host marker-health warning** (`cli/src/commands/validate.rs`) — new `check_host_marker_health` walks `manifest.injections` from the local `.straymark/dist-manifest.yml`, calls `inject::inspect_marker_health` on every declared target that exists, and emits a `ValidationWarning` (rule `host-marker-health`) when the structure is malformed. Message names the specific issue (asymmetric counts, no-canonical-block, end-before-begin) and points the operator at `straymark update-framework` or `straymark repair` to auto-repair. Merges into the existing `ValidationResult` flow so the `--fix` path and re-validation also pick it up. If the manifest cannot be loaded, the check is silently skipped.
- **`doc_count == 0` no longer suppresses host-marker warnings** — when a project has no `.straymark/` documents but a host file has malformed markers, the warning still surfaces. The "no documents found" hint only fires when the validation result is empty.

### Adopter guidance

Run `straymark update-cli` to install `cli-3.15.0`. The next `straymark update-framework` (whenever a new framework release lands) auto-repairs any `.cursorrules` or other host file that accumulated a duplicate end marker — no manual editing required. Adopters who want a heads-up before re-running the update can call `straymark validate` now; malformed host files appear in the warning summary under the `host-marker-health` rule.

The Sentinel `.cursorrules` duplication observed after a recent `straymark update-framework` resolves automatically on the next framework update once `cli-3.15.0` is in place. No framework bump is required — `fw-4.17.0` from the previous release cycle remains current.

---

## Framework 4.17.0 — Emergent-observation design meta-pattern

Names a design property of the StrayMark framework that was already present but unnamed: *formal cross-referencing (frontmatter linkage, canonical sections, stable IDs) composed with cultural permission to surface beyond the asked task* produces agents that, while reading only the framework documentation, surface dissonance between canonical sources. This composition produced the Sentinel observation that crystallized as `CHARTER-CHAIN-EVOLUTION.md` Pattern 1 in `fw-4.16.0`. The new doc and a new Principle codify the meta so it can be preserved deliberately, and lists four open application axes (MCARD ↔ deployed model, SBOM ↔ lockfiles, ADR ↔ contradicting implementation, Constitution Check ↔ framework bump) as candidates for future N=1-validated extension. No CLI bump.

### Added (Framework)

- **`dist/.straymark/00-governance/EMERGENT-OBSERVATION-DESIGN.md`** (EN, ES, zh-CN) — new canonical doc naming the meta-pattern. Sections: Status (v0, N=1), Why this document exists, The two design properties (formal linkage + cultural permission), Empirical case (Sentinel #150 → #156), Pyramid of instances (Pattern 1, Pattern 2, charter drift, follow-ups backlog drift, TDE-vs-`R<N>` escalation, external audit checkpoint), Anti-patterns, Open application axes, Authority/acceptance flow, Open questions, Related. Structure mirrors `CHARTER-CHAIN-EVOLUTION.md` for cross-doc consistency.
- **`PRINCIPLES.md` §8 — Cross-Source Dissonance Surfacing** (EN, ES, zh-CN) — new principle condensing the cultural rule into PRINCIPLES.md alongside the existing seven. Points to the meta doc for the pyramid of existing applications.

### Changed (Framework)

- **`AGENT-RULES.md §6 "Be Proactive"** (EN, ES, zh-CN) — expanded with a new sub-bullet *"Surface dissonance between canonical sources"* citing Principle #8 and listing concrete examples (stale spec, accumulated `R<N>` matching TDE criteria, ADR contradicted by code, follow-ups crossing threshold, post-close audit findings). Existing three bullets unchanged.
- **Cross-links added** in `CHARTER-CHAIN-EVOLUTION.md` (`## Related` — EN/ES/zh-CN), `SPECKIT-CHARTER-BRIDGE.md` (`## See also` / `## Ver también` / `## 另请参阅` — EN/ES/zh-CN), `FOLLOW-UPS-BACKLOG-PATTERN.md` (new `## Related` / `## Relacionado` / `## 相关` section before footer — EN/ES/zh-CN), and `STRAYMARK.md §11` (new row in "When to Load Additional Documents" table — EN only; STRAYMARK.md has no i18n versions).
- **Governance footers** bumped to `fw-4.17.0` across `PRINCIPLES.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md`, `QUICK-REFERENCE.md`, `CHARTER-CHAIN-EVOLUTION.md`, `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN + ES + zh-CN) and the root `dist/.straymark/QUICK-REFERENCE.md`. PRINCIPLES.md migrated from the legacy `StrayMark v1.0.0` footer convention to the `fw-X.Y.Z` framework versioning convention used by sibling governance docs.
- **`STRAYMARK.md §11 "When to Load Additional Documents"** — new row: *"Wondering why agents surface things you didn't ask"* → `EMERGENT-OBSERVATION-DESIGN.md`. Lets adopters discover the meta-pattern from the canonical entry-point doc.

### Adopter guidance

`straymark update-framework` brings the new doc, the new Principle, and the §6 expansion. No CLI install required — `cli-3.14.1` from the `fw-4.16.2` cycle remains current. The four "Open application axes" listed in the new doc are tracked in a follow-up upstream RFC issue filed after this release lands.

---

## Framework 4.16.2 / CLI 3.14.1 — `straymark repair` per-target restore (Issue #156 follow-up)

Fixes the underlying CLI bug surfaced in the [Issue #156 closure comment](https://github.com/StrangeDaysTech/straymark/issues/156#issuecomment-4465050814): `straymark repair` ignored a single missing injection target (e.g. a deleted `AGENTS.md`) when `STRAYMARK.md` was still present, because both the download trigger and the inject path were gated on `STRAYMARK.md` being absent. From this release, both gates are removed and `repair` mirrors `straymark update-framework`'s per-target behavior for the set of files declared under `dist-manifest.yml::injections:`.

### Fixed (CLI)

- **`straymark repair` per-target restore** (`cli/src/commands/repair.rs`) — two gating bugs removed:
  - `check_needs_download()` now reads the local `dist-manifest.yml` and flags the download as needed if any declared injection target is missing. Before this change a deleted directive file (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.cursorrules`, `.cursor/rules/straymark.md`) was silently ignored because the function only looked at essential framework files.
  - `restore_missing_files()` replaces the `if !straymark_md.exists()` gate around the inject loop with a per-target filter (`missing_injections`). Each declared injection target is now checked individually; only the missing ones load their template from the ZIP and run through `inject::inject_directive`. Targets already present are left untouched — `repair` does not overwrite an installed file.
- **`cli/src/commands/repair.rs::tests`** — five new unit tests cover the helper-level behavior: `check_needs_download` returns true for a missing injection target / false when all are present / still true for missing essential files (regression for the prior behavior); `missing_injections` filter identifies per-target gaps and is empty when all targets are present. End-to-end coverage of the download + extract path remains out of scope for the unit suite (it requires a real release ZIP and network); the release workflow exercises that path on every tag.

### Changed (Framework)

- **`dist/STRAYMARK.md §Directive Injection Markers`** — the missing-target behavior bullet added in `fw-4.16.1` is rewritten to describe the post-`cli-3.14.1` behavior: `init`, `update-framework`, AND `repair` all walk `manifest.injections` and create any missing target file. A short historical note preserves the pre-fix behavior for adopters running older CLIs.
- **Governance footers** bumped to `v4.16.2` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md` (EN + ES + zh-CN) and the `CHARTER-CHAIN-EVOLUTION.md` footer (3 languages).
- **Version tables** in `README.md` + i18n and `CLI-REFERENCE.md` + i18n bumped to `fw-4.16.2` / `cli-3.14.1`. The EN `CLI-REFERENCE.md` CLI row had been left at `cli-3.13.2` since the `fw-4.15.0` cycle — corrected in this patch.

### Adopter guidance

`straymark update-cli` brings `cli-3.14.1` with the fix. `straymark update-framework` brings the docs sync. Adopters who had been working around the bug by manually re-creating a deleted `AGENTS.md` before running `repair` no longer need that step — `straymark repair` will restore any single missing injection target without requiring the rest of the install to be broken.

---

## Framework 4.16.1 — docs polish (Issue #156 follow-up)

Documentation-only patch addressing two non-blocking observations raised in the [Issue #156 closure comment](https://github.com/StrangeDaysTech/straymark/issues/156#issuecomment-4465050814) after the adopter pulled `fw-4.16.0` / `cli-3.14.0`. No CLI bump.

### Changed (Framework)

- **`dist/STRAYMARK.md §Directive Injection Markers`** — added an explicit clarification of how each command treats a missing injection target. `straymark init` and `straymark update-framework` walk `dist-manifest.yml::injections:` end-to-end and create the target file if it does not exist. `straymark repair` is narrower: it re-injects directives only when `STRAYMARK.md` itself is missing. The practical guidance for an adopter who deleted a single target while keeping `STRAYMARK.md` is to re-run `straymark update-framework`, not `straymark repair`. Resolves the AGENTS.md observation from the closure comment.
- **`docs/adopters/CLI-REFERENCE.md` (EN + ES + zh-CN)** — clarified that the `--threshold N` flag is the only override path for `straymark charter refresh-suggest` in v0.2; there is no `config.yml` field for the heuristic threshold yet (tuning lives at the operator-invocation level until a second adopter validates a project-wide default).
- **Versioning tables** in `README.md` + i18n and `CLI-REFERENCE.md` + i18n bumped to `fw-4.16.1`. EN CLI-REFERENCE versioning row had been left at `fw-4.15.0` through the `fw-4.16.0` cycle — fixed in this patch.
- **Governance footers** bumped to `v4.16.1` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md` (EN + ES + zh-CN).

### Adopter guidance

`straymark update-framework` brings the clarifications. No CLI install required — `cli-3.14.0` from the `fw-4.16.0` cycle remains current.

---

## Framework 4.16.0 / CLI 3.14.0 — Charter-chain evolution patterns (Issue #156)

Codifies two patterns surfaced by the Sentinel adopter after seven consecutive CommsHub Charters (Issue [#156](https://github.com/StrangeDaysTech/straymark/issues/156)): a **pre-declare SpecKit refresh** that absorbs accumulated chain-level learnings before the next Charter is declared, and a **post-close audit-driven Batch N.4 amendment** that handles bounded external-audit findings on the same execute branch without opening a new Charter. Both are operator-driven; the framework ships canonical guidance in `00-governance/CHARTER-CHAIN-EVOLUTION.md` (EN/ES/zh-CN), opt-in telemetry slots in the schema, and two new read-only/scaffolding CLI helpers. Also fixes the `straymark charter audit --merge-into` bug where the v0 re-audit guard rejected the empty-array placeholder `external_audit: []`, breaking the post-close audit-cycle round-trip.

### Added (Framework)

- **`dist/.straymark/00-governance/CHARTER-CHAIN-EVOLUTION.md`** (EN, ES, zh-CN) — new canonical doc covering both patterns (Pattern 1 = pre-declare SpecKit refresh; Pattern 2 = post-close audit-driven amendment), each with when-it-applies + mechanics + telemetry + empirical anchor from Sentinel CHARTER-18. Establishes `06-evolution/<name>-rfc.md` as the canonical adopter-local home for in-flight RFCs and `00-governance/<NAME>.md` as the upstream-accepted home.
- **`dist/.straymark/schemas/charter-telemetry.schema.v0.json`** — two new optional sub-objects under `charter_telemetry`:
  - `pre_declare_refresh` (enabled, refresh_pr, refresh_aidec, plus integer counts for reusable_patterns / code_gaps / discipline_patterns / empirical_corrections / operator_decisions).
  - `post_close_amendment` (applied, trigger ∈ {external_audit, production_incident, deferred_implementation}, ailog_id, findings_closed, files_modified, effort_hours).
  - Schema stays v0 — the N=1-domain caveat from the existing $comment carries forward; the addendum is documented inline.
- **`dist/.straymark/templates/charter/charter-telemetry-template.yaml`** — commented stubs for the two new blocks with guidance on when to populate them.
- **`dist/STRAYMARK.md` §15.A and §15.B** — two new subsections inside the Charter chapter describing each pattern at a glance and citing the helper commands. Quick CLI surface listing extended with `refresh-suggest` and `amend`.
- **`dist/.straymark/templates/charter/charter-template.md`** — Batch Ledger task note pointing readers to §15.B + `straymark charter amend` when audit findings arrive after `status: closed`.
- **`dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md`** — short cross-reference at the top of "Spec maintenance during multi-Charter execution" pointing to CHARTER-CHAIN-EVOLUTION.md Pattern 1 as the canonical extension.

### Added (CLI)

- **`straymark charter refresh-suggest <module> [--threshold N]`** *(new)* — read-only. Reads the last 3 closed Charters whose `charter_id` contains the module (case-insensitive substring), computes the rolling mean of `agent_quality.r_n_plus_one_emergent_count`, and prints a recommendation (refresh-now / not-needed / insufficient-data). Default threshold 6. Always exits 0 — informational, never a CI gate. Located at `cli/src/commands/charter/refresh_suggest.rs`.
- **`straymark charter amend <CHARTER-ID> --trigger <kind> --ailog-title <title> [--findings-closed N] [--merge-into <PATH>]`** *(new)* — scaffolds the three artifacts of a post-close Batch N.4 amendment: creates a new AILOG stub under `agent-logs/` with `risk_level: high` + `amends:` pointing back to the most-recent prior AILOG; appends a `## Historical correction (YYYY-MM-DD)` subsection to that prior AILOG; renders the `post_close_amendment:` YAML block (printed to stdout or merged via `--merge-into`). Does NOT touch git — the operator decides when to commit. Located at `cli/src/commands/charter/amend.rs`.
- **`cli/tests/charter_refresh_suggest_test.rs`** and **`cli/tests/charter_amend_test.rs`** — 9 new integration tests covering: trigger / no-trigger / insufficient-data / zero-match / threshold override (refresh-suggest); AILOG creation + historical correction append + merge-into round-trip + closed-status guard + invalid-trigger rejection (amend).

### Fixed (CLI)

- **`straymark charter audit --merge-reports --merge-into <PATH>` placeholder support** (`cli/src/commands/charter/audit.rs:370-432`) — the v0 anti-duplicate guard rejected any presence of `external_audit:`, including the empty placeholder `external_audit: []`. The new implementation parses the YAML semantically and distinguishes three cases: missing → append (original behavior), empty placeholder `[]` → **replace in place** (new — fixes Issue #156 sub-issue), populated array → still rejected with a clearer error message (anti-duplicate guard intact). `straymark charter close` now writes the `external_audit: []` placeholder by default so the post-close audit cycle round-trips cleanly without manual YAML editing. New regression test `audit_merge_into_replaces_empty_external_audit_placeholder` pins the fix.

### Changed (Framework)

- **Governance footers** bumped to `v4.16.0` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md` (EN + ES + zh-CN).
- **Version tables** in `README.md` (root + ES + zh-CN) and `CLI-REFERENCE.md` (EN + ES + zh-CN) bumped to `fw-4.16.0` / `cli-3.14.0`. CLI command listings extended with the two new subcommands.

### Adopter guidance

- `straymark update-framework` brings the new template fields, the canonical doc in three languages, and the schema additions. Both new schema sub-objects are **optional** — existing telemetry files validate unchanged.
- `straymark update-cli` brings `cli-3.14.0` with the bug fix for `--merge-into` and the two new subcommands. The `external_audit: []` placeholder now emitted by `charter close` is harmless for v0.1 adopters whose tooling treats the schema as v0 — the schema permits the empty array.
- After update, run `straymark charter refresh-suggest <module>` on any module with 3+ closed Charters to check whether the heuristic suggests a refresh; the command is read-only and side-effect-free.

---

## Framework 4.15.0 / CLI 3.13.2 — AGENTS.md universal injection

Adds `AGENTS.md` as a first-class injection target. `AGENTS.md` is the open standard for AI coding agents, donated to the Agentic AI Foundation (Linux Foundation, 2025) and read by Claude Code, OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue, Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q and others. Before this release, StrayMark injected directives only into platform-specific files (`CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.cursorrules`, `.cursor/rules/straymark.md`); adopters using any of the 15+ CLIs that read `AGENTS.md` instead had to copy directives by hand. From `fw-4.15.0`, `straymark init` and `straymark update-framework` keep `AGENTS.md` in sync with `STRAYMARK.md` automatically.

### Added (Framework)

- **`dist/dist-templates/directives/AGENTS.md`** — new template (reference shape, parallel to `CLAUDE.md` / `GEMINI.md`). Marker block points to `STRAYMARK.md`; below the markers, a "minimum viable" body declares identity, review requirements, pre-commit checklist, regulatory frontmatter snippet, NIST AI 600-1 risk categories, observability rules, and the naming convention — sufficient for any reader that cannot follow the relative link.
- **`dist/dist-manifest.yml` `injections:`** — new entry `target: AGENTS.md` (reference shape, no embed). Placed first in the list to mark it as the universal entry point.
- **`STRAYMARK.md` § Directive Injection Markers** — explicit mention of `AGENTS.md` with the standard's context (Agentic AI Foundation donation, reader list) and the coexistence rule (CLI-specific files coexist with `AGENTS.md` for platform-specific identity strings).
- **`cli/tests/inject_test.rs`** — three new integration tests:
  - `test_agents_md_template_has_markers` — verifies the shipped template has the right marker shape and pointer.
  - `test_manifest_declares_agents_md_injection` — verifies the manifest declares the injection so init/update/remove all pick it up.
  - `test_agents_md_coexists_with_preexisting_file` — pins the append-when-no-markers behavior against the very common case of an adopter who already has an `AGENTS.md`.

### Added (CLI)

- **`AGENTS.md` in `LEGACY_DIRECTIVE_TARGETS`** (`cli/src/commands/remove.rs:13`) — the data-driven `clean_directives` path (which reads `.straymark/dist-manifest.yml`) already cleans `AGENTS.md` correctly for `fw-4.15.0` installations. This change extends the legacy fallback that fires when the local manifest is missing or fails to parse, so `straymark remove` cleans `AGENTS.md` defensively in that edge case rather than leaving it orphaned. Follows the existing convention of keeping `LEGACY_DIRECTIVE_TARGETS` in sync with the manifest's `injections:` list.

### Changed (Framework)

- **Governance footers** bumped to `v4.15.0` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md`, `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN + ES + zh-CN, plus the top-level `dist/.straymark/QUICK-REFERENCE.md`).
- **Version tables** in `README.md` (root + i18n) and `CLI-REFERENCE.md` (EN + ES + zh-CN) bumped to `fw-4.15.0`. Canonical-output examples follow.
- **AI Agent Support sections** in `README.md`, `ADOPTION-GUIDE.md` and equivalent i18n files now list `AGENTS.md` as the universal entry point with the full reader-CLI inventory.
- **`init` / `update` descriptions** in `CLI-REFERENCE.md` and `ADOPTION-GUIDE.md` (EN + ES + zh-CN) list `AGENTS.md` first among directive files.

### Adopter guidance

`straymark update-framework` brings the new template and injects `AGENTS.md` at the project root. The injection follows the same rules as every other directive target: it creates the file if absent, replaces the marker block on subsequent runs, and appends safely when the file pre-exists without StrayMark markers (very common in 2026 — many adopters already hand-maintain an `AGENTS.md`). `straymark update-cli` brings the matching `cli-3.13.2` binary so the `remove` legacy fallback also covers `AGENTS.md`; updating only the framework is safe because the common cleanup path is manifest-driven and already includes the new target.

If your `.gitignore` excludes `AGENTS.md`, adjust it before `update-framework` so the injection lands in version control.

---

## Framework 4.14.3 — Spec-refresh discipline for multi-Charter execution (closes #150 Asks 1, 2, 4)

Framework-only patch that closes a governance gap reported by Sentinel after running a single `specs/002-commshub/plan.md` through **seven consecutive Charters** over ~1 month. The bridge doc (`SPECKIT-CHARTER-BRIDGE.md`) covered Charter *declaration* but said nothing about *spec maintenance during multi-Charter execution* — and naively re-running `/speckit-plan` regenerates assertions about already-shipped user stories that the actual code does not implement, propagating stale state into future audits.

Ships **Cubeta A** of the issue's two-bucket plan: pure governance documentation. **Cubeta B** (the `straymark spec-drift` CLI that mechanizes Gate (a), plus a cross-Charter `lessons-learned` index per [#146 Proposal D](https://github.com/StrangeDaysTech/straymark/issues/146)) is deferred to a dedicated post-announcement Charter — tracked separately so the context survives.

### Added (Framework)

- **`dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md`** (EN + i18n/es + i18n/zh-CN) — new section **"Spec maintenance during multi-Charter execution"** with the empirical anchor (Sentinel CHARTER-07..17 cycle, 12 unreflected learnings). Subsections:
  - **When to refresh** — four heuristics (≥3 closed Charters, ≥4 weeks + ≥2 Charters, `R<N>(new)` count >6, target US touches refined infra). Explicit guidance to *skip* the refresh when none hold.
  - **How to refresh: scope-limited prompt** — name the target phase, list locked sections, cite refinement AILOGs, forbid `tasks.md` regeneration.
  - **Three mechanical gates** — (a) Validation against code reality (diff non-target-phase entities/endpoints vs migrations/handler signatures), (b) Granular hunk-by-hunk review, (c) Two-PR split (refresh PR separate from Charter-fill PR).
  - **Why NOT re-run `/speckit-tasks`** — regenerating destroys `[X]` completion marks and `*CHARTER-NN:* <sha>*` annotations that form the historical trace. Manual edit for the target phase only.
  - **Constitution Check re-evaluation cadence** — codifies per-Charter (recommended) + per-spec-refresh (mandatory) + NOT per-framework-bump alone, closing the implicit-cadence ambiguity reported by Sentinel.
  - **Roadmap: `straymark spec-drift`** — names the deferred CLI explicitly so adopters reading the policy know mechanization of Gate (a) is coming post-announcement.
- **Anti-pattern entry** — new bullet against re-running `/speckit-tasks` mid-execution, pointing to the new section for the safe path.

### Changed (Repo-level docs, not shipped via `straymark init`)

- **`docs/contributors/WHAT-IS-A-CHARTER.md`** §4 Mode A — cross-link inserted right after the SpecKit-driven flow diagram, surfacing the "what happens when a single spec drives many Charters?" question with a pointer to the new bridge-doc section. Contributors landing on the conceptual doc now find the operational discipline at the natural decision point.
- **Governance footers** updated to `v4.14.3` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md`, `FOLLOW-UPS-BACKLOG-PATTERN.md`, `SPECKIT-CHARTER-BRIDGE.md` (EN + ES + zh-CN, plus the top-level `dist/.straymark/QUICK-REFERENCE.md`).
- **Version tables** in `README.md` (root + i18n) and `CLI-REFERENCE.md` (EN + ES + zh-CN) bumped to `fw-4.14.3`. Canonical-output examples (`Framework updated to fw-4.14.3`, etc.) follow.

### Deferred — Cubeta B (post-announcement Charter)

The empirical pattern around #150 reinforces the gap surfaced by [#146 Proposal D](https://github.com/StrangeDaysTech/straymark/issues/146) (cross-Charter `lessons-learned` index). Both #146 D and #150 Ask 3 are mechanics that live *above* the Charter as a unit — the missing "cross-Charter knowledge layer". A dedicated Charter post-announcement will design this layer cohesively: lessons-learned index + `straymark spec-drift` CLI + possible umbrella `straymark spec` command. A tracking issue is filed to preserve the context across the announcement cycle.

### Why a patch release for docs only

`dist/.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md` is shipped to every adopter via `straymark init`. Without a framework bump, `straymark update-framework` would not bring the new section to existing installations. Sentinel needs the canonical recommendation before filling CHARTER-18 — the ~50% probability of inheriting a critical/high finding from stale-premise inheritance is real and time-sensitive.

### Adopter guidance

`straymark update-framework` brings the updated bridge doc and footers. No behavior changes: the CLI is unchanged (`cli-3.13.1` remains the matching CLI version), no schemas or templates moved. Adopters running multi-Charter specs should read the new section *before* declaring their next Charter against an aging spec.

---

## Framework 4.14.2 / CLI 3.13.1 — TDE terminal `resolved` (closes #149)

Closes [#149](https://github.com/StrangeDaysTech/straymark/issues/149) — surfaced by Sentinel post-CHARTER-17 housekeeping. TDE adopters who keep documents on disk as audit history after the debt is paid had no canonical status to mark the closure; `accepted` / `superseded` / `deprecated` all carry the wrong semantics. The validator rejected `resolved` with `META-003`.

This release ships **Option A** of the issue's proposal triplet (flat enum + `resolved`) and documents **Option B** (per-doc-type lifecycle vocabulary) as the deliberate next evolution.

### Added (CLI)

- **`resolved`** is now a valid value in `VALID_STATUSES` (`cli/src/validation.rs:48`). Adopter-facing effect: `straymark validate` accepts `status: resolved` on TDE documents without `META-003`. Two new tests pin the behavior:
  - `test_validate_tde_resolved_terminal_state` — TDE with `status: resolved` passes.
  - `test_validate_rejects_non_canonical_ailog_terminals` — `final`, `closed`, `completed` (Sentinel's invented AILOG terminals reported in #149) continue to fail with `META-003`. Adopters using these on AILOGs should migrate to `accepted` per `TEMPLATE-AILOG.md` and `DOCUMENTATION-POLICY.md §6`.

### Changed (Framework)

- **`dist/.straymark/00-governance/DOCUMENTATION-POLICY.md` §3 (EN + i18n/es + i18n/zh-CN)** — lifecycle diagram extended with the `resolved` terminal branch (TDE-only); new `resolved` row in the status table with the principled distinction from `accepted` / `superseded` / `deprecated`; §6 table column updated to note that TDE enters at `identified` and has its own terminal `resolved`.
- **`dist/.straymark/templates/TEMPLATE-TDE.md` (EN + i18n/es + i18n/zh-CN)** — frontmatter `status: identified` annotated with the `→ resolved` transition; new optional `## Resolution` body section (omit while the debt is still open) with fields for Resolved by / Date / Verification / Notes.

### Deliberately deferred (Option B)

The principled fix to the per-doc-type lifecycle vocabulary problem is to promote the flat `VALID_STATUSES` enum to a `HashMap<DocType, Vec<&str>>` so each doc type has its own canonical state machine and the validator inspects `doc.doc_type` before deciding which set to apply. The reasons to defer:

- Option A unblocks adopters today (Sentinel reports 2 TDE files with `status: resolved`); shipping it is ~30 LOC + docs.
- Option B is ~150 LOC + a per-type test matrix expansion + non-trivial decisions about which terminals each type should have. Doing it pre-announcement risks calcifying choices we'd rather make based on a second adopter's lifecycle needs.
- The `VALID_STATUSES` constant doc-comment in `validation.rs` and the explanatory paragraph in `DOCUMENTATION-POLICY.md §3` both name the trade-off explicitly: "TDE is the only type today with a custom terminal; the validator accepts `resolved` globally as a stop-gap. Issue #149 Option B will scope `resolved` strictly to TDE; until then, using it on non-TDE documents passes validation but is semantically incorrect."

### Adopter guidance

- `straymark update-framework` brings the new `TEMPLATE-TDE.md` and `DOCUMENTATION-POLICY.md`. Existing TDE files are unaffected — the template change is purely additive.
- `straymark update-cli` brings the relaxed validator. TDE files marked `status: resolved` start passing immediately.
- Sentinel-side AILOGs marked `final` / `closed` / `completed` continue to fail validation. The fix is to migrate those to `accepted` (the canonical AILOG terminal per `TEMPLATE-AILOG.md`) and capture the "work completed" semantics in the body or in the originating Charter's telemetry, not in the AILOG status field.

---

## Framework 4.14.1 — Docs sync for batch-complete + Batch Ledger discoverability

Framework-only patch that closes the documentation gap left by `fw-4.14.0` / `cli-3.13.0`. The release added the `batch-complete` subcommand and the Batch Ledger workflow but didn't surface them in three places adopters and contributors discover the project from: the root `README.md` (and i18n overlays), the project's user-facing comparison/feature lists, and **`dist/STRAYMARK.md`** — the governance file shipped to every adopter via `straymark init`.

### Changed (Framework)

- **`dist/STRAYMARK.md` §15 (Charter lifecycle)** — new "Batch update" stage row inserted between "In progress" and "Drift check", documenting the per-batch ledger update pattern and pointing at `straymark charter batch-complete CHARTER-NN <N>`. The "Drift check" row now mentions that pending `### Batch N` entries cause a hard fail (with `--no-batch-ledger-check` bypass).
- **`dist/STRAYMARK.md` Quick CLI surface** — added the `batch-complete` line and annotated the `drift` line with "AILOG-aware + Batch Ledger gate".
- **Governance footers** updated to `v4.14.1` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md`, `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN + ES + zh-CN, plus the top-level `dist/.straymark/QUICK-REFERENCE.md`).

### Changed (Repo-level docs, not shipped via `straymark init`)

- **`README.md`** (root, EN + i18n/es + i18n/zh-CN) — "Features → CLI Tools" bullet expanded to include `batch-complete` and the Batch Ledger gate on `drift`; the table-of-commands row for `straymark charter <subcommand>` updated likewise.
- **`docs/adopters/CLI-REFERENCE.md`** (EN + ES + zh-CN) — version tables + canonical-output examples bumped to `fw-4.14.1`. The detailed `batch-complete` section and the extended `drift` description added in `fw-4.14.0`/`cli-3.13.0` remain unchanged; their `*(cli-3.13.0+ + fw-4.14.0+)*` minimum-version annotations are intentionally preserved (those mark when the feature was introduced, not the current release).

### Why a patch release for docs only

The `dist/STRAYMARK.md` change is governance content shipped to every adopter — without bumping the framework, `straymark update-framework` would not bring the new content to existing installations. The cost of a patch tag is one CI run; the value is that any new or existing adopter who runs `straymark init` or `straymark update-framework` sees the canonical Charter lifecycle table with the Batch Ledger pattern visible.

### Adopter guidance

`straymark update-framework` brings the updated `STRAYMARK.md` and governance footers. No behavior changes: the CLI is unchanged (`cli-3.13.0` remains the matching CLI version), and no schemas or templates moved.

---

## Framework 4.14.0 / CLI 3.13.0 — Sentinel feedback cycle: shellcheck cleanup, regex extension, multi-batch AILOG ledger

Closes three upstream issues filed by Sentinel during its CHARTER-17 cycle, all field-validated downstream and ready for upstream canonization. The unifying theme is **pre-announcement hardening**: each issue was discovered in real adopter usage, the fix is additive (zero ruptura para existing artifacts), and Sentinel can revert its local workaround once this release lands.

### Added (Framework)

- **`## Batch Ledger` section in `TEMPLATE-AILOG.md`** (EN + i18n/es + i18n/zh-CN) — opt-in canonical structure for AILOGs of Charters that span 3+ batches or >1 day of execution. Each batch entry starts as `(pending)`; `straymark charter batch-complete` substitutes the placeholder; `straymark charter drift` fails on any leftover at Charter close. Single-batch AILOGs continue to use `## Actions Performed` and skip the ledger entirely — the section is purely additive.
- **Charter template §Tasks guidance** (EN + i18n/es + i18n/zh-CN) — new task #6 explicitly tells the Charter author to maintain a `## Batch Ledger` and run `batch-complete` per batch when execution is multi-batch. Single-batch Charters skip the step.

### Changed (Framework)

- **`dist/.github/workflows/docs-validation.yml`** — naming regex extended from `[0-9]{3}` to `[0-9]{3}[a-z]?` (closes [#145](https://github.com/StrangeDaysTech/straymark/issues/145)). The optional single-letter suffix resolves same-day same-sequence filename collisions without renumbering downstream entries (e.g. `AILOG-2026-05-02-028b` when `-028-` is taken). Lowercase only, single letter only — discourages multi-letter ad-hoc labels that would erode the convention.
- **`dist/.github/workflows/docs-validation.yml`** — shellcheck cleanup (closes [#143](https://github.com/StrangeDaysTech/straymark/issues/143)). Removed unused `ERRORS=0` in the high-risk ETH compliance step; grouped repeated `echo ... >> "$GITHUB_STEP_SUMMARY"` blocks (SC2129) into `{ ... } >> "$..."` redirects for atomicity and readability; replaced `grep -v X | wc -l` with `grep -vc X` (SC2126); fixed `while read file` → `while IFS= read -r file` (SC2162). Adopters who add actionlint to their CI (e.g., Sentinel via `StrangeDaysTech/sentinel#72`) can now run `actionlint -color` cleanly without needing `-shellcheck=` or per-adopter `-ignore` patterns.
- **`dist/.straymark/schemas/charter.schema.v0.json`** + **`dist/.straymark/schemas/charter-telemetry.schema.v0.json`** — `originating_ailogs` pattern extended to accept the optional letter suffix, in lockstep with the workflow regex.

### Added (CLI)

- **`straymark charter batch-complete <CHARTER-ID> <N>`** — new subcommand. Marks a Charter batch as complete in the originating AILOG's `## Batch Ledger`, substituting the `(pending)` placeholder. Three modes: interactive prompts by default (TTY-only, three short questions: files touched, tests, design note); one-shot via `--note "..."` (designed for agents and scripts); `--non-interactive` requires `--note` and aborts cleanly if missing. Refuses to overwrite an already-completed batch. Closes Proposal C of [#146](https://github.com/StrangeDaysTech/straymark/issues/146).
- **`cli/src/ailog.rs`** (new module) — shared helpers for AILOG file discovery (`find_ailog_file`, `agent_logs_dir`) and `## Batch Ledger` parsing/editing (`parse_batch_ledger`, `pending_batches`, `write_batch_section`, `ensure_pending`). The file-discovery helpers were previously private to `commands/charter/drift.rs`; promoted to share with `batch_complete`.

### Changed (CLI)

- **`straymark charter drift`** — new **Batch Ledger gate**. When the Charter status is `in-progress` or `closed`, the command also checks every originating AILOG's `## Batch Ledger` for entries left as `(pending)` and fails with a clear diagnostic listing the missing batches. AILOGs without a ledger contribute nothing (the section is opt-in). New `--no-batch-ledger-check` flag bypasses the gate for adopters consolidating the ledger post-close. Charters in `declared` state never trip the gate (nothing has been executed yet).
- **`cli/src/charter_schema.rs`** — `TEST_SCHEMA` literal updated to mirror the framework schema's `[0-9]{3}[a-z]?` pattern; three new tests pin the behavior (`accepts_ailog_id_with_letter_suffix`, `rejects_ailog_id_with_multiletter_suffix`, `rejects_ailog_id_with_uppercase_suffix`).

### Deferred

- **Proposal D of [#146](https://github.com/StrangeDaysTech/straymark/issues/146)** — cross-Charter `lessons-learned.md` index — intentionally **not** in this release. The artifact introduces a new canonical taxonomy (`LL-YYYY-MM-DD-NNN` IDs, frontmatter, promotion CLI, discovery integration with `explore`) and warrants a dedicated Charter post-announcement. Sentinel's local workaround for D stays in place; we'll revisit upstream after the web/announcement cycle lands.

### Adopter guidance

- `straymark update-framework` brings the new template sections and schema changes. Existing AILOGs without `## Batch Ledger` continue to work — the drift gate only activates when the section is present.
- `straymark update-cli` brings the new `batch-complete` subcommand and the extended `drift` gate. Both are backward-compatible at the CLI surface (no existing flags renamed).
- **Sentinel reconciliation path**: revert the local `actionlint -shellcheck=` flag, the local regex extension in `docs-validation.yml`, and the `CLAUDE.md` "Multi-batch Charter discipline (TRANSIENT)" subsection — all three are now load-bearing upstream.

---

## Framework 4.13.4 — Close translation gaps (ISO-25010 reference + Charter template zh-CN)

Framework-only patch that closes two translation coverage gaps surfaced by an audit of the project's i18n consistency. The audit confirmed that skills, workflows, schemas, and CLI-internal i18n are correctly aligned with the project principle (LLM-processed assets stay EN-only; human-primary artifacts get translated). Two specific files were the exception.

### Added (Framework)

- **`dist/.straymark/00-governance/i18n/es/ISO-25010-2023-REFERENCE.md`** (new) — Spanish translation of the ISO/IEC 25010:2023 software quality reference doc. Preserves the table structure (9 quality characteristics, sub-characteristics, 2023-vs-2011 changes column) and the canonical Spanish equivalents of the standard's terminology (`Adecuación funcional`, `Capacidad de interacción`, `Confiabilidad`, etc.).
- **`dist/.straymark/00-governance/i18n/zh-CN/ISO-25010-2023-REFERENCE.md`** (new) — Simplified Chinese translation, same structure, uses the Chinese-localized terminology (`功能适合性`, `交互能力`, `可靠性`, `安全（Safety）`, etc.).
- **`dist/.straymark/templates/charter/i18n/zh-CN/charter-template.md`** (new) — Simplified Chinese translation of the Charter scaffold template. Brings zh-CN to parity with EN canonical and the existing ES overlay (both already shipped). All 6 format-conventions footer notes translated; structure and placeholders intact.

### Empirical context

Surfaced while auditing the framework's translation matrix end-to-end: of the 7 categories where translation parity matters (templates, governance docs, adopter docs, README, TUI i18n strings, charter template, audit prompt), all were complete or alignment-consistent except for these three files. The remaining `docs/contributors/TRANSLATION-GUIDE.md` gap was left intentional — its target audience already reads English to read the guide.

### Adopter guidance

`straymark update-framework` brings the three new files. Spanish and Chinese governance review now references `ISO-25010-2023-REFERENCE.md` in the operator's preferred language; `straymark new charter` from a zh-CN project (`.straymark/config.yml` with `language: zh-CN`) now scaffolds the Charter template in Chinese instead of falling back to the English canonical.

---

## Framework 4.13.3 / CLI 3.12.3 — Audit prompt becomes EN-canonical + CLI wires i18n resolution

Aligns the external-audit cycle with the framework's localization convention. Before this release, `dist/.straymark/audit-prompts/audit-prompt.md` was the **only** framework artifact whose canonical content lived in Spanish — every other template, skill, workflow, and governance doc had EN canonical at the root and `i18n/es/` (plus optionally `i18n/zh-CN/`) as overlays, resolved by `resolve_localized_path` (`cli/src/utils.rs:146`). The audit prompt was a Sentinel-derived artifact (parameterized but never translated to EN) and the CLI's `straymark charter audit` command hardcoded the canonical path without going through the i18n resolver — so even if an overlay file existed, the CLI would not have picked it up.

Surfaced empirically when [Sentinel](https://github.com/StrangeDaysTech/sentinel) audited the `straymark explore` rendering and the user noted that audit-cycle templates were ES while all other templates and skills were EN.

### Changed (Framework)

- **`dist/.straymark/audit-prompts/audit-prompt.md`** — rewritten as EN canonical. The seven universal sections (ABSOLUTE RULE, Your role, Scope rules, Step 2 mandatory verification, Step 5 severity calibration, What you must NOT do, Output format) preserved verbatim from the Sentinel-derived original, translated to English. Placeholders (`{{charter_id}}`, `{{charter_title}}`, `{{charter_path}}`, `{{charter_content}}`, `{{git_range}}`, `{{git_diff}}`, `{{ailog_paths}}`, `{{ailog_contents}}`, `{{audit_role}}`, `{{schema_path}}`, `{{project_context}}`) intact and unchanged. HTML documentation header updated to describe the new EN-canonical / `i18n/es/` overlay layout instead of the v0 "future canonical path" note.
- **`dist/.straymark/audit-prompts/i18n/es/audit-prompt.md`** (new) — the Spanish translation now lives here. Resolved automatically when `.straymark/config.yml` declares `language: es`. Spanish-speaking adopters (Sentinel) continue to receive a Spanish audit prompt.
- **Etapa 12 Pub/Sub example generalized.** The Step 5 severity-calibration example previously cited Sentinel's "Etapa 12 Pub/Sub stub vs gochannel" case verbatim. Replaced with a generic equivalent ("declared deferral, not a defect" — a charter that introduces a thin adapter slated for replacement in a future Charter) that illustrates the same anti-inflation discipline without tying the prompt to Sentinel's stack. Applied identically to EN and ES.
- **`dist/dist-manifest.yml`** — version bumped to `4.13.3`. `audit-prompts/i18n/` ships under `.straymark/` (already covered by the `.straymark/` entry in the manifest's `files:` list, no manifest change beyond the version field).
- **Governance footers** updated to `v4.13.3` across `QUICK-REFERENCE.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `C4-DIAGRAM-GUIDE.md`, `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN + ES + zh-CN, plus the top-level `dist/.straymark/QUICK-REFERENCE.md`).

### Changed (CLI)

- **`cli/src/commands/charter/audit.rs`** — `prepare_unified_prompt` now reads the project's `language` from `.straymark/config.yml` via `StrayMarkConfig::resolve_language` (`cli/src/config.rs:93-101`) and resolves the template path through `resolve_localized_path` (`cli/src/utils.rs:146-154`), the same pattern used by `straymark new`, `straymark charter new`, and `straymark explore` for their localized assets. No new abstractions introduced — just wired the existing resolver to the existing config reader.
- **Integration tests** (`cli/tests/charter_audit_test.rs`) — three new tests pin the wiring: `language: es` resolves the ES overlay; `language: zh-CN` with no overlay falls back to EN canonical; explicit `language: en` always picks the canonical EN file.

### Empirical context

The inconsistency was structural (single-artifact ES in an otherwise EN-canonical framework) rather than a bug — `straymark charter audit` worked, the prompt was a competent Spanish prompt, modern auditor LLMs handle it. The cost was twofold: (1) cross-model audit calibration suffers when reports come back in Spanish from Gemini/Claude/Copilot — the delta between auditors competes with normal Spanish phrasing variance instead of being a pure semantic signal; (2) any non-Spanish adopter received a Spanish prompt the first time they ran the audit cycle, which broke the framework's documented convention.

### Adopter guidance

- **Spanish-speaking adopters (Sentinel)**: after `straymark update-framework && straymark update-cli`, keep `language: es` in `.straymark/config.yml` — the ES prompt now resolves automatically via the i18n overlay. The only content difference from the previous version is the generalized example (the "Etapa 12 Pub/Sub" anecdote was replaced with a generic equivalent in the same Spanish voice).
- **English (default) adopters**: receive the EN prompt out of the box. The prior Spanish artifact is no longer the canonical content at the root.
- **zh-CN adopters**: fallback to EN canonical until someone contributes `dist/.straymark/audit-prompts/i18n/zh-CN/audit-prompt.md`. No CLI work required to enable that — `resolve_localized_path` already supports it.
- **Adopters who locally edited their audit-prompt.md**: `straymark update-framework` will overwrite the canonical file with the new EN content. If you customized the prompt, either reapply your edits on top of the EN canonical (recommended — converges with the framework) or move your customization into your project's `i18n/<lang>/audit-prompt.md` overlay (which `straymark update-framework` does not touch).

---

## CLI 3.12.2 — `straymark explore` shows Charter frontmatter correctly

Fixes a long-standing bug where the **Metadata** panel of `straymark explore` rendered every Charter as `Status: ? UNKNOWN` regardless of the actual `status:` value in the Charter frontmatter. Surfaced by [Sentinel](https://github.com/StrangeDaysTech/sentinel) while reviewing a closed Charter (`CHARTER-13`) in the TUI — the screen showed UNKNOWN even though the Charter's frontmatter clearly read `status: closed`.

### Fixed (CLI)

- **`cli/src/tui/document.rs`** — `Document::load` parsed every `.md` against `DocFrontMatter`, whose `status` enum only knows `draft|accepted|deprecated|superseded`. Charter frontmatter (`status: declared|in-progress|closed`) fell through to the `#[serde(other)] Unknown` variant, and the disjoint fields (`charter_id`, `effort_estimate`, `trigger`, `originating_ailogs`, `originating_spec`) were silently dropped. The loader now dispatches to `crate::charter::parse_charter` for paths under `.straymark/charters/NN-slug.md` and returns a typed `DocumentMetadata::Charter(_)` variant.
- **`cli/src/tui/widgets/metadata_panel.rs`** — renders a Charter-specific layout when the variant is `Charter`: `Charter ID`, `Status` (with the correct vocabulary and color: yellow ○ declared / cyan ◐ in-progress / green ■ closed), `Effort` (XS/S/M/L color-graded), `Trigger` (truncated single-line), `Origin`, and `Related` links (materialized from `originating_ailogs` + `originating_spec`, navigable via `Enter` like any other related link).
- **`cli/src/tui/ui.rs`** — `metadata_panel_height` now computes a sensible vertical reserve for both governance docs and Charters; previously it under-reserved for Charters and the panel could clip its lower fields.
- **`cli/src/tui/app.rs`** — `tag_count`, `related_count`, and `metadata_enter` now go through the new `Document::tags()` / `Document::related()` helpers so the existing tag-search and follow-related-link interactions work for both variants without any further branching at the call site.

### Changed (CLI)

- **`cli/src/charter.rs`** — `is_charter_filename` is now `pub` so the TUI loader can use it for dispatch. No behavioral change.
- **`cli/src/tui/i18n_strings.rs`** — adds `Charter ID:`, `Effort:`, `Trigger:`, `Origin:` translations (ES + zh-CN).

### Empirical context

The `Status: ? UNKNOWN` rendering was misleading because the frontmatter *did* parse — just against the wrong schema. `serde_yaml::from_str::<DocFrontMatter>` succeeded with every charter field defaulted and `status = Some(Unknown)` via `#[serde(other)]`, which is why the panel showed an explicit UNKNOWN instead of falling back to `No frontmatter`. The fix avoids re-parsing at render time by routing the schema choice at load time, keyed on the canonical `.straymark/charters/NN-slug.md` path — the same heuristic `discover_charters` uses to skip the adopter-maintained `README.md` status board.

### Adopter guidance

`straymark update-cli` brings the corrected binary. No project state changes required. The fix is purely TUI-side; `straymark charter` subcommands (`new`, `list`, `status`, `close`, `audit`, `drift`) were already on the correct schema and are not touched.

---

## Framework 4.13.2 — Complete the Charter path migration to `.straymark/charters/`

Completes the path migration that `fw-4.12.0` started ([#119](https://github.com/StrangeDaysTech/straymark/issues/119)). That release aligned all `straymark charter` CLI subcommands to `.straymark/charters/` (single source of truth via `charter::charters_dir(project_root)`) but missed three classes of artifact still pointing at the legacy `docs/charters/` path. Surfaced empirically by the Sentinel adopter during external-audit prep when reviewing the framework's pre-PR hook behavior.

### Fixed (Framework)

- **`dist/.straymark/hooks/pre-pr.sh`** — load-bearing fix. The hook gated on `[ ! -d docs/charters ]` and the in-progress Charter glob was `docs/charters/*.md`. For any adopter on the canonical `.straymark/charters/` layout (i.e., everyone post-`fw-4.12.0`), the hook **silently exited 0 without running drift check** — `straymark init --hooks` installed a no-op. Now correctly scans `.straymark/charters/*.md` (3 hits in the script: comment + directory test + glob).
- **`dist/.straymark/schemas/charter-telemetry.schema.v0.json`** — `charter_id.description` field referenced `docs/charters/` as the canonical Charter file location. Now matches `.straymark/charters/`.
- **`dist/.straymark/templates/charter/charter-template.md`** (EN + ES) — step 3 of "Closing the charter" instructed *"Move the row in `docs/charters/README.md`"*; adopters following the template literally pointed at a non-existent path. Now `.straymark/charters/README.md`.

### Changed (Framework — user-facing docs)

- **`README.md` + ES + zh-CN** — `straymark validate --include-charters` description aligned to `.straymark/charters/`.
- **`CLI-REFERENCE.md` + ES + zh-CN** — multiple references (flag descriptions, command prose, example outputs) aligned to `.straymark/charters/`. Includes the example outputs that the CLI itself emits — those showed the legacy path which would have confused operators comparing their actual CLI output against the docs.

### Empirical context

Surfaced while [Sentinel](https://github.com/StrangeDaysTech/sentinel) prepared an external audit cycle and noticed that the upstream `fw-4.13.1` install retained `docs/charters/` in schemas + templates + hook + docs even though the CLI binary itself was already on `.straymark/charters/`. The pre-PR hook bug was the most operationally consequential: any adopter running `straymark init --hooks` since `fw-4.12.0` had a silently-broken hook.

### Not adding a migration helper

This release does not introduce a CLI command to detect adopter projects still on the legacy `docs/charters/` layout — that migration was already declared a [pre-1.0 breaking change in `fw-4.12.0`](https://github.com/StrangeDaysTech/straymark/blob/main/CHANGELOG.md#framework-4120--cli-3120--charter-discoverability--path-alignment) with the documented one-command fix (`git mv docs/charters .straymark/charters`). The improved CLI error message added there continues to apply.

### Adopter guidance

- Existing `fw-4.13.1` adopters: `straymark update-framework` brings the corrected hook, schema, template, and docs. No charter file moves required (those already happened at `fw-4.12.0`). Operators using `--hooks` will see drift check actually run for the first time since the canonical-path adoption.
- Operators who were on `docs/charters/` and never migrated: see the `fw-4.12.0` entry for the one-command migration.

---

## Framework 4.13.1 — TDE trigger refinements + FU→TDE promotion path schema v0.1

Lands Sentinel's empirical-validation feedback from [#128 comment](https://github.com/StrangeDaysTech/straymark/issues/128#issuecomment-4426059594) (3 TDEs filed in [StrangeDaysTech/sentinel#61](https://github.com/StrangeDaysTech/sentinel/pull/61), all 4 trigger criteria validated; two textual ambiguities + one schema gap surfaced). See [#135](https://github.com/StrangeDaysTech/straymark/issues/135) for the broader 4-tier roadmap; this release is **Tier 1** (governance-only, no CLI changes).

### Changed (Framework)

- **`AGENT-RULES.md §3` "TDE vs `R<N> (new, not in Charter)`" — heritage trigger** (EN + ES + zh-CN). Disambiguates two TDE-worthy shapes that the prior wording conflated:
  - **Strict heritage** — prior Charter introduced the debt; subsequent Charters propagate without re-introducing (legacy DB schema decision, deferred config). Inherited by transitive contact.
  - **Pattern propagation** — prior Charter set a pattern; subsequent Charters re-introduce the same debt by following the pattern (handler shape that omits `RequireScope`, test scaffolding that bypasses HTTP middleware). Fix is at the pattern level, not at any single Charter.

  Sentinel's TDE-002 case (CommsHub HTTP layer test coverage gap) was pattern-propagation, not strict heritage; the heuristic captured it correctly but the wording read as strict-only.

- **`AGENT-RULES.md §3` multi-module trigger** (EN + ES + zh-CN). Reformulated from "applies to multiple modules or multiple Charters" to "applies to multiple modules **or Charter execution boundaries**". Captures governance-trail debt that spans sessions without spanning code modules — e.g., a deferred classification in CHARTER-04 that passes silently through CHARTER-08 → CHARTER-13 and only surfaces under a fresh CI gate. Sentinel's TDE-003 case (2 legacy MVP AILOGs pending human review).

- **`FOLLOW-UPS-BACKLOG-PATTERN.md` "Promotion to TDE"** (EN + ES + zh-CN). Documents two equally-valid promotion shapes:
  - **Promotion of existing entry** — open FU registered weeks/Charters ago, lived through ≥1 close, meets the criteria. Standard flow.
  - **Retroactive promotion at creation** — debt recognized as TDE-worthy *during* a retrospective; TDE created first, FU added to the registry with `Status: promoted` from birth. Cross-references TDE back to the originating `R<N>` / calibrator finding / deferred classification.

  Both produce the same end state (`Status: promoted`, `Promoted to: TDE-...`); drift script treats them identically.

### Added (Framework)

- **`FOLLOW-UPS-BACKLOG-PATTERN.md` frontmatter schema v0.1** (EN + ES + zh-CN). Canonicalizes operator-maintained counters in the registry header: `total_open`, `total_promoted`, `total_closed_in_session`, `total_phase_blocked`. `total_promoted` is the new entry, mirroring Sentinel's adopted convention. Operator-driven; the drift script does not update them automatically.

### Empirical context

Sentinel ran all 4 trigger criteria against 3 retrospective TDEs:

| TDE | Triggers (of 4) | Notes |
|-----|----------------:|-------|
| TDE-001 (RequireScope architectural gap) | 4/4 unambiguous | Textbook trigger application |
| TDE-002 (HTTP layer test coverage gap) | 3/4 clean + 1 marginal | "Heritage" marginal → wording refined in this release |
| TDE-003 (legacy MVP AILOGs review) | 3/4 clean + 1 marginal | "Multi-module" marginal → wording refined in this release |

Heuristic captured all 3 cases correctly **with the pre-4.13.1 wording**; refinements remove residual ambiguity for adopter N=2+.

### Not in this release (per [#135](https://github.com/StrangeDaysTech/straymark/issues/135))

- **Tier 2** (ship `check-followups-drift.sh` in `dist/.straymark/scripts/`) — deferred, no adoption friction signal yet
- **Tier 3** (soft integration with `straymark charter close`) — deferred, no friction signal
- **Tier 4** (`straymark followups list/status/promote/drift` CLI subcommand trio) — deferred indefinitely against second-adopter validation gate per design principle #12

### Adopter guidance

- Existing fw-4.13.0 adopters: `straymark update-framework` brings the refinements in. No document edits required — the changes are clarifications, not breaking schema changes. `total_promoted` is opt-in (omit the field if you don't use it; the registry still parses fine).

---

## CLI 3.12.1 — Validator accepts TDE `identified` status

Closes [#130](https://github.com/StrangeDaysTech/straymark/issues/130). Latent in `cli-3.12.0` and earlier; fully exposed by the `fw-4.13.0` TDE activation trigger (which makes TDE creation likely). Surfaced empirically while verifying #128: a freshly-created TDE via `straymark new --doc-type tde` shipped with the canonical `status: identified` per `TEMPLATE-TDE.md` + `DOCUMENTATION-POLICY.md §6`, but `straymark validate` immediately rejected it with `META-003 Invalid status 'identified'`. Validator's hardcoded enumeration was missing the only non-`draft`/non-`accepted` default in the type matrix.

### Fixed (CLI)

- **`validation.rs:45`** — `VALID_STATUSES` now includes `identified` alongside `draft`, `review`, `accepted`, `superseded`, `deprecated`. No per-doc-type dispatch yet — flat list still applies to all 12+ document types, but every documented default now lives in the list. A per-type validation set is deferred to v1 against a second-adopter validation gate (same gate as the follow-ups CLI per `FOLLOW-UPS-BACKLOG-PATTERN.md`).
- **`cli/tests/validate_test.rs`** — new regression test `test_validate_tde_document_valid()` exercises the canonical `status: identified` flow. The test fails against unpatched validators (proven during TDD) and would have caught the bug at `fw-4.13.0` ship time.

### Changed (Framework)

- **`DOCUMENTATION-POLICY.md §3 "Document Statuses"`** (EN + ES + zh-CN) — lifecycle diagram and table extended to document `identified` as the TDE-only entry state. Functionally equivalent to `draft` for lifecycle gating, semantically distinct so adopter analytics can distinguish "agent-discovered debt" from "human-drafted document". Reconciles §3 prose with the §6 per-type default table that already declared TDE → `identified`. Ships as part of `fw-4.13.0` — no separate framework bump.

### Adopter guidance

- Adopters on `cli-3.12.0` or earlier with one or more TDE documents in `06-evolution/technical-debt/` will see `straymark validate` start passing once they upgrade via `straymark update-cli`. No document edits required; the canonical `status: identified` is now accepted.
- Operators who manually changed TDE documents to `status: draft` to work around the bug may revert to `status: identified` for semantic clarity, but the workaround keeps working too.

---

## Framework 4.13.0 — TDE activation trigger

Closes [#128](https://github.com/StrangeDaysTech/straymark/issues/128). The TDE (Technical Debt) document type had structural shape — template, destination folder, autonomy boundary — but no operational activation trigger in the agent-facing governance. Empirical signal from the Sentinel adopter (primary, fw-4.12.0): zero TDEs created across 13 closed Charters despite ≥7 instances of transversal debt being routed through parallel mechanisms (`R<N> (new, not in Charter)` in AILOG `§Risk`, or follow-ups in `follow-ups-backlog.md`). This release adds the trigger and the `R<N>` vs TDE disambiguation, plus a promotion path from the follow-ups backlog. Governance-docs-only — no CLI changes; the `straymark debt list/status/close` subcommand trio is deferred to v1 (same gate as the follow-ups CLI per `FOLLOW-UPS-BACKLOG-PATTERN.md`).

### Added (Framework)

- **TDE activation trigger** in `AGENT-RULES.md §2 "When to Document"` (EN + ES + zh-CN). New row routes transversal technical debt to TDE creation.
- **TDE vs `R<N> (new, not in Charter)` disambiguation** in `AGENT-RULES.md §3` (EN + ES + zh-CN). Four canonical triggers for TDE: *heritage from prior Charter*, *applies to multiple modules/Charters*, *requires dedicated Charter outside current scope envelope*, *requires human prioritization/assignment*. If none apply, the debt is an `R<N>` row in the current Charter's AILOG; if any apply, file a TDE.
- **TDE row in "When to Document"** quick tables: `QUICK-REFERENCE.md` (EN + ES + zh-CN) and `STRAYMARK.md §6`. Surfaces the trigger at the every-session entry points.
- **Promotion path FU → TDE** in `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN + ES + zh-CN). New status `promoted`, new `Destination: TDE-YYYY-MM-DD-NNN` value, new `Promoted to:` entry field, plus a "Promotion to TDE" section with criteria (mirroring the `AGENT-RULES.md §3` heuristics) and the operator-driven workflow. Post-Charter close checklist now includes "promote un-resolved transversal entries to TDE."
- **`promoted_from_followup: FU-NNN` frontmatter field** in `TEMPLATE-TDE.md` (EN + ES + zh-CN). Optional, populated when the TDE originates from a backlog entry, preserves traceability.
- **Activation-trigger note** in the TDE template body (EN + ES + zh-CN) so the agent reading the template at creation time sees the four canonical triggers + pointer to `AGENT-RULES.md §3`.

### Changed (Framework)

- **`/straymark-new` skill** (3 surfaces — `.claude/skills/`, `.gemini/skills/`, `.agent/workflows/`): the TDE suggestion row split into two — `TODO`/`FIXME`/`HACK` comments remain as a code-smell trigger, and a new row covers the architectural trigger (heritage, transversal, dedicated Charter, human prioritization) pointing to `AGENT-RULES.md §3`.

### Fixed (Framework)

- **`/straymark-status` skill paths** (3 surfaces — `.claude/skills/`, `.gemini/skills/`, `.agent/workflows/`). Five doc types had stale directory paths that diverged from the canonical layout in `AGENT-RULES.md §5` and `STRAYMARK.md §10`: ADR (`04-architecture/decisions/` → `02-design/decisions/`), REQ (`03-requirements/` → `01-requirements/`), TES (`05-testing/` → `04-testing/`), INC (`06-operations/incidents/` → `05-operations/incidents/`), TDE (`06-operations/tech-debt/` → `06-evolution/technical-debt/`). The TDE bug was surfaced while verifying #128: `/straymark-new` would have written a TDE to the correct path, but `/straymark-status` would never have found it. The other four are pre-existing drift fixed in the same pass.
- **`/straymark-adr` shortcut path in CLI-REFERENCE.md** (EN + ES + zh-CN). Same `04-architecture/decisions/` → `02-design/decisions/` correction.

### Adopter guidance

- Sentinel will create the 3 TDEs surfaced during CHARTER-13 close ceremony retrospective (R7 RequireScope architectural gap, HTTP layer test coverage gap, legacy AILOGs with `review_required: false`) as empirical validation of the trigger heuristic in a follow-up PR.
- Existing projects on `fw-4.12.0` get the new framework files via `straymark update-framework`. The trigger guidance is documentation-only — no migration is required.

---

## Framework 4.12.0 / CLI 3.12.0 — Charter discoverability + path alignment

Closes the two Charter-related gaps surfaced by real adopters in the issue tracker: **Charter were structurally invisible to the CLI** ([#119](https://github.com/StrangeDaysTech/straymark/issues/119) — `straymark charter list/audit/close` hardcoded `docs/charters/` while the framework already validated `.straymark/charters/`) and **Charter were conceptually invisible to onboarding agents** ([#113](https://github.com/StrangeDaysTech/straymark/issues/113) — agents following the canonical entry points could not discover Charter as a workflow concept).

### Added (Framework)

- **`STRAYMARK.md` §15 — Charter as bounded units of work.** Dedicated section explaining what a Charter is, when to declare one, the lifecycle (`declared` → `in-progress` → `closed`), and how it relates to AILOG / ADR / SpecKit. Charter trigger row added to §6 (When to Document); Charter row added to §9 (Autonomy Limits), §10 (folder map), §11 (When to Load), §13 (Quick Type Reference).
- **`SPECKIT-CHARTER-BRIDGE.md`** in `dist/.straymark/00-governance/` (EN + ES + zh-CN). Documents *when* a SpecKit feature yields a Charter (4 yes-conditions, 3 no-conditions), 4 granularity heuristics ("one Charter per shippable cut, NOT per User Story"), creation timing within the SpecKit pipeline, frontmatter linkage in both directions (`originating_spec` / `originating_charter`), 5 anti-patterns, 5 non-fit cases.
- **Skill `/straymark-charter-new`** across the three skill surfaces (`.claude/skills/`, `.gemini/skills/`, `.agent/workflows/`). Drives `straymark charter new` with the right flags (`--from-spec` vs `--from-ailog` vs none, effort estimate). Skill explicitly does *not* flip status, run drift, or run audit — those have their own surfaces.
- **Charter trigger** in the directive templates (`dist/dist-templates/directives/{CLAUDE.md,GEMINI.md,copilot-instructions.md}`) — agents see Charter alongside AILOG / AIDEC / ADR / ETH triggers in the pre-commit checklist.
- **`Charters` block in `straymark status`** — declared / in-progress / closed counts (or a friendly hint when empty); colorized status keyed on lifecycle stage; surfaces unparseable Charter files as a warning row.

### Changed (Framework)

- **Charter templates moved to `dist/.straymark/templates/charter/`** subdirectory (with `i18n/es/` co-located inside). Visually distinguishable from auxiliary doc templates, addresses one of the contributing factors of #113 (templates indistinguishable from auxiliary).
- **`QUICK-REFERENCE.md` (EN + ES + zh-CN)** — adds "Bounded Units of Work — Charter" subsection alongside the doc-type tables, `charters/` entry in the folder tree, Charter trigger row in When-to-Document, `/straymark-charter-new` in the skills table.
- **`/straymark-status` and `/straymark-new` skills (×3 surfaces)** — `/straymark-status` now scans `.straymark/charters/` and surfaces gaps; `/straymark-new` recognizes Charter intent and *redirects* to `/straymark-charter-new` (Charter is not a `straymark new` doc type).

### Fixed (CLI)

- **`straymark charter list/audit/close/drift/new` honor `.straymark/charters/`** as the canonical Charter location, matching what `straymark init` and `straymark status` already validate. Eliminates 5 hardcoded `docs/charters/` references through a new `charter::charters_dir(project_root)` single source of truth.
- **Improved error messages** in `audit/close/drift`: instead of opaque `Charter X not found in docs/charters/`, the CLI now reports the searched path and hints at `straymark charter list`:

  ```
  error: Charter CHARTER-02 not found in .straymark/charters/.
    hint: run `straymark charter list` to see discovered Charters.
  ```

- **`dist/.straymark/scripts/check-charter-drift.sh`** matches against `.straymark/charters/*` instead of `docs/charters/*`.

### Breaking change (CLI, pre-1.0)

- Projects with charters under the legacy `docs/charters/` (Sentinel pre-rebrand layout) need to relocate them. Migration is one command: `git mv docs/charters .straymark/charters`. Pre-1.0 SemVer permits the change; the improved error message points operators at it.

### Adopter guidance

- Existing projects on `fw-4.11.0` get the new framework files (skills, templates, governance docs, directive triggers) via `straymark update-framework`.
- Charter telemetry sidecars (`*.telemetry.yaml`) now share `.straymark/charters/` with the declarative `.md` files by design — no migration needed; `straymark charter close` already wrote them there in 4.11.0.

---

## Framework 4.11.0 / CLI 3.11.0 — StrayMark rebranding

The project formerly known as DevTrail is now **StrayMark**. The decision was made on 2026-05-08 by the operator after external trademark conflict research, motivated by **legal certainty over the project's mark**. See [`ADR-2026-05-08-001`](docs/decisions/ADR-2026-05-08-rebranding-straymark.md) for the full record.

This release ships the rebrand end-to-end across the **live state** of the project. **Historical state is preserved literally** — prior CHANGELOG entries, release titles, tags, the `devtrail-cli@3.10.0` crate (not yanked), and closed issues/PRs all retain the "DevTrail" name as historical record. The single Sentinel adopter (operator-owned) migrates manually via `mv .devtrail .straymark` + `git mv DEVTRAIL.md STRAYMARK.md` + updating refs in their `CLAUDE.md`/`AGENT.md`.

### Changed (Framework + CLI)

- **GitHub repository renamed**: `StrangeDaysTech/devtrail` → `StrangeDaysTech/straymark`. GitHub maintains automatic 301 redirects from the old URL for at least one year (so existing clones, badges, and external links continue to resolve).
- **Framework root path**: `.devtrail/` → `.straymark/`, `DEVTRAIL.md` → `STRAYMARK.md`. Adopters reinstalling get the new layout; existing adopters migrate manually (no fallback logic in CLI).
- **CLI binary**: `devtrail` → `straymark`.
- **CLI crate (crates.io)**: published as `straymark-cli@3.11.0`. The legacy `devtrail-cli@3.10.0` is preserved on crates.io (not yanked) as historical record.
- **Skills (3 platforms)**: `/devtrail-{ailog,aidec,adr,mcard,sec,status,new,audit-prompt,audit-execute,audit-review}` → `/straymark-*` across `.claude/skills/`, `.gemini/skills/`, `.agent/workflows/`.
- **Asset filename prefixes**: `devtrail-fw-X.Y.Z.zip` → `straymark-fw-X.Y.Z.zip`, `devtrail-cli-vX.Y.Z-*` → `straymark-cli-vX.Y.Z-*`.
- **Documentation rebranded** end-to-end: README, CLAUDE.md, governance docs (3 langs), adopter docs (3 langs), CONTRIBUTING, CODE_OF_CONDUCT, all framework distributables.
- **Rust identifiers**: `DevTrailConfig` → `StrayMarkConfig`, `DevTrailDocument` → `StrayMarkDocument`, `GITHUB_REPO` const updated.

### Preserved (immutable historical record)

- All commits, commit messages, and git history.
- Tags published before this release (`fw-4.10.0`, `cli-3.10.0`, and earlier) and their release titles ("DevTrail Framework X.Y.Z", "DevTrail CLI X.Y.Z").
- Prior CHANGELOG entries below (`## Framework 4.10.0 — Follow-ups backlog pattern`, etc.) — preserved literally with the "DevTrail" name and old repo URLs.
- All previously merged PRs, closed issues, and their bodies/comments.
- `devtrail-cli@3.10.0` on crates.io.
- Sentinel adopter's AILOGs and Charters that reference "DevTrail" — those are Sentinel's history, not this repo's.

### Versioning continuity

Versions continue the existing series — **no reset to 0.1.0**. The product trajectory is unbroken; only the name changes. Tag prefixes (`fw-`, `cli-`) are agnostic to the project name and continue unchanged.

### Adopter impact

Single known adopter (`StrangeDaysTech/sentinel`, operator-owned) migrates manually. No third-party adopters are known. Any silent adopters (improbable) are protected for at least one year by the GitHub redirect; the legacy `devtrail-cli@3.10.0` crate continues to be installable.

The README does not yet include an etymological paragraph explaining the meaning of "StrayMark" — that is a deliberate follow-up, deferred from the rebranding execution per operator instruction.

### Errata — migration step missing from original release notes *(documented 2026-05-12, after Sentinel adopter encountered it)*

The "migrates manually via `mv .devtrail .straymark` + `git mv DEVTRAIL.md STRAYMARK.md` + updating refs in `CLAUDE.md`/`AGENT.md`" step above is **incomplete**. It does not mention the cleanup of the per-platform skill directories. Adopters that ran `straymark update-framework` (or applied the rebrand manually) ended up with **30 orphaned legacy items** sitting alongside the new ones:

| Surface | Orphan pattern | Count |
|---|---|---:|
| `.gemini/skills/` | `devtrail-{adr,aidec,ailog,audit-{execute,prompt,review},mcard,new,sec,status}/` | 10 dirs |
| `.claude/skills/` | same pattern | 10 dirs |
| `.agent/workflows/` | `devtrail-*.md` | 10 files |

The `name:` field **inside** each orphaned `SKILL.md` was rewritten to `straymark-*` during the rebrand (probably via mass sed), so each pair has identical `name` but two different directory paths. Gemini CLI surfaces this as 10 `⚠ Skill conflict detected: "straymark-X" from devtrail-X/SKILL.md is overriding the same skill from straymark-X/SKILL.md` warnings at startup. Claude Code and the `.agent/workflows/` consumer do not warn out loud but exhibit equivalent override behavior.

**Complete migration steps** (amending the line above for any adopter migrating from a `fw-≤4.10.x` install):

```bash
# Renames already documented above
mv .devtrail .straymark
git mv DEVTRAIL.md STRAYMARK.md

# Skill directory cleanup — missing from original release notes
rm -rf .gemini/skills/devtrail-*
rm -rf .claude/skills/devtrail-*
rm .agent/workflows/devtrail-*.md

# Update references in agent directive files (CLAUDE.md, GEMINI.md, etc.) as already documented
```

Sentinel applied this cleanup in `StrangeDaysTech/sentinel#62` (2026-05-12). No known third-party adopters, so the impact is bounded to the single project that already fixed it. This errata is preserved here as historical record for any future major rename or repository-level prefix change.

### Errata — root-level installer / SBOM / LICENSE residuals *(documented 2026-05-12)*

Four files in the repository root were missed by the rebrand sweep. The installer pair was the operationally consequential one — broken from the merge of `fw-4.11.0` until this errata:

- **`install.sh` / `install.ps1`** — internals still referenced `REPO=StrangeDaysTech/devtrail`, `BINARY=devtrail` (or `devtrail.exe`), and the legacy asset name `devtrail-cli-v${VERSION}-${TARGET}.{tar.gz,zip}`. Meanwhile `release-cli.yml` had already been producing `straymark-cli-*` assets with the `straymark` binary, so any user piping the installer per the (already-rebranded) README/CLI-REFERENCE instructions hit a GitHub 404. Now aligned: `StrangeDaysTech/straymark`, `straymark`/`straymark.exe`, asset prefix `straymark-cli-`. Windows `LOCALAPPDATA` install path also moved from `DevTrail\bin` → `StrayMark\bin` (no shim — same policy applied to `.devtrail` → `.straymark` and to the skill-dir cleanup above).
- **`devtrail.spdx` → `straymark.spdx`** — `git mv` plus content update of `DocumentName`, `DocumentNamespace`, `PackageName`, `PackageDownloadLocation`, `PackageHomePage`, `PackageCopyrightText` ("DevTrail Contributors" → "StrayMark Contributors"), and `PackageDescription`. `PackageVersion` advanced from the stale `1.0.0` to `3.12.1` to match the CLI binary the SBOM describes.
- **`LICENSE`** — copyright line updated to "StrayMark Contributors" in the live state. The MIT text itself is unchanged.

Historical entries elsewhere in this CHANGELOG and the `devtrail-cli@3.10.0` crate are preserved as historical record per the policy stated at the top of this file.

---

## Framework 4.10.0 — Follow-ups backlog pattern (governance docs)

Documents the follow-ups backlog convention contributed by the Sentinel adopter via [issue #111](https://github.com/StrangeDaysTech/devtrail/issues/111). The pattern was empirically validated in `StrangeDaysTech/sentinel` CHARTER-12 (47 AILOGs accumulated across CHARTER-08 → CHARTER-11). Adopters reaching ~20+ AILOGs benefit from a central registry + per-AILOG drift detection script + agent integration in `CLAUDE.md` / `AGENT.md`.

This release is **docs only** — no CLI changes, no schema changes, no audit flow changes. Cristalization as a `devtrail followups` subcommand is deferred until a second adopter validates the pattern.

### Added (Framework)

- **NEW governance pattern document** `FOLLOW-UPS-BACKLOG-PATTERN.md` describing the reproducible convention: registry shape (`.devtrail/follow-ups-backlog.md` with `fully_extracted_ailogs` frontmatter), 5 buckets (`ready` / `time-triggered` / `charter-triggered` / `phase-blocked` / `operational`), per-entry schema (FU-NNN / Origin / Status / Trigger / Destination / Cost / Notes), drift detection script (3 modes: default / `--apply` / `--scan-all`), agent integration block, adoption walkthrough, reference implementation in Sentinel CHARTER-12 (PRs #53/#54), and open questions (bucket heuristic, schema validation, audit-cycle integration, cristalization path).
- **Pointers** added in `AGENT-RULES.md` (new "Patterns" section) and `QUICK-REFERENCE.md` (new "Patterns" table) referencing the new pattern document.
- **Translations** to ES and zh-CN under `dist/.devtrail/00-governance/i18n/{es,zh-CN}/FOLLOW-UPS-BACKLOG-PATTERN.md`.

### Changed (Framework)

- Footer version bumps from `v4.9.0` to `v4.10.0` across governance docs (EN + ES + zh-CN) and version-table references in adopter docs.

### Status

The pattern is **v0** — proven in N=1 domain (Sentinel). It may evolve into a CLI helper after a second-domain validation. Adopters who implement it now follow the documented convention; their local script + registry is fully portable and survives a future cristalization unchanged.

---

## Framework 4.9.0 / CLI 3.10.0 — Audit v1: zero copy/paste flow with auditor-side CLI tool use

Closes the four axes reported in [issue #102](https://github.com/StrangeDaysTech/devtrail/issues/102) by Sentinel during its first primary-adopter run of the v0 audit-skills (CHARTER-07 of CommsHub Etapa 2). The release is **one integrated iteration** rather than four separate patches — Sentinel re-runs CHARTER-07 once after this lands, with the full v1 flow, instead of multiple times against partial fixes.

This is the largest single audit-flow refactor since v0 shipped. Operators now invoke three skills in sequence (`audit-prompt` → `audit-execute` × N → `audit-review`) over canonical filesystem paths under `.devtrail/audits/`, and **never copy/paste prompts or reports**. The unified prompt template lifts the seven universal sections from Sentinel's pre-DevTrail audit skill (contributed via the issue), parameterized against Charter doc + originating AILOGs + git range. The review evolves from "validate + merge YAML" to a six-section consolidated analysis (Executive summary / Scope / Per-auditor evaluation / Remediation plan P0-P4 / Discarded / Auditor ratings).

### Added (Framework)

- **NEW skill `devtrail-audit-execute` (3 platforms)** — runs inside an auditor-side CLI (gemini-cli, claude-cli, copilot-cli, codex-cli). Reads the prompt at the canonical path, audits with tool use citing `path:line`, writes a report keyed on the auditor's model id. Auto-discovery when CHARTER-ID argument is omitted (D14). Wait-for-all-audits warning at completion is load-bearing for parallel-CLI workflows.
- **NEW unified prompt template** `dist/.devtrail/audit-prompts/audit-prompt.md` (325 lines) lifting the seven universal sections from Sentinel's `audit/SKILL.md`: REGLA ABSOLUTA — SOLO LECTURA, Tu rol (anti-cheerleader), Reglas de alcance, Paso 2 verificación obligatoria, Paso 5 calibración severidad (anti-inflation/deflation with the Etapa 12 example preserved as labeled real adopter case), Lo que NO debes hacer, Formato de salida.
- **AGENT-RULES.md §12 Audit checkpoint** updated for the 3-skill sequence + canonical paths under `.devtrail/audits/`. Wait-for-ALL-audits warning surfaces in both the message text and the rules of engagement.

### Changed (Framework)

- **Skills `devtrail-audit-prompt` and `devtrail-audit-review` rewritten** for v1: prompt skill no longer surfaces prompts inline (writes to canonical path; operator opens auditor CLIs). Review skill evolves to consolidated analysis generator producing `review.md` with 6 sections + 5-verdict vocabulary (VALID / PARTIALLY VALID / MISATTRIBUTED / FALSE POSITIVE / DUPLICATE) + 4-criterion weighted auditor rating (Scope precision 25% / Technical depth 25% / Bug detection 30% / False positive rate 20%). Both lifts Sentinel's `audit-review/SKILL.md` mature pre-DevTrail.
- **Adopter docs** (CLI-REFERENCE, WORKFLOWS, ADOPTION-GUIDE, QUICK-REFERENCE) in 3 langs aligned to v1 flow.

### Removed (Framework, BREAKING within `v0.x` schemas)

- DELETE `dist/.devtrail/audit-prompts/auditor-primary.md` (154 lines), `auditor-secondary.md` (131 lines), `calibrator-reconciler.md` (173 lines). Replaced by the single unified `audit-prompt.md`.

### Added (CLI)

- **NEW flag `--prepare`** on `devtrail charter audit` — generates the unified prompt at `.devtrail/audits/<id>/audit-prompt.md`. Default action when no other action flag is passed.
- **NEW flag `--merge-reports`** — reads N `report-*.md` files from the canonical audit dir, validates each against `audit-output.schema.v0.json`, emits/merges the `external_audit` YAML. Replaces the v0 two-step `--calibrate` then `--finalize`.
- **`--merge-into <PATH>`** combines with `--merge-reports` (or deprecated `--finalize`); strict `requires = "finalize"` removed.
- **Schema `audit-output.schema.v0.json` evolved**: `audit_role` enum extended to `["auditor", "auditor-primary", "auditor-secondary"]` (v1 unified value + v0 legacy). NEW optional `evidence_citations: integer (>=0)` for review-skill weighting. `calibratorOutput.auditors_reconciled.maxItems` removed (v1 supports N≥2).

### Changed (CLI)

- **`git_range` default** changes from `HEAD~1..HEAD` to `origin/main..HEAD` (with fallback to `origin/master..HEAD`, then to `HEAD~1..HEAD` with stderr warning when no upstream is reachable). Fixes R11(A): Sentinel CHARTER-07 had 8 commits on a feature branch; v0 default sent only the last commit to auditors.
- **Canonical audit path migration**: `audit/charters/<CHARTER-ID>/` → `.devtrail/audits/<CHARTER-ID>/`. Per propuesta D13: namespaced under `.devtrail/` to avoid collisions with adopter-defined `audit/` folders; structure leaves room for future audit-unit categories beyond Charter.
- **Resolved prompt is one file, not two**: `audit-prompt.md` (was `auditor-{primary,secondary}.prompt.md`).
- **Reports keyed on model slug**: `report-<sluggified-model-id>.md` (was `auditor-{primary,secondary}.md`).

### Fixed (CLI)

- **R10 — resolver respects HTML comment bounds.** Issue #102: `auditor-primary.md` template's documentation header listed placeholders with literal `{{name}}` syntax, and the global `String::replace` expanded them inside the `<!-- ... -->` block, duplicating ~30k tokens of payload. Resolver now scans for comment ranges before substituting and skips placeholder replacement inside them. Unclosed comments terminate the scan early (conservative).
- **`render_external_audit_yaml` uses canonical Charter id** in `audit_notes:` instead of literal `<charter-id>` placeholder (pre-existing bug fixed as side-effect of refactor).

### Deprecated (CLI)

- **`--calibrate`** — emits warning explaining the v1 flow has no separate calibrate step (`/devtrail-audit-review` skill handles the calibrator role inline) and exits with error. Hidden in `--help`.
- **`--finalize`** — deprecated alias for `--merge-reports`. Emits warning and routes through the new path. Hidden in `--help`.

### BREAKING (deliberate, within experimental v0.x schemas)

- Convention of paths changes from `audit/charters/` to `.devtrail/audits/`. Audits in flight that used v0 paths (Sentinel CHARTER-07 paused state) need to be re-run under v1 — the v0 outputs stay as historical evidence at the v0 path.
- The 3 v0 prompt templates are removed. Adopters who customized them must port their changes to the unified `audit-prompt.md`.
- The CLI no longer reads from `audit/charters/<id>/` — only from `.devtrail/audits/<id>/`.

### Tests

- 5 new unit tests for the R10 resolver fix (HTML comment boundaries).
- 3 new integration tests for the `git_range` default change (R11(A)) — uses `init_repo_with_remote_main` helper with isolated bare-repo TempDirs to avoid parallel-test collisions.
- 9 new fixture tests for the unified prompt template (canonical path, 7 universal sections, expected placeholders, didactic Etapa 12 example, Sentinel credit, evidence discipline, schema accepts v1 + legacy, evidence_citations optional, calibrator supports N≥2).
- 17 charter_audit integration tests rewritten for v1 (10 new + 7 v0-tests-ported-to-v1 paths/flags).
- 4 new fixture tests for `devtrail-audit-execute` skill (per-platform frontmatter + cross-platform parity asserting D14 elements + wait warning + path:line discipline).
- audit_skill_test parity assertions updated for the rewritten audit-prompt and audit-review skills (six-section structure, 5-verdict vocabulary, 4-criterion rating, `external-audit-pending.yaml` for Branch B).

### Credit

The seven universal sections of the unified prompt template, the six-section structure of the consolidated review, the five-verdict vocabulary, and the four-criterion weighted auditor rating all lift directly from Sentinel's pre-DevTrail audit-skills (`audit/SKILL.md` and `audit-review/SKILL.md`), contributed via [issue #102](https://github.com/StrangeDaysTech/devtrail/issues/102) by José Villaseñor Montfort (StrangeDaysTech). Sentinel-specific hardcodes (paths, headings, build commands) were parameterized; didactic examples (Etapa 12 Pub/Sub stub vs gochannel active) preserved as labeled real adopter cases.

---

## Framework 4.8.0 / CLI 3.9.0 — External audit skills + workflow checkpoint

Phase 1 of `Propuesta/devtrail-audit-skills.md`: closes the back-half of the external multi-model audit cycle by surfacing it inside the AI assistant in the loop, and codifies a soft (never-enforced) workflow checkpoint where the agent proactively offers the audit at the right moment. External audit remains **fully optional** — the Charter's declarative scope + drift check + AILOG discipline already provide rigorous closure without it. The skills only add UX-inline; the underlying CLI orchestration is unchanged in shape, only extended with a new `--merge-into` flag to close the manual copy-paste loop.

This release also fixes a pre-existing bug in `devtrail charter audit --finalize` where the rendered `audit_notes:` field contained the literal placeholder `<charter-id>` instead of the canonical Charter id.

### Added (Framework)

- **Skill `/devtrail-audit-prompt CHARTER-ID`** (3 platforms): `dist/.claude/skills/devtrail-audit-prompt/SKILL.md`, `dist/.gemini/skills/devtrail-audit-prompt/SKILL.md`, `dist/.agent/workflows/devtrail-audit-prompt.md`. Wraps `devtrail charter audit` PREPARE — surfaces both auditor prompts inline in the conversation so the operator can paste them into 2 LLMs of different families without leaving the IDE.
- **Skill `/devtrail-audit-review CHARTER-ID`** (3 platforms): same install shape. Validates the operator-saved auditor responses, runs the calibrator inline (the agent driving the conversation IS a valid calibrator since heterogeneity is required only for the auditor pair), and triggers `devtrail charter audit --finalize --merge-into` so the `external_audit:` array lands directly in the Charter telemetry. Branch B (telemetry not yet present) writes `audit/charters/<id>/external-audit-pending.yaml` for later manual merge.
- **AGENT-RULES.md §12 "Audit Checkpoint"** (3 langs: EN / ES / zh-CN): codifies the 4 boolean triggers, the literal YES/NO message text, the YES/NO heuristics (security surface, new component, AILOG risks, large + first Charter, explicit cross-model request, **arborist 2× threshold complexity signal with graceful-degradation when the `analyze` feature is absent**), and the rules of engagement (emit once per Charter, never block, not counted as a metric). Permanent v0+v1 design decision — the checkpoint will never be escalated to enforcement.
- **Adopter docs surfaced** the new skills + checkpoint in `WORKFLOWS.md`, `CLI-REFERENCE.md` (new `## Skills` section listing all 9 shipped skills), `ADOPTION-GUIDE.md` (new `## External Audit (Optional)` section), and `QUICK-REFERENCE.md` (skills table expanded from 1 row to all 9), across all 3 languages.

### Added (CLI)

- **`devtrail charter audit --finalize --merge-into <PATH>`** *(`cli-3.9.0+`)* — appends the rendered `external_audit:` array directly into the telemetry YAML at `<PATH>` instead of printing it to stdout. String-level append at indent 2 under `charter_telemetry:` (preserves the hand-written shape from `charter close`; no full re-serialization → no comment loss). v0 deliberately rejects re-audit (file already has `external_audit:`) with a clear error — operator reconciles manually rather than risk silent duplication. Missing telemetry path errors with explicit guidance to run `devtrail charter close` first or omit the flag.

### Fixed (CLI)

- **`render_external_audit_yaml` now emits the canonical Charter id** in `audit_notes:` instead of the literal placeholder `<charter-id>`. The function takes the canonical id as a parameter; both the stdout path and the new `--merge-into` path produce correct output.

### Tests

- 4 new integration tests in `cli/tests/charter_audit_test.rs` covering the `--merge-into` flag (happy path, missing telemetry, re-audit guard, clap rejection without `--finalize`).
- 8 new fixture tests in `cli/tests/audit_skill_test.rs` covering both audit skills across 3 platforms (per-platform frontmatter shape + cross-platform parity of load-bearing guidance).
- 4 new fixture tests in `cli/tests/checkpoint_guidance_test.rs` covering the §12 Audit Checkpoint section across 3 languages (presence + cross-language parity of language-agnostic anchors).

### Documentation only

- Versioning tables, governance footers, and CLI output examples updated from `fw-4.7.1` / `cli-3.8.1` to `fw-4.8.0` / `cli-3.9.0` across 22 files (3 languages).

---

## Framework 4.7.1 / CLI 3.8.1 — O3 resolved (`--no-ailog-suppress` always emits INFO line)

Closes the last `pending design discussion` carried forward from issue #81:
**O3** — when `devtrail charter drift --no-ailog-suppress` was passed and there
was nothing for the AILOG-aware filter to suppress (N=0), the output was
**byte-identical** to the default. Operators couldn't tell whether the
suppression logic ran and found nothing, or whether the flag was wired
incorrectly.

Resolution voted by Sentinel CHARTER-06 telemetry on issue #91 (option (c)
"--no-ailog-suppress only", with the N=0 confirming-line extension):

- Default mode stays silent at N=0 — no new noise in the common case.
- `--no-ailog-suppress` always emits at least one line confirming dispatch:
  - At N=0: `INFO: AILOG-aware suppression bypassed (would have suppressed: 0 paths)`
  - At N>0: `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s) listed above as drift)` followed by a per-path list with the AILOG ID that documents the risk.

The asymmetry matches the `git diff --stat` shape — silent default, signal on
explicit opt-in. Operators with dispatch suspicion now have a one-flag debug
path that always says something.

### Fixed (CLI)

- **`devtrail charter drift --no-ailog-suppress`** — emits a confirming
  `INFO:` line at end of output regardless of N. The line names the count
  and (when N>0) lists each path that would have been suppressed with its
  documenting AILOG ID. Default mode (suppression on) is unchanged: silent
  at N=0, existing `AILOG-suppressed: N path(s)` block at N>0.

  Implementation: `compute_ailog_suppressions` now runs unconditionally so
  the count is available regardless of whether suppression is applied. The
  flag controls only whether the suppression mutates the rendered drift
  list.

### Tests

- 3 new integration tests:
  - `charter_drift_no_ailog_suppress_emits_info_line_when_n_zero` (the
    primary case the issue is about).
  - `charter_drift_no_ailog_suppress_emits_info_line_when_n_nonzero`
    (count + per-path listing).
  - `charter_drift_default_stays_silent_when_n_zero` (negative test:
    confirms we did NOT add noise to the common case).
- 414/414 tests pass (3 new on top of 411 from cli-3.8.0).

### Empirical signal that drove this decision

Sentinel CHARTER-06 was constructed deliberately to exercise the
N>0 path (`subscriber.go` declared in the Charter, named in
AILOG-2026-05-03-034's `## Risk` section, not modified during execution).
The captured outputs confirmed the byte-identical-at-N=0 ambiguity and
voted for option (c) with a one-line N=0 confirmation. Full telemetry is
on issue #91.

A secondary observation from the same run — that paths in the WARNING
block don't carry an inline `[suppressed]` annotation, forcing the reader
to scan top-to-bottom to know a WARNING was OK'd — is **not bundled here**.
It's a UX polish item flagged for separate validation; tracked in a
follow-up issue rather than rolled into this patch to keep the change
surgical.

### What's NOT in this release

- Inline `[suppressed]` annotation on WARNING block items (separate issue).
- HTTP API clients for `charter audit` (Phase 3 v1).
- Inter-family heterogeneity auto-enforcement.

---

## Framework 4.7.0 / CLI 3.8.0 — Phase 3 (multi-model external audit, orchestration-only) + open frictions F2/F5/F7

The first feature-bearing release since Phase 2 (fw-4.6.0/cli-3.7.0). Lands the
six PRs of Phase 3 + open frictions F2/F5/F7 as a coordinated bundle. The
remaining gap from issue #81 (F2/F5/F7) is closed; observation O3
(`INFO: 0 paths suppressed` always-on log) remains pending design discussion.

**Compatibility.** No breaking changes. New commands (`devtrail charter audit`)
are additive; existing `charter new`, `approve`, and `charter close` behaviors
are extended (new flags, new auto-write fields, refined output) without
modifying defaults that adopters depended on.

**Architectural decision A1 (Phase 3).** Multi-model external audit ships as
**orchestration-only** — the CLI prepares prompts, validates outputs against
a schema, and prints findings ready for telemetry. It does **not** invoke LLM
APIs. The operator pastes prompts into their auditor of choice (Copilot,
Gemini, Claude, etc.) and saves responses to canonical paths. Rationale lives
in PR #85 + the Phase 3 plan: implementing 3 HTTP clients is 1-2 weeks +
perpetual maintenance when APIs change, and the human-in-the-loop shape
matches Sentinel's empirical `/plan-audit` pattern that motivated Phase 3 in
the first place. v1 may add HTTP clients when a real adopter reports a need.

### Added (Framework)

- **`dist/.devtrail/audit-prompts/auditor-primary.md`** + **`auditor-secondary.md`** + **`calibrator-reconciler.md`** — three prompt templates for the dual-audit + calibrator cycle. Primary and secondary are structurally identical (heterogeneity lives in the auditor MODEL, not in different prompts). Each declares the categorization rules (hallucination / implementation_gap / real_debt / false_positive) and discipline rules ("don't fabricate findings", "no external sources beyond the prompt"). PR #85.
- **`dist/.devtrail/schemas/audit-output.schema.v0.json`** — JSON Schema Draft 2020-12 for the markdown files auditors and the calibrator produce. `oneOf` discriminator on `audit_role` distinguishes auditor outputs (fresh findings) from calibrator outputs (reconciliation across the two). `findings_by_category` enum matches the `external_audit` array in `charter-telemetry.schema.v0.json` so the audit cycle output integrates directly into Charter telemetry. Marked **experimental v0** — same N=1-domain caveat as the other Phase schemas. PR #85.

### Added (CLI)

- **`devtrail charter audit <CHARTER-ID>` (PR #86, Phase 3 v0).** Three steps invokable independently:
  - **Default** = step 1 (PREPARE): resolves the auditor prompts against the Charter content + git diff + originating AILOGs, writes to `audit/charters/<CHARTER-ID>/prompts/`. Per [RFC #82](https://github.com/StrangeDaysTech/devtrail/issues/82) the resolved prompts persist before any external action.
  - **`--calibrate`** = step 2: validates both auditor outputs against the schema, resolves the calibrator prompt with their findings embedded.
  - **`--finalize`** = step 3: validates all 3 outputs, prints a YAML-formatted `external_audit` block ready to paste into Charter telemetry, points to the calibrator's reconciliation summary.
  - Each step is a filesystem mutation. Files persist between steps — operator can prepare, walk away, come back days later, calibrate.
- **`devtrail charter new --slug <value>`** *(F1 fix carried forward from cli-3.7.2)* — explicit slug override for cases where title-derived slugs would lose meaningful suffixes.
- **`devtrail approve --quiet`** *(F5, PR #88)* — suppresses the per-document success message, the F4 idempotent-skip message, and the `review_required:false` info-warning. Useful for bulk approve runs. **Does NOT silence the high-risk warning** (see below) — bulk-approving high-risk docs without seeing it is exactly the failure mode `--quiet` would otherwise enable.

### Changed (CLI)

- **`devtrail charter new --from-ailog`** *(F2, PR #87)* — now auto-extracts the first 1-2 sentences of the referenced AILOG's `## Summary` (or `## Context`) section and injects them in the body's Origin line, replacing the `[Add 1-line context]` placeholder. Falls back gracefully when the AILOG is not found, has no extractable section, or yields empty. Strips inline markdown markup (bold/italic) but preserves code spans. Caps at 240 chars with ellipsis. Sentinel CHARTER-02..05 evidence: the placeholder was rarely filled in by adopters.
- **`devtrail approve`** *(F5, PR #88)* — emits a stderr `WARNING: <id> has risk_level: <level> — ensure thorough human review` when the document being approved has `risk_level: high` or `critical`. Defense-in-depth, not enforcement — approval still proceeds. The warning is **always-on** even under `--quiet`.
- **`devtrail charter close --from-template --non-interactive`** *(F7, PR #89)* — output now differentiates first-run (template just dropped, telemetry didn't exist before) from subsequent-run (telemetry exists, schema validation passed). First-run prints "Telemetry template created — edit the YAML to fill in… then re-run"; subsequent-run prints "Telemetry schema validation passed. Charter close finalized." Pre-fix, both cases printed the same "next: edit the telemetry YAML and re-run" message regardless of state.

### Documentation

- **CLI-REFERENCE.md** (EN canonical) gains a full **`### devtrail charter audit`** section with the 3-step flow, the layout produced under `audit/charters/<CHARTER-ID>/`, the heterogeneity-recommendation note, and a worked example transcript across the three steps. Plus updates to the validate / approve / charter close sections for the new flags and behaviors. README EN/ES/zh-CN command tables list `audit` as a charter subcommand.

### Tests

411/411 → 411 (no test count change; existing infra catches the new
behavior via parameterized expansions). Specifically:
- 5 unit + 7 integration tests for `charter audit` (PR #86) covering the 3-step flow, schema validation, mutually-exclusive flags, error paths.
- 8 unit + 2 integration tests for F2 (PR #87): Summary precedence, Context fallback, no-section case, 240-char truncation, markdown stripping, leading sentences, end-to-end backfill, graceful fallback when AILOG missing.
- 6 integration tests for F5 (PR #88): high-risk warning, critical-risk warning, no warning on low/medium, --quiet preserves WARNING, --quiet suppresses idempotent-skip, --quiet suppresses review_required:false info.
- 3 integration tests for F7 (PR #89): first-run guidance, subsequent-run "finalized", invalid-yaml-fails-clearly.

### What's NOT in this release

- HTTP API clients for OpenAI / Google / Anthropic (Phase 3 v1 if a real adopter requires).
- Inter-family heterogeneity automatic enforcement (recommendation documented; auto-detection deferred to v1).
- O3 (`INFO: 0 paths suppressed` always-on log on drift) — pending design discussion.

---

## Framework 4.6.2 / CLI 3.7.2 — Phase 2 patches part 2 (F1, F8 + wildcard glob from issue #81 update)

Second round of patches surfaced by Sentinel CHARTER-02..05 telemetry
([issue #81 update](https://github.com/StrangeDaysTech/devtrail/issues/81#issuecomment-update)).
After fw-4.6.1 / cli-3.7.1 fixed F3 / F4 / F6, executing four more
Charters in Sentinel re-prioritized the remaining frictions: **F1
(slug truncation) reproduced 4/4 times consecutively** — was UX
polish in the original report, now the most consistently reproducing
friction in the CLI. **F8 (closed_at)** required manual workaround on
every Charter close (4× consecutive). **Wildcard glob in drift script**
was a new finding from CHARTER-04 that any future bulk Charter would
hit. Plus **observation O1** validated empirically as a feature
(governance paths always-in-scope correctly suppressed governance
noise without hiding a stray `git add -A` of project files).

**Compatibility.** No breaking changes. New behavior is additive
(`--slug` flag, automatic `closed_at` writing, glob resolution in drift).

### Fixed (CLI)

- **F1 — `devtrail charter new` mid-word slug truncation.** A title that
  overflows the 50-char slug limit by 1-2 chars used to produce a partial
  word fragment (Sentinel CHARTER-04: title ending in `... true` →
  filename `…-required-t.md`). Two changes:
  - `slugify` now backs up to the last `-` boundary at-or-before the
    limit and drops any partial token, never producing a mid-word cut.
    Conservative: when the next char in the original is a `-` (or
    end-of-string), the truncated view is already at a complete
    boundary and is kept verbatim.
  - New `--slug <value>` flag lets the operator override the
    title-derived slug entirely (Sentinel CHARTER-05: title with a
    meaningful `… Plan 04 F3` suffix that otherwise gets lost). The
    override is normalized through the same slugifier so it cannot
    smuggle in characters that break the filename.

- **F8 — `devtrail charter close` did not auto-write `closed_at`.** Per
  Sentinel CHARTER-02..05 telemetry, the field had to be added manually
  4× consecutively. The CLI now writes `closed_at: <today>` to the
  frontmatter alongside the `status: closed` bump. If the Charter
  already had a `closed_at` (e.g., a prior close that was reverted),
  the value is refreshed to today rather than left stale.

### Fixed (Framework)

- **Drift script wildcard glob resolution.** The bash drift script
  already supported the historical `prefix...suffix` ellipsis
  wildcard. Now it also resolves the more conventional
  `prefix*suffix` glob form: `*` is converted to `.*` for the regex
  match, and the same logic applies in both directions (declared
  glob suppresses "declared but not modified" when at least one
  matching file was modified, and suppresses "modified but not
  declared" when a modified path matches a declared glob).
  Sentinel CHARTER-04 declared `AILOG-*.md` for a parameterized bulk
  set; pre-fix the script extracted the literal string and reported
  spurious drift.

### Documented (Framework)

- **O1 ("always in scope" rule for governance paths) is a designed
  feature, not a bug.** Empirically validated in Sentinel CHARTER-04:
  a stray `git add -A` staged unrelated user-untracked files
  (`.claude/skills/`, `cmd/sentinel/sentinel`); the rule correctly
  suppressed governance noise without hiding the genuine project-file
  expansion. CLI-REFERENCE.md `devtrail charter drift` section now
  has explicit subsections for "Wildcard support" and "Designed:
  governance paths are always in scope", with the empirical citation
  to issue #81 W2. A `--strict-scope` flag that disables the rule
  remains on the table for cli-3.8.0 if a real adopter reports the
  asymmetry as friction.

### Tests

- 4 new unit tests for the F1 slug helper (mid-word truncation
  reproduction, hard-cut fallback for hyphenless slugs, trailing-`-`
  trim, helper purity).
- 4 new integration tests for `devtrail charter new` (word-boundary
  truncation end-to-end, `--slug` override, `--slug` normalized
  through slugifier, empty `--slug` falls back to title).
- 2 new integration tests for F8 (`closed_at` auto-write when
  absent, refresh of stale `closed_at` to today).
- 1 new integration test for the drift wildcard glob (real git repo,
  Charter declares `component-*.rs`, all matching modified files
  satisfy the glob).
- All 16/16 test groups pass (393 individual tests, 11 new on top of
  the 382 carried forward from cli-3.7.1).

### What's NOT in this release

- F2 (AILOG context backfill in Origin line) — bundled for cli-3.8.0.
- F5 (high-risk approve warning + verbose) — bundled for cli-3.8.0.
- F7 (charter close output differentiation first-run vs revalidation)
  — bundled for cli-3.8.0.
- O3 (`INFO: 0 paths suppressed` always-on log) — pending design discussion.
- Phase 3 (multi-model external audit) — separately scoped.

---

## Framework 4.6.1 / CLI 3.7.1 — Phase 2 patches (F3, F4, F6 from issue #81)

Empirical validation of fw-4.6.0 / cli-3.7.0 in Sentinel CHARTER-02
([issue #81](https://github.com/StrangeDaysTech/devtrail/issues/81))
surfaced 8 reproducible frictions. This release fixes the three that the
report classified as **real bugs** (Medium severity, in commands the docs
imply are clean). UX polish items (F1, F2, F5, F7, F8) and design
discussions (O1, O3) are deferred to cli-3.8.0 and later.

**Compatibility.** No breaking changes. Existing Charters, AILOGs, AIDECs,
and approved documents continue to work unchanged. The only behavior change
adopters need to be aware of is **F4** — `devtrail approve` no longer
silently re-applies; pass `--force` for the legitimate cycles
(`revisions_requested → approved` iteration, multi-reviewer hand-off).

### Fixed (Framework)

- **F3 — `check-charter-drift.sh` regex too greedy.** The bash drift script
  used to extract backtick-quoted paths from **any column** of the `## Files
  to modify` table, including textual references in the "Change" column
  (e.g., "follows the pattern of `docs/plans/README.md`"). Such references
  were parsed as declared deliverables and produced false-positive omission
  warnings. The fix tightens the awk pre-processor to extract only column 1
  of markdown table rows, while preserving backward compatibility with
  bullet-list `## Files to modify` sections (non-table content is still
  passed through). Header and separator rows are also filtered explicitly.
  Script syntax remains POSIX bash; no new dependencies. Reproduces from a
  clean Charter — see new test
  `charter_drift_ignores_path_references_in_change_column`.

### Fixed (CLI)

- **F4 — `devtrail approve` silent re-application.** Re-running `approve`
  on a document that already had `reviewed_by`/`reviewed_at`/`review_outcome`
  in frontmatter and a `## Approval` section in the body used to silently
  overwrite the frontmatter and append a duplicate body block (resulting in
  two blocks with potentially conflicting `--notes`). The fix detects
  existing approval state and, by default, performs an **idempotent skip**
  with an informative message naming the existing reviewer and date. The
  new `--force` flag is the explicit gate for the legitimate cycles:
  - `revisions_requested → approved` iteration (same reviewer amending),
  - multi-reviewer hand-off (different reviewer adding to history).

  Implementation: `Frontmatter.review_outcome.is_some() && !force` ⇒ exit
  Ok(0) without touching the document. Existing single-approval and
  multi-reviewer flows are unchanged with `--force`.

- **F6 — `devtrail charter close` did not sync body mirror line.** The
  command bumped `frontmatter status: declared|in-progress` to `closed`
  correctly, but left the body's prose mirror line
  (`> **Status (mirrored from frontmatter — source of truth is above):** declared.`)
  untouched. The template's own promise that the body line mirrors
  frontmatter was broken by the CLI. The fix adds a body-sync step
  immediately after the frontmatter mutation: a string-anchored matcher
  finds lines containing `mirrored from frontmatter` (EN) or `espejado del
  frontmatter` (ES), locates the `):**` marker, and replaces the status
  word with `closed`. Lines without the canonical anchor are left alone
  (preserves user-added `**Status:**` markers in body). Best-effort: if
  the line shape is corrupted, the function is a no-op rather than
  guessing.

### Tests

- 6 new unit tests for `sync_body_status_mirror`: EN form, ES form,
  in-progress→closed transition with hyphen, leaves unrelated `**Status:**`
  lines alone, no-op when anchor absent, preserves corrupted lines.
- 1 new integration test for F3 (`charter_drift_ignores_path_references_in_change_column`)
  exercising a real git repo where the Change column contains a
  backtick-quoted path.
- 1 new integration test for F6 (`charter_close_syncs_body_status_mirror_line`)
  asserting both frontmatter and body are updated atomically.
- 2 new integration tests for F4: idempotent skip without `--force`,
  full multi-reviewer flow with `--force`.
- The previous test `approve_replaces_existing_approval_fields` was
  updated and split into the two F4 cases above (the old behavior it
  pinned is now a bug under the new contract).

**388/388 tests pass** (6 new + 382 carried forward).

### What's NOT in this release

- F1 (slug truncation), F2 (AILOG context backfill), F5 (high-risk approve
  warning), F7 (close output differentiation), F8 (auto `closed_at`) —
  bundled for cli-3.8.0.
- O1 (`--strict-scope` flag for the "always in scope" rule), O3
  (`INFO: 0 paths suppressed` always-on log) — pending design discussion.
- Phase 3 (multi-model external audit) — separately scoped.

---

## Framework 4.6.0 / CLI 3.7.0 — Phase 2: telemetry, drift, approval workflow

The first feature-bearing release since the repositioning (fw-4.5.x). Phase 2 of `Propuesta/devtrail-cli-roadmap.md` lands as 7 bisect-safe PRs (#73–#79), grouped here for reviewers. The release closes the empirical loop the Sentinel experiment opened: telemetry at Charter close, drift detection at Charter close, and a canonical approval signal for `review_required: true` documents (resolving issue #67).

**Compatibility.** No breaking changes. Existing Charters, AILOGs, AIDECs, and templates remain valid. New CLI commands and framework artifacts are additive; new template fields ship as commented YAML so existing documents continue to validate. Adopters pick up the changes via `devtrail update-framework` + `devtrail update-cli`.

### Added (Framework)

- **`dist/.devtrail/schemas/charter-telemetry.schema.v0.json`** — JSON Schema Draft 2020-12 for the post-execution telemetry recorded at Charter close. Derived from `Propuesta/devtrail-charter-telemetry.md` v0.3 with the 4 fields refined by Sentinel: `external_audit` as array (dual-audit calibration), `outcome.scope_change_notes` with F1...FN encoding, `agent_quality.r_n_plus_one_emergent_count`, `qualitative.format_iteration`. Marked **experimental v0** — same N=1-domain caveat as `charter.schema.v0.json`. (PR #73)
- **`dist/.devtrail/templates/charter-telemetry-template.yaml`** — commented YAML skeleton mirroring the schema. Used by `devtrail charter close --from-template` as the starting point for manual edits. (PR #73)
- **`dist/.devtrail/scripts/check-charter-drift.sh`** — bash script (~165 lines) ported from Sentinel `scripts/check-plan-drift.sh`. Detects declared-but-not-modified files and undocumented scope expansion. Validated empirically with zero false positives across PLAN-05 retrospective + PLAN-06 prospective in Sentinel. Path surface adapted (`docs/plans/` → `docs/charters/`) and section heading detection extended to EN/ES/zh-CN. (PR #73)
- **`dist/.devtrail/hooks/pre-pr.sh`** — opt-in pre-push hook that runs `devtrail charter drift` automatically on Charters with `status: in-progress`. Per principle #6 (cognitive discipline > raw productivity), the hook is virtuous when consented to and never installed by default. Manual install: `cp .devtrail/hooks/pre-pr.sh .git/hooks/pre-push`. CLI flag: `devtrail init --hooks`. (PR #79)
- **Approval workflow canonization** (PR #76, closes issue #67): 3 optional frontmatter fields (`reviewed_by`, `reviewed_at`, `review_outcome`) added to all 11 templates that need formal approval (AIDEC, ETH, MCARD, ADR, DPIA, INC, SEC + China: PIPIA, CACFILE, TC260RA, AILABEL) across EN, ES, and zh-CN — **33 template files**. Fields ship as commented YAML so existing documents continue to validate. The presence of `review_outcome` is the canonical "human has reviewed" signal; `review_required: true` remains as historical record after approval (it's not toggled to `false`).

### Changed (Framework)

- **`dist/.devtrail/00-governance/DOCUMENTATION-POLICY.md`** (EN + ES + zh-CN):
  - §2 Optional Fields extended with `reviewed_by`, `reviewed_at`, `review_outcome`.
  - New §3.5 "Recording Approval" section explaining closure semantics, body section format (compatible with existing `## Approval` tables in 7 templates), the multi-reviewer convention for v1 (chronological body blocks; structured array deferred), and the CLI tooling.

### Added (CLI)

- **`devtrail charter close <CHARTER-ID>`** (PR #74) — record post-execution telemetry and bump status to `closed`. Two modes:
  - **Interactive** (default): walks the schema field by field — trigger, effort, agent quality, outcome, qualitative — rendering YAML directly so the output is stable, comment-free, and validated against the schema before disk write. Target time: 5–10 min.
  - **`--from-template [--non-interactive]`**: copies the YAML skeleton next to the Charter for manual editing (CI / scripted use). Pre-fills `charter_id`, title, and `closed_at`. Idempotent.
  - Telemetry storage: `.devtrail/charters/CHARTER-NN.telemetry.yaml` (lateral file, not embedded in Charter frontmatter — per roadmap §A2: frontmatter is declarative ex-ante, telemetry is voluminous ex-post).
- **`devtrail charter drift <CHARTER-ID>`** (PR #75) — wraps `check-charter-drift.sh` with **AILOG-awareness**: paths reported as "declared but not modified" are silenced when they appear in the `## Risk` / `## Riesgos` / `## 风险` section of any AILOG referenced by the Charter's `originating_ailogs`. This is mitigation R2 of the Sentinel experiment — friction was virtuous when emitting cross-agent signal, ceremony when alerting on already-documented risks. Flags: `--range REV..REV` (default `HEAD~1..HEAD`), `--no-ailog-suppress` (disable suppression), `--path DIR`. Bash delegation only; pure-Rust fallback for Windows-without-bash deferred until requested.
- **`devtrail approve <doc-id>`** (PR #77) — record a formal human approval. Writes the three approval frontmatter fields and appends the canonical `## Approval` body section in one atomic edit. Flag-driven for CI (`--outcome <approved|revisions_requested|rejected> --reviewer <id> [--at YYYY-MM-DD] [--notes "..."]`); falls back to interactive prompts on TTY when flags are absent. Resolves any DocType by canonical prefix, supports re-approval (latest-wins in frontmatter, both blocks preserved chronologically in body for the multi-reviewer convention).
- **`devtrail validate --check-pending-reviews [--max-pending-days N]`** (PR #78) — surfaces documents with `review_required: true` and no `review_outcome` older than the threshold. **Warn-only** (never errors): per principle #6, useful for CI dashboards of the approval backlog without blocking unrelated PRs. Default threshold: 14 days.
- **`devtrail init --hooks`** (PR #79) — copies `.devtrail/hooks/pre-pr.sh` to `.git/hooks/pre-push` after init. Refuses to overwrite existing hooks; skips silently if not a git repo.

### Changed (CLI)

- New shared module `cli/src/prompts.rs` — interactive helpers (string, u32, bool, enum, comma-separated array, multiline) with `require_interactive()` guard, used by `charter close` and `approve`.
- New `cli/src/telemetry_schema.rs` — JSON Schema validator for telemetry YAML, mirroring `charter_schema.rs`.
- New `cli/src/charter_schema.rs::yaml_to_json_value` — exposed as `pub` (renamed from private `yaml_to_json`) so `telemetry_schema` and other future schema validators can reuse the conversion without duplication.
- `cli/src/document.rs::Frontmatter` — added `reviewed_by`, `reviewed_at`, `review_outcome` as `Option<String>` so `validate --check-pending-reviews` can read them.

### Notes

- **No schema changes to `charter.schema.v0.json`** — Phase 2 is additive in the Charter ecosystem, not breaking.
- **Empirical validation gate** — Sentinel's PLAN-05 + PLAN-06 fixtures (zero false positives) are the contractual reference for `charter drift`. The integration tests in PR #75 reproduce the equivalent shape against a real git repo. The Sentinel-side telemetry artifacts (5 PLAN-NN.telemetry.yaml files) are the cross-validation set for the schema crystallized here.
- **i18n parity** — DOCUMENTATION-POLICY §3.5 ships in all three languages this release; CLI-REFERENCE EN gets the new command sections in this release, ES + zh-CN command-section translations are deferred to fw-4.6.x (the EN canonical surface advances first, ES + zh-CN translations of the command tables stay at the current command list with a brief note).

### Test plan summary

**382/382 tests pass** across 15 test groups: 4 hook-install unit tests, 6 approve integration tests, 4 drift integration tests, 5 charter-close integration tests, 4 pending-review integration tests, plus the existing 359 covering the rest of the CLI. Manual smoke of the interactive `charter close` flow remains an open item before tag.

---

## Framework 4.5.1 — i18n catch-up + ADOPTION-GUIDE reframe (ES + zh-CN follow up to fw-4.5.0)

Completes the repositioning shipped in `fw-4.5.0` for the Spanish and Simplified Chinese surfaces, and reframes `docs/adopters/ADOPTION-GUIDE.md` (English) — which had been overlooked in `fw-4.5.0` and was still leading with the *"ISO 42001-aligned AI governance platform"* framing. After this release, the canonical engineering-discipline-first positioning is consistent across all three languages.

### Changed (Framework)

- **`docs/adopters/ADOPTION-GUIDE.md`** (EN): same reframe pattern as `README.md` and `DEVTRAIL.md` in fw-4.5.0 — opening *What is DevTrail?*, *Why Now?*, *Who is it for?*, *Benefits*, *Standards Compliance* sections rewritten engineering-first; *What DevTrail is NOT* extended; *Benefits* table reordered (engineering discipline first, AI-assisted development second, regulatory compliance third); a new *Primary user* paragraph mirrors the README persona section.
- **`docs/i18n/es/README.md`**: full ES translation of the fw-4.5.0 EN reframe — new headline (*"La disciplina cognitiva que tus proyectos asistidos por IA necesitan"*), reframed *El Problema* / *La Solución*, new *¿Para quién es DevTrail?*, *Principios de Diseño*, *Límites Honestos*, and *Compliance* sections; the previously redundant *Alineación con Estándares* trailer block is removed (now lives inside *Compliance*); a brief *Cobertura regulatoria de China* table is added with a link to the zh-CN README for adopters operating in mainland China; final tagline updated.
- **`docs/i18n/es/adopters/ADOPTION-GUIDE.md`**: ES translation of the EN ADOPTION-GUIDE reframe.
- **`dist/.devtrail/00-governance/i18n/es/DOCUMENTATION-POLICY.md`**: opening *Marco de Gobernanza* section replaced with *Por qué existe esta política* (mirror of the EN reframe).
- **`docs/i18n/zh-CN/README.md`**: full zh-CN translation of the fw-4.5.0 EN reframe — new headline (*"你的 AI 辅助项目所需的认知纪律"*), reframed *问题* / *解决方案*, new *DevTrail 的适用人群*, *设计原则*, *诚实的边界*, and *合规性* sections (with the existing detailed Chinese opt-in subsection now nested under *合规性*); final tagline updated.
- **`docs/i18n/zh-CN/adopters/ADOPTION-GUIDE.md`**: zh-CN translation of the EN ADOPTION-GUIDE reframe.
- **`dist/.devtrail/00-governance/i18n/zh-CN/DOCUMENTATION-POLICY.md`**: opening *治理框架* section replaced with *本策略为何存在* (mirror of the EN reframe).
- **Version footers / examples bumped to `v4.5.1` / `fw-4.5.1`** across the EN canonical surface and i18n: `dist/dist-manifest.yml`; `dist/.devtrail/QUICK-REFERENCE.md`; `dist/.devtrail/00-governance/{QUICK-REFERENCE,AGENT-RULES,DOCUMENTATION-POLICY,C4-DIAGRAM-GUIDE}.md` (EN); `dist/.devtrail/00-governance/i18n/{es,zh-CN}/{QUICK-REFERENCE,AGENT-RULES,DOCUMENTATION-POLICY,C4-DIAGRAM-GUIDE}.md` (8 files); `README.md`, `docs/i18n/es/README.md`, `docs/i18n/zh-CN/README.md` (3 versioning tables); `docs/adopters/CLI-REFERENCE.md` and the ES + zh-CN counterparts (3 files, 7 example outputs each); `docs/adopters/ADOPTION-GUIDE.md` and the ES + zh-CN counterparts.

### Notes

- **No schema changes, no template changes, no CLI behavior changes.** This release is documentation-and-positioning only; adopters who pick it up via `devtrail update-framework` get the new EN, ES, and zh-CN governance docs and the new `dist-manifest.yml` version stamp.
- **i18n parity restored.** After this release, `README.md`, `DEVTRAIL.md`, `DOCUMENTATION-POLICY.md`, and `ADOPTION-GUIDE.md` carry the same engineering-discipline-first framing in EN, ES, and zh-CN. Operational governance files (`AGENT-RULES.md`, `QUICK-REFERENCE.md`, `C4-DIAGRAM-GUIDE.md`) had no positioning content to reframe — only their version footers were bumped.
- **GitHub repository description and topics updated** to match the new positioning (one-line description and topic list refreshed via `gh repo edit`).

---

## Framework 4.5.0 — repositioning: engineering-discipline-first, compliance as side effect (EN canonical docs)

This release does not add features; it realigns the canonical English-language positioning to match how DevTrail is actually used and the explicit hierarchy of `Propuesta/devtrail-design-principles.md`. The previous framing led with compliance ("AI Governance Platform for Responsible Software Development") which inverted Principle #4 — *regulatory compliance is a side effect, not the product* — and Principle #2 — *the primary user is the senior engineer orchestrating agents, not the compliance officer*. This release restates the product in those terms.

Scope of this release is **EN canonical only**. Spanish and Simplified Chinese translations are deferred to a follow-up release (`fw-4.5.x`) so the EN positioning ships in a focused, reviewable PR; until then, ES and zh-CN docs continue to reflect the prior framing.

### Changed (Framework)

- **`README.md`** (EN) rewritten:
  - New headline: *"The cognitive discipline your AI-assisted projects need"* (was: *"AI Governance Platform for Responsible Software Development"*).
  - **`## The Problem`** reframed around how AI agents lose coherence over many turns and accumulate hidden technical debt — not around regulatory pressure.
  - **`## The Solution`** reframed as a *framework + CLI that externalizes the cognitive discipline of senior software engineering work* into versioned files alongside the code. Compliance is presented as the side effect when the discipline is real.
  - **New `## Who is DevTrail for`** persona section: primary user is the senior engineer orchestrating agents; tech leads, compliance officers, and adopters in regulated environments are explicit secondary audiences (never at the primary user's expense). Anti-positioning bullets enumerate what DevTrail is *not* trying to be.
  - **New `## Design Principles`** section: the 12 principles summarized one-line each, with a link to the full polished document at `Propuesta/devtrail-design-principles.md` for the empirically-annotated v0.2.2 version.
  - **New `## Honest Limits`** top-level section operationalizing Principle #10 (*honesty about what the tool does not do*).
  - **`## Compliance`** is now a single dedicated section (the previous *Standards Alignment* + *China Regulatory Compliance* fragments are unified) with an opening paragraph that frames compliance as a *consequence of doing the engineering work well*, not as the product.
  - The Features subsection *Compliance Automation* is renamed to *CLI Tooling* and now leads with `devtrail charter` as the unit of agent execution.
- **`dist/DEVTRAIL.md`**: opening *Governance Context* section replaced with *Why these rules exist* — leads with externalizing senior-engineering cognitive discipline; lists the regulatory frameworks as evidence the artifacts align with, not as the goal. The *Fundamental Principle* is broadened from *"No significant change without a documented trace"* to *"No significant change without a documented trace — and a constrained decision space for the agent."*
- **`dist/.devtrail/00-governance/DOCUMENTATION-POLICY.md`**: opening *Governance Framework* section replaced with *Why this policy exists*. Same engineering-first framing as `DEVTRAIL.md`. The standards list is preserved verbatim and remains authoritative; only the positioning around it changes.
- **`cli/Cargo.toml`** description updated from *"CLI tool for DevTrail - Documentation Governance for AI-Assisted Development"* to *"CLI for DevTrail — the cognitive discipline your AI-assisted projects need"* (visible on crates.io).
- **`Propuesta/devtrail-design-principles.md`** polished (v0.2.1 → v0.2.2): internal Sentinel-specific references generalized for public readability (PR #70). Now linked publicly from `README.md` and `DEVTRAIL.md`.

### Notes

- **No schema changes.** No template changes. No CLI behavior changes. Adopters who pick up fw-4.5.0 via `devtrail update-framework` get the new English `DEVTRAIL.md` + `00-governance/` files; their project-level `.devtrail/config.yml` and existing documents are unaffected.
- **i18n parity gap is intentional and time-bounded.** The Spanish and Simplified Chinese governance docs (`i18n/es/`, `i18n/zh-CN/`) and the `docs/i18n/es/`, `docs/i18n/zh-CN/` README + CLI-REFERENCE retain their fw-4.4.2 language and version footer until the i18n catch-up release. Their `*DevTrail v4.4.2*` footers are correct for their content; only the EN canonical surface advances to v4.5.0 in this release.
- **Why minor and not patch.** This release changes the *meaning* the canonical docs project to readers — the headline of the README, the opening of `DEVTRAIL.md`, the opening of `DOCUMENTATION-POLICY.md`. By Keep a Changelog conventions and DevTrail's own semver discipline (CLAUDE.md), repositioning of canonical surface is *changed* (minor), not *fixed* (patch). Schema and template stability is unaffected.

---

## CLI 3.6.1 — `devtrail charter new` "Next steps" output renumbers correctly when origin is set

### Fixed (CLI)

- `devtrail charter new` "Next steps" output skipped from step `2.` to step `4.` when `--from-ailog` or `--from-spec` was passed. The conditional origin-step (step 3 in the no-origin path) was suppressed correctly, but the remaining steps had hardcoded numbers (`println!("    1. ...")`, `println!("    2. ...")`, `println!("    4. ...")`) that did not re-sequence. Adopters following `--from-ailog` or `--from-spec` saw a numbering gap that hinted at a missing step. Reported as F1 of `AILOG-2026-05-02-028` in Sentinel during the first end-to-end execution of CHARTER-01 (format v4 atomic Charter closure pattern).
- The fix extracts the step list into a pure function `next_steps(from_ailog, from_spec) -> Vec<String>` that builds the steps as data and applies dynamic numbering via `enumerate()`. Four unit tests verify the behavior across all three origin paths and guard against regression to the hardcoded form.

### Notes

- Editorial / cosmetic fix only. No behavior change beyond the printed output. No schema changes. Existing Charters created with cli-3.6.0 are unaffected.
- This is the first CLI patch driven directly by an empirical Charter-execution finding (Sentinel `AILOG-2026-05-02-028` §cli-3.6.0 frictions encountered §F1). It validates the disposition rule established by `CHARTER-01` of Sentinel: bug-class frictions with obvious fix paths flow upstream as patches without ceremony, while observation-class frictions accumulate in the AILOG until 3+ patterns emerge.

---

## Framework 4.4.2 — atomic Charter closure pattern (format v4)

First Charter-driven release. Originating Charter: `sentinel/docs/charters/01-format-v4-atomic-charter-closure-pattern.md` (Sentinel repo). Originating decision: AIDEC-2026-05-02-001 (Sentinel `.devtrail/07-ai-audit/decisions/`), which formalized the canon gap discovered via PLAN-07 of Sentinel (closed 2026-05-02): the step "update the Plan-doc post-merge if the AILOG documented divergencias" was mentioned in TEMPLATE.md and `docs/plans/README.md §Cómo cerrar un plan` but had no systematic trigger — implementer relied on memory, drift remediation lagged the main PR by days when memory failed.

This release ports the Sentinel-internal fix upstream as **format v4** of the Charter template. Editorial only — no schema changes, no breaking changes, adopters do not need to migrate existing Charters.

### Changed (Framework)

- **`dist/.devtrail/templates/charter-template.md`** (EN) gains five updates:
  1. **Step 1 of `## Charter Closure` becomes "Atomic update (format v4)"**: if the drift check (Tasks #7) reported drift not already in the AILOG, edit `## Files to modify` and/or add a `## Closing notes` block in the **same commit/PR**, before submitting. No housekeeping PR deferral.
  2. **New top-level `## Closing notes` section** near the end of the body (not inside the HTML comment), with template structure and references to PLAN-05 and PLAN-07 of Sentinel as historical examples. Designed to be omitted entirely when no drift was detected — empty `## Closing notes` is noise.
  3. **Convention #5** in the comment block rewritten to require atomic update at Tasks #7 time, not post-merge housekeeping. Cites PLAN-07 as the empirical case that demonstrated the failure mode and AIDEC-2026-05-02-001 as the formalization.
  4. **Trigger placeholder** broadened: `[1-line: what observable signal...]` → `[1-line: what concrete signal — observable event, declared decision, metric threshold, or infrastructure milestone — justifies executing this Charter now]`. Surfaced during the originating Charter review — Charters derived from AIDECs/ADRs have declarative triggers, not observable ones, and the previous wording pushed authors to invent pseudo-observable signals.
  5. **`## Risks` placeholder** gains a guidance comment: "Each mitigation should specify: (a) concrete trigger or threshold, (b) action committed, (c) what happens if the mitigation itself fails, (d) where follow-up insights are captured." Surfaced during R4 review of the originating Charter — without guidance, authors leave these four properties implicit, which a human reviewer catches but a solo implementer does not.
- **`dist/.devtrail/templates/i18n/es/charter-template.md`** (ES) mirrors the same five changes in Spanish.
- **`Propuesta/que-es-un-charter.md`** gains a new §1.3 "Atomic Charter closure (format v4)" describing the pattern and its empirical origin.

### Also (governance hygiene)

- **Promotes 4 files from `Propuesta/` to public git history**: `que-es-un-charter.md`, `devtrail-cli-roadmap.md`, `devtrail-thesis-validation.md`, `devtrail-design-principles.md`. These were referenced from shipped docs (CHANGELOG, CLI-REFERENCE EN/ES/zh-CN) since fw-4.4.0 but had never been committed — adopters following the links got 404. Discovered during the originating Charter's execution. The other 3 `Propuesta/` files (`devtrail-charter-telemetry.md`, `devtrail-cloud-proposal.md`, `devtrail-studio-vision.md`) remain local-only because no shipped doc references them.

### Notes

- This release ships **no breaking changes**. Existing Charters created with fw-4.4.0 / fw-4.4.1 remain valid. Adopters pick up format v4 via `devtrail update-framework`; the new step in `## Charter Closure` and the new `## Closing notes` section apply to Charters created from this template version forward.
- Format v4 is **editorial only** — no `charter.schema.v0.json` changes. Items deferred to schema v0.x evolutions (e.g., `status: blocked` enum, `trigger_kind` field) await empirical data from 3+ real Charters before being designed; the format v4 Charter §Out of scope captures the deferral explicitly.
- Cross-repo Charter pattern (this Charter being executed in Sentinel while implementing changes in the DevTrail repo) is **not codified** in the template — it was a coyuntural arrangement for empirical Phase 1 validation, not a pattern to inherit to adopters. Most adopters keep code and governance in the same repo. The asymmetry is documented in the originating Charter's R1/R4 as a historical note, not a promoted variant.

---

## Framework 4.4.1 — `docs-validation.yml` workflow recognizes all DocTypes and recent governance files

### Fixed (Framework)

- `dist/.github/workflows/docs-validation.yml` had three pre-existing divergences that surfaced once Charters Phase 1 made adopters revisit the framework:
  - **`VALID_PATTERN` regex** missed the four China-specific DocTypes (`PIPIA`, `CACFILE`, `TC260RA`, `AILABEL`) that shipped earlier and incorrectly listed two stale prefixes (`OPS`, `DOC`) that aren't real DocType variants. A China-region adopter creating `PIPIA-2026-01-15-001-foo.md` would have the workflow reject it as "Invalid naming".
  - **`TYPES` array in the `governance-metrics` job** was a 12-element list missing the same four China DocTypes. The metrics summary on `$GITHUB_STEP_SUMMARY` always reported zero PIPIA/CACFILE/TC260RA/AILABEL counts, even when those documents existed.
  - **`EXCLUDED` regex** (referenced in 5 places) listed only 8 framework files but the framework now ships ~18 additional governance / reference files at the root of `00-governance/` and `03-implementation/` (C4-DIAGRAM-GUIDE, MANAGEMENT-REVIEW-TEMPLATE, AI-KPIS, AI-LIFECYCLE-TRACKER, AI-RISK-CATALOG, AI-GOVERNANCE-POLICY, CHINA-REGULATORY-FRAMEWORK, CAC-FILING-GUIDE, GB-45438-LABELING-GUIDE, PIPL-PIPIA-GUIDE, ISO-25010-2023-REFERENCE, OBSERVABILITY-GUIDE, TC260-IMPLEMENTATION-GUIDE, the four NIST-AI-RMF guides). When an adopter ran `devtrail update-framework` on a tracked checkout, the workflow flagged each of these as "Invalid naming" and the job failed.

### Changed (Framework)

- The workflow now uses a **whitelist approach** instead of the blacklist. A new top-level `env: DOC_TYPE_PREFIXES` enumerates the 16 canonical DocType prefixes (in sync with `cli/src/document.rs::DocType::ALL_PREFIXES`); per-step checks skip files whose basename does not start with one of those prefixes. Framework / governance / template files are silently skipped with no manual exclude list to maintain. The `governance-metrics` job derives its `TYPES` array from the same env var.
- `cli/src/document.rs::ALL_PREFIXES` gains a doc-comment pointing at the workflow's env var, and the workflow's env var has a comment pointing back. Adding a new DocType still requires updating both sides, but the bidirectional pointer makes the obligation discoverable from either entry point.

### Notes

- This is a Framework-only patch. CLI behavior is unchanged; `cli-3.6.0` is unaffected. Existing adopters can pick up the fix with `devtrail update-framework`.
- Charter validation in CI is intentionally still out of scope for this workflow — the `--include-charters` flag on `devtrail validate` is opt-in and the workflow's path filter (`.devtrail/**`) does not cover `docs/charters/`. Adding Charter validation to CI is queued for cli-3.7.0 along with `--include-charters --staged` integration.

---

## Framework 4.4.0 / CLI 3.6.0 — Charters as a first-class entity

The first user-visible step of the post-Sentinel roadmap (`Propuesta/devtrail-cli-roadmap.md` Fase 1). Crystallizes the Charter pattern — bounded, auditable units of work declared ex-ante and validated ex-post — that emerged from the 6-cycle Sentinel `/plan-audit` experiment. The artifact was historically called "Plan" in Sentinel; renamed to **Charter** to disambiguate from GitHub SpecKit's `plan.md`. Sentinel's historical files preserve "Plan"; everything DevTrail ships from this release on uses "Charter".

### Added (Framework)

- **Charter template** at `dist/.devtrail/templates/charter-template.md` (EN + ES, ports Sentinel's `TEMPLATE.md v3` with the 6 validated format conventions: Local/Production verification split, time-based effort, structured sub-sections, R<N+1> emergent risks, Charter Closure section, auto-checklist drift). Localized parallel under `templates/i18n/es/`.
- **Charter JSON Schema** at `dist/.devtrail/schemas/charter.schema.v0.json` (Draft 2020-12, marked `experimental`). Required fields: `charter_id`, `status` (`declared`/`in-progress`/`closed`), `effort_estimate` (`XS`/`S`/`M`/`L`), `trigger`. Mutually-exclusive optional fields: `originating_ailogs` array or `originating_spec` path. The `v0` suffix and additional-properties:true posture leave room for evolution; v1.0 stable requires a second-domain adopter (see `Propuesta/devtrail-thesis-validation.md` §6).
- **Two anonymized canonical examples** at `dist/docs/examples/charters/CHARTER-01-anomaly-thresholds.md` (M-effort feature) and `CHARTER-02-baseline-recompute.md` (XS-effort admin endpoint), derived from Sentinel PLAN-05 / PLAN-06 with identifiers anonymized but structural conventions preserved.

### Added (CLI)

- **`devtrail charter new`** scaffolds a Charter from the framework template into `docs/charters/NN-slug.md`. Three origin paths supported, mutually exclusive at the clap level: `--from-ailog AILOG-YYYY-MM-DD-NNN` (post-MVP / maintenance mode — the Sentinel case), `--from-spec specs/.../spec.md` (greenfield mode driven by SpecKit), or neither (Charter scaffolded without an explicit origin, to be filled in manually before status moves to `in-progress`). `--type XS|S|M|L` defaults to `M`. Sequential numbering is project-local; concurrency on parallel branches is documented as a known v0 limitation.
- **`devtrail charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any]`** enumerates Charters as a tight table (NN, STATUS, EFFORT, ORIGIN, TITLE) with width-adaptive columns. Files that fail to parse are reported as warnings to stderr; the command lists what it can.
- **`devtrail charter status [CHARTER-ID] [--path <dir>]`** with an ID resolves the full charter_id, the `CHARTER-NN` prefix, or just the numeric NN; numeric matching is permissive across zero-padding (`10` matches both `CHARTER-10` and `CHARTER-010`). Without an ID, prints the 5 most recent Charters by NN descending. Status output flags Phase 2 features (`charter close`, `charter drift`) as not yet available.
- **`devtrail validate --include-charters`** validates `docs/charters/*.md` against the Charter schema (shape, enums, mutual exclusion of origin types) plus referential integrity (`originating_ailogs` IDs resolve to AILOG files; `originating_spec` path exists). Default `false` so projects that don't yet use Charters keep working unchanged. Schema-level errors emit hint-rich messages; missing schema emits a single warning rather than failing per-Charter. Currently honored only in the all-mode path; `--staged` integration is queued for cli-3.7.0.
- **`devtrail explore` Charters view (TUI)**: a synthetic "Charters" group is appended to the navigation tree when at least one Charter exists. Charter files render with a `CH` badge, are searchable / sortable like governance docs, and the `charter_id` resolves through `find_by_ref` so a related-link from any document can navigate to a Charter. Group label translates to `Charters` (es loanword) / `章程` (zh-CN).

### Changed (CLI)

- **`devtrail validate`** gains the `--include-charters` opt-in flag described above. No change to the existing pipeline when the flag is absent.

### Notes

- The Charter pattern is empirically validated in **a single project (Sentinel) on a single domain (Go backend)**. Per principle #12 of `Propuesta/devtrail-design-principles.md`, the schema, template, and tooling ship as `v0` / experimental. Stabilization to `v1.0` requires validation in a second domain (frontend, ML pipeline, infra-as-code) — see `Propuesta/devtrail-thesis-validation.md` §6 for the full N≈2-3 argument.
- Phase 2 of the CLI roadmap (`Propuesta/devtrail-cli-roadmap.md` §4) adds `charter close` (interactive telemetry capture at Charter cierre) and `charter drift` (file-vs-commit drift check, port of Sentinel's `check-plan-drift.sh`). Phase 3 adds `charter audit` (multi-model external audit with inter-family heterogeneity constraint).
- This release ships **no breaking changes**. Existing adopters can update via `devtrail update-cli` and `devtrail update-framework`; their existing flow remains identical until they opt into the Charter commands.

---

## CLI 3.5.3 — `devtrail update` no longer leaks package internals into adopter projects

### Fixed (CLI)
- `devtrail update` (and `devtrail update-framework`) used to copy the framework's internal `dist-manifest.yml` and `dist-templates/` directory into the root of the adopter project. Both are package-internal artifacts: the manifest is the catalogue the CLI reads from the release ZIP, and `dist-templates/` is the source of agent-directive injections that are read into memory and merged via marker blocks — neither is meant to land on disk in the target project. `devtrail init` already filtered correctly via `manifest.files`; only the update path was inconsistent. Update now applies the same whitelist, so only files declared in the release manifest are copied. Existing projects affected by the bug can clean up by deleting `dist-manifest.yml` and `dist-templates/` from their project root and running `devtrail update-framework` again to regenerate `.devtrail/.checksums.json` without orphan entries.

---

## CLI 3.5.2 — Remove Undocumented Vim-Style Aliases (`l`, `h`)

### Changed (CLI)
- `devtrail explore` no longer treats lowercase `l` as a synonym for `Enter` / `Right` (open document / expand group) and no longer treats lowercase `h` as a synonym for `Esc` / `Left` (back / collapse). These bindings were never documented in the `?` help popup nor the status bar, and `l` clashed with the language switcher key `L` introduced in cli-3.5.0 — users pressing `L` could land on `l` if Shift slipped, accidentally opening a document instead of cycling languages. The documented `j` / `k`, `g` / `G`, and `n` / `N` keys (all listed in the help popup) remain unchanged.
- "Fullscreen document mode, vim-style keybindings" is now described as "alternate `j` / `k` keys for `↓` / `↑`" in `docs/adopters/CLI-REFERENCE.md` (EN / ES / zh-CN). DevTrail no longer claims vim compatibility — only specific documented alternates.

---

## CLI 3.5.1 — Metadata Panel and Welcome Screen i18n Coverage

### Fixed (CLI)
- The Metadata panel (title `Metadata`, the empty-state `No document selected`, and field labels `Status:`, `Created:`, `Agent:`, `Confidence:`, `Risk:`, `Review:`, `Tags:`, `Related:`, plus the `(Enter: search)` / `(Enter: follow)` hints and `⚠ REQUIRED`) now switches with the active language. Field labels are padded to a consistent visual width so values stay aligned across `en` / `es` / `zh-CN`.
- The Document panel title and welcome screen (`DevTrail Explorer` brand line aside) now translate: `Documentation Governance for AI-Assisted Development`, `Total documents:`, `Quick start`, the keyboard-shortcut descriptions, `Developed by`, and the repo-root fallback notice. Brand and company names stay in their canonical form on purpose.
- Frontmatter values themselves (status, tags, related IDs, dates) are still read verbatim from each document — they're authored content, not UI strings.

---

## CLI 3.5.0 — TUI i18n Polish: Translated Shell, Live Switcher, Locale Auto-Detect

Three changes that complete the language-aware `devtrail explore` work started in 3.4.0. Together they make the TUI feel native to non-English users instead of just "translated docs inside an English shell."

### Added (CLI)
- **Translated TUI shell**: navigation tree group/subgroup labels, sort hints, status-bar key hints, notifications, and the `?` help popup all render in `es` and `zh-CN` when the active language is non-English. Untranslated strings fall back to English silently. New module `cli/src/tui/i18n_strings.rs` is the single lookup point — extending to a new locale is one entry per call site.
- **Live language switcher**: press `L` inside `devtrail explore` to cycle the display language (`en → es → zh-CN → en`) without quitting. The doc index is rebuilt on the spot, the document body cache is dropped so the next open reads the localized file, and the status bar shows a translated notification (`Idioma: es`, `语言: zh-CN`). Documented in the help popup.
- **OS locale auto-detection**: when a project has no `.devtrail/config.yml`, `devtrail explore` / `new` / `status` now read `$LC_ALL` / `$LANG` and map a POSIX locale (e.g., `zh_CN.UTF-8`, `es_MX.UTF-8`) to the nearest supported language. Existing projects with a config file are unaffected — an explicit `language: en` is still treated as a deliberate user choice and never overridden by env vars. Traditional Chinese (`zh_TW`, `zh_HK`) returns `None` because DevTrail only ships Simplified.

### Changed (CLI)
- New `DevTrailConfig::resolve_language(project_root)` is now the single entry point used by `explore`, `new`, and `status`, so all three commands agree on the effective language. Resolution order: `--lang` flag > `config.language` (when config file exists) > OS locale > `"en"`.

---

## CLI 3.4.1 — Code-Block Background No Longer Fragments on Narrow Panels

### Fixed (CLI)
- Fix the gray background of fenced code blocks in `devtrail explore` breaking into truncated stripes when the document panel is narrower than the longest code line. The renderer used to pad each code line to the longest line and let `Paragraph::wrap` re-flow it, which dropped trailing styled whitespace at the wrap point and left visible gaps between content rows. The code-block renderer now hard-wraps lines into chunks no wider than the panel itself (visual-column aware, UTF-8 / CJK safe, indentation preserved), so each visual row paints its own uninterrupted gray gutter regardless of terminal size or live resizes. Blank lines inside code blocks also keep their background.

---

## CLI 3.4.0 — Language-Aware `devtrail explore`

### Added (CLI)
- `devtrail explore` now resolves framework governance docs (`QUICK-REFERENCE`, `AGENT-RULES`, `CHINA-REGULATORY-FRAMEWORK`, `PIPL-PIPIA-GUIDE`, etc.) in the language set by `language` in `.devtrail/config.yml`. With `language: zh-CN` or `es`, the navigation tree, titles, and document body all switch to the translated variant — the English original is used silently as a fallback when no translation exists. CJK rendering relies on the Unicode-safe layout work done in 3.2.3 / 3.2.4.
- New `--lang <code>` flag on `devtrail explore` to override the configured language for a single session (e.g., `devtrail explore --lang zh-CN`). Resolution order: `--lang` > `config.language` > `en`.
- Adopter-authored content under subgroups (`02-design/decisions/`, `05-operations/incidents/`, etc.) is intentionally never localized — it has no canonical i18n sibling.

### Changed (CLI)
- Shared `utils::resolve_localized_path()` is now the single source of truth for `i18n/<lang>/<filename>` lookups. `devtrail new` (templates) and `devtrail explore` (governance docs) both delegate to it.

---

## Framework 4.3.0 / CLI 3.3.0 — China Regulatory Coverage (TC260, PIPL, GB 45438, CAC, GB/T 45652, CSL)

DevTrail now supports six Chinese AI / data regulations as an opt-in regional scope. Existing projects are unaffected — Chinese frameworks activate only when `regional_scope: china` is added to `.devtrail/config.yml`.

### Added (Framework)
- 4 new document templates: `TEMPLATE-PIPIA.md`, `TEMPLATE-CACFILE.md`, `TEMPLATE-TC260RA.md`, `TEMPLATE-AILABEL.md` — translated to `es` and `zh-CN`.
- 5 new governance guides under `dist/.devtrail/00-governance/` — `CHINA-REGULATORY-FRAMEWORK.md`, `TC260-IMPLEMENTATION-GUIDE.md`, `PIPL-PIPIA-GUIDE.md`, `CAC-FILING-GUIDE.md`, `GB-45438-LABELING-GUIDE.md` — with full `es` and `zh-CN` translations.
- China-specific sections appended to `TEMPLATE-MCARD`, `TEMPLATE-DPIA`, `TEMPLATE-INC`, `TEMPLATE-ETH`, `TEMPLATE-SBOM`, `TEMPLATE-AILOG` — activated by `regional_scope: china`.
- `regional_scope` field documented in `.devtrail/config.yml` with explanatory comments. Default `[global, eu]` preserves backward compatibility.

### Added (CLI)
- 4 new `DocType` variants: `Pipia`, `Cacfile`, `Tc260ra`, `Ailabel`. Filtered out of `devtrail new` unless `china` is in `regional_scope`.
- 6 new `Standard` variants and checkers: `china-tc260`, `china-pipl`, `china-gb45438`, `china-cac`, `china-gb45652`, `china-csl`.
- New `--region <global|eu|china|all>` flag on `devtrail compliance`. The default behavior now respects `regional_scope` from config; `--all` still runs every standard.
- `devtrail compliance --standard <name>` accepts six new identifiers.
- 12 new validation rules: `CROSS-004` through `CROSS-011` and `TYPE-003` through `TYPE-006`. China rules are skipped when `china` is not in scope.
- 20 new optional frontmatter fields covering TC260, PIPL, GB 45438, CAC, GB/T 45652, and CSL profiles.
- `devtrail metrics` document-count breakdown now includes the 4 China-specific types when present.
- 30+ new tests (unit + integration) covering checkers, validation, config, and the opt-in dispatch.

### Notes
- TC260 v2.0 is treated as `Recommended` (not yet a binding GB). Status will be promoted in a future release if it is published as a GB.
- CSL 2026 reporting windows (1h / 4h+72h+30d) are enforced as cross-rules but DevTrail does not validate actual submission to authorities — it documents intent and plan.

---

## CLI 3.2.5 — Smarter Table Column Allocation in `explore`

### Fixed (CLI)
- Table column widths in `devtrail explore` are now allocated with a water-fill strategy: narrow columns (e.g. `CWE`, `Severity`) receive exactly their natural width and the excess flows to the columns that need it (e.g. `Description`, `Remediation`). Previously, a proportional pass gave every column a slice of the terminal budget regardless of need, which caused the narrow columns to hoard space and the wide ones to wrap unnecessarily. This is what produced the "fixes itself, breaks, fixes itself again" behavior users saw while resizing the terminal.

---

## CLI 3.2.4 — Unicode-Safe Rendering Across TUI and Commands

### Fixed (CLI)
- Scrollbar in `devtrail explore` no longer leaks document text through the track; the document body now renders in a dedicated column and the scrollbar thumb has a correct viewport-proportional size.
- `devtrail explore` navigation tree, metadata panel, status bar, and Markdown code blocks now measure text in visual columns (via `unicode-width`) instead of bytes. Titles, tags, related-document links, paths, and the status bar all stay aligned with CJK, accented characters, and emoji.
- `devtrail validate`: filename-date parsing is now UTF-8-safe. Filenames with multi-byte characters where ASCII was expected fail with a clean `NAMING-001` error instead of risking a byte-boundary panic.
- `devtrail analyze` and `devtrail status` tables no longer misalign when paths, function names, or project directories contain non-ASCII characters.
- `devtrail new`: sequence-number and slug computation switched from byte slicing to char-safe operations.

### Changed (CLI)
- `unicode-width` is now a direct (always-compiled) dependency. Previously it was an optional transitive dep under feature `tui`.
- New shared helpers `visual_width`, `truncate_visual`, and `pad_right_visual` in `utils.rs`, used by every layout site that previously confused bytes with columns.

---

## CLI 3.2.3 — UTF-8 Crash Fix in `explore` Tables

### Fixed (CLI)
- Fix panic in `devtrail explore` when rendering Markdown tables whose cells contain multi-byte UTF-8 characters (em-dash `—`, CJK ideograms, accented characters, emoji). Cell wrapping now uses `char_indices()` for safe slicing and measures text in visual columns via `unicode-width`, so table borders also stay aligned with Chinese and double-wide content.

---

## CLI 3.2.2 — crates.io README Broken Links Fix

### Fixed (CLI)
- Convert all relative links in README to absolute GitHub URLs so badges, documentation links, and language switcher all resolve correctly on crates.io

---

## CLI 3.2.1 — crates.io README Language Links Fix

### Fixed (CLI)
- Use absolute GitHub URLs for Español and 简体中文 language links in README so they resolve correctly on crates.io

---

## Framework 4.2.0 / CLI 3.2.0 — Simplified Chinese (zh-CN) Localization

### Added (Framework)
- **Simplified Chinese (zh-CN)**: Complete localization as the third supported language alongside English and Spanish
  - 12 document templates (AILOG, ADR, AIDEC, DPIA, ETH, INC, MCARD, REQ, SBOM, SEC, TDE, TES)
  - 12 governance documents (AGENT-RULES, AI-GOVERNANCE-POLICY, AI-KPIS, AI-LIFECYCLE-TRACKER, AI-RISK-CATALOG, C4-DIAGRAM-GUIDE, DOCUMENTATION-POLICY, GIT-BRANCHING-STRATEGY, MANAGEMENT-REVIEW-TEMPLATE, OBSERVABILITY-GUIDE, PRINCIPLES, QUICK-REFERENCE)
  - 5 NIST implementation guides (AI RMF Govern/Map/Measure/Manage + GenAI Risks)
  - 6 user-facing docs (README, CONTRIBUTING, CODE_OF_CONDUCT, ADOPTION-GUIDE, CLI-REFERENCE, WORKFLOWS)

### Added (CLI)
- **Generic language support**: Template resolution now supports any configured language, not just hardcoded `es`

### Changed (Framework)
- Language navigation links updated across all three languages (EN, ES, zh-CN) in governance and documentation files
- Language navigation links added to English governance files (PRINCIPLES, DOCUMENTATION-POLICY, AGENT-RULES) that previously lacked them

---

## CLI 3.1.1 — crates.io README Fix

### Fixed (CLI)
- Include project README in crates.io package (copy from repo root during CI publish)
- Restore `readme` field in `Cargo.toml` pointing to local copy

---

## CLI 3.1.0 — crates.io Distribution & Smart Self-Update

### Added (CLI)
- **crates.io distribution**: `cargo install devtrail-cli` now available as an installation method
- **Install method detection**: `devtrail update-cli` auto-detects whether the CLI was installed via cargo or prebuilt binary and uses the appropriate update mechanism
- **`--method` flag**: Override auto-detection on `update-cli` and `update` commands (`auto`, `github`, `cargo`)
- **`devtrail about`**: Now displays the detected installation method
- **CI**: `release-cli.yml` workflow publishes to crates.io after GitHub Release upload

### Changed (CLI)
- `Cargo.toml`: Added `include` field for crate packaging, removed `readme` path (outside crate boundary)

---

## Framework 4.1.1 — Status Skill Complexity Fix

### Fixed (Framework)
- **devtrail-status skill**: Replace outdated ">10 lines of changes" heuristic with `devtrail analyze --output json` (cognitive complexity, threshold 8) as the primary method for AILOG triggers, with >20 lines fallback when CLI is unavailable
- Updated across all 3 platform variants: Claude Code, Gemini, and generic agent workflow

---

## CLI 3.0.1 — Validate False Positive Fix

### Fixed (CLI)
- **REF-001**: Only validate `related:` references that match DevTrail document ID patterns (AILOG-*, AIDEC-*, ADR-*, etc.). Skips task IDs, requirement IDs, risk IDs, external paths, and other non-document references
- **SEC-001**: `Bearer` and `token:` moved from errors to warnings — common in documentation describing auth flows. Actual secret patterns remain as errors

---

## Framework 4.1.0 / CLI 3.0.0 — Complexity-Based Documentation & Ecosystem

### Added (CLI)
- **`devtrail analyze`** command — Code complexity analysis (cognitive + cyclomatic) powered by [arborist-metrics](https://github.com/StrangeDaysTech/arborist)
  - Output formats: text (colored), json, markdown
  - 12 languages: Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift
  - Configurable threshold: CLI flag → `.devtrail/config.yml` → default (8)
- **`devtrail new`** command — Interactive document creation with type suggestion based on context
  - Supports all 12 document types
  - Analyzes git diff to suggest appropriate type
- **`--staged` flag** for `devtrail validate` — Validate only staged documents (pre-commit hooks)

### Changed (Framework)
- **Documentation triggers redesigned**: `devtrail analyze --output json` is now the primary method for determining when to create AILOGs. The >20 lines heuristic is preserved as fallback when the CLI is unavailable
  - Updated across all governance docs, agent directives, skills/workflows (18 files, EN + ES)
- Agent directives (Claude, Gemini, Copilot, Cursor) updated with complexity-based pre-commit checklist

### Changed (CLI)
- All 12 arborist-metrics languages enabled (was limited subset)
- Legacy scripts removed, replaced with CLI commands in all docs

### Added (Docs)
- arborist-metrics promotion in README (EN + ES) — Open Source Ecosystem table
- Documentation trigger notes in CLI-REFERENCE (EN + ES)

### Changed (CI/CD)
- Release workflows unified: both trigger on tag push with automatic release creation
- Idempotent releases: create if missing, upload with `--clobber` if exists
- Auto-delete previous releases on new version (keeps only latest per component)
- GitHub Actions updated to Node.js 24 compatible versions (checkout v6, upload-artifact v7, download-artifact v8)
- Version verification: workflows check Cargo.toml / dist-manifest.yml matches tag
- `workflow_dispatch` added to both workflows for manual re-runs

### Removed
- Legacy `auditoria/` directory (one-time audit reviews, findings already addressed)
- Legacy `docs/archive/` (obsolete planning documents)
- Legacy shell scripts (replaced by CLI commands)

---

## Framework 4.0.0 / CLI 2.1.0 — Advanced Automation & Ecosystem

### Added (CLI)
- **`devtrail audit`** command — Generate audit trail reports with timeline, traceability map, risk distribution, and compliance summary
  - Output formats: text (colored terminal), markdown, json, html (with SVG pie chart)
  - Filters: `--from`/`--to` date range, `--system` component filter
  - Traceability graph built from document `related:` fields using BFS

### Added (Framework)
- **C4-DIAGRAM-GUIDE.md** — Complete guide for C4 Model diagrams with Mermaid syntax (EN + ES)
  - Examples for all 4 levels: Context, Container, Component, Code
  - PlantUML alternative syntax
  - Integration guidance for ADR and REQ documents
- `api_changes` field in TEMPLATE-ADR.md frontmatter for tracking API endpoint changes
- `api_spec_path` field in TEMPLATE-REQ.md frontmatter for OpenAPI/AsyncAPI spec references
- Architecture Diagram section in TEMPLATE-ADR.md with Mermaid C4 placeholder
- Sections 10 (C4 Model) and 11 (API Specification Tracking) in AGENT-RULES.md
- Terminal compatibility notes in skill files for box-drawing character fallback

### Changed
- QUICK-REFERENCE.md: Added C4 Model reference to regulatory alignment table
- Updated CLI-REFERENCE.md, README.md with 13 commands (EN + ES)

---

## Framework 3.2.0 / CLI 2.0.0 — Compliance Automation & Metrics

### Added (CLI)
- **`devtrail compliance`** command — Check regulatory compliance (EU AI Act, ISO 42001, NIST AI RMF)
  - Output formats: text, markdown, json
  - Per-standard or `--all` mode with percentage scores
- **`devtrail metrics`** command — Governance metrics and documentation statistics
  - Period filtering, review compliance rate, risk distribution, agent activity, trends

### Added (Framework)
- AI-RISK-CATALOG.md — Risk catalog mapped to 12 NIST AI 600-1 categories + ISO 42001 Annex C
- AI-LIFECYCLE-TRACKER.md — AI system lifecycle tracking mapped to ISO 42001 Annex A.6
- AI-KPIS.md — Governance KPI tracking template
- MANAGEMENT-REVIEW-TEMPLATE.md — ISO 42001 Clause 9.3 review agenda
- OBSERVABILITY-GUIDE.md — OpenTelemetry integration guide with 10 sections (EN + ES)
- NIST AI RMF implementation guides: MAP, MEASURE, MANAGE, GOVERN
- NIST-AI-600-1-GENAI-RISKS.md — Detailed 12 GenAI risk categories

---

## Framework 3.1.0 / CLI 1.4.0 — New Document Types & Validation

### Added (CLI)
- **`devtrail validate`** command — Validate documents with 13 rules (NAMING, META, CROSS, TYPE, REF, SEC, OBS)
  - `--fix` flag for automatic corrections
  - Exit code 1 on errors, 0 on warnings-only
- Document parsing engine (`document.rs`) — Shared by validate, compliance, metrics, audit
- Validation engine (`validation.rs`) — Extensible rule-based validation

### Added (Framework)
- **TEMPLATE-SEC.md** — Security Assessment (STRIDE threat model, OWASP ASVS)
- **TEMPLATE-MCARD.md** — Model/System Card (Mitchell et al. 2019)
- **TEMPLATE-SBOM.md** — Software Bill of Materials (SPDX/CycloneDX aligned)
- **TEMPLATE-DPIA.md** — Data Protection Impact Assessment (GDPR Art. 35)
- Skills: `/devtrail-sec`, `/devtrail-mcard` (Claude, Gemini, generic agent)
- Updated `/devtrail-new` and `/devtrail-status` for 12 document types
- Compliance CI jobs in docs-validation.yml

---

## Framework 3.0.0 / CLI 1.3.0 — Regulatory Base & Standards Update

### Changed (Framework)
- **IEEE 830 → ISO/IEC/IEEE 29148:2018** in TEMPLATE-REQ.md (External Interfaces, V&V, Traceability)
- **ISO/IEC 25010:2011 → 2023** in TEMPLATE-ADR.md and TEMPLATE-REQ.md (9 quality characteristics)
- **ISO/IEC/IEEE 29119-3:2021** alignment in TEMPLATE-TES.md (3-level hierarchy, 29119 terminology)
- Regulatory fields added to all templates: `eu_ai_act_risk`, `nist_genai_risks`, `iso_42001_clause`
- OpenTelemetry optional sections in TEMPLATE-REQ, TEMPLATE-TES, TEMPLATE-INC, TEMPLATE-AILOG

### Added (Framework)
- **AI-GOVERNANCE-POLICY.md** — ISO 42001 Clauses 4-10 governance template
- **ISO-25010-2023-REFERENCE.md** — Quality characteristics reference
- EU AI Act, NIST GenAI, GDPR sections in ETH, INC, and AILOG templates
- Observability rules in AGENT-RULES.md (Section 9)
- Expanded agent directives with pre-commit checklists
- New folders: `08-security/`, `09-ai-models/`

### Added (CLI)
- Support for 12 document types (was 8): SEC, MCARD, SBOM, DPIA
- New directories in `init`, `status`, `repair`, `explore`

### Changed (CLI)
- Cross-validation rules in pre-commit hooks and CI

---

*DevTrail is maintained by [Strange Days Tech](https://strangedays.tech).*
