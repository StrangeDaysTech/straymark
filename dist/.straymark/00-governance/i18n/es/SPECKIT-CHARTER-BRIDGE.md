# Puente SpecKit ↔ Charter de StrayMark

> **Estado**: Patrón empírico (`v0`). Cristaliza tras validarse contra un segundo dominio (Principio #12). Refinar vía PRs cuando surjan nuevos casos de uso.

## El problema que este documento resuelve

[SpecKit](https://github.com/StrangeDaysTech/speckit) te da `spec.md`, `plan.md` y `tasks.md` para una feature. StrayMark te da Charters, AILOGs, AIDECs, ADRs. **Ningún documento canónico explicaba cuándo una feature de SpecKit debe producir un Charter, qué granularidad usar, quién dispara la creación, ni cuándo.** Reportado como el artefacto central del [issue #113](https://github.com/StrangeDaysTech/straymark/issues/113) — un *gap* de descubribilidad que llevaba a los agentes (Claude, Gemini, Copilot) a construir modelos mentales binarios (`SpecKit = planeación, StrayMark = audit-trail`) y a descartar silenciosamente la tercera capa (work-as-auditable-shippable-unit) donde viven los Charters.

Este archivo es la respuesta.

## Modelo mental

Tres capas, con *handoffs*:

| Capa | Vive en | Propósito | Dueño |
|------|---------|-----------|-------|
| **1. Especificación** | `specs/NNN-feature/{spec,plan,tasks,research,quickstart}.md` | Qué es la feature, por qué existe, cómo se implementará a nivel técnico. SpecKit produce esto vía `/speckit-specify` → `/speckit-plan` → `/speckit-tasks`. | Operador (con asistencia del agente). |
| **2. Unidad acotada de ejecución** | `.straymark/charters/NN-slug.md` | El contrato de un único corte enviable de la feature. Empareja el alcance ex-ante (archivos, riesgos, subset de tareas) con la telemetría ex-post (drift, audit, lecciones). | Operador declara el Charter; el agente ejecuta dentro del mismo. |
| **3. Traza de implementación** | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` (más AIDECs y ADRs cuando aplique) | Registro día-a-día de qué se hizo, por qué, con qué nivel de confianza. Cada AILOG referencia al Charter vía `originating_charter:` (o el Charter agrega los AILOGs vía `originating_ailogs:`). | El agente los crea mientras trabaja; el operador revisa. |

**El puente es el Charter.** Las specs son demasiado de alto nivel para hacer drift-check ("¿enviaste la spec?" no se puede contestar en un horizonte útil). Los AILOGs son demasiado de bajo nivel para enviar contra ellos ("¿enviaste este AILOG?" es la unidad equivocada). Los Charters están en la granularidad correcta: un contrato de alcance estable contra el que puedes auditar en días, no en meses.

## ¿Cuándo una feature de SpecKit produce un Charter?

Una feature de SpecKit debe producir **al menos un Charter** cuando *cualquiera* de las siguientes se cumple:

1. El `tasks.md` de la feature tiene **5 o más tareas** que no puedes completar en una sola sesión.
2. La feature abarca **2 o más fases de SpecKit** (Setup, Foundation, User Stories, Polish, etc.) que pretendes enviar juntas como una unidad.
3. El trabajo amerita una **auditoría externa** (revisión cross-modelo, cross-equipo) al cierre.
4. Quieres **telemetría medible** al cierre (effort estimate vs. real, conteo de drift, lecciones).

**No** debe producir un Charter cuando:

- La feature es lo suficientemente pequeña para enviarse en una sola sesión (<1 día, <5 tareas). Usa AILOGs solamente — el overhead del Charter excede la ganancia de auditabilidad.
- La feature es **puramente planeación** (sin código todavía). Espera hasta que exista `tasks.md`; el contrato del Charter necesita tareas concretas que enumerar.
- La feature es **mantenimiento** sin alcance planeado (ej. "arreglar bugs según aparezcan"). Para mantenimiento ad-hoc, los AILOGs son suficientes.

## Heurísticas de granularidad

Cuando una feature amerita Charters, elige granularidad por **unidad enviable**, no por unidad estructural. Concretamente:

### Heurística 1 — Un Charter por corte enviable

Si la feature tiene Fases (ej. el típico Foundation → US1 → US2 → US3 → Polish de SpecKit), el **primer Charter envuelve el corte de fundación** (todo lo que envía junto como `v0.1`). Charters subsecuentes envuelven cortes subsecuentes. *Effort estimate* **M** es el bucket mediano para un corte enviable; **L** para un corte de feature completa.

```
specs/001-peek-mvp-foundation/
├── spec.md
├── plan.md
└── tasks.md  →  CHARTER-01 (Foundation: T001-T012, effort M)
                  CHARTER-02 (peek MVP: T013-T044, effort L)
```

### Heurística 2 — NO por User Story

Las User Stories son demasiado granulares. Una US que toma 2-3 tareas pertenece *dentro* de un Charter, no como su propio Charter. Telemetría por US es ruido; telemetría por corte enviable es señal.

### Heurística 3 — NO por feature

Una feature que se envía en dos cortes (ej. MVP → polish) merece dos Charters, no uno. El contrato del Charter contra el que puedes hacer drift-check es "lo que envió este corte", no "lo que eventualmente construimos".

### Heurística 4 — Caso borde: ≥10 tareas en 4+ fases

Cuando una feature es excepcionalmente grande, un tercer Charter (o partir el corte de fundación en "scaffolding" + "core") puede estar justificado. Usa effort estimate **L** como tope; si estimarías **XL**, esa es señal de que la feature debe re-especificarse.

## Cronología de creación

```
/speckit-specify  → spec.md
/speckit-plan     → plan.md
/speckit-tasks    → tasks.md
                    ↓
                ┌────────────────────────────────────────┐
                │  ★ PUNTO DE DECLARACIÓN DEL CHARTER ★  │
                │                                        │
                │  Operador corre `straymark charter new`│
                │   --from-spec specs/NNN-feature/spec.md│
                │   --type <M|L>                         │
                │                                        │
                │  Status del Charter: declared          │
                │  → Operador llena scope, files, tasks  │
                │  → status: in-progress al ejecutar     │
                └────────────────────────────────────────┘
                    ↓
/speckit-implement  → tareas ejecutadas
                    → AILOGs creados (`originating_charter:` → Charter)
                    ↓
straymark charter drift CHARTER-NN  → check archivos-vs-commit
straymark charter audit CHARTER-NN  → auditoría externa (opcional)
straymark charter close CHARTER-NN  → telemetría, status: closed
```

**Invariante clave**: declara el Charter *antes* de que `/speckit-implement` arranque. El Charter es un contrato; declararlo después de la ejecución vacía el drift check.

## Vinculación en frontmatter

El frontmatter del Charter cita explícitamente la feature de SpecKit:

```yaml
charter_id: CHARTER-01-workspace-foundation
status: declared
effort_estimate: M
trigger: tasks.md tiene 12 tareas ordenadas en 2 fases; envíar como v0.1.
originating_spec: specs/001-peek-mvp-foundation/spec.md
```

La dirección inversa (spec → Charter) es por convención — lista el Charter activo en la sección "Phase 5: Implementation Tracking" de la spec si tu template de `plan.md` la tiene. SpecKit actualmente no tiene un slot de schema para esto; convención emergente.

Los AILOGs creados durante la ejecución deben citar al Charter:

```yaml
id: AILOG-2026-05-08-005
title: T013, T016-T026 — US1 P1 MVP core + TUI + peek bin
agent: claude-code-v4.7
confidence: high
risk_level: medium
review_required: false
originating_charter: CHARTER-02-peek-mvp-foundation
```

## Mapa del ciclo de vida

| Fase de SpecKit | Evento del Charter | CLI de StrayMark |
|-----------------|-------------------|------------------|
| `/speckit-tasks` completo | **Declarar Charter** | Skill `/straymark-charter-new` o `straymark charter new --from-spec …` |
| Primera tarea inicia | Operador cambia `declared` → `in-progress` | (edición manual de frontmatter) |
| Cada tarea ejecutada | AILOG producido (cuando lo amerite §6 de STRAYMARK.md) | `/straymark-ailog` |
| Decisión mayor encontrada | AIDEC producido | `/straymark-aidec` |
| Cambio arquitectónico | ADR producido | `/straymark-adr` |
| Última tarea hecha, antes de cerrar | Drift check | `straymark charter drift CHARTER-NN` |
| Revisión externa opcional | Auditoría multi-modelo | `straymark charter audit CHARTER-NN` + `/straymark-audit-prompt` + `/straymark-audit-execute` + `/straymark-audit-review` |
| Corte enviado | Cerrar Charter | `straymark charter close CHARTER-NN` (status: `closed`, telemetry yaml emitido) |

## Anti-patrones

**No abras un Charter "por si acaso".** Un Charter sin un corte enviable claro se convierte en una wishlist. El operador termina cerrándolo como `closed: aborted` y la telemetría no significa nada.

**No abras un Charter por User Story.** Telemetría por US es demasiado ruidosa para informar estimaciones futuras. Agrega.

**No omitas el campo `originating_spec`.** Aunque el Charter envuelva trabajo que no tiene una spec de SpecKit, define `originating_ailogs:` en su lugar. Charters sin origen son un anti-patrón (señalan motivación no documentada).

**No corras `straymark charter audit` sin las CLIs auditoras disponibles.** La auditoría es orchestration-only — `straymark` no llama a APIs de LLM. Si no tienes N CLIs auditoras listas, salta el paso; cierra el Charter sin auditoría externa.

**No cambies status a `closed` antes del drift check + yaml de telemetría.** `straymark charter close` hace ambos atómicamente; el cierre manual salta invariantes.

## Cuándo este patrón no aplica

Este puente asume un flujo de feature manejado por SpecKit con implementación multi-tarea y multi-sesión. No aplica a:

- **Features de una sola sesión** — usa AILOGs solamente.
- **Trabajo solo de arquitectura, sin implementación** (ej. "diseñar el siguiente schema") — usa ADRs.
- **Refactors puros sin nuevo comportamiento** — usa AILOGs + etiqueta con `refactor:`.
- **Respuesta a incidentes y hotfixes** — usa INC + AILOG.
- **Entregables sólo de cumplimiento** (ej. refresh trimestral del DPIA) — usa el doc type relevante directamente.

Si tu trabajo encaja en alguno de los anteriores, *no declares Charter*. El costo del Charter excede el valor cuando no hay corte enviable que envolver.

## Ver también

- `STRAYMARK.md` §6 (Cuándo Documentar) y §15 (Charters como unidades acotadas de trabajo)
- `.straymark/templates/charter/charter-template.md` — template declarativo
- `.straymark/templates/charter/charter-telemetry-template.yaml` — template de telemetría
- `.straymark/schemas/charter.schema.v0.json` — JSON Schema del frontmatter declarativo
- `.straymark/schemas/charter-telemetry.schema.v0.json` — JSON Schema de telemetría
- `.claude/skills/straymark-charter-new/SKILL.md` (y equivalentes Gemini / agnóstico)

> **Contexto empírico citado** (issue #113): Suite Rust CLI/TUI greenfield, onboarding de Claude Opus 4.7 vía los puntos de entrada canónicos (`STRAYMARK.md`, constitución del proyecto, checklist de `CLAUDE.md`, skills `/straymark-*` disponibles, `/straymark-status`). Los Charters fueron *eventualmente* adoptados (2 Charters: foundation + MVP) sólo tras prompt explícito del usuario — confirmando que el gap era sistémico, no específico de la sesión. Este documento elimina ese gap.

---

*Idiomas*: [English](../../SPECKIT-CHARTER-BRIDGE.md) | Español | [简体中文](../zh-CN/SPECKIT-CHARTER-BRIDGE.md)
