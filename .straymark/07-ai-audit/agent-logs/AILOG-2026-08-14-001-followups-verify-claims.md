---
id: AILOG-2026-08-14-001
title: followups verify --claims — batch re-derivation of registry code claims against the tree (#419, PR 2 of CHARTER-02)
status: accepted
created: 2026-08-14
agent: qoder-cli-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
files_modified:
  - cli/src/tree_grep.rs
  - cli/src/commands/followups/verify_claims.rs
  - cli/src/commands/followups/mod.rs
  - cli/src/commands/analyze_declared_vs_wired.rs
  - cli/src/main.rs
  - cli/tests/followups_test.rs
  - cli/Cargo.toml
  - Cargo.lock
  - CHANGELOG.md
  - docs/adopters/CLI-REFERENCE.md
  - docs/i18n/es/adopters/CLI-REFERENCE.md
  - docs/i18n/zh-CN/adopters/CLI-REFERENCE.md
observability_scope: none
tags: [followups, verify, claims, reference-resolution, adopter-feedback, cli, charter-02]
related:
  - AILOG-2026-08-13-001
---

# AILOG: followups verify --claims (#419, PR 2)

## Summary

Issue #419 case 2: a remediation AILOG claimed a function "has no callers"
while the tree already had three call sites — the registry states mechanical
claims about the code, and nothing re-derives them. PR 1 gave id citations a
resolver; this PR gives the registry's *code claims* one. The per-entry
`followups verify` puts a premise in front of a human; `--claims` re-derives
the claims the tree can answer on its own, in batch, warn-first.

Defect class covered (design constraint 3): **registry drift vs the tree**.

## Decision

Three checks, all grep tier (design constraint 2 — no AST, cross-stack):

- `CLAIM-PATH-GONE`: a backticked path (`src/old/parser.rs`) or bare filename
  (`validation.rs`) that no longer exists anywhere in the tree.
- `CLAIM-SYMBOL-GONE`: a backticked symbol with zero word-boundary
  occurrences outside `.straymark/`.
- `CLAIM-STALE-DEAD`: an entry asserting "no callers / not wired / unused /
  dead code" whose symbol is now mentioned by ≥2 files. The two-file
  threshold is the precision guard: one file can be definition-plus-use and
  grep cannot tell them apart, so it stays quiet there.

Classification skips everything that is not a code claim — flags,
placeholders, URLs, whitespace-bearing prose, versions, and dashed shapes
(FU/CHARTER/rule ids; the validate rules from PR 1 own that class). Claims
are read from the description, `Premise`, and `Notes` fields of `open` /
`in-progress` entries (or one explicit entry). Warn-first (design
constraint 1): findings print, exit code stays 0.

The (glob, regex) walker behind `analyze declared-vs-wired`'s
`collect_symbols` moved into a shared `tree_grep` module; `regex` and `glob`
moved from the optional `analyze` feature to required dependencies, since
the new mode needs them unconditionally (user decision on the charter plan).

## Baseline (first run against this repository)

`followups verify --claims` on this repo: **3 open entries scanned, 0
findings** — clean. The batch correctly ignored multi-word command spans
(`straymark update`) and closed/superseded entries.

## Actions Performed

- `cli/src/tree_grep.rs` (new): `collect_symbols` (moved verbatim),
  `read_text_tree` (skips `.git`/`target`/`node_modules`/`.straymark`,
  symlinks, non-UTF-8), `symbol_occurrences` (word-boundary count). 4 unit
  tests, incl. the two moved from `analyze_declared_vs_wired`.
- `cli/src/commands/followups/verify_claims.rs` (new): batch mode, span
  extraction/classification, the three checks, warn-first reporting. 3 unit
  tests.
- `FollowupsCommands::Verify`: positional `FU-NNN` → optional
  (`required_unless_present = "claims"`); `--claims` conflicts with the
  per-entry write flags. Per-entry behavior unchanged.
- `cli/Cargo.toml` 3.47.0; `regex` + `glob` required; `analyze` feature
  keeps only `arborist-metrics`.
- 7 integration tests in `followups_test.rs` (phantom path/symbol flagged
  with exit 0; stale-dead flagged at ≥2 files; closed entries ignored;
  clean tree reports clean; fu_id filter; `--claims` × `--verified`
  conflict; missing fu_id without `--claims` fails).
- CHANGELOG (cli-3.47.0); CLI-REFERENCE EN + es + zh-CN.

## Risks

- R2 (Charter): `--claims` false positives on prose that looks like a claim.
  Mitigation: warn-first, classification skips non-claims, two-file
  threshold on STALE-DEAD. First run on this repo: zero noise on 3 entries.
  No new risk surfaced during implementation.
- R3 (new, not in Charter): `cli/src/commands/followups/verify.rs` was
  declared in the Charter ("dispatch `--claims` …") but did not need
  changes — the dispatch landed in `cli/src/main.rs` next to every other
  command dispatch, and the per-entry mode is untouched. Conversely,
  `cli/src/commands/followups/mod.rs` was NOT declared and needed a
  one-line `pub mod verify_claims;` registration — recorded here per the
  drift-check protocol.

## Validation

- `cargo test --workspace` green (incl. 48 followups integration tests).
- Dogfood: `followups verify --claims` on this repo — clean (baseline above).
- Dogfood: `verify <id> --claims` on a fixture registry filters the batch to
  that entry; `verify` without fu_id and without `--claims` fails (clap).

## Follow-ups

- REF-003 / GUARD-001 / CLAIM-* severity flips to Error: only after the
  warn-first baseline is measured across adopters. (Captured in the
  Charter's out-of-scope; FU-007 tracks the REF-003 side.)
