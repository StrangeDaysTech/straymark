---
id: AILOG-2026-09-17-002
title: Baton inventory hygiene — skip nested checkouts, ignore follow-up examples (#434, #431)
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
  - experiment-baton/src/scan.rs
  - experiment-baton/src/units.rs
  - experiment-baton/src/coherence.rs
  - experiment-baton/src/codescan.rs
  - experiment-baton/tests/inventory_hygiene.rs
observability_scope: none
tags: [baton, adopter-feedback, inventory, track-c]
related:
  - 07-ai-audit/agent-logs/AILOG-2026-09-13-001-baton-task-inheritance.md
---

# AILOG: Baton inventory hygiene (#434, #431)

## Summary

Baton counted work that is not the project's live work. It walked into other git checkouts nested
in the project (#434), and it read the shipped registry's commented `### FU-NNN` example as a
follow-up (#431). Both inflate Track C denominators, and the first one also duplicates unit IDs.

## Context

Found while reviewing Estoa's #430. On Estoa's local checkout, one linked worktree under
`.worktrees/` inflated the inventory to 1946 tasks, with 1839 duplicated unit IDs. #431 was
reported by Estoa itself: its 43/50 raw rows were 42/49 real units.

## Actions Performed

1. `scan::is_nested_checkout`: a sub-directory with its own `.git` entry is another checkout. A
   `.git` file marks a linked worktree or a submodule, and a `.git` directory marks a nested clone.
   The check is structural, so there is no gitignore parsing and nothing for adopters to configure.
2. All three Baton walkers use it: `units::find_files` (tasks, batch ledgers),
   `coherence::Inventory::scan` and `codescan::walk_code`.
3. `read_followups` inventories only live entries:
   - text inside HTML comments (including multi-line and inline ones) and fenced blocks is ignored;
   - IDs must have the canonical `FU-<digits>` shape that `straymark followups` also requires;
   - metadata lines only attach to the entry heading directly above them. Before, a `- **Work
     verb**:` line under an unrelated `###` heading or inside a commented example could fill an
     earlier entry's field.
4. `tests/inventory_hygiene.rs`: 6 tests. Each walker is checked against a fixture copied three
   times (root, worktree, nested clone). The shipped empty registry is read from `dist/` as data,
   plus the #431 reproduction and a mixed registry of live entries and examples. All 6 fail on the
   previous code.

## Decisions Made

- Skipping any directory with `.git`, rather than matching names like `.worktrees`. Worktrees can
  live anywhere, and submodules and nested clones are other repositories whose governance is not
  this project's work.
- Out of scope, as stated in #434: `straymark_core::architecture::collect_source_files` has the
  same blind spot. Fixing it needs a `core` bump, which also affects the CLI and Loom. Copies of
  `specs/` inside the main tree (for example, evidence snapshots) cannot be told apart
  structurally, so handling them would need an explicit exclude setting, gated on an adopter
  needing it.
- The CLI half of #431 (`straymark validate` warning on the template's example line) belongs to
  the CLI and is handled with the CLI follow-up work (#432).

## Impact

Measured read-only with the binaries built from `main` and from this branch (git status unchanged
in every repository):

| Corpus | Before | After |
|---|---|---|
| Estoa (local) | task 1946, batch 22, follow-up 6, duplicated IDs 1839 | task 79, batch 11, follow-up 5, duplicated IDs 0 |
| LNXDrive | follow-up 18 | follow-up 17 (template placeholder) |
| Sentinel | unchanged | unchanged (its one duplicated ID is an AILOG with two `### Batch 1` headings: adopter content) |

- **Performance / Security / Privacy / Environmental**: N/A. Still read-only.

## Verification

- [x] `cargo test -p straymark-baton --locked`: 90 passed (84 existing + 6 new)
- [x] The 6 new tests fail against the unfixed source
- [x] `cargo clippy -p straymark-baton --all-targets --locked -- -D warnings`
- [x] Read-only measurement on three adopter corpora
