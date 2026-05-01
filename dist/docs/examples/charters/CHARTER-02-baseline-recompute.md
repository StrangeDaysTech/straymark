---
charter_id: CHARTER-02-baseline-recompute
status: closed
effort_estimate: XS
trigger: "Operator post-onboarding fast-track: anomaly dashboard shows correlated spikes after batch service onboarding"
originating_ailogs: [AILOG-2026-01-20-002]
note: "Anonymized example derived from Sentinel PLAN-06 (baseline-recompute-job). See devtrail-cli-roadmap.md §3.1 for the porting context."
closed_at: "2026-01-30"
---

# Charter: Baseline re-compute manual via admin endpoint

> **Status (mirrored from frontmatter — source of truth is above):** closed. Effort: XS (~30 min).
>
> **Origin:** follow-up of [upstream issue F7] (deferred from AILOG-2026-01-20-002 [upstream issue]).

<!-- Anonymized example derived from Sentinel PLAN-06.
     Format conventions match charter-template.md (6 conventions distilled from the
     Sentinel /plan-audit experiment). Structural conventions preserved verbatim;
     identifiers anonymized. See devtrail-cli-roadmap.md §3.1. -->

## Context

`StartActivityBaselineJob` runs `RefreshActivityBaselines` every 24 hours and naturally absorbs source changes (event-rate fallback → request-count cache authoritative). The daily refresh re-absorbs baseline bias in ~7 days (the SQL query window).

The operational gap: when a batch of services is onboarded to the MetricsPoller simultaneously — or when the dashboard `anomaly_evaluations_total{outcome="critical"}` shows spikes correlated with recent mapping changes — the operator has no way to force a re-compute and must wait for the next natural tick.

A prior issue left `RefreshActivityBaselines` exported in the Repository interface and the SQL idempotent via UPSERT (ON CONFLICT DO UPDATE). The only thing missing is a manual trigger via admin endpoint. Effort XS, high operational value for fast-track post-onboarding.

## Scope

**In scope:**

1. New method `Service.RefreshActivityBaselines(ctx, caller) (*BaselineRefreshResult, error)` that calls the existing Repository method, measures duration, publishes an audit event and returns metadata to the caller.
2. SUPER_ADMIN guard at the handler edge (same pattern as the policy-handler `requireSuperAdmin`). No DEVOPS-relax — consistent with the per-service-thresholds Charter.
3. New route `POST /api/v1/admin/baselines/refresh` (no path params; refresh is global by design, not per-service).
4. Audit event `monitor.baselines.refreshed` with payload `{caller, duration_ms, completed_at}` and classification `WARNING` (admin operation that affects the AnomalyDetector).
5. Unit tests handler (3): auth gate, happy path, transient repo error mapping.
6. Unit tests service (2): repo error propagation, event published with duration and caller.

**Out of scope:**

- Per-service refresh (filter `WHERE service_id=$1` in a new SQL query): the current use-case is post-onboarding batch, not point-fix per-service. If a concrete ticket emerges, a follow-up Charter is opened.
- Asynchrony (background job with status polling): typical refresh duration with real MVP data is <1s; a sync endpoint with a 5min timeout covers 99% of cases.
- Endpoint in the policy or identity module — the job lives in the monitor module (which owns the AnomalyDetector + `RefreshActivityBaselines`), so the handler also lives there.
- Dedicated OTel metric (`baseline_manual_refresh_duration_ms`): the audit event with `duration_ms` already covers operational observability. If an SLO need emerges, it can be added later.

## Files to modify

| File | Change |
|---|---|
| `src/services/monitor/service.<ext>` | Add method `RefreshActivityBaselines(ctx, caller) (*BaselineRefreshResult, error)` to the `Service` interface + impl in `serviceImpl`. New type `BaselineRefreshResult` with `DurationMS`, `StartedAt`, `CompletedAt`. |
| `src/services/monitor/handler.<ext>` | Private helper `requireSuperAdmin` (clone of the policy module pattern). New route `POST /api/v1/admin/baselines/refresh` registered in `RegisterRoutes` with handler `refreshBaselines`. |
| `src/services/monitor/handler_test.<ext>` | 3 tests: auth required (401), forbidden (403 without SUPER_ADMIN), happy path (200 + event published). |
| `src/services/monitor/service_test.<ext>` | 2 tests: repo error propagated without event published, happy path with event + correct payload. |
| `src/services/monitor/models.<ext>` | New types `BaselineRefreshedData` (event payload) + `BaselineRefreshResult` (Service response shape). |
| `src/services/audit/consumer.<ext>` | Add `"monitor.baselines.refreshed"` to `auditTopics`. |
| `src/services/audit/classifier.<ext>` | WARNING rule for the new topic. |
| `specs/contracts/events.md` | New event entry `monitor.baselines.refreshed` + subscription matrix. |
| `.devtrail/07-ai-audit/agent-logs/AILOG-...md` | New, `risk_level: low` (idempotent operation, RBAC SUPER_ADMIN, no schema changes). |

