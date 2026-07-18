---
last_scan: YYYY-MM-DD
schema_version: v1
total_open: 0
total_promoted: 0
total_closed_in_session: 0
total_phase_blocked: 0
total_suspected_closed: 0
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs: []
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
- **Premise**: <the load-bearing assumption this entry rests on>                       (optional)
- **Verified-at**: <YYYY-MM-DD the premise was last re-checked against the code>       (optional)
- **Notes**: <free-form context>

An entry is a dated, decaying *hypothesis*, not an instruction. Its premise may
have been false at capture or gone stale since. Re-verify it at execution — the
cheap moment — not at capture: `straymark followups verify FU-NNN --verified`
(or `followups promote FU-NNN --premise-verified`). See the pattern doc's
"Epistemic status" section.
-->

## Bucket: ready

## Bucket: time-triggered

## Bucket: charter-triggered

## Bucket: phase-blocked

## Bucket: operational
