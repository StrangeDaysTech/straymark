---
last_scan: 2026-08-05
schema_version: v1
total_open: 3
total_promoted: 0
total_closed_in_session: 1
total_phase_blocked: 0
total_suspected_closed: 0
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-08-04-003
  - AILOG-2026-08-05-003
  - AILOG-2026-08-05-004
---

# Follow-ups Backlog

> Central registry of `§Follow-ups` and `R<N> (new, not in Charter)` entries across AILOGs.
> Maintained by `straymark followups drift --apply`; counters are CLI-owned.
> Convention: `.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` ·
> Schema: `.straymark/schemas/follow-ups-backlog.schema.v1.json`

<!--
Entry shape (v1 — optional fields marked):

### FU-NNN — <short description>
- **Origin**: AILOG-NNNN-NN-NN-NNN <pointer to source section>
- **Origin-class**: ex-ante-planning | testing | telemetry | staging | real-env-bug   (optional)
- **Status**: open | in-progress | suspected-closed | closed | superseded | promoted
- **Severity**: normal | blocking                                                     (optional)
- **Trigger**: ready | <calendar date> | when <X> | <other>
- **Destination**: chore | mini-charter | charter-replanning | operations | <charter-id> | <TDE id>
- **Cost**: <effort estimate>
- **Labels**: <free tags, comma-separated>                                            (optional)
- **Notes**: <free-form context>
-->

## Bucket: ready

### FU-001 — (new) Consider wiring the merge-driver setup into `straymark init` behind a prompt (see R2).
- **Origin**: AILOG-2026-08-04-003 §Follow-ups
- **Source-hash**: b455feec70c6
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-08-04.

### FU-002 — Verify `QWEN.md` and `.qwen/skills/` land through a real `straymark init` and a real `straymark update` once…
- **Origin**: AILOG-2026-08-05-003 §Follow-ups
- **Source-hash**: db86ed449fc7
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-08-05.

### FU-003 — Confirm which customization root Antigravity discovers (`.agent/` vs `.agents/`) before tagging `fw-4.42.0`, and flip…
- **Origin**: AILOG-2026-08-05-004 §Follow-ups
- **Source-hash**: 9055de213adb
- **Status**: closed
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-08-05. · [2026-08-06 · AILOG-2026-08-05-004] Resolved 2026-08-06: operator ran an interactive agy session against the probe project; agy listed BOTH straymark-probe-alias (.agent/) and straymark-probe-canonical (.agents/). .agent/ is a real alias — shipped channel unchanged. Headless 'agy -p' does not exercise workspace customization discovery.

### FU-004 — After the tag: verify on a real `straymark update` from fw-4.41.0 that both retired directories disappear and the…
- **Origin**: AILOG-2026-08-05-004 §Follow-ups
- **Source-hash**: 4a131ad2ba05
- **Status**: open
- **Trigger**: TBD
- **Destination**: TBD
- **Cost**: TBD
- **Notes**: Auto-appended by `straymark followups drift --apply` 2026-08-05.

## Bucket: time-triggered

## Bucket: charter-triggered

## Bucket: phase-blocked

## Bucket: operational
