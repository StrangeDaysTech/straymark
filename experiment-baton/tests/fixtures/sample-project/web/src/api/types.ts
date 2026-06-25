// Health types — consumes GET /api/v1/services/{id}/health.
// NOTE (fixture): this encodes the *assumed* contract of issue #304 — wrong
// field names, wrong enum values, and per-component metrics the backend never
// modeled. B3's coherence engine flags the mismatch against the Go producer.

export type HealthStatus = 'GREEN' | 'YELLOW' | 'RED' | 'UNKNOWN';

export interface ComponentHealth {
  name: string;
  status: HealthStatus;
  latency_p95_ms?: number;
  cpu?: number;
  memory?: number;
}
