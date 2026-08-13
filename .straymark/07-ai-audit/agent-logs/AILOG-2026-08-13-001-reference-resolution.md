---
id: AILOG-2026-08-13-001
title: Name resolution for the markdown layer — generalized id reference checks and validate --commit-msg (#419, PR 1 of CHARTER-02)
status: accepted
created: 2026-08-13
agent: qoder-cli-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
files_modified:
  - cli/src/validation.rs
  - cli/src/commands/validate.rs
  - cli/src/main.rs
  - cli/tests/validate_test.rs
  - cli/Cargo.toml
  - CHANGELOG.md
  - .straymark/charters/02-mechanical-verifiers-for-the-markdown-layer-419.md
  - .straymark/charters/README.md
observability_scope: none
tags: [validate, reference-resolution, hooks, adopter-feedback, cli, charter-02]
related:
  - AILOG-2026-08-07-001
---

# AILOG: reference resolution (#419, PR 1)

## Summary

Issue #419 (Sentinel field report): the code has mechanical verifiers; the
governance markdown does not. A remediation commit in the adopter's repo cited
a phantom AILOG — format right, date plausible, file absent — and nothing in
the toolchain rejected it, because nothing resolved id-shaped references
outside Charter frontmatter. This PR gives the markdown layer its missing
"name resolution": every id-shaped token the framework owns (dated document
ids, FU-*, CHARTER-*) now resolves against a single index, on three surfaces.

Defect class covered (per the issue's design constraint 3): **phantom
references** — citations that look load-bearing and resolve to nothing.

## Decision

Three surfaces, two severities, one index:

1. **`validate --commit-msg <file>`** (new mode, **Error**). Extracts
   id-shaped tokens from a commit message and exits 1 on any that do not
   resolve. Designed for commit-msg hooks the way `--staged` is designed for
   pre-commit. Blocking from day one is safe here: the message is fully
   author-written and the id shapes are framework-owned, so precision is
   total (the issue's own argument).
2. **REF-001 promoted Warning → Error.** `related:` references must resolve.
   Same total-precision argument. Documented as a behavior break in the
   CHANGELOG; adopter fix is mechanical (correct or remove the dangling id).
3. **REF-003 (new, Warning)**. Bodies of dated documents and Charters are
   scanned for id-shaped tokens that do not resolve. Warn-first per the
   issue's design constraint 1 — legacy content and *intentional* phantom
   citations (test fixtures, examples) trip it, so the flip to Error waits
   for a measured baseline (see Baseline below).

All three resolve against `IdIndex`, built once per run: canonical ids of
discovered documents + every FU id the registry knows (entries and body
mentions, the same known-set FOLLOWUP-UNTRACKED-ID uses) + charters
(`CHARTER-NN` and `CHARTER-NN-slug`). The tokenizer is a generalization of
the existing `scan_fu_ids` scanner — same boundary rules — now covering all
families; `scan_fu_ids` remains as a filter over it. FU tokens in AILOG
bodies are exempt from REF-003 because FOLLOWUP-UNTRACKED-ID already owns
that class; double-reporting the same token is noise.

## Baseline (first run against this repo)

REF-003 on our own `.straymark/` found real drift, warn-first working as
designed: four documents cite charter ids that do not resolve in this tree
(Sentinel-side or never-created charters), and one AIDEC cites FU ids the
registry had forgotten. The precise inventory lives in FU-007's notes —
recorded for cleanup; not fixed in this PR (scope). One emergent behavior
worth naming: citing the drifted FU ids in FU-007's title made them resolve,
because the index harvests every FU id the registry body mentions — the same
known-set rule FOLLOWUP-UNTRACKED-ID already uses for pruned-entry
provenance bullets. The registry now "knows" them as tracked cleanup.

Our own Charter for this work initially tripped REF-003 four times; two were
reworded (external artifact cited in id shape — exactly the rewording the
fix_hint suggests), one resolves once this AILOG exists, and one is the
intentional phantom fixture in the Charter's Verification commands (a
must-FAIL case needs a non-resolving id by construction). Accepted: one
REF-003 warning on the Charter, documented here.

## Actions Performed

- `cli/src/validation.rs`: `scan_straymark_ids` tokenizer (dated ids,
  FU-NNN(-NNN), CHARTER-NN; slug-stripping; word-boundary rules), `IdIndex`,
  `validate_commit_msg` (COMMIT-REF-001, Error), `check_id_references`
  (REF-003, Warning) wired into `validate_all` and `validate_paths`;
  REF-003 for Charter bodies wired into `validate_charters`;
  `check_related_exist` re-wired to the index and promoted to Error;
  `find_document_by_id` deleted (superseded by the index).
- `cli/src/commands/validate.rs` + `cli/src/main.rs`: `--commit-msg <FILE>`
  flag (conflicts with `--staged`/`--agent`/`--fix`) and the `run_commit_msg`
  mode.
- Tests: 8 unit (tokenizer shapes/boundaries, index over a tempdir project,
  REF-001 severity, commit-msg blocking + dedup, REF-003 warn + frontmatter
  skip + AILOG FU exemption) and 4 integration (`--commit-msg` pass/fail/no-id,
  REF-003 end-to-end); the REF-001 integration test now asserts failure.
- CHARTER-02 declared and flipped to in-progress; charters README row added.

## Risks

- R1 (Charter): REF-001 → Error breaks adopters with dangling `related:`
  refs. Mitigation: CHANGELOG calls it a behavior break; the adopter fix is
  mechanical. No new risk surfaced during implementation.

## Validation

- `cargo test --workspace` green (unit + integration, incl. the updated
  REF-001 test asserting failure).
- Dogfood: `validate . --include-charters` on this repo — 0 errors; REF-003
  warnings as baselined above.
- Dogfood: `validate --commit-msg` against a message citing a phantom id
  fails with COMMIT-REF-001; against one citing real ids passes.

## Follow-ups

- **FU-007** — REF-003 baseline cleanup: reword, resolve, or deliberately accept the drifted citations in CHARTER-01 and AIDEC-2026-07-18-001. Trigger: before any REF-003 severity flip to Error.
- REF-003 / GUARD-001 (PR 3) severity flip to Error: only after the warn-first baseline is measured across adopters. (Captured in the Charter's out-of-scope; promoted to an entry if adopted.)
