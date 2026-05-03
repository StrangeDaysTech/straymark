<!--
Anonymized example derived from a real AILOG that documented the
introduction of an anomaly detector to a Go backend service. This is
the AILOG referenced as `originating_ailogs` by CHARTER-01-anomaly-thresholds.md
in the same examples directory — the pair illustrates the canonical
"Charter as follow-up of an AILOG" pattern.

Sentinel-specific identifiers (module names, internal issue numbers,
PR refs, infrastructure hostnames, reviewer emails, dates) have been
replaced with generic placeholders. Technical reasoning, the Decision
section structure, the Risk numbering, and the Verification commands
are preserved verbatim per the example-anonymization rules in
devtrail-cli-roadmap.md §3.1.

For browsing only — `devtrail init` does not install this file. Adopters
who want a starting AILOG scaffold use `devtrail new --doc-type ailog`
against TEMPLATE-AILOG.md.
-->

---
id: AILOG-2026-01-15-001
title: "AnomalyDetector + Activity Baselines (statistical RPM anomaly detection)"
status: accepted
created: 2026-01-15
agent: claude-code-v1.0
confidence: high
review_required: true
reviewed_by: reviewer@example.com
reviewed_at: 2026-01-28
review_outcome: approved
risk_level: medium
eu_ai_act_risk: minimal
nist_genai_risks: [confabulation, information_integrity]
iso_42001_clause: [6, 8]
lines_changed: 450
files_modified:
  - db/migrations/008_activity_baselines_rls.sql
  - db/queries/monitor/baselines.sql
  - db/generated/baselines.sql.go
  - db/generated/models.go
  - src/core/config.<ext>
  - src/services/monitor/anomaly_detector.go
  - src/services/monitor/anomaly_detector_test.go
  - src/services/monitor/job_baseline.go
  - src/services/monitor/models.go
  - src/services/monitor/repository.go
  - src/services/monitor/service.go
  - src/services/monitor/wire.go
  - src/main.<ext>
  - src/main.<ext> (DI wiring)
  - .env.example
  - specs/001-service-mvp/post-mvp-backlog.md
observability_scope: event:status.anomaly_detected event:status.anomaly_critical
tags: [post-mvp, anomaly, statistics, rpm, feature]
related:
  - specs/001-service-mvp/post-mvp-backlog.md
  - specs/001-service-mvp/data-model.md
  - specs/001-service-mvp/contracts/events.md
---

# AILOG: AnomalyDetector + Activity Baselines

## Summary

Closes the upstream issue. Activates for the first time the `activity_baselines` table that existed as a skeleton since an earlier migration, and turns on statistical anomaly detection over the RPM reported by services in their heartbeats.

Components:

- **`AnomalyDetector`** (`src/services/monitor/anomaly_detector.go`): evaluates each heartbeat against the baseline for its `(service_id, day_of_week, hour_of_day)` bucket and publishes events `status.anomaly_detected` (z ≥ 3σ) and `status.anomaly_critical` (z ≥ 5σ). Stateless, nil-receiver-safe, best-effort (errors never block ingestion).
- **`StartActivityBaselineJob`** (`job_baseline.go`): daily goroutine that invokes `repo.RefreshActivityBaselines`, an UPSERT that recomputes `(avg_rpm, stddev_pop, sample_count)` per bucket over the last 7 days of heartbeats with `jsonb_typeof(checks->'rpm') = 'number'`.
- **Migration 008** (`db/migrations/008_activity_baselines_rls.sql`): closes two gaps detected when reviewing the original migration against `data-model.md §3.8` — missing RLS and missing `updated_at` column.
- **3 anomaly types**: `rpm_drop`, `rpm_spike`, `zero_activity`. All three modeled as string constants to avoid magic strings.
- **Configuration via env vars**: `ANOMALY_DETECTION_ENABLED`, `ANOMALY_DEVIATION_FACTOR`, `ANOMALY_CRITICAL_FACTOR`, `ANOMALY_MIN_SAMPLES`, `ANOMALY_ZERO_ACTIVITY_MIN_AVG`. Defaults 3.0 / 5.0 / 7 / 1.0.

