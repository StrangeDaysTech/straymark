---
id: AILOG-2026-08-04-003
title: Lote 3 adopter fixes — structural merge driver for the follow-ups registry (#391) and untracked FU-id validation warning (#392)
status: accepted
created: 2026-08-04
agent: qodercli-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 0
files_modified: []
observability_scope: none
tags: [follow-ups, merge-driver, git, validation, drift, adopter-feedback, cli]
related: []
---

# AILOG: Lote 3 adopter fixes — structural merge driver for the follow-ups registry (#391) and untracked FU-id validation warning (#392)

## Summary

Third remediation batch from the 2026-08-04 open-issue triage. #391 closes
the registry's parallel-PR hazard: `.straymark/follow-ups-backlog.md` is a
single CLI-owned file, so every concurrent PR that touches follow-ups
conflicts on it, and resolving textually (take one side, re-run
`drift --apply`) silently reverted the other side's closures — statuses
live only in the file, and a re-extraction renumbers ids, so even
comparing ids cannot detect the loss. Fix: a structural git merge driver
(`straymark followups merge-driver`) plus title-based dedup in
`drift --apply`. #392 closes the silent-mention gap: FU ids declared
outside an AILOG's `## Follow-ups` section were invisible to the extractor
with no warning; `validate` now flags unregistered ones.

## Context

Both issues were reported against the follow-ups backlog pattern
(fw-4.21.0+). The registry's design makes titles, not ids, the stable
identity of an entry: ids are positional (`max(existing) + 1` at
extraction time) and renumber whenever entries are regenerated, while
titles survive. Both fixes lean on that invariant.

## Actions Performed

1. **#391 — structural merge driver.**
   - `cli/src/followups.rs`: new `merge_registries(base, ours, theirs)`
     with `status_rank`, `normalize_title` (whitespace-collapsed,
     lowercased) and a `MergeReport`. Entries are matched across sides
     **by title**; the higher-rank status wins (`open` < `in-progress` <
     `suspected-closed` < `closed`/`superseded`/`promoted`), so a closure
     made on either side survives; equal-rank disagreements keep `ours`
     and are reported. Entries only in `theirs` are appended (renumbered
     on id collision), deletions by `theirs` are respected unless `ours`
     changed the entry's status (modify/delete → kept + reported),
     `Notes` accepts append-only extensions from `theirs`, and the
     frontmatter (`fully_extracted_ailogs` union, newest `last_scan`,
     counters) is recomputed from the merged body.
   - `cli/src/commands/followups/merge_driver.rs` (new): the git-driver
     entry point (`%O %A %B` contract; exit 0 = merged, nonzero = git
     marks the file conflicted). Lenient on a missing/unparseable base;
     strict on ours/theirs.
   - `cli/src/main.rs`: `straymark followups merge-driver <base> <ours>
     <theirs>` subcommand with gitattributes setup in the doc comment.
   - `cli/src/commands/followups/drift.rs`: `--apply` now skips
     candidates whose normalized title already exists in the registry,
     protecting declarations that moved section (and re-extractions after
     any conflict resolution) from spawning a duplicate `open` entry that
     shadows the operator's status.
2. **#392 — untracked FU-id warning.**
   - `cli/src/validation.rs`: new `check_followup_mentions` — for every
     AILOG, FU ids (`FU-NNN` and charter-scoped `FU-NNN-NNN`) mentioned
     **outside** the document's own `## Follow-ups` section and absent
     from the registry emit a warn-only `FOLLOWUP-UNTRACKED-ID` issue
     with a fix hint. Mentions of registered ids (legitimate
     cross-references) stay quiet, and the check is skipped entirely when
     the project has no registry.
3. **Docs.** CLI-REFERENCE updated in en/es/zh-CN (merge-driver section
   with the reconciliation table and once-per-clone gitattributes setup,
   the new validate rule, and the title-dedup note on `drift --apply`).
   `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` gained a
   "Parallel PRs — structural merge (cli-3.41.0+)" subsection.

## Modified Files

| File | Change Description |
|------|--------------------|
| `cli/src/followups.rs` | `merge_registries` + `status_rank` / `normalize_title` / `MergeReport`; `is_followup_heading` made `pub` |
| `cli/src/commands/followups/merge_driver.rs` | git merge-driver entry point (new) |
| `cli/src/commands/followups/mod.rs` | `pub mod merge_driver` |
| `cli/src/main.rs` | `MergeDriver` subcommand + dispatch |
| `cli/src/commands/followups/drift.rs` | title-based dedup in `detect_drift_candidates` |
| `cli/src/validation.rs` | `check_followup_mentions` + `scan_fu_ids` (`FOLLOWUP-UNTRACKED-ID`) |
| `cli/tests/followups_test.rs` | 2 merge-driver tests + 1 title-dedup test |
| `cli/tests/validate_test.rs` | 2 untracked-FU-id tests |
| `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` | parallel-PR merge subsection + title-dedup note |
| `docs/adopters/CLI-REFERENCE.md` (+ i18n es/zh-CN) | merge-driver section, validate rule, drift note |
| `.gitignore` | `/.qoder/` (local agent config, mirrors `/.claude/`) |

## Decisions Made

- **#391 — match by title, not id.** Ids are positional and do not
  survive regeneration; the title is the registry's stable identity. A
  structural merge keyed on ids would mis-pair entries after any
  re-extraction.
- **#391 — status rank, not timestamp.** The merge driver sees three file
  versions with no reliable clock; rank order makes "a closure made on
  either side survives" the mechanical outcome, and equal-rank
  disagreements stay visible (stderr) instead of being guessed.
- **#392 — warn only, and only for unregistered ids.** A registered id
  mentioned elsewhere is a legitimate cross-reference; the hazard is a
  declaration the extractor cannot see **and** nothing tracks. Warn-only
  keeps the rule advisory for projects mid-adoption.
- **Folded in: `/.qoder/` gitignore.** Lote 2 ignored only
  `/.qoder/skills/`; Qoder also writes `.qoder/settings.local.json`,
  which is per-machine config like `/.claude/`.

## Impact

- **Functionality**: parallel PRs touching the registry merge without
  operator intervention and without losing closures; untracked FU-id
  declarations surface at `validate` instead of silently never being
  extracted.
- **Performance**: N/A
- **Security**: N/A
- **Privacy**: N/A
- **Environmental**: N/A

## Verification

- [x] Code compiles without errors
- [x] Tests pass — `cargo test --no-fail-fast` in `cli/`: all suites green
  except the pre-existing `audit_template_test::unified_template_has_seven_universal_sections`
  failure documented in AILOG-2026-08-04-001 (R1). New tests: 3 in
  followups_test (merge closures/unions, deletion + conflict visibility,
  title dedup) and 2 in validate_test (unregistered warns, no-registry
  stays quiet), all passing.
- [x] Manual review performed
- [ ] Security scan passed (if risk_level: high/critical) — N/A (low)
- [ ] Privacy review completed (if handling PII) — N/A

## Risk

- R1 (known, accepted): pre-existing failing test on main — unchanged from
  AILOG-2026-08-04-001.
- R2 (new): the merge driver requires once-per-clone setup
  (`.gitattributes` + `git config merge.*.driver`); a future
  `straymark init`/`update-framework` hook could offer to wire it
  automatically, gated on adopter demand.
- R3 (new): title-based dedup compares normalized titles, so a
  deliberately *reworded* re-declaration of the same follow-up still
  extracts a second entry — accepted; that case is operator-visible at
  triage, unlike the silent-status-loss this batch closes.

## Follow-ups

- (new) Consider wiring the merge-driver setup into `straymark init`
  behind a prompt (see R2).

---

<!-- AILOG generated by qodercli-v1.0 | StrayMark | https://strangedays.tech -->
