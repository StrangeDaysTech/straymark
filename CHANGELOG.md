# Changelog

All notable changes to StrayMark (formerly DevTrail; rebranded 2026-05-08, see [`ADR-2026-05-08-001`](docs/decisions/ADR-2026-05-08-rebranding-straymark.md)) will be documented in this file. Historical entries below preserve the "DevTrail" name where present — that was the project's name at the time of those releases.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [independent versioning](README.md#versioning) for Framework (`fw-`) and CLI (`cli-`).

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
