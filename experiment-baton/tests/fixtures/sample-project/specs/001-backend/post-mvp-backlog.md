# Post-MVP Backlog — Backend

## 2. Modelo de salud por componente

- **ID**: PM-002
- **Prioridad**: P1
- **Estado**: CERRADO (2026-04-24) — Opción C aplicada
- **Origen**: decisión registrada en AILOG-2026-04-24-006

**Descripción**: extiende el contrato de salud a per-componente (score + state
por componente) en un JSONB versionado v2 (`metrics` + `components`). Cada
componente expone `name`, `state`, `detail` — **no** métricas raw.
