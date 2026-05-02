# Changelog

All notable changes to DevTrail will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [independent versioning](README.md#versioning) for Framework (`fw-`) and CLI (`cli-`).

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
