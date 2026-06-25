# Feature Spec 005 — Frontend Dashboard

## 6. Functional requirements

- **FR-010** MUST mostrar el Health Dashboard: estado por componente y métricas
  (latencia P95, error rate, CPU, memoria). Consume `GET /api/v1/services/{id}/health`.

Nota: este spec asume la forma del contrato de salud del backend pero no enlaza
la decisión de backlog que lo definió (el edge faltante del caso #304).
