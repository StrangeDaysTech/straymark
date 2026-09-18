---
id: AILOG-2026-09-17-004
title: Baton batch inheritance — nearest declaration (#428)
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
  - experiment-baton/tests/batch_inheritance.rs
  - experiment-baton/tests/task_inheritance.rs
  - experiment-baton/06-work-verb-schema-ratification.md
  - experiment-baton/07-track-c-adopter-kit.md
observability_scope: none
tags: [baton, adopter-feedback, inheritance, track-c]
related:
  - 07-ai-audit/decisions/AIDEC-2026-09-17-001-baton-batch-parent-nearest-declaration.md
---

# AILOG: Baton batch inheritance (#428)

## Summary

Baton now reads batch declarations. A batch takes the nearest declaration: its own ledger line,
else its AILOG's frontmatter, else the Charters whose ledger that AILOG is, when they agree. Before,
every batch was `undeclared`, and the ratification and the Track C kit disagreed on where a batch
declares.

## Context

Estoa's #428. The ratification placed batch verbs on a ledger line, while the framework's AILOG
template and the Track C kit used the AILOG frontmatter, and `read_batches` read neither. This
follows the task inheritance of #430 and the inventory hygiene of #435, and reuses both: the
Charter parent index and the comment-aware scanning.

## Actions Performed

1. The parent-declaration code moves into its own section of `units.rs`, shared by tasks and
   batches:
   - `CharterOrigin` gains `ledger_ailog`, resolved as `charter batch-complete` does
     (`originating_ailogs[0]`, else `execution_ailogs[0]`); a link field that is not a list of ids
     counts as unreadable;
   - `ailog_key` compares ids on the CLI's five-segment key;
   - `agreed` is the all-parents-agree rule, now used by both granularities.
2. `read_batches` scans live ledger text only (HTML comments, fences and other sections are
   ignored) and collects each batch's own `Work verb` / `Design provenance` lines. An override
   replaces the parent's declaration as a whole. `batch_parent` resolves the AILOG frontmatter,
   else the agreeing ledger Charters; an unreadable AILOG frontmatter blocks inheritance.
3. `task_inheritance_blockers` → `inheritance_blockers`. The CLI note now covers `--granularity
   batch` and says "inheritance from Charters disabled"; the #430 test assertion follows.
4. `tests/batch_inheritance.rs` has 9 cases: the #428 reproduction, both ledger links, non-ledger
   and `…-001b` links, agreement, AILOG-over-Charter precedence, whole-declaration overrides,
   examples and other sections, an unreadable AILOG and an unreadable Charter. 8 of them fail
   before the change; the ninth is a negative case.
5. Ratification §3 note and kit §2 table describe the one contract.

## Decisions Made

`AIDEC-2026-09-17-001`: the maintainer chose "nearest declaration" over ledger-only (which would
leave the AILOG slot inert) and over any Charter mentioning the AILOG (which would make motivating
Charters into parents).

## Impact

Measured read-only (git status unchanged in each corpus). Batches classified, previously all
`undeclared`:

| Corpus | Classified batches | Source of the declaration |
|---|---|---|
| Sentinel | 22 | Charters 54 (`operate`) and 55 (`implement/new`), plus two others, through their ledger AILOGs |
| LNXDrive | 7 | Charter 02 through its ledger AILOG |
| Estoa | 8 | AILOG frontmatter |

Spot-checked against the source AILOGs and Charters. **Performance / Security / Privacy /
Environmental**: N/A; still read-only.

## Verification

- [x] `cargo test -p straymark-baton --locked`: 112 passed
- [x] `cargo clippy -p straymark-baton --all-targets --locked -- -D warnings`
- [x] Read-only measurement on three adopter corpora
