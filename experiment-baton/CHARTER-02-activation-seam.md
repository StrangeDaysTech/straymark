---
charter_id: CHARTER-02-activation-seam
status: declared
effort_estimate: M
trigger: "Fase 1 entregó la detección read-only on-demand; el caso #304 (telemetría mockup, contrato de salud) se origina cuando un agente codifica ANTES de que nadie corra el diagnóstico. La señal debe llegar en tiempo de autoría."
originating_concept: experiment-baton/01-baton-concept.md
originating_research: experiment-baton/02-speckit-integration-research.md
related_issues: [316, 304, 303]
---

# Charter: Baton — costura de activación (extensión SpecKit, hook `before_implement`)

> **Status (espejo del frontmatter; la fuente de verdad es el frontmatter):** declared. Effort: M.
> **Origen:** concepto [01-baton-concept.md](01-baton-concept.md) §4.1 (costura de *activación*) + investigación [02-speckit-integration-research.md](02-speckit-integration-research.md) §2 (extensiones/hooks de SpecKit v0.11). Cierra el issue #316.
> **Encuadre:** continuación de la Fase 1 (Coherence Bridge). Reutiliza el motor read-only ya construido; **no toca modelos**. La Fase 2 (routing económico) sigue siendo posterior.

## Context

La Fase 1 entregó la **costura de lectura**: un diagnóstico `coherence`/`overlay` read-only, on-demand. Pero la deriva del #304 (el frontend construido contra un contrato asumido; la telemetría mockup) nace cuando un agente **codifica antes** de que nadie corra el diagnóstico. La detección on-demand llega tarde.

Esta es la **costura de activación** (la 2ª de las tres del documento de investigación): una **extensión SpecKit** que engancha el evento de ciclo de vida **`before_implement`** para correr el motor de coherencia de la Fase 1 **en tiempo de autoría** — justo antes de que el agente escriba código. Es el pedido #3 del issue #304 ("drift signal at authoring time, not just audit time") en su forma más fuerte.

El precedente existe y está verificado en Sentinel (research §2.2): la extensión `agent-context` de SpecKit ya corre `after_specify`/`after_plan` para refrescar el contexto del agente. Baton enviará una extensión análoga que, en `before_implement`, **verifica coherencia** en vez de solo refrescar contexto.

## Scope

**In scope:**

1. **Paquete de extensión SpecKit `straymark`** — `extension.yml` (schema 1.0) que declara un comando `speckit.straymark.coherence-check` y lo auto-registra en el hook **`before_implement`** (research §2.1/§2.2). Estructura espejo de `.specify/extensions/git/` y `.specify/extensions/agent-context/`.
2. **El comando de coherencia en autoría** — invoca el motor de la Fase 1 (`straymark-baton coherence`) **acotado al feature/spec activo** (no a todo el repo, por costo y relevancia) y surface al agente los findings blocking + warnings de alta confianza **antes** de implementar.
3. **Acotamiento por feature** — resolver el spec activo (vía `.specify/feature.json` / el branch de feature) y limitar el análisis a sus contratos/consumidores, para que el hook sea rápido.
4. **Descubrimiento del binario `straymark-baton`** — el comando localiza el binario (instalado, en `PATH`, o vía `cargo run` en desarrollo) y degrada con un aviso claro si no está disponible (nunca rompe el flujo de SpecKit).
5. **Política de gate configurable** — por defecto **surface/advisory** (el humano/agente decide), con opción de gate duro (`fail before_implement` ante blocking) vía config de la extensión.
6. **Dogfood read-only en Sentinel** — instalar la extensión, disparar `before_implement`, confirmar que surface los findings #304-class; cero mutación.

**Out of scope:**

