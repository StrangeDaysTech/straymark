---
id: AILOG-2026-09-17-005
title: drift title fidelity — long local ids and underscores (#433)
status: accepted
created: 2026-09-17
agent: claude-opus-5-1m
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_integrity]
iso_42001_clause: []
files_modified:
  - cli/src/commands/charter/new.rs
  - cli/src/followups.rs
  - cli/src/commands/followups/drift.rs
  - cli/src/commands/charter/close.rs
  - cli/tests/followups_test.rs
  - cli/Cargo.toml, Cargo.lock, version tables, CLI-REFERENCE drift row x3, CHANGELOG.md
observability_scope: none
tags: [followups, drift, adopter-feedback, cli]
related:
  - 07-ai-audit/agent-logs/AILOG-2026-09-17-003-followups-declare.md
---

# AILOG: drift title fidelity (#433)

## Summary

`followups drift --apply` changed extracted titles without saying so: a long bold local id replaced
the description, and every `_` / `*` was stripped, even inside code. Both are fixed, and entries left
without a description now trigger a warning. CLI 3.49.1.

## Context

The adopter's hypotheses were half right. The underscore hypothesis was correct:
`strip_inline_markup` removed `_` and `*` everywhere, code spans included. The long-id hypothesis
was not. There is no 7-character limit on the id pattern. `leading_bold_title` takes a bold lead of
**15 or more characters** as the title (#365), and `FU-BARRIDOS-006` is exactly 15 characters
(`FU-ABCDEFG-005`, at 14, falls through). The description was dropped from the title and nothing
warned about it.

## Actions Performed

1. `strip_inline_markup` (shared with `charter new`): code spans are copied verbatim, with the
   closing run matched to the opening backtick count; an underscore run between two alphanumerics
   is kept (CommonMark: never an emphasis delimiter); other emphasis markers are still stripped.
2. `leading_bold_title`: a single-token bold span is a tag, not a title
   (`followups::title_is_bare`).
3. `drift::warn_bare_titles`: `drift --apply` and `charter close` list the entries whose title is
   only an id.
4. Tests:
   - unit: the markup stripper, and `entry_title` over the issue's bullets;
   - integration: the issue reproduction, the warning, and a registry holding a pre-fix title that
     must stay "in sync" (hash-stable dedup).

   The integration tests for the reproduction and the warning fail on `main`.

## Decisions Made

- **Existing titles are not rewritten.** The title is the merge driver's match key (#391).
  Retitling on one branch and not on another would break matching across open branches. Dedup
  keys on the unchanged Source-hash, so re-scanning an AILOG extracted under the old titles creates
  no duplicates. This was verified both on a copy of the reference corpus and in a test.
- **Patch release (3.49.1)** on top of the untagged 3.49.0.

## Impact

Regenerating a copy of the reference adopter's full registry from its AILOGs (Sentinel, 520
entries) changes 58 titles: all of them recover a `_`/`*`, or a description lost to a bold code
span or id. None of them regresses. The copy made with 3.49.0 is reported "in sync" by the fixed
binary. **Performance / Security / Privacy / Environmental**: N/A.

## Verification

- [x] `cargo test -p straymark-cli`: all pass (5 new tests)
- [x] Clippy: no new warnings (18/24, same as `main`)
- [x] Corpus comparison as above; the adopter's repository was read-only (copies only)
