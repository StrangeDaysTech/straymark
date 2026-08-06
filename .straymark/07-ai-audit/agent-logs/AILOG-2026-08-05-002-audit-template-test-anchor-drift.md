---
id: AILOG-2026-08-05-002
title: audit_template_test — pin sections by content, not by step ordinal (closes the anchor drift red since v1.2)
status: accepted
created: 2026-08-05
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 77
files_modified:
  - cli/tests/audit_template_test.rs
observability_scope: none
tags: [testing, audit-prompt, i18n, guard-markers, tech-debt]
related:
  - AILOG-2026-08-04-001
---

# AILOG: audit_template_test — pin sections by content, not by step ordinal

## Summary

`audit_template_test::unified_template_has_seven_universal_sections` had
been failing on `main` since audit-prompt v1.2, across two releases. The
template was right and the test was wrong: v1.2 (#382) inserted
"Enumerate callers of new public entry points" as Step 3, pushing severity
calibration from Step 5 to Step 6, and the test pinned the heading
including its ordinal. Fixed by anchoring on the stable half of each
heading, plus two new invariants that catch the class rather than the
instance.

## Context

Recorded as R1 in AILOG-2026-08-04-001 with two candidate readings — update
the test's anchors, or restore a Step 5 calibration section. Verification
settles it: the section exists and is intact in all three copies (EN dist,
ES dist, governance-in-force), only renumbered. So the failure was never
product drift; it was a test asserting an ordinal that no one promised
would hold.

That is the same shape as the guard-marker class this project keeps
hitting: an assertion whose subject is incidental (a number that shifts
when a step is inserted) rather than the property it means to protect (the
step exists and says what it must). The ordinal is not the contract.

## Actions Performed

1. **Anchors made ordinal-free.** The two step-bearing entries in the
   seven-section list now match `— Verify each task (MANDATORY)` and
   `— Calibrate severity against the project's REAL configuration`. The
   five non-step anchors were already stable and are unchanged.
2. **`template_steps_are_numbered_without_gaps` (new).** Extracts the
   ordinals of `### Step N —` / `### Paso N —` headings and asserts they
   run `1..N` in order, in both locales. This is the invariant the old test
   was reaching for by accident: inserting a step mid-sequence and
   forgetting to renumber is cheap to do and invisible to a skim.
3. **`es_template_tracks_the_en_procedure` (new).** Asserts the ES template
   carries the same number of procedure steps as the EN canonical. fw-4.38.1
   exists because v1.2 landed only in the governance copy and the EN dist
   template, leaving Spanish adopters on v1.1 with no signal; step count is
   the cheapest invariant that would have caught it. The ES template is now
   `include_str!`-ed alongside the EN one.
4. **Comment corrected** in `unified_template_carries_anti_inflation_didactic_example`,
   which named Step 5 as the home of the anti-inflation example.

## Modified Files

| File | Change Description |
|------|--------------------|
| `cli/tests/audit_template_test.rs` | ordinal-free anchors; `step_ordinals` helper; 2 new tests; ES template constant; comment fix |

## Decisions Made

- **Fix the test, not the template.** The template is correct in all three
  copies; restoring a "Step 5 calibration" heading would have re-broken the
  numbering to satisfy an assertion.
- **Guard the class, not the instance.** Re-pinning the anchor to "Step 6"
  would go red again the next time a step is inserted. The numbering
  invariant and the locale-parity invariant make the next occurrence fail
  for the right reason, with a message that names it.
- **Step count as the parity proxy.** A full structural diff between
  locales would be stronger and far more brittle (headings are translated).
  Count catches the failure that actually happened.

## Impact

- **Functionality**: none — test-only change. No product file touched.
- **Performance**: N/A
- **Security**: N/A
- **Privacy**: N/A
- **Environmental**: N/A

## Verification

- [x] Code compiles without errors
- [x] `cargo test` at the workspace root: **40 suites green, zero failures** —
  the first fully green run since the anchor drift appeared. `audit_template_test`
  goes 9 → 11 tests, all passing.
- [x] Manual review performed — confirmed the severity-calibration section is
  present and intact in `dist/.straymark/audit-prompts/audit-prompt.md`
  (Step 6), its ES counterpart (Paso 6), and the governance-in-force copy.
- [ ] Security scan passed (if risk_level: high/critical) — N/A (low)
- [ ] Privacy review completed (if handling PII) — N/A

## Follow-ups

- **`followups drift` reports "registry in sync" from an empty scan window.**
  The default range is `origin/main..HEAD`, which on a synced `main` is empty
  — so on `main` the command always reports sync, whatever the registry
  holds. `--scan-all` on this repo surfaces 13 unextracted entries across 7
  AILOGs, including the follow-up that declared *this* work. The claim is
  global ("registry in sync") while the evidence is a window; it should
  either say which window it scanned or widen when the window is empty.
  Same failure family as GH #392: the registry looks complete while
  silently missing items.
- **No `cargo test` job in CI** — the deferral (decided 2026-06-11: CI runs
  codex-sync and build only) is why a red test survived two releases. It was
  reasonable when taken; this is the first concrete cost of it.
