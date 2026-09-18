---
id: AILOG-2026-09-17-001
title: Baton task inheritance — maintainer review amendments (#430)
status: accepted
created: 2026-09-17
agent: claude-opus-5-1m
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: [human_ai_config, information_integrity]
iso_42001_clause: []
files_modified:
  - experiment-baton/src/units.rs
  - experiment-baton/src/main.rs
  - experiment-baton/tests/task_inheritance.rs
  - experiment-baton/06-work-verb-schema-ratification.md
  - experiment-baton/07-track-c-adopter-kit.md
  - .straymark/07-ai-audit/decisions/AIDEC-2026-09-13-001-baton-task-parent.md
observability_scope: none
tags: [baton, adopter-feedback, inheritance, review]
related:
  - 07-ai-audit/agent-logs/AILOG-2026-09-13-001-baton-task-inheritance.md
  - 07-ai-audit/decisions/AIDEC-2026-09-13-001-baton-task-parent.md
---

# AILOG: Baton task inheritance — maintainer review amendments (#430)

## Summary

Maintainer review of #430 (Estoa's fix for #427) found the implementation correct but two
behaviours worth changing before merge. Both are amended on the PR branch, on top of the
adopter's commit, so its authorship and its own AILOG stay intact.

## Context

Measured read-only against the three local adopter corpora (git status unchanged in each):
the original rule classified 86 Sentinel tasks (all of `007-usageguard`), 39 in Estoa and
none in LNXDrive. Sentinel shows the review concern directly: its specs 002–005 carry 2–8
Charters each (the charter-chain pattern), so under "exactly one parent" a spec loses its
task classification the moment it gains a second Charter — retroactively, for tasks already
done. Uniqueness never proved task ownership either: one Charter may cover part of `tasks.md`.

## Actions Performed

1. **Agreeing parents inherit.** Every Charter whose `originating_spec` resolves to the
   task's sibling `spec.md` is a candidate parent; the task inherits when all candidates
   declare the same `(work_verb, design_provenance)`. Any disagreement, or a candidate that
   declares nothing, keeps the task `undeclared`. Agreement is on the *declaration*, not on
   the class the current classifier derives from it.
2. **Parents read from raw frontmatter.** Candidate parents now come from
   `read_frontmatter_yaml`, not the typed parser: a Charter with an out-of-schema field
   (e.g. `effort_estimate: XXL`) still counts as a parent — neither hidden nor disabling
   inheritance project-wide. Only unreadable YAML (or a non-string `originating_spec`)
   leaves the parent inventory incomplete and disables inheritance, as before.
3. **Say why.** `classify` and `route` print a `note:` on stderr naming the Charters that
   disabled inheritance (new `units::task_inheritance_blockers`); JSON on stdout is unchanged.
4. Test harness `Drop` no longer unwraps (a panic while unwinding aborts the test binary).
5. AIDEC, ratification §3 note and Track C kit updated to the amended rule.

## Decisions Made

Recorded as an amendment in `AIDEC-2026-09-13-001` (option 2 refined to "all explicit parents
agree"), chosen by the maintainer in this session.

## Impact

- **Functionality**: more tasks inherit when a spec's Charters agree; none inherit through
  disagreement. On today's corpora the counts are unchanged (Sentinel's multi-Charter specs
  declare no verb; Estoa's specs have one Charter each).
- **Performance**: one raw-frontmatter read per Charter per inventory. N/A in practice.
- **Security / Privacy / Environmental**: N/A. Still read-only; `route` still requires `--dry-run`.

## Verification

- [x] `cargo test -p straymark-baton --locked`: 97 passed (13 in `task_inheritance.rs`,
      replacing the "even when they agree" case and adding typed-invalid and CLI-note cases)
- [x] `cargo clippy -p straymark-baton --all-targets --locked -- -D warnings`
- [x] Read-only re-measurement on Sentinel / Estoa / LNXDrive, git status unchanged
- [x] Manual review performed

## Additional Notes

Estoa's local corpus is inflated by a separate, pre-existing inventory bug (Baton walks nested
git worktrees under `.worktrees/`), handled in its own PR, not here.
