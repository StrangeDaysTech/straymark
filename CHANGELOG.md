# Changelog

All notable changes to DevTrail will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [independent versioning](README.md#versioning) for Framework (`fw-`) and CLI (`cli-`).

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