## Context

Origin: `specs/001-service-mvp/data-model.md` (the `activity_baselines` table declared as "Future Phase"), `plan.md` (`AnomalyDetector` component), `contracts/events.md` (events `status.anomaly_detected` and `status.anomaly_critical` with schemas already defined). The post-MVP backlog marked the item as "define when prioritized"; the operator prioritized it today.

## Decision

### Data source assumption

The principal design decision was where the detector takes the "observed RPM" from. The service only persists heartbeats (presence, not volume); there is no MetricsPoller and no integration with a cloud monitoring backend (those live in a deferred upstream issue).

Agreed with the operator (2026-01-15): assume services report `rpm` as a key inside `IncomingHeartbeat.Checks` (which is `map[string]any`). Services that don't report it are exempt without error. This decision:

- Zero new infra (heartbeats already carry `checks` JSONB).
- Lets a future deferred upstream issue replace the source without touching the detector.
- False positives are impossible by design for silent services (no `rpm` reported → detector skips).

### Daily refresh vs incremental

Welford on-line (incremental per heartbeat) is more reactive but introduces race conditions and complicates the empty-first-boot case. Chose UPSERT daily over a rolling window:

```sql
INSERT INTO activity_baselines (...)
SELECT service_id, EXTRACT(DOW...), EXTRACT(HOUR...),
       AVG((checks->>'rpm')::numeric),
       COALESCE(STDDEV_POP(...), 0),
       COUNT(*)::INT, NOW()
FROM heartbeats
WHERE received_at >= NOW() - INTERVAL '7 days'
  AND jsonb_typeof(checks->'rpm') = 'number'
GROUP BY ...
ON CONFLICT (service_id, day_of_week, hour_of_day) DO UPDATE ...
```

`STDDEV_POP` (population stddev) instead of `STDDEV_SAMP` because we are describing observed historical behavior, not extrapolating to a larger population. It matters when `sample_count` is small (n=2 gives var=2 vs var=4).

`jsonb_typeof(checks->'rpm') = 'number'` filters non-numeric values before the `::numeric` cast, preventing a malformed value in one heartbeat from breaking the refresh for all services.

### Thresholds via env vars (not per-service)

`PolicyEngine` already exposes `RateLimits` per-service in `src/core/interfaces/policy.<ext>`, but extending it for anomaly thresholds expands surface without proven demand. Defaults of 3σ / 5σ are industry-standard (>99.7% / >99.99994% intervals); the operator can tune via env vars without recompiling. Per-service is a follow-up when there's signal of real use.

> *Note: this follow-up materialized later as CHARTER-01-anomaly-thresholds (see the Charter examples directory) — exactly the pattern of "AILOG observes a constraint, a Charter eventually addresses it" the framework is designed to capture.*

### `DurationMinutes = 0`

`AnomalyCriticalData.DurationMinutes` is in the event schema from `events.md`. Real tracking (how many minutes the anomaly has been ongoing) requires per-service in-memory state that survives between heartbeats and clears on return-to-normal. For the MVP version of this feature I left it at 0 with a TODO documented in this AILOG. The principal info travels in `status.anomaly_detected`, which does carry `DeviationFactor` and baseline.

### Migration 008

When inspecting the earlier migration against `data-model.md §3.8` I found two gaps:

1. The `activity_baselines` table did not have RLS, even though `service_id` is a pivot column and the doc lists it.
2. `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` was missing in the DDL even though the doc lists it.

Migration 008 closes both before the detector consumes the table in production. Idempotent via `ADD COLUMN IF NOT EXISTS`.

### Event schema preserved

`AnomalyDetectedData` and `AnomalyCriticalData` are modeled in `monitor/models.go` with the contract's exact field names (`current_value`, `baseline_avg`, `baseline_stddev`, etc.). This lets a future consumer (e.g., the audit module's universal consumer) deserialize without coupling to the package.

## Verification

