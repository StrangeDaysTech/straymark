---
id: AILOG-2026-09-13-001
title: Baton task inheritance through an unambiguous Charter origin
status: review
created: 2026-09-13
agent: codex-cli-v0.154.0
confidence: high
review_required: true
risk_level: medium
work_verb: implement
design_provenance: new
eu_ai_act_risk: not_applicable
nist_genai_risks: [human_ai_config, information_integrity]
iso_42001_clause: []
observability_scope: none
tags: [baton, adopter-feedback, inheritance]
related: [07-ai-audit/decisions/AIDEC-2026-09-13-001-baton-task-parent.md]
---

# AILOG: Baton task inheritance

## Context and actions

Estoa's authorized adopter contribution (Discussion #426, issue #427) found the
ratified task inheritance absent on e83cc064. A synthetic test failed before the
change (3 failed, 5 passed). The fix reads the parent declaration using the existing
originating_spec relation, then feeds the unchanged classifier and router.

Scope: experiment-baton/src/units.rs, tests/task_inheritance.rs, the prototype status
note in the ratification and the current adopter kit. Batch placement documentation
now follows the ratified inline slot (#428); batch harvesting remains pending.
No versions, framework installation, execution policy or model dispatch changed.

## Decision and verification

The linked AIDEC records why multiple parents remain undeclared. The nine new tests
exercise inventory through routing and CLI, all verbs, provenance, filtered output,
missing and ambiguous parents, invalid declarations, out-of-project symlinks,
read-only behavior and mandatory --dry-run. One early test incorrectly expected
an auditor's default tier to be frontier; inspection showed economic and the test
was corrected, without changing routing policy. Final checks: `cargo test -p straymark-baton --locked --offline` passed 93 tests
(84 existing + 9 new), with no failed or ignored tests; zero-test binary/doc harnesses
are not counted. `cargo clippy -p straymark-baton --all-targets --locked --offline
-- -D warnings`, rustfmt on the new test file and `straymark validate` passed.
The latter reports existing warnings; full Charter schema validation in this clean
clone is unavailable because regenerable framework files are intentionally absent.
No second framework installation was created.

## Limits and pending review

This is repeatability of a deterministic implementation, not human calibration.
Risk escalation remains implementer-only and derives from follow-up severity;
Charter risk_level and Bridge findings do not feed that mechanism. The same-author
check here is not an independent audit. No human documentary approval is recorded.

## Follow-ups

Batch inventory/inheritance and placement implementation remain tracked in #428.
Multi-Charter task ownership and direct spec declaration require a defined mapping;
this patch conservatively leaves such tasks undeclared. Track C needs prospective
units and retrospective labels over 2–4 weeks; no concordance or real savings claimed.
