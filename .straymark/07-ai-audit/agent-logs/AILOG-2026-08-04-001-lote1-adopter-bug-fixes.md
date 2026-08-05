---
id: AILOG-2026-08-04-001
title: Lote 1 adopter bug fixes — drift range, i18n ledger, amend scaffold, telemetry round, merge-reports signal
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
tags: [charter-audit, telemetry-schema, i18n, adopter-feedback, cli]
related: []
---

# AILOG: Lote 1 adopter bug fixes — drift range, i18n ledger, amend scaffold, telemetry round, merge-reports signal

## Summary

Remediation batch for five adopter-reported issues (GitHub #397, #389, #390,
#401, #402), all surfaced by the ri-ceiba/Sentinel audit cycles. Each fix is
small and localized; together they close the "the CLI contradicts itself"
class of defect: a two-dot drift range reporting the base branch's changes,
a ledger parser blind to the framework's own translated headings, a scaffold
that fails the CLI's own validator, a schema that refuses what the CLI emits,
and a silent field omission at merge time.

## Context

The open-issue triage of 2026-08-04 classified the recent `adopter-feedback`
issues into quick-fix and design buckets. This AILOG covers the quick-fix
batch ("Lote 1"). #398 (duplicate of #401) was closed in the same session
after code-level verification of the duplication.

## Actions Performed

1. **#397** — `dist/.straymark/hooks/pre-pr.sh` now invokes
   `charter drift --range "$UPSTREAM...HEAD"` (three-dot / merge-base) so the
   pre-push check reports only what the branch changed, not changes the base
   gained while the branch lived. The charter templates (en/es/zh-CN, dist +
   installed copies) now recommend the same range.
2. **#389** — `parse_batch_ledger` (cli/src/ailog.rs) locates the ledger via
   the ASCII `(Batch Ledger)` marker instead of an exact `## Batch Ledger`
   match, so the es (`## Bitácora por Lote (Batch Ledger)`) and zh-CN
   (`## 批次台账 (Batch Ledger)`) headings parse. All consumers
   (batch-complete, drift gate, audit) share the single helper.
3. **#390** — `charter amend` scaffold now emits the four META-001-required
   fields (`status: draft`, `created`, `agent`, `confidence`) so the
   generated AILOG passes `straymark validate`. Enum-valued fields carry
   valid values with inline comments prompting operator updates; the
   frontmatter parses with serde_yaml, so template-style bracketed
   placeholders were not an option.
4. **#401** — `charter-telemetry.schema.v0.json` (dist + installed copy)
   accepts optional `round` (string) on `external_audit` items; `$comment`
   records the v0.3 addendum. Added a regression test that validates the
   CLI's rendered `--merge-reports [--round]` YAML against the shipped
   schema, so the CLI can never again emit YAML its own schema refuses.
5. **#402** — `--merge-reports` now warns (stderr, naming file and auditor)
   when a validated report lacks `audit_quality`, instead of silently
   emitting a lame telemetry entry. Exit code unchanged: the field is
   optional in `audit-output.schema.v0.json` by design.

## Modified Files

| File | Change Description |
|------|--------------------|
| `cli/src/ailog.rs` | Batch Ledger heading lookup keyed on `(Batch Ledger)` marker + i18n tests |
| `cli/src/commands/charter/amend.rs` | scaffold emits META-001-complete frontmatter + regression test |
| `cli/src/commands/charter/audit.rs` | audit_quality absence warning + shipped-schema round-trip test |
| `cli/tests/charter_audit_test.rs` | integration test for the audit_quality warning |
| `dist/.straymark/hooks/pre-pr.sh` | two-dot → three-dot range (+ rationale comment) |
| `dist/.straymark/schemas/charter-telemetry.schema.v0.json` | optional `round` on external_audit items |
| `dist/.straymark/templates/charter/charter-template.md` (+ i18n es/zh-CN) | recommend three-dot drift range |

## Decisions Made

- **#402 — warn, don't reject.** `audit_quality` is declared optional in the
  report schema ("set when consolidating into telemetry"); a hard failure
  would block legitimate single-pass merges. A named warning preserves
  signal without new friction. Rejection was the considered alternative.
- **#390 — valid defaults over placeholders.** `agent: straymark-cli-amend`
  + `confidence: medium` (both flagged with inline comments) instead of
  template-style bracketed placeholders, because the scaffold must be
  immediately valid YAML *and* pass META-001/status/confidence enum checks.
- **#397 — scope held to drift.** The same two-dot shape exists in the
  `audit --prepare` diff range and `followups drift` defaults; those
  semantics differ (over-coverage vs mis-attribution) and are tracked
  separately below rather than folded in.

## Impact

- **Functionality**: adopters on es/zh-CN can use batch-complete; multi-round
  audits can close; amendments validate; pre-push drift stops misreporting.
- **Performance**: N/A
- **Security**: N/A
- **Privacy**: N/A
- **Environmental**: N/A

## Verification

- [x] Code compiles without errors
- [x] Tests pass — `cargo test` in `cli/`: all suites green except
  `audit_template_test::unified_template_has_seven_universal_sections`,
  which was verified to fail identically on the clean tree (pre-existing
  main drift: expects a Step 5 anchor the shipped audit-prompt.md no longer
  carries). All 5 new/updated tests pass.
- [x] Manual review performed
- [ ] Security scan passed (if risk_level: high/critical) — N/A (low)
- [ ] Privacy review completed (if handling PII) — N/A

## Risk

- R1 (known, accepted): pre-existing failing test on main
  (`audit_template_test` anchor drift) — not introduced by this change.
- R2 (new, not in Charter): `charter audit --prepare` and
  `followups drift` defaults still use two-dot `origin/main..HEAD` for
  `git diff`; the same base-advancement noise applies. Evaluate whether the
  semantics want three-dot there too (audit diffs would shrink to
  branch-only coverage).
- R3 (new, not in Charter): `.straymark/` installed copies (schemas,
  templates, hooks) are gitignored in this repo and were synced manually
  from `dist/`; future `update-framework` runs will reconcile them.

## Follow-ups

- (new) Audit whether `charter audit --prepare` and `followups drift`
  should adopt three-dot ranges too — companion of GH #397 (see R2).
- (new) Fix the pre-existing `audit_template_test` anchor drift on main
  (see R1) — either update the test's expected anchors or restore the
  Step 5 calibration section in `dist/.straymark/audit-prompts/audit-prompt.md`.

---

<!-- AILOG generated by qodercli-v1.0 | StrayMark | https://strangedays.tech -->
