---
id: AILOG-2026-06-27-001
title: Baton — discontinue title-scan; declared work_verb as the sole classification signal
status: accepted
created: 2026-06-27
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 470
files_modified: [experiment-baton/src/signals.rs, experiment-baton/src/classify.rs, experiment-baton/src/route.rs, experiment-baton/src/telemetry.rs, experiment-baton/src/units.rs, experiment-baton/src/main.rs, experiment-baton/tests/dry_run_router.rs, experiment-baton/tests/fixtures/governance-corpus]
observability_scope: none
tags: [baton, phase2, classification, work-verb, title-scan, adopter-feedback]
related: [AILOG-2026-06-26-002, CHARTER-03-dry-run-router]
---

# AILOG: Baton — declared work_verb as the sole classification signal

## Summary

Implements decision **#332** (registered after the Sentinel adopter calibration,
#331): **discontinue title-scan** as a classification mechanism. The declared
**`work_verb`** (+ `design_provenance`), captured at authoring, becomes the sole
authoritative classification signal. A unit with no declared verb is
**unclassifiable** — routed up to frontier conservatively, with a nudge to declare
the verb — never guessed from the title.

## Why (the adopter evidence)

The Sentinel calibration (#331, E1/E2/E3) proved title-substring classification is
not just imprecise but **unsafe**: high+medium precision 0.57 with **4 errors
*downward*** (frontier work routed cheap), all from reading the *object noun /
incidental token* rather than the work verb — `interfaces/audit.go` (filename),
`(commit hash)`, "Audit remediation", "make test-live". The root cause is
structural, not tunable:

- **Object-vs-verb collisions are irreducible.** Sentinel has a module *named*
  `Audit`; "audit" as a domain noun (something you *implement*) is indistinguishable
  by keyword from "audit" as a verb (something you *do*).
- **No schema to scan against.** The title is unconstrained, schema-less, often
  bilingual free text. Reinforcing the scan per adopter is an unbounded keyword
  treadmill — the #321 "calibrated to one stack" anti-pattern.
- **Fake savings are worse than an honest gap.** Part of the scan's "saving" was
  false (critical work sent to a weak model). "Unknown → declare the verb" is the
  honest state.

## What changed

- **`signals.rs`** — removed the title-scan machinery (`Cue`, `scan_cues`,
  `CUE_TABLE`, `matches_at_word_start`). Added `WorkVerb`
  (design/implement/audit/operate) and `DesignProvenance` (new/upstream) with
  controlled-vocabulary parsing. `UnitSignals` carries the declared verb/provenance.
- **`classify.rs`** — `class: Option<TaskClass>`. Verb→class, with the
  residual-cognitive-load refinement: `implement` + `design_provenance=upstream`
  → `operator` (instruments prior design = mechanical). Undeclared → `None`. A
  declared verb is High confidence (the author knew it for free).
- **`route.rs`** — undeclared (`None`) → frontier (conservative default, R1); the
  high-risk Implementer escalation is retained.
- **`telemetry.rs`** — `conflict_fraction` (a title-scan artifact) replaced by
  `undeclared_fraction` (the actionable nudge metric). The saving now provably
  rests on declared units (`low_confidence_savings_fraction` → 0).
- **`units.rs`** — harvest `work_verb`/`design_provenance` from charter frontmatter
  (`read_frontmatter_yaml`) and from follow-up `- **Work verb**:` /
  `- **Design provenance**:` lines. Batch/Task have no declaration slot in the
  prototype → undeclared.
- **`main.rs`** — `classify`/`route` rendering for `Option<class>` + the undeclared
  nudge.

## Verification

- `cargo test -p straymark-baton` ✓ — full suite rewritten for the new model
  (declared-verb mapping, upstream→operator degrade, undeclared→frontier, telemetry
  undeclared metric); `cargo clippy` clean.
- **Sentinel dogfood** (read-only, `git status` intact): the legacy corpus declares
  no verbs → **100% undeclared → all routed to frontier → 0 saving → "not routable",
  with the "declare a work_verb" nudge.** That is the truthful clean-cut behaviour:
  no fake savings; the gap is an action, not a number.

## Scope boundary

This is the **Baton-side prototype** (self-contained in the experiment), de-risking
before graduation. Deferred, on purpose:

- **Framework graduation** — `work_verb`/`design_provenance` as first-class
  governance fields (schemas, templates, validator nudge, the authoring-time
  discipline) is a `fw-X.Y.Z` change, gated on schema ratification (the adopter
  explicitly asked not to instrument against an unratified schema).
- **Finer-grained declaration** — how batch/task units declare a verb (no
  frontmatter slot yet) is part of the framework design.
- **Forward-validation** — do real-world authors declare verbs correctly? — is
  StrayMark's job post-adoption over a varied corpus (#332).
- **B2 auto-assist** — using the intent-provenance edges to *suggest*
  `design_provenance: upstream` when a unit only touches already-defined contracts.

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no personal
data, no model inference. Read-only over the target tree (NFR1).
