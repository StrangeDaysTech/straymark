---
id: AILOG-2026-09-18-001
title: core nested-checkout rule — every project-wide walker skips other checkouts (#434 follow-up)
status: accepted
created: 2026-09-18
agent: claude-opus-5-1m
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: [information_integrity]
iso_42001_clause: []
files_modified:
  - core/src/walk.rs
  - core/src/lib.rs
  - core/src/architecture/gather.rs
  - cli/src/tree_grep.rs
  - cli/src/analysis_engine.rs
  - experiment-baton/src/scan.rs
  - core/Cargo.toml, cli/Cargo.toml, experiment-loom/Cargo.toml, experiment-baton/Cargo.toml, Cargo.lock
  - CHANGELOG.md, experiment-loom/CHANGELOG.md, version tables x6
observability_scope: none
tags: [core, architecture, loom, baton, adopter-feedback]
related:
  - 07-ai-audit/agent-logs/AILOG-2026-09-17-002-baton-inventory-hygiene.md
---

# AILOG: core nested-checkout rule (#434 follow-up)

## Summary

#435 taught Baton's walkers to skip another checkout nested in the project. The same blind spot
existed in `straymark-core`'s source scanner and in two CLI walkers. The rule now lives once, in
`core::walk::is_nested_checkout`, and all of them use it. Released as core 0.10.1, CLI 3.49.2 and
Loom 0.7.1.

## Context

Demonstrated on an adopter repository with one linked worktree under `.worktrees/`. `straymark
architecture generate`, writing to a scratch `--out`, produced 15 components, one of them
`.worktrees` (glob `.worktrees/**`): a full copy of the repo, 5606 source files against 5235 in the
real tree. With this change the same command produces 14 components and no `.worktrees`. The
repository was read-only in both runs (git status unchanged).

## Actions Performed

1. `core/src/walk.rs`: `pub fn is_nested_checkout(dir)`. A sub-directory with its own `.git` (a
   file for a linked worktree or submodule, a directory for a nested clone) is another checkout.
2. `core::architecture::collect_source_files_with` skips such directories. This covers
   `architecture generate | sync | validate`, `status --where`, Loom's architecture view and
   Intent plane, and Baton's overlay.
3. CLI: `tree_grep::walk` (used by `followups verify --claims` and `analyze declared-vs-wired`) and
   `analysis_engine::walk_recursive` (`analyze`) use the same rule.
4. Baton: `scan::is_nested_checkout` is now a re-export of the `core` rule. Its behavior is
   unchanged.
5. Tests: `core::walk` itself, and one per walker. Each walker test uses a directory name that is
   not already excluded (the first `analyze` fixture used `vendor/`, which `analyze` already
   excludes, so it passed without the fix; it was moved to `third_party/`). All three walker tests
   fail with the guard removed.
6. Versions move atomically: core 0.10.0 → 0.10.1, and the `straymark-core` bound moves in the
   CLI, Loom **and** Baton Cargo.toml, per the lesson from #282. CLI 3.49.2, Loom 0.7.1, CHANGELOG
   and Loom CHANGELOG.

## Decisions Made

- **One rule, in `core`.** Walkers keep their own name lists, because those differ by purpose, but
  the "another checkout" rule is shared so that no tool can disagree with another about what the
  project contains.
- **Patch versions.** The change adds an API to core and fixes behavior; no public API breaks.
- Baton is not re-released: its behavior is identical, and the next Baton release will pick up the
  new dependency bound.

## Impact

- **Functionality**: architecture models, `status --where`, Loom and `analyze` / `verify --claims`
  no longer read another checkout as the project's code.
- **Performance**: one `stat` per visited directory. Negligible.
- **Security / Privacy / Environmental**: N/A.

## Verification

- [x] `cargo test --workspace`: 1041 passed
- [x] Clippy: core and Baton clean with `-D warnings`; Loom clean; CLI unchanged from the baseline
      (18/24)
- [x] Before/after `architecture generate` on the adopter repository, read-only