- Cualquier routing/tier/presupuesto/token/costo de modelos → Fase 2.
- Mutar SpecKit o auto-corregir el contrato (sigue siendo solo diagnóstico).
- Resolver la limitación de keying de tipos generados (#313) — ortogonal; la extensión usa el motor tal cual y se beneficiará cuando #313 cierre.
- Publicar un release de `straymark-baton` en crates.io (sigue experimental); la distribución del binario se resuelve en §Files/decisión, no aquí como release formal.
- Graduar el motor a `straymark-core`.

## Files to modify

| File | Change |
|---|---|
| `experiment-baton/extension/straymark/extension.yml` | New — manifiesto de la extensión SpecKit (commands + hook `before_implement`) |
| `experiment-baton/extension/straymark/commands/speckit.straymark.coherence-check.md` | New — el comando de verificación en autoría (prompt + invocación) |
| `experiment-baton/extension/straymark/README.md` | New — instalación + descubrimiento del binario + config de gate |
| `experiment-baton/src/main.rs` | Posible flag `--spec <id>` / `--feature` para acotar el report al feature activo (confirmar al ejecutar) |
| `experiment-baton/src/coherence.rs` | Posible filtro por spec activo (acotamiento) — typed, sin tocar core |
| `experiment-baton/specs/002-activation-seam/` | New — spec SpecKit (WHAT/HOW/tasks) de esta fase |
| `experiment-baton/AILOG-YYYY-MM-DD-NNN-*.md` | New — log(s) de ejecución |

> Paths confirmados al ejecutar (reconnaissance-first). Antes de codificar el comando, leer un `command.md` real de `.specify/extensions/git/commands/` en Sentinel para imitar el formato exacto.

## Verification

### Local checks

- `cargo build -p straymark-baton` (+ el flag de acotamiento si se añade) ✓.
- El `extension.yml` valida contra el schema de extensiones de SpecKit (research §2.2): `provides.commands` + `hooks.before_implement` bien formados.
- Test: con la extensión apuntando a un fixture, un `before_implement` simulado invoca el comando y emite los findings esperados (acotados al feature).
- `cargo test --workspace` + `cargo clippy` verdes.

### Production smoke (after install / dogfood)

- Instalar la extensión en Sentinel (read-only sobre el repo), disparar `before_implement` para el feature `005-frontend-dashboard`, confirmar que surface el finding #304-class (consumidor vs decisión no referenciada) **antes** de implementar.
- `git status` en Sentinel intacto tras el dogfood (NFR1).

## Risks

- **R1 — El binario `straymark-baton` puede no estar instalado** en el proyecto adopter. Severidad: media. Mitigation: descubrimiento robusto (PATH / ruta configurable / `cargo run` en dev) + degradación con aviso; el hook **nunca rompe** el flujo de SpecKit si el binario falta.
- **R2 — Costo del hook** si corre el análisis de todo el repo en cada `before_implement`. Severidad: media. Mitigation: acotar al feature/spec activo (§Scope 3); medir y, si hace falta, cachear.
- **R3 — Deriva de versión de SpecKit** (formato de extensión/hooks). Severidad: baja. Mitigation: anclar `requires.speckit_version`; probado contra 0.11.x (el de Sentinel).
- **R4 — Falsos positivos en autoría erosionan confianza** (más sensible que on-demand, porque interrumpe). Severidad: alta. Mitigation: por defecto advisory (no gate duro); surface solo blocking + warnings de alta confianza; reutiliza el sesgo conservador ya calibrado en B5.
- **R5 — Distribución del binario experimental.** Severidad: media. Mitigation: documentar instalación manual / `cargo install --path` mientras Baton sea experimental; un release formal se decide al graduar.

## Tasks

1. Sync `main`, partir de la rama del experimento.
2. Reconnaissance: leer `.specify/extensions/{git,agent-context}/extension.yml` + un `command.md` reales (en Sentinel) para imitar formato y mecánica del hook.
3. Redactar `specs/002-activation-seam/` (WHAT/HOW/tasks).
4. Implementar el acotamiento por feature en el motor/CLI (si se confirma necesario).
5. Crear el paquete de extensión `straymark` (extension.yml + command.md + README).
6. Tests locales (validación del manifiesto + invocación sobre fixture).
7. Dogfood read-only en Sentinel; AILOG (`risk_level`, `review_required`).
8. Verificación local; drift check; commit + PR. Cierra #316.

## Charter Closure

- Atomic update (format v4): si se detecta drift al cerrar, reconciliar en el **mismo PR**, documentando en `## Closing notes`.
- Post-merge drift check.
- Frontmatter: `declared` → `in-progress` al arrancar; `in-progress` → `closed` al cerrar (+ `closed_at`).
- No borrar el archivo.
- **Gate de éxito:** con la extensión instalada, un `before_implement` en Sentinel surface al menos un finding #304-class al agente **antes** de implementar, sin romper el flujo de SpecKit ni mutar el repo.
