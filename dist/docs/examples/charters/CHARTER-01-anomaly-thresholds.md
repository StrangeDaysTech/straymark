---
charter_id: CHARTER-01-anomaly-thresholds
status: closed
effort_estimate: M
trigger: "First false-positive anomaly ticket from a service with irregular traffic"
originating_ailogs: [AILOG-2026-01-15-001]
note: "Anonymized example derived from Sentinel PLAN-05 (per-service-anomaly-thresholds). See straymark-cli-roadmap.md §3.1 for the porting context."
closed_at: "2026-01-28"
---

# Charter: Per-service anomaly threshold overrides via PolicyEngine

> **Status (mirrored from frontmatter — source of truth is above):** closed. Effort: M (~1.5h).
>
> **Origin:** follow-up of AILOG-2026-01-15-001 ([upstream issue] anomaly detector). Forked from a "large features" backlog as Feature 1.

<!-- Anonymized example derived from Sentinel PLAN-05.
     Format conventions match charter-template.md (6 conventions distilled from the
     Sentinel /plan-audit experiment). Structural conventions preserved verbatim;
     identifiers anonymized. See straymark-cli-roadmap.md §3.1. -->

## Context

Today `AnomalyDetectorConfig` (DeviationFactor 3σ, CriticalFactor 5σ, MinSamples 7, ZeroActivityMinAvg 1.0) is **global** — passed to the detector via DI wiring at boot, immutable at runtime. The originating issue documented that services with irregular traffic (batch jobs, cron-driven workloads) generate false positives: a job that runs 1× every 4 hours has very high `StdDevRPM` in its activity bucket, and `DeviationFactor=3.0` flags it as anomalous when it is expected behavior.

The solution is per-service overrides stored in `policies.data.anomaly_thresholds`. The `policies` table already supports service-specific policies (the `system` row is default; rows with `policy_id = "service:<id>"` override). The detector already has a parallel pattern we copy: `s.policy.GetHealthProfile(ctx, hb.ServiceID)` is called per-heartbeat for layering of `HealthProfile`. This Charter adds `GetAnomalyThresholds` symmetrically.

Central trade-off: the detector acquires a dependency on `PolicyQuerier` (today it has none — `cfg` is baked in at construction). Mitigation: if `PolicyQuerier.GetAnomalyThresholds` fails, the detector uses the global default without aborting — **fail-open toward pre-Charter behavior**.

## Scope

**In scope:**

1. New type `interfaces.AnomalyThresholds` in `src/core/interfaces/policy.<ext>` with the 4 overrideable fields (`DeviationFactor`, `CriticalFactor`, `MinSamples`, `ZeroActivityMinAvg`). `Enabled` is not exposed — that flag stays global (operational kill-switch, not per-service).
2. Extend `policy.PolicyData` with `AnomalyThresholds *interfaces.AnomalyThresholds` (`omitempty`).
3. Extend the `interfaces.PolicyQuerier` interface with `GetAnomalyThresholds(ctx, serviceID) (*AnomalyThresholds, error)`. Implementation in `PolicyEvaluator` (mimics `ResolveHealthProfile`).
4. Extend `monitor.StubPolicyQuerier.GetAnomalyThresholds` returning `nil, nil` (no override) — existing tests are not affected.
5. `AnomalyDetector` gains a new field `policy interfaces.PolicyQuerier` (may be nil → pre-Charter behavior). In `Evaluate`, before `classifyAnomaly`, resolve effective config: start with `d.cfg`, layer overrides from `s.policy.GetAnomalyThresholds(ctx, hb.ServiceID)` when not nil. If lookup fails, log warn and use global cfg.
6. `policy` Service: new method `SetAnomalyThresholds(ctx, caller, serviceID, thresholds *AnomalyThresholds)`. SUPER_ADMIN guard. Passing `nil` un-sets the override and reverts to global. Repository upserts a service-specific policy row.
7. `policy` Handler: `PUT /api/v1/services/{service_id}/anomaly-thresholds` with body `{deviation_factor?, critical_factor?, min_samples?, zero_activity_min_avg?}`. Empty body `{}` un-sets override. Validation: positive floats, `critical_factor >= deviation_factor`, `min_samples >= 1`. Audit event `policy.anomaly_thresholds.changed` with `{service_id, previous, current, changed_by}`.
8. AuditTrail wiring: add topic to `consumer.auditTopics` + classification rule `WARNING`.
9. Tests:
   - Unit handler: SUPER_ADMIN guard, body validation (negative floats rejected, critical < deviation rejected), happy path set + unset.
   - Unit service: `SetAnomalyThresholds` validates + persists + publishes event; previous/current correct.
   - Unit anomaly_detector: with `PolicyQuerier` returning override, the detector uses overridden thresholds; with `nil` policy or policy returning `nil, nil`, uses global cfg; with error in GetAnomalyThresholds, fail-open to global cfg.
   - Integration: round-trip (request → DB → audit_records with WARNING) + verifies that an override changes the detector behavior end-to-end.