## Verification

### Local checks

Commands executable literal in clean shell — include explicit setup of dependencies.

```bash
<build-command>
<test-command-scoped> src/services/monitor/ src/services/audit/

# Explicit setup for security/vulnerability scanners
<install-and-run-security-scanner> src/services/monitor/ src/services/audit/
<install-and-run-vulnerability-scanner> src/services/monitor/ src/services/audit/
```

### Production smoke (after deploy)

Commands that only apply after deploy to a real environment. External auditors skip this section.

```bash
# Force refresh after onboarding a batch of services to the MetricsPoller.
TOKEN="$(<auth-cli> print-identity-token)"
curl -X POST "https://${SERVICE_HOST}/api/v1/admin/baselines/refresh" \
  -H "Authorization: Bearer $TOKEN"

# Verify audit_records persisted under WARNING.
<production-db-cli> connect <service-db> -- \
  -c "SELECT context FROM audit_records \
      WHERE action='monitor.baselines.refreshed' \
      ORDER BY timestamp DESC LIMIT 1"

# Verify activity_baselines.updated_at reflects the manual refresh.
<production-db-cli> connect <service-db> -- \
  -c "SELECT MAX(updated_at) FROM activity_baselines"
```

## Risks

- **R1 — Long refresh blocks the handler until timeout**: the global refresh scans heartbeats from the last 7 days. With 100+ services and current-month partitioning this may approach 30s. Mitigation: 5-minute timeout in the handler (matching the perTickTimeout used by `StartActivityBaselineJob`). If that timeout consistently approaches, GIN-index optimization or asynchrony emerge as follow-ups.

- **R2 — Concurrency with the daily job**: if the operator calls the endpoint while the daily ticker is mid-tick, both run `RefreshActivityBaselines` in parallel. Mitigation: the SQL is UPSERT-idempotent — the second one "wins" and leaves data consistent; PostgreSQL serializes UPSERTs by (service_id, day_of_week, hour_of_day). No data race. Double computation is the acceptable cost of a rare operation.

- **R3 — Privilege escalation**: any role other than SUPER_ADMIN might want this button. Mitigation: explicit SUPER_ADMIN gate in the handler (pattern from the prior Charter). If DEVOPS needs it in production, open a follow-up Charter with operational justification.

- **R4 — Noisy audit event**: if an automated script calls the endpoint every minute, `audit_records` inflate. Mitigation: cooldown is managed operationally (this is not an endpoint a script should hit regularly); the WARNING classification helps make abuse visible. If real abuse emerges, a later Charter adds caller-level cooldown.

## Tasks

1. Sync main, branch `post-mvp/baseline-recompute-job`.
2. `monitor/models.<ext>`: types `BaselineRefreshResult` (response) + `BaselineRefreshedData` (event payload).
3. `monitor/service.<ext>`: extend `Service` interface + impl `RefreshActivityBaselines`.
4. `monitor/handler.<ext>`: helper `requireSuperAdmin` + route `POST /api/v1/admin/baselines/refresh` + handler `refreshBaselines`.
5. `audit`: topic + WARNING rule.
6. `events.md` bump + subscription matrix.
7. Unit tests (handler 3 + service 2).
8. AILOG (`risk_level: low`, `review_required: false` — idempotent operation, no schema changes).
9. Local verification passes clean.
10. **Auto-checklist drift** (Phase 2 of the CLI roadmap): `devtrail charter drift CHARTER-02 <range>`. If it reports omissions, complete; if scope expansion, document in AILOG. Until Phase 2 ships, run Sentinel's `check-plan-drift.sh` manually.
11. Commit + push + open PR.

## Charter Closure

Closed post-merge. The implementation matched the Charter declaration with no major drift; the `effort_estimate: XS` was met within margin (~40 min real vs ~30 min estimate, 1.33x — within the noise band of XS work). Drift script reported 0 omissions and 0 scope expansions.

This Charter is a useful counter-example to CHARTER-01: small, well-bounded work where the format conventions add minimal overhead. The same template that supports an M-sized Charter with 5+ risks scales cleanly down to XS.

---

<!--
Architectural decisions made during planning:

- Global endpoint (not per-service). Reason: the originating issue documents the
  real use-case (batch onboarding fast-track), not point-fix per-service. Global
  refresh reuses existing SQL without new indexes or queries.

- Endpoint in the monitor module (not policy or identity).
  Reason: the job and Repository.RefreshActivityBaselines live here; the handler
  must live where the logic is.

- SUPER_ADMIN-only in v1. Reason: consistent with the per-service-thresholds
  Charter. DEVOPS expansion as explicit follow-up if real tickets request it.

- Sync (not async). Reason: typical refresh <1s, 5-min timeout covers worst case.
  Asynchrony adds complexity (status polling, job queue) without immediate
  operational value.

- Audit event with `WARNING` classification. Reason: parity with other admin-
  operation events — all admin operational changes the operator wants visible.
-->
