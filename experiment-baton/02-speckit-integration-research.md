# Baton — Investigación: mecanismos de integración de SpecKit y el caso #304

> **Versión:** 1.0
> **Fecha:** 24 de junio de 2026
> **Rama:** `docs/baton-speckit-research-304`
> **Issue:** StrayMark #304 — *Cross-spec decision propagation* (filed from adopter Sentinel)
> **Relacionado:** #303 (cross-boundary contract verification en el audit prompt)
> **Alimenta:** la Fase 1 (Coherence Bridge) de [01-baton-concept.md](01-baton-concept.md)

Este documento resuelve la sub-decisión #3 de Baton ("investigar el sistema de hooks/extensiones de SpecKit") ampliándola, a tu pedido, a las **integraciones**. La investigación combina:

- **Grounding local autoritativo:** la instalación real de **SpecKit v0.11.3** en Sentinel (`/home/montfort/StrangeDaysTech/sentinel/.specify/`). Es la fuente de verdad de la mecánica.
- **Contexto del ecosistema:** documentación oficial y artículos (ver §6, *Fuentes*).
- **El caso #304:** un fallo real de deriva cross-spec en Sentinel que ancla por qué esto importa.

---

## 1. Hallazgo principal

SpecKit dejó de ser "tres comandos que generan markdown". En v0.11.x es una **plataforma con superficie de extensión de primera clase**: un ciclo de vida con **eventos enganchables (hooks)**, un sistema de **extensiones** (comandos + plantillas + config + hooks), un sistema de **integraciones** (qué agente de IA está conectado, instalado como *skills*), **workflows** con *review gates*, y una **memoria persistente de proyecto** (`.specify/memory/`).

**Conclusión accionable para Baton:** existen **tres costuras de integración** concretas para lograr "que StrayMark no ignore los diseños de SpecKit", y las tres habrían cazado el caso #304:

