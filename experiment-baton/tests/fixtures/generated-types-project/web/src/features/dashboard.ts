// Call sites that bind a generated type to its route — the keying source for
// #313. The typed client adds the `/api/v1` base, so paths here are relative and
// use `${...}` interpolation for params.
import { api } from '@/api/client';
import type { DashboardHealth, ServiceRow } from '@/api/types.gen';

export const fetchHealth = (serviceId: string) =>
  api.get<DashboardHealth>(`/services/${serviceId}/health`);

export const fetchServices = () => api.get<ServiceRow[]>('/services');