```bash
sqlc generate                                              # OK (generates baselines.sql.go)
go build ./...                                             # OK
go vet ./...                                               # OK
go test -short ./src/services/monitor/...                  # 13 unit tests green (TestAnomalyDetector_*)
go test -short ./...                                       # full suite green
go test -tags=integration -run 'TestIntegration_RefreshActivityBaselines|TestIntegration_AnomalyDetectorEndToEnd' \
        -timeout 5m ./src/services/monitor/...             # 3/3 green
<security-scanner> --exclude-generated ./...               # 0 issues, 5 nosec
```

Unit coverage (`monitor/anomaly_detector_test.go`):

- Nil receiver no panic.
- Disabled cfg emits nothing, no lookup performed.
- No `rpm` in checks → silent skip (no lookup).
- `baseline == nil` → silent skip.
- `sample_count < min_samples` → silent skip.
- Lookup error → log warn, no panic, no events.
- Reading inside the envelope → no events.
- Spike (z=4) → `rpm_spike` only.
- Drop (z=-4) → `rpm_drop` only.
- Critical spike (z=10) → both events.
- `zero_activity` with historical avg > 1 → emits (even with stddev=0).
- `zero_activity` with avg < `ZeroActivityMinAvg` → skip.
- Bucket key correctly derived from `received_at` (DOW + hour).
- `extractRPM` accepts float64/int/int64/json.Number; rejects string/bool.

Integration coverage:

- 7 heartbeats across 7 distinct DOWs → 7 baseline rows with `sample_count=1` each; today's bucket has avg=100, stddev=0.
- 5 heartbeats in the same bucket → 1 baseline row with avg=100, stddev_pop=√50 ≈ 7.0711, sample_count=5.
- `ProcessHeartbeat` with a pinned baseline and anomalous rpm → both events reach the bus subscriber.

## Risk

- **R1 — False positives in seasonal services**: static thresholds (3σ/5σ) don't understand deploy ramps or maintenance windows. *Mitigation*: env vars allow trivial tuning; events go to the audit module with WARNING, not direct paging. Per-service is a follow-up.
- **R2 — Refresh cost over large N**: scanning 7 days × `jsonb_typeof()` × `STDDEV_POP` per (service, dow, hour) could become noticeable when N > 50 services and heartbeats are dense. *Mitigation*: the `jsonb_typeof(checks->'rpm') = 'number'` filter enables a future expression index. Documented as a follow-up in the backlog.
- **R3 — Heartbeats without RPM**: most MVP services don't report RPM today (only cpu/memory/latency/error_rate). The detector emits nothing for them, which is correct but invisible. Observability metric pending as follow-up.
- **R4 — RLS added late to the table**: migration 008 turns on RLS over a table that was open until today. Zero existing rows (the table wasn't being filled in MVP), so no risk of historical exposure.
- **R5 — Nil detector in legacy tests**: updated `newTestService` in `service_test.go` to pass `nil` as the detector, and added a test `TestAnomalyDetector_NilReceiverIsNoOp` that protects that path. Existing heartbeat tests stay green.

## Follow-ups

- **Metric `service.anomaly_evaluations_total{outcome}`** (skipped / detected / critical) — operational observability over how many heartbeats activate the detector vs how many it skips for missing RPM or baseline. Trivial to add.
- **Real `DurationMinutes`** — per-service in-memory tracking of the first anomalous heartbeat with cleanup on return-to-normal. Allows reporting real duration in `AnomalyCriticalData`.
- **Per-service thresholds via PolicyEngine config** — useful when a critical service (e.g., billing) needs stricter gates than the global default. *(This follow-up was eventually picked up as CHARTER-01-anomaly-thresholds, see the Charter examples directory.)*
- **Expression index on `heartbeats.checks`** — `CREATE INDEX ... ON heartbeats USING GIN ((checks->'rpm'))` only if the refresh cost becomes noticeable.
- **Switch to cloud-monitoring source** — when a deferred MetricsPoller upstream issue lands, the detector can consume RPM observed by the poller without changing its interfaces.

## Approval

**Approved**: 2026-01-28 by `reviewer@example.com`.

Approved retroactively as part of a housekeeping bulk-approval cycle. Code shipped to main on or before AILOG creation date; behavior validated by elapsed operation without incidents reported.