1. **Lectura** — ingerir `.specify/memory/` + los specs al grafo de gobernanza/arquitectura (el plano de *intención*).
2. **Activación** — registrar una **extensión StrayMark** con **hooks** en eventos del ciclo (`after_specify`, `after_plan`, `before_implement`) para verificar coherencia **en tiempo de autoría**, no solo en auditoría.
3. **Modelado** — introducir **edges de procedencia de contrato cross-spec** (el pedido central de #304).

---

## 2. La arquitectura de integración de SpecKit (verificada en Sentinel v0.11.3)

### 2.1 Ciclo de vida con hooks por evento

`.specify/extensions.yml` declara hooks sobre un ciclo de vida explícito. Los **eventos** (cada uno con variante `before_` y `after_`) son:

`constitution`, `specify`, `clarify`, `plan`, `tasks`, `implement`, `checklist`, `analyze`, `taskstoissues`.

Cada hook tiene esta forma (ejemplo real del `agent-context`):

```yaml
after_plan:
  - extension: agent-context
    command: speckit.agent-context.update
    enabled: true
    optional: true
    priority: 10
    prompt: Execute speckit.agent-context.update?
    description: Refresh agent context after planning
    condition: null
```

Campos: `extension`, `command`, `enabled`, `optional`, `prompt`, `description`, `condition`, `priority`. Setting global `auto_execute_hooks: true`. **Este es el punto de inyección de Baton.**

### 2.2 Extensiones

Una extensión vive en `.specify/extensions/<id>/extension.yml` y declara (ejemplo: la extensión `git`, autor `spec-kit-core`):

```yaml
schema_version: "1.0"
extension:
  id: git
  name: "Git Branching Workflow"
  version: "1.0.0"
requires:
  speckit_version: ">=0.2.0"
provides:
  commands:                       # nuevos slash-commands (archivos .md)
    - name: speckit.git.commit
      file: commands/speckit.git.commit.md
  config:                         # plantillas de config versionables
    - name: "git-config.yml"
      template: "config-template.yml"
hooks:                            # se auto-registran en el ciclo
  before_implement: { command: speckit.git.commit, optional: true, ... }
```

Sentinel tiene dos extensiones instaladas:
- **`git`** — ramas de feature + auto-commit en cada evento del ciclo.
- **`agent-context`** — *"Manages coding agent context/instruction files (e.g., CLAUDE.md) with **project-specific plan references**"*. Su comando `speckit.agent-context.update` corre en `after_specify` y `after_plan`.

> **`agent-context` es el precedente directo de lo que Baton necesita.** Ya existe la idea de "tras spec/plan, refrescar lo que el agente sabe del plan del proyecto". Pero solo actualiza una sección gestionada del archivo de contexto; **no verifica coherencia ni propaga decisiones cross-spec**. Baton puede enviar una extensión análoga que sí lo haga.

El ecosistema oficial reporta **70+ extensiones comunitarias** (Jira, Azure DevOps, GitHub Issues) y **quality gates** para seguridad, testing y **drift detection** — confirmando que "verificación de coherencia como extensión" es un patrón soportado y esperado.

### 2.3 Integraciones (distinto de extensiones)

Una **integración** es el *agente de IA conectado al proyecto*. Estado en `.specify/integration.json`:

```json
{ "version": "0.11.3", "installed_integrations": ["claude"], "default_integration": "claude" }
```

SpecKit soporta **~29 integraciones nombradas** (Claude Code, Copilot, Gemini CLI, Cursor, Windsurf, Codex CLI, Qwen, Kiro, Goose, etc.) + una genérica. Cada integración se instala con su manifiesto + hashes (`.specify/integrations/<name>.manifest.json`). La integración `claude` instala SpecKit **como Claude Skills**:

```
.claude/skills/speckit-{specify,plan,tasks,implement,clarify,analyze,
                         checklist,constitution,converge,taskstoissues}/SKILL.md
```

> **Relevante para Baton (driver económico):** la noción de "integración = agente" ya es nativa de SpecKit, y `v0.10+` añadió **multi-install controlado de integraciones**. Eso significa que un mismo proyecto puede tener varios agentes registrados — exactamente el sustrato sobre el que un router consciente de costo escogería *cuál* agente/modelo ejecuta *qué* paso.

### 2.4 Workflows con review gates

`.specify/workflows/speckit/workflow.yml` define el ciclo SDD completo como pasos orquestados con **compuertas humanas**:

```yaml
steps:
  - { id: specify, command: speckit.specify, integration: "{{ inputs.integration }}" }
  - { id: review-spec, type: gate, message: "Review the generated spec...", options: [approve, reject], on_reject: abort }
  - { id: plan, command: speckit.plan, ... }
  - { id: review-plan, type: gate, ... }
  - { id: tasks, command: speckit.tasks, ... }
  - { id: implement, command: speckit.implement, ... }
```

El campo `integration` es parametrizable (`"auto"` = la del proyecto). **SpecKit ya tiene un primitivo de orquestación con gates y selección de integración por paso** — Baton no parte de cero en este eje.

### 2.5 La memoria persistente — `.specify/memory/` (el "plan maestro" que se olvida)

Aquí está el corazón del problema del §2.2 del concepto. `.specify/memory/` en Sentinel contiene **el plano global estructurado**:

- `constitution.md` (las reglas no negociables; SpecKit ahora hace *constitution enforcement during implementation*).
- `INDEX.md`, `Project Sentinel - Documento de Visión y Decisiones.md`.
- **Una arquitectura y requisitos por módulo:** `Arquitectura - PolicyEngine.md`, `Requisitos - PolicyEngine.md`, `Arquitectura - StatusCenter.md`, … (12+ módulos), `Arquitectura de Sentinel.drawio`, `Mapa de Navegación - Dashboard Sentinel.md`.

> **Prueba directa de tus dos anécdotas:** existe `.specify/memory/Arquitectura - PolicyEngine.md` — el módulo PolicyEngine **estaba diseñado y documentado** en la memoria de SpecKit. Pero StrayMark/Loom **nunca leen `.specify/memory/`**, así que la proyección de arquitectura no sabe que PolicyEngine debería existir. Cuando un agente dispersó funciones de policy en otros módulos, ninguna señal lo contradijo. Lo mismo con el plan global de módulos interrelacionados que motivó la telemetría mockup.

---

## 3. El caso #304, mapeado a las costuras de integración

### 3.1 Qué pasó (verificado en Sentinel)

Dos specs escritos con meses de diferencia divergieron en un **triple mismatch**:

- **`spec 001`** modeló salud a nivel de servicio. Su backlog post-MVP **PM-002** (`specs/001-sentinel-mvp/post-mvp-backlog.md:62`, decidido en `AILOG-2026-04-24-006`) extendió el modelo a **salud per-componente** en un JSONB versionado v2 (`metrics` + `components`, cada componente con `{score, state}`).
- El contrato backend real (`internal/modules/statuscenter/handler.go:109-127`): `componentResponse { Name, State, Detail }` — **sin métricas raw**; enum `OPERATIONAL/DEGRADED/MAJOR_OUTAGE/IDLE` (`interfaces/status.go:8-16`).
- **`spec 005`** (frontend, `FR-010`) pidió "estado por componente y métricas (latencia P95, error rate, CPU, memoria)" **sin referenciar nunca PM-002**. El frontend escribió a mano `web/src/api/types.gen.ts` con un contrato **asumido**: campos `status`/`score` (vs. `state`/`health_score`), enum `GREEN/YELLOW/RED` (vs. `OPERATIONAL/...`), y `latency_p95_ms`/`error_rate`/`cpu`/`memory` **que no existen en el backend**.
- Los **mocks de test codificaban el contrato asumido**, así que los e2e pasaban en verde. El bug solo apareció en staging: `TypeError: t.find is not a function`. Remediación en `AILOG-2026-06-24-001`, follow-ups `FU-005-005` / `FU-005-006`.

### 3.2 La información existía; faltaba el *edge*

| Existía | Dónde | Faltaba el edge hacia |
|---|---|---|
| Decisión de contrato per-componente (PM-002) | `specs/001/post-mvp-backlog.md`, `AILOG-2026-04-24-006` | ← spec 005 nunca lo referenció |
| Contrato backend real (campos/enum) | `statuscenter/handler.go`, `interfaces/status.go` | ← el frontend asumió en vez de derivar |
| Arquitectura/requisitos del módulo | `.specify/memory/Arquitectura - StatusCenter.md` | ← nadie reconcilia memory vs. implementación |
| Requisito del consumidor (FR-010) | `specs/005/spec.md:151` | ← sin procedencia de "quién define este contrato" |

**No existe ningún mecanismo** —ni en SpecKit ni en StrayMark— que diga *"spec 005 consume el contrato producido por StatusCenter / PM-002"*. Verificado: los charters de Sentinel ni siquiera usan `originating_spec`; los specs no tienen metadata de dependencia cross-spec.

> Nota: SpecKit **sí** trae `speckit.analyze` (consistencia entre artefactos) y `before_analyze`/`after_analyze`. Pero `analyze` opera **intra-feature** (spec↔plan↔tasks de *un* spec); no cruza specs, ni compara contra el código implementado ni contra la gobernanza de StrayMark. Ese es exactamente el hueco que Baton cubre, sin duplicar lo que SpecKit ya hace.

### 3.3 Cómo cada costura lo habría cazado

- **Costura 1 (lectura):** al ingerir `.specify/memory/` + specs, el Coherence Bridge habría tenido el contrato per-componente de PM-002 y el FR-010 del consumidor en el mismo grafo. Al proyectar contra el código real, `latency_p95_ms`/`cpu`/`memory` aparecen como **campos exigidos por un consumidor que ningún productor modela** → finding.
- **Costura 2 (hook en `before_implement` del spec 005):** justo antes de codificar el frontend, el hook surface: *"FR-010 consume el contrato de salud definido en PM-002 (spec 001); valores enum y campos difieren de lo que asumes."* Esto es el pedido #3 de #304 ("drift signal at authoring time, not just audit time").
- **Costura 3 (edges de procedencia):** con un edge `spec005:FR-010 --consumes--> statuscenter:health-contract <--defines-- PM-002`, la pregunta "¿dónde está la verdad de este contrato?" tiene **una** respuesta, no tres. Un consumidor construido contra una forma vieja se vería como edge *colgante/envejecido* en Loom.

---

## 4. Implicaciones de diseño para el Coherence Bridge (Fase 1)

1. **Adaptador de SpecKit versionado** (decisión §10 del concepto: *adaptador, no acoplamiento*). Anclar al `speckit_version` (Sentinel: `0.11.3`) y al `integration_state_schema`. Leer: `.specify/memory/**`, `specs/**/{spec,plan,tasks}.md`, `specs/**/post-mvp-backlog.md`, `specs/**/contracts/**`, `.specify/extensions.yml`.
2. **Empezar por la costura de lectura (read-only).** Es la de mayor valor inmediato y cero riesgo: ingerir el plano de intención y extender la proyección de Loom a *intención vs. gobernanza vs. código*. Entrega el finding del §3.3 sin tocar SpecKit.
3. **Luego enviar una extensión `straymark` para SpecKit.** Reutiliza el patrón de `agent-context`: comando `speckit.straymark.coherence-check` enganchado en `after_specify`, `after_plan` y sobre todo `before_implement` (gate de coherencia en autoría). Esto materializa la integración "fuerte y continua" que pediste.
4. **Modelar procedencia de contrato cross-spec** como tipo de edge nuevo en el modelo de arquitectura — el corazón de #304.
5. **No duplicar `speckit.analyze`.** Posicionar Baton como *complemento*: `analyze` cubre coherencia intra-spec; Baton cubre cross-spec + spec↔código↔gobernanza. Documentar la frontera.
6. **Smell "el mock codifica la suposición"** (#304 propuesta 4, largo plazo): marcar fixtures de test que afirman una forma de contrato que ningún handler/schema productor corrobora.

---

## 5. Preguntas abiertas que esta investigación deja para el Charter

| # | Pregunta | Nota |
|---|---|---|
| A | ¿Procedencia de contrato como metadata **en el spec** (campo nuevo) o inferida por el Bridge? | Lo segundo evita modificar SpecKit; lo primero es más explícito. Posible híbrido |
| B | ¿La extensión StrayMark se distribuye vía el **catálogo de SpecKit** o se instala junto al framework de StrayMark? | El catálogo da alcance; la instalación local da control |
| C | ¿Cómo se relaciona esto con #303 (verificación en audit time)? | #303 = detección post-hoc en el prompt de auditoría; Baton = propiedad estructural en autoría. **Complementarios, deben compartir el modelo de contrato** |
| D | ¿El Bridge lee `.specify/memory/` (docs de arquitectura por módulo) además de `specs/`? | Sí — ahí estaba PolicyEngine. Pero el formato de `memory/` es libre (markdown humano), no estructurado como `spec.md`. Requiere parsing tolerante o convención |
| E | ¿Baton aprovecha el **workflow + gates** de SpecKit como punto de orquestación del router (Fase 2+)? | El primitivo de gates + `integration` por paso ya existe; encaja con el driver económico |

---

## 6. Fuentes

Grounding local (autoritativo): `/home/montfort/StrangeDaysTech/sentinel/.specify/` (SpecKit v0.11.3) y los artefactos del caso #304 en Sentinel.

Ecosistema:
- [github/spec-kit (repo)](https://github.com/github/spec-kit)
- [Spec Kit Documentation](https://github.github.io/spec-kit/) · [Extensions reference](https://github.github.io/spec-kit/reference/extensions.html) · [Integrations reference](https://github.com/github/spec-kit/blob/main/docs/reference/integrations.md)
- [DeepWiki — specify integration](https://deepwiki.com/github/spec-kit/4.5-specify-integration) · [Slash commands reference](https://deepwiki.com/github/spec-kit/5-slash-commands-reference)
- [Spec-Kit Extensions: making SDD your own — Hidde de Smet](https://hiddedesmet.com/speckit-extensions)
- [GitHub Spec Kit takes off — Visual Studio Magazine](https://visualstudiomagazine.com/articles/2026/05/12/github-spec-kit-takes-off-as-antidote-to-piecemeal-vibe-coding.aspx)

---

*Fin del documento de investigación — v1.0.*