**Out of scope:**

- Per-tenant (cross-service) overrides — only per-service.
- Override of the `Enabled` flag per-service — stays global as operational kill-switch.
- Client UI/CLI — pure REST, curl works.
- DEVOPS role-relaxation for the endpoint — deferred to a follow-up Charter if real tickets request the access.
- Automatic discovery of "override candidate" services based on historical metrics — deferred.
- Formal schema validation (JSON Schema) of `policy_data.anomaly_thresholds` — deferred to a follow-up.

## Files to modify

| File | Change |
|---|---|
| `src/core/interfaces/policy.<ext>` | New `AnomalyThresholds` type + new method on `PolicyQuerier`. |
| `src/services/policy/models.<ext>` | `PolicyData.AnomalyThresholds` + `AnomalyThresholdsChangedData` payload. |
| `src/services/policy/evaluator.<ext>` | `ResolveAnomalyThresholds` method (mimics `ResolveHealthProfile`). |
| `src/services/policy/service.<ext>` | Implement `GetAnomalyThresholds` + new `SetAnomalyThresholds` method with SUPER_ADMIN guard + publish event + cache invalidate. Persists via `GetServicePolicy + CreatePolicyVersion` (read-modify-write); does not require new repository method. |
| `src/services/policy/handler_privacy.<ext>` | New handler `setAnomalyThresholds` + structs + route `PUT /api/v1/services/{service_id}/anomaly-thresholds`. |
| `src/services/policy/handler_test.<ext>` | Handler tests (4): success set, success unset, unauthenticated 401, invalid body 422. |
| `src/services/policy/service_test.<ext>` | Service tests (5): SUPER_ADMIN guard rejects Admin/Devops, valid set, valid unset, validation rejects critical<deviation, event published with previous/current. |
| `src/services/policy/evaluator_test.<ext>` | Resolver tests (3): system policy → nil thresholds; service policy with thresholds → returns those; service policy without AnomalyThresholds → fallback to nil. |
| `src/services/monitor/anomaly_detector.<ext>` | `AnomalyDetector` gains `policy interfaces.PolicyQuerier`. `Evaluate` resolves effective config layering global cfg + overrides. Fail-open on error. |
| `src/services/monitor/anomaly_detector_test.<ext>` | Tests (4): nil policy → uses global; policy with override → uses override; policy with nil thresholds → uses global; policy returns error → log + global. |
| `src/services/monitor/stub_policy.<ext>` | `GetAnomalyThresholds` added (returns `nil, nil` by default). |
| `src/main.<ext>` (DI wiring) | Regenerate or update DI graph. The injection of `policy` to `NewAnomalyDetector` happens here; the monitor service file does not need changes. |
| `src/services/audit/consumer.<ext>` | Add `"policy.anomaly_thresholds.changed"` to `auditTopics`. |
| `src/services/audit/classifier.<ext>` | WARNING rule for the new topic. |
| `src/integration/integration_test.<ext>` | End-to-end integration test: PUT request → policy row updated → heartbeat with irregular traffic → with override does not flag anomaly, without override does. |
| `specs/contracts/events.md` | New event entry `policy.anomaly_thresholds.changed` + subscription matrix. |
| `.straymark/07-ai-audit/agent-logs/AILOG-...md` | New, `risk_level: medium` (admin endpoint + changes detector inner loop). |

## Verification

### Local checks

Commands executable literal in clean shell — include explicit setup of dependencies.

```bash
<build-command>                                # e.g. cargo build, go build, npm run build
<test-command-scoped> src/services/policy/ src/services/monitor/ src/services/audit/

# Explicit setup for security/vulnerability scanners
# (Pattern: implicit PATH lookups generate false-positive 'real_debt' from external auditors.)
<install-and-run-security-scanner>
<install-and-run-vulnerability-scanner>

# Integration with testcontainers (~3 min):
<integration-test-runner> -run 'TestIntegration_AnomalyThresholds' src/integration/
```

### Production smoke (after deploy)

Commands that only apply after deploy to a real environment. External auditors skip this section.

```bash
# Set override for a service with known irregular traffic.
TOKEN="$(<auth-cli> print-identity-token)"
curl -X PUT "https://${SERVICE_HOST}/api/v1/services/${SVC_ID}/anomaly-thresholds" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"deviation_factor": 5.0, "critical_factor": 8.0, "min_samples": 14}'

# Verify audit_records persisted under WARNING.
<production-db-cli> connect <service-db> -- \
  -c "SELECT context FROM audit_records \
      WHERE action='policy.anomaly_thresholds.changed' \
      ORDER BY timestamp DESC LIMIT 1"

# Verify that anomaly_evaluations_total{outcome="critical"} for that
# service falls to 0 in the next 24h with the override applied.
```

## Risks

