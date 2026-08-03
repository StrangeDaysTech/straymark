---
id: AILOG-2026-08-02-001
title: Baton — Track C adopter kit (forward-validation handoff, post-graduation)
status: accepted
created: 2026-08-02
agent: qoder
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 175
files_modified: [experiment-baton/07-track-c-adopter-kit.md, experiment-baton/05-adopter-test-plan.md, experiment-baton/PLAN-avance-post-calibracion.md]
observability_scope: none
work_verb: operate
design_provenance: upstream
tags: [baton, track-c, forward-validation, adopter, work-verb]
related: [AILOG-2026-06-27-001, 06-work-verb-schema-ratification, PLAN-avance-post-calibracion]
---

# AILOG: Baton — Track C adopter kit (forward-validation handoff)

## Summary

Produces the adopter-facing handoff kit for **Track C** (forward-validation), now
unblocked because its only dependency — Track A, schema graduation — shipped in
**fw 4.38.0 / cli 3.40.0** (2026-07-29). New document
[`07-track-c-adopter-kit.md`](07-track-c-adopter-kit.md); the pre-graduation plan
[`05-adopter-test-plan.md`](05-adopter-test-plan.md) gets a supersession banner;
the post-calibration plan marks Track A done and links the kit from Track C.

## Why

The 05 plan predates decision #332: it asks adopters to run the E1/E2/E3 trio
against the old title-scan classifier. After the declared-`work_verb` turn
(AILOG-2026-06-27-001) and ratification (06), two of the three experiments are
obsolete by design:

- **E2** (signal enrichment to raise confidence) — the signal is now the declared
  verb itself; nothing to enrich.
- **E3** (real costs) — the post-calibration plan explicitly says illustrative
  costs suffice for forward-validation.

And **E1 changes question**: with classification deterministic over the declared
verb (classify.rs: declared → High, undeclared → unclassifiable/frontier), the
oracle question is no longer "does the classifier predict right" but **"do authors
declare the verb correctly in production"** — which is exactly the forward-validation
gap #332 step 3 assigned to StrayMark.

## What changed

- **`07-track-c-adopter-kit.md` (new)** — the adopter handoff: preconditions
  (fw ≥ 4.38.0), declaration placement table, vocabulary + the three determination
  rules (foundational-contract = implement; upstream degrade; non-work = operate),
  the simplified E1 (sample 20–30 declared units, retrospective true_verb /
  true_provenance labeling, agreement ≥ 0.8 target, error-direction watch), the
  friction questions (vocabulary coverage, provenance usage, undeclared fraction),
  the explicit "what we do NOT ask" list, the Track C done-criterion, and the
  read-only guarantees.
- **`05-adopter-test-plan.md`** — supersession banner pointing to 07; preserved
  as the historical record of the pre-graduation calibration.
- **`PLAN-avance-post-calibracion.md`** — Track C header records that Track A
  shipped (fw 4.38.0) and links the kit.

## Verification

Docs-only change; no code touched. Cross-checked against the shipped sources:

- Vocabulary/placement/rules quoted from `06-work-verb-schema-ratification.md`
  (ratified 2026-06-27) — no drift.
- Field names/slots match the graduated dist templates
  (`charter-template.md`, `TEMPLATE-AILOG.md`, `follow-ups-backlog.md`).
- CLI behaviour described (declared → High; undeclared → frontier + nudge;
  `route` requires `--dry-run`) matches `src/classify.rs` and `src/main.rs`.
- Done-criterion restates the Track C gate from the plan (≥1 adopter, ≥ 0.8,
  no systematic downward errors).

## Scope boundary

This is the *kit*, not the validation itself: Track C now waits on adopter
production use (2–4 weeks of declared verbs) before the simplified E1 runs. No
backfill of the legacy corpus is requested — undeclared remains the honest state.

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no personal
data, no model inference. Read-only over the target tree.
