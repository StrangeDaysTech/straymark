// AUTO-GENERATED — do not edit. All API types live in one file with no per-type
// endpoint anchors (the shape that collapses onto one coarse contract under
// nearest-anchor keying — issue #313). The route↔type binding lives at the call
// sites in ../features/dashboard.ts, not here.

export type Semaphore = 'GREEN' | 'RED' | 'UNKNOWN';

// Consumer of GET /api/v1/services/{id}/health — encodes the *assumed* contract:
// wrong enum (Semaphore vs the backend's HealthState) and a phantom `cpu` the
// producer never models.
export interface DashboardHealth {
  name: string;
  status: Semaphore;
  cpu?: number;
}

// Consumer of GET /api/v1/services — matches its producer exactly (no drift).
export interface ServiceRow {
  service_id: string;
  display_name: string;
}
