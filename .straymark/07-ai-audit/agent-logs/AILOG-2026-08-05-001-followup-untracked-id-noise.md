---
id: AILOG-2026-08-05-001
title: FOLLOWUP-UNTRACKED-ID — recognize the two id spaces so the rule stays readable in adopters with history (#392)
status: accepted
created: 2026-08-05
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 234
files_modified:
  - cli/src/validation.rs
  - cli/tests/validate_test.rs
  - cli/Cargo.toml
  - Cargo.lock
  - CHANGELOG.md
  - docs/adopters/CLI-REFERENCE.md
  - docs/i18n/es/adopters/CLI-REFERENCE.md
  - docs/i18n/zh-CN/adopters/CLI-REFERENCE.md
  - README.md
  - docs/i18n/es/README.md
  - docs/i18n/zh-CN/README.md
observability_scope: none
tags: [follow-ups, validation, adopter-feedback, cli, false-positives]
related:
  - AILOG-2026-08-04-003
---

# AILOG: FOLLOWUP-UNTRACKED-ID — recognize the two id spaces so the rule stays readable in adopters with history (#392)

## Summary

Field report on the rule shipped in cli-3.41.0. Run against an adopter with
history, `FOLLOWUP-UNTRACKED-ID` produced **192 warnings across 66
documents**, nearly all false positives. The rule's own premise turns
against it at that volume: a signal that appears 192 times stops being
read, and the real cases drown. The matching criterion now asks whether
the extractor could ever have seen the id — covering every form the
registry remembers, plus ids the document itself declares — which takes
the same adopter to **14 warnings, all true positives**, including the
case that opened the issue.

## Context

Two identifier spaces coexist in any adopter that has run the registry for
a while, and only one of them was ever consulted:

- The **registry id** the CLI assigns (`max + 1` at extraction, rendered
  `### FU-NNN — …`). The only first-class identifier.
- The adopter's **author id**, charter-scoped (`FU-0NN-0NN`), written in
  the AILOG at declaration time. The framework does not define it: it has
  no field in `Entry`, no entry in the backlog schema, and `drift --apply`
  neither reads nor preserves it. It survives only as text inside the
  entry title, because the heading parser puts everything after the
  separator into the description.

Citing the author id in prose is the *better* citation — it names the
Charter the item came from, where the registry id is opaque to a reader —
so matching on `fu_id` alone punished the more traceable habit. A second
shape had the same fate: entries closed and pruned by triage leave their
record in a closure section, not as a `### FU-NNN` entry, so every
back-reference to a closed item warned too.

Measured over the reference adopter's 192 warnings (90 distinct ids):

| criterion | ids covered | warnings left |
|---|---|---|
| entry ids only (cli-3.41.0) | — | 192 |
| \+ ids appearing in an entry title | 49 / 90 | 106 |
| \+ any mention anywhere in the registry body | 81 / 90 | 16 |
| \+ ids the document declares in its own `## Follow-ups` | — | 14 |

The 9 ids that survive every criterion are the genuine defects — the
issue's own case among them, in the exact document and lines that
motivated the report.

## Actions Performed

1. **`cli/src/validation.rs` — `check_followup_mentions`.**
   - The known-id set is now the union of entry ids and every id
     `scan_fu_ids` finds across the registry body: author-id aliases
     inside titles, `Notes` back-references, closure sections for pruned
     entries.
   - Per document, ids found **inside** its follow-ups sections are
     collected as declarations, and candidates outside are judged after
     the pass — so a declaration further down the document covers an
     earlier prose mention. Prose citing an id the document itself
     declares is visible to the extractor by construction, even before
     `drift --apply` has run.
   - Message and hint reworded to state the criterion actually applied
     ("appears nowhere in the registry"), and the design doc-comment
     rewritten to record why each shape stays quiet.
2. **`cli/tests/validate_test.rs`.** The shared registry fixture gained an
   entry carrying an author-id alias in its title and a closure section;
   four tests added — alias in title, id only in a closure section,
   id declared later in the document's own follow-ups section (all
   silent), and the regression case from the issue (still warns, exactly
   once).
3. **Docs.** The validate rule's description rewritten in the three
   CLI-REFERENCE locales; CHANGELOG section for CLI 3.41.1; version bump
   and the six versioning tables.

## Modified Files

| File | Change Description |
|------|--------------------|
| `cli/src/validation.rs` | known-id set widened to the registry body; per-document declaration set; message/hint/doc-comment rewritten |
| `cli/tests/validate_test.rs` | fixture extended (alias entry + closure section); 4 tests |
| `cli/Cargo.toml`, `Cargo.lock` | 3.41.0 → 3.41.1 |
| `CHANGELOG.md` | CLI 3.41.1 section |
| `docs/adopters/CLI-REFERENCE.md` (+ i18n es/zh-CN) | rule description + versioning table |
| `README.md` (+ i18n es/zh-CN) | versioning table |

## Decisions Made

- **Silence, not a new severity.** The field report proposed an
  informational level for "exists, cited under another identifier". Both
  variants were measured: silence takes the adopter to 14 lines, an
  informational level leaves 176 lines on screen under a different label —
  reproducing the noise the report was about. The distinction is real but
  does not need to be printed.
- **Any mention in the registry, not just entry titles.** Matching titles
  alone was the narrower proposal and covers 49 of 90 ids (192 → 106).
  The remaining 45% are back-references to pruned entries, which are the
  same class of legitimate citation. Scanning the registry body covers
  both with one criterion instead of two.
- **The author id stays a convention, not a field.** Making it
  addressable would mean a field in `Entry`, the backlog schema, `drift`,
  `find_entry` and the framework pattern. The issue asks for a readable
  signal, not a change to the identity model. Recorded as a follow-up.
- **No `dist/` change.** A clarifying note in the backlog pattern would
  have dragged a framework release for one sentence; the same clarification
  lives in the adopter-facing CLI-REFERENCE instead.

## Impact

- **Functionality**: the rule keeps flagging exactly the defect it was
  built for (a follow-up coined where the extractor cannot see it) while
  the citation habits of a mature registry no longer trigger it. Nothing
  that warned for a true reason stops warning.
- **Performance**: one extra scan of the registry body per run. N/A.
- **Security**: N/A
- **Privacy**: N/A
- **Environmental**: N/A

## Verification

- [x] Code compiles without errors
- [x] Tests pass — `cargo test` at the workspace root: all suites green
  except the pre-existing `audit_template_test::unified_template_has_seven_universal_sections`
  failure documented in AILOG-2026-08-04-001 (R1), confirmed to fail on
  the base branch with these changes stashed. 6 follow-up-mention tests
  passing (2 existing + 4 new).
- [x] Manual review performed — read-only dogfood against the reference
  adopter: 192 → 14 warnings, and the issue's own case still reported at
  both its lines.
- [x] `straymark validate .` on this repo: 5 untracked-id warnings, the
  same 5 as before (true positives — this registry holds one entry).
- [ ] Security scan passed (if risk_level: high/critical) — N/A (low)
- [ ] Privacy review completed (if handling PII) — N/A

## Follow-ups

- The charter-scoped author id is a convention with no home in the model:
  no field in `Entry`, absent from the backlog schema, and `drift --apply`
  preserves it only by accident — the title extractor can drop it when a
  bold lead wins, which was observed in the reference adopter. Either
  promote it to a declared field with its own traceability, or document it
  as prose-only and stop leaving its survival to chance.
- The rule runs only in the full `validate` pass, not in `validate
  --staged`, so the pre-commit hook never sees it. Coherent (it needs the
  whole registry) but worth an explicit decision.