- **R1 — `PolicyQuerier.GetAnomalyThresholds` fails at runtime and blocks heartbeats**: today `s.anomaly.Evaluate` is best-effort (does not block ProcessHeartbeat). The new lookup runs inside Evaluate, so a PolicyQuerier failure must not propagate. Mitigation: `Evaluate` wraps the lookup in a wrapper returning `(nil, error)`; on error, log warn + use `d.cfg` (fail-open).

- **R2 — Stale PolicyEngine cache serves old thresholds for 30s post-update**: `ConfigCache` has TTL 30s. After `PUT /anomaly-thresholds`, the AnomalyDetector may keep using prior thresholds for up to 30s. Mitigation: invalidate cache explicitly in `SetAnomalyThresholds` (`s.cache.Invalidate(serviceID)` already exists). Integration test verifies new threshold applies immediately.

- **R3 — Cross-validation `critical_factor >= deviation_factor` not applied on read**: if an operator writes directly to the DB with an invalid value (bypassing the endpoint), the detector would use bad thresholds. Mitigation: not defensive — direct DB writes are operator responsibility. Document in struct comment. Validation happens at API boundary only.

- **R4 — `AnomalyDetector.Evaluate` loses performance with one extra round-trip to PolicyEngine per heartbeat**: each heartbeat today does 1 baseline query; this Charter adds 1 thresholds query. Mitigation: `ConfigCache` has TTL 30s — most heartbeats hit cache. Verify with benchmark that p95 of `health_evaluations_duration_ms` does not exceed 10ms (still dominated by baseline lookup, not threshold lookup).

- **R5 — DI graph circularity after adding `policy` to AnomalyDetector**: today the monitor service receives `policy` in `NewService`; AnomalyDetector lives within the monitor. Passing `policy` from `NewService` to `NewAnomalyDetector` is direct, introduces no cycle. Verify after DI regeneration.

## Tasks

1. Sync main, branch `post-mvp/per-service-anomaly-thresholds`.
2. `interfaces/policy.<ext>`: add `AnomalyThresholds` struct + extend `PolicyQuerier` interface.
3. `policy/models.<ext>`: extend `PolicyData` with `AnomalyThresholds` + `AnomalyThresholdsChangedData` payload.
4. `policy/evaluator.<ext>`: `ResolveAnomalyThresholds`.
5. `policy/service.<ext>`: `GetAnomalyThresholds` + `SetAnomalyThresholds` with guard + publish + cache invalidate (read-modify-write via `GetServicePolicy + CreatePolicyVersion`).
6. `policy/handler_privacy.<ext>`: new handler + route + structs.
7. `audit`: topic + WARNING rule.
8. `monitor/stub_policy.<ext>`: stub method (returns nil, nil).
9. `monitor/anomaly_detector.<ext>`: new field `policy` + resolve effective config in `Evaluate`. Fail-open on error.
10. `src/main.<ext>` (DI wiring): regenerate / update so DI passes `policy` to the AnomalyDetector constructor directly.
11. Unit tests (handler + service + evaluator + anomaly_detector + audit).
12. Integration test (round-trip + behavioral verification).
13. `events.md` bump + subscription matrix.
14. AILOG (`risk_level: medium`, `review_required: true` because it is an admin endpoint).
15. Local verification passes clean.
16. Commit + push + open PR.

## Charter Closure

Closed post-merge. The implementation introduced 3 emergent risks (R6, R7, R8) documented in the AILOG, plus two drifts caught only by external multi-model audit (F4: forgotten evaluator tests; F5: hallucinated injection point). The retroactive update of this Charter file is itself the canonical example of why the format includes a Charter Closure section. The drift-check tooling (Phase 2 of the CLI roadmap) is designed to catch the F4/F5-class drifts before commit, not after audit.

---

<!--
Architectural decisions made during planning:

- Thresholds live in `policies.data` (JSONB), NOT in `service_configs.data`.
  Reason: policies already host HealthProfile/Quotas/RateLimits with the same
  override shape; service_configs is webhook/branding.

- Endpoint lives in the policy module, NOT in identity.
  Reason: gcp-resource is in identity because the `services` table is identity's.
  Anomaly thresholds modify PolicyData. Follows the privacy profiles pattern.

- AnomalyDetector keeps `cfg AnomalyDetectorConfig` as GLOBAL DEFAULTS.
  Reason: backwards-compat with existing tests; override is optional;
  old tests keep passing with `nil` policy.

- Fail-open on PolicyQuerier lookup error.
  Reason: anomaly evaluation is best-effort today; a downed PolicyEngine
  must not degrade worse than pre-Charter.

- SUPER_ADMIN-only in v1.
  Reason: consistent with privacy profiles + safer default. DEVOPS expansion
  as explicit follow-up if tickets request it.

- Explicit cache invalidation in SetAnomalyThresholds.
  Reason: ConfigCache TTL of 30s would delay override application up to 30s.
  Immediate invalidation closes the gap.
-->
