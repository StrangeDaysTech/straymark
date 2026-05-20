# Documentación para contribuidores

Recursos para personas que contribuyen a StrayMark — ya sea leyendo el código, escribiendo traducciones, proponiendo cambios, o tratando de entender el *por qué* detrás de la forma del framework.


---

## Conceptos

El material atemporal — leer esto para entender *qué* es StrayMark y *por qué* tiene la forma que tiene, antes de abrir un PR que toque la superficie del framework.

| Documento | Qué cubre |
|---|---|
| [`DESIGN-PRINCIPLES.md`](DESIGN-PRINCIPLES.md) | Los doce principios jerárquicos que gobiernan las decisiones de producto. Incluye anotaciones empíricas v0.2 de los ciclos de validación en los principios #6 (disciplina cognitiva), #9 (simplicidad) y #12 (velocidad = velocidad del aprendizaje). |
| [`WHAT-IS-A-CHARTER.md`](WHAT-IS-A-CHARTER.md) | Alcance conceptual del artefacto Charter: una declaración ex-ante de una unidad de trabajo con contrato de verificación y ancla de auditoría. Mapea la frontera entre lo que cubre el `plan.md` de GitHub SpecKit y lo que cubre un Charter de StrayMark — no son lo mismo. |

## Guías de flujo de trabajo

| Documento | Qué cubre |
|---|---|
| [`TRANSLATION-GUIDE.md`](/docs/contributors/TRANSLATION-GUIDE) | Reglas y convenciones para traducir la documentación de StrayMark a idiomas adicionales. Leer antes de enviar un PR que agregue o modifique un archivo de `i18n/`. *(Solo en inglés por ahora.)* |

## Propuestas históricas (archivadas)

Propuestas y roadmaps de la evolución del proyecto previa al CLI, preservadas como contexto — explican cómo emergió la forma actual. **No** se mantienen hacia adelante; la fuente canónica del comportamiento actual son el código, los schemas bajo `dist/.straymark/schemas/` y el CHANGELOG. Examinarlas en GitHub en [`docs/decisions/proposals/`](https://github.com/StrangeDaysTech/straymark/tree/main/docs/decisions/proposals):

| Archivo | Fecha del snapshot | Qué capturó |
|---|---|---|
| `2026-04-30-thesis-validation.md` | 2026-04-30 | Validación empírica de la tesis del producto contra seis ciclos de Sentinel (backend Go) — el cuerpo de evidencia que motivó las anotaciones v0.2 de `DESIGN-PRINCIPLES.md`. |
| `2026-04-30-charter-telemetry.md` | 2026-04-30 | Esquema de instrumentación de telemetría para observar la ejecución de Charters en proyectos reales. La versión normativa ahora vive en `dist/.straymark/schemas/charter-telemetry.schema.v0.json`. |
| `2026-05-03-cli-roadmap.md` | 2026-05-03 | Roadmap de implementación en tres fases del CLI Rust, con criterios de cierre. Las fases 1–3 ya están shippeadas en `cli-3.x`. |
| `2026-05-03-audit-skills-design.md` | 2026-05-03 | Diseño de las skills `/straymark-audit-prompt` y `/straymark-audit-review` como checkpoints humano-en-el-loop. Implementado en `fw-4.8.0`. |
| `2026-05-03-audit-skills-rollout.md` | 2026-05-03 | Plan operacional de rollout de las audit skills (criterios de gating, telemetría, shipping por fases). |
| `2026-05-04-audit-cli-flow.md` | 2026-05-04 | Rediseño del flujo de auditoría externa tras el primer encuentro empírico con un Charter L multi-commit (Sentinel CHARTER-07). Implementado en `cli-3.10+`. |

Los ADRs (registros de decisión arquitectónica) del código actual viven en GitHub en [`docs/decisions/`](https://github.com/StrangeDaysTech/straymark/tree/main/docs/decisions).

---

*Ver también: [`../adopters/`](../adopters/ADOPTION-GUIDE.md) para documentación dirigida a equipos que adoptan StrayMark en sus propios proyectos, incluyendo [`ADOPTION-GUIDE.md`](../adopters/ADOPTION-GUIDE.md), [`CLI-REFERENCE.md`](../adopters/CLI-REFERENCE.md) y [`WORKFLOWS.md`](../adopters/WORKFLOWS.md).*
