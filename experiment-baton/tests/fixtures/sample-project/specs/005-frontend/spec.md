# Feature Spec 005 — Frontend Dashboard

## 6. Functional requirements

- **FR-010** MUST mostrar el Health Dashboard: estado por componente y métricas
  (latencia P95, error rate, CPU, memoria). Consume `GET /api/v1/services/{id}/health`.

Nota: este spec NO referencia la decisión PM-002 del spec 001 (el edge faltante
del caso #304).
