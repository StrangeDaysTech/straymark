# Ratificación del esquema `work_verb` / `design_provenance`

> **De:** experimento Baton · **Estatus:** `ratified` · **Fecha:** 2026-06-27 ·
> **Desbloquea:** paso 2 de [#332](https://github.com/StrangeDaysTech/straymark/issues/332)
> (graduación de los campos al framework).
> **Trazabilidad:** #332 (decisión) · #331 (evidencia de adoptante E1/E2/E3) ·
> #328 (la palanca es la señal estructurada) · #321 (anti-patrón "calibrado a un stack").

## Propósito

El giro arquitectónico de Baton (#332, prototipo en #333) descontinuó el *title-scan* y dejó al
**`work_verb` declarado en autoría** (+ `design_provenance`) como la **única señal autoritativa** de
clasificación. La graduación de esos campos al framework (plantillas/schemas/validador) está
condicionada por una restricción que el adoptante Sentinel puso explícita: *no instrumentar contra un
esquema sin ratificar*.

Este documento **es** esa ratificación. Fija el vocabulario, las reglas de determinación, el
placement (incluido el grano fino) y la postura de enforcement. Una vez mergeado, el esquema queda
ratificado y la graduación al framework (`fw-X.Y.Z`) queda desbloqueada como el siguiente PR.

El esquema está calibrado contra el corpus de gobernanza real de Sentinel (762 unidades):
[`proposal-declared-verb.md`](adopter-calibration-sentinel/proposal-declared-verb.md) +
[`verbs.yml`](adopter-calibration-sentinel/verbs.yml). El prototipo ya implementa los enums base en
[`src/signals.rs`](src/signals.rs) y los consume en [`src/classify.rs`](src/classify.rs).

## 1. Los dos campos

### `work_verb` — la señal autoritativa

Enum **cerrado**, vocabulario controlado (lowercase). Es lo que el autor (humano + agente) declara al
crear la unidad, porque lo sabe gratis y con certeza (costo ≈ 0 tokens).

| valor | significado | ejemplos |
|---|---|---|
| `design` | Decisión de arquitectura/patrón/escalabilidad **abierta**; autoría de spec. | Decidir un patrón de partición; diseñar un contrato de salud cross-componente; autorar un spec/arquitectura. |
| `implement` | Lógica de servicio; fix con diagnóstico; query compleja/optimización; tooling nuevo; **definir un contrato/interface fundacional acotado**. | Provider abstraction; fix de race READ COMMITTED; construir un analizador; escribir los 4 ports de un módulo. |
| `audit` | Revisión / verificación / contraste **independiente** contra la realidad. | Smokes + WCAG audit; public-shape audit; contrastar un mock contra el contrato real. |
| `operate` | Trabajo mecánico: tests/test-infra/migraciones, scaffolding, docs, cierre/ceremonia, bulk. | Autorar unit tests; bump de schema; bulk-approve; meta-nota de cierre; anotar un doc de arquitectura. |

### `design_provenance` — la carga cognitiva residual

Enum **cerrado** `{new, upstream}`. **Opcional**, default `new`. Captura si el pensamiento difícil ya
se gastó aguas arriba: una unidad cuyo trabajo sólo *instrumenta un diseño ya hecho en otra etapa* es
mecánica, aunque su `work_verb` aparente sea `implement`.

- `new` — la decisión difícil ocurre **en esta unidad**.
- `upstream` — el diseño ya existe; esta unidad sólo lo transcribe/cablea.

## 2. Reglas de determinación

Estas reglas cierran la ambigüedad que un enum por sí solo no resuelve. Son el corazón de la
ratificación.

1. **Regla del contrato fundacional.** "Definir un contrato/interface fundacional acotado" es
   **`implement`**, NO `design`. `design` se reserva a arquitectura **abierta** (patrón,
   escalabilidad) y autoría de spec. (Hallazgo del piloto, [`verbs.yml`](adopter-calibration-sentinel/verbs.yml#L11-L13);
   en la muestra calibrada hay 0 `design` real — consistente con E1, donde `planner` = 0.)

2. **Carga cognitiva residual (degradación).** `implement` + `design_provenance: upstream`
   **degrada a `operator`** (instrumentar diseño previo = mecánico). Es lo que separó
   `CHARTER-28` (wire de métricas declaradas en CHARTER-18 → operator) y `CHARTER-13` batch-1
   (queries ya generadas por CHARTER-07 → operator) de trabajo `implement` con diseño nuevo.

3. **`design_provenance` sólo es significativo para `implement`.** Un `design` cuyo diseño "ya
   existe aguas arriba" es una contradicción (si el diseño ya está hecho, el trabajo no es diseñar,
   es implementar). Por eso la degradación se ancla en `implement`, no en `design`. `audit` y
   `operate` ignoran `design_provenance`. Esto **ratifica** la regla tal como la implementa el
   prototipo ([`classify.rs`](src/classify.rs#L59)), resolviendo el "implement|design" más laxo de la
   propuesta original.

4. **No-trabajo → `operate`.** Higiene, ceremonia de cierre, meta-notas (p.ej. FU-136, FU-174):
   no se introduce un 5º valor. Se mapean a `operate` (rutea barato igual). El "en rigor NO-trabajo"
   del piloto es una observación de telemetría, no un tier.

### Mapa verbo → tier (el resultado)

| `work_verb` | tier | nota |
|---|---|---|
| `design` | planner | |
| `implement` | implementer | degrada a **operator** si `design_provenance: upstream` |
| `audit` | auditor | |
| `operate` | operator | |

## 3. Placement y regla de grano homogéneo

El verbo se declara **al grano más fino que ya tiene slot de declaración** — sin introducir
frontmatter nuevo en unidades que no lo tienen.

| Unidad | Slot de declaración |
|---|---|
| **Charter** | Frontmatter: `work_verb:` / `design_provenance:`. |
| **Follow-up** | Líneas en la entrada: `- **Work verb**:` / `- **Design provenance**:`. |
| **Batch** | Línea `- **Work verb**:` en el AILOG ledger del batch (mismo formato que follow-up). |
| **Task** (SpecKit) | **Sin slot propio.** Hereda del charter/spec padre. **Cero frontmatter nuevo en `tasks.md`.** |

**Regla de grano homogéneo + herencia:**

- Se declara una sola vez, al grano homogéneo más amplio. Batch y Task **heredan** el verbo del
  charter/follow-up padre por defecto.
- **Override sólo por heterogeneidad:** si un charter mezcla, p.ej., diseño + parte mecánica, se
  declara el verbo al grano más fino que sí tenga slot (por batch, vía la línea del ledger). No se
  fragmenta artificialmente una unidad homogénea.

**Nota de honestidad (estado del prototipo).** Hoy el prototipo **no** implementa la herencia: cosecha
el verbo del charter frontmatter y de las líneas de follow-up, y deja batch/task como `undeclared`
([`src/units.rs`](src/units.rs)). La herencia es una **regla ratificada aquí**, pendiente de
implementar en la graduación al framework (o en un follow-on de Baton). Documentarla ahora evita que
la graduación invente un mecanismo de declaración por-task que esta ratificación descarta.

## 4. Undeclared = estado honesto

Una unidad **sin verbo declarado es inclasificable** (`class = None`): se rutea conservadoramente
hacia arriba (frontier) **y se emite un nudge** ("declara el verbo"), **nunca** una conjetura de baja
confianza desde el título ([`src/classify.rs`](src/classify.rs#L51), [`src/route.rs`](src/route.rs)).
El hueco se vuelve una *acción* de higiene, no un número fingido. La telemetría reporta
`undeclared_fraction` (la métrica accionable), no un `conflict_fraction` artefacto del title-scan.

## 5. Postura de enforcement (para la graduación)

Cuando estos campos gradúen al framework:

- **Campo opcional**, no requerido. (`charter.schema.v0.json` ya tiene `additionalProperties: true`,
  así que añadirlos es puramente aditivo.)
- El validador marca **falta o invalidez** del verbo como **warning / nudge advisory** — al estilo
  de un check `coherence`, no un CI-gate (`exit ≠ 0`).
- Coherente con v0 experimental y con la cautela del adoptante: el corpus legacy (100% undeclared en
  Sentinel) no debe romperse por declarar un esquema. `undeclared` es estado honesto, no un error.

## 6. Anti-gaming y forward-validation

- **El título degrada de oráculo a cross-check advisory**, y **sólo** en la herramienta de
  calibración periódica — **nunca** en la ruta de clasificación/ruteo. (`scan_cues`/`CUE_TABLE` ya se
  eliminaron del ruteo en #333.)
- **Mitigación de deriva/gaming del verbo:** vocabulario pequeño + calibración periódica (el mismo
  loop E1) + el nudge de clasificabilidad.
- **Forward-validation** (¿declaran bien los autores en un corpus variado post-adopción?) es el
  **paso 3 de #332**, responsabilidad de StrayMark, **fuera del alcance** de esta ratificación.

## 7. Destino de graduación (mapeo — NO se aplica en este documento)

Para el PR de graduación, ya desbloqueado por esta ratificación:

- `dist/.straymark/schemas/charter.schema.v0.json` — añadir `work_verb` (enum) + `design_provenance`
  (enum, default `new`), opcionales (aditivo, sin tocar `required`).
- Plantilla de charter + plantillas de follow-up — añadir los campos con el vocabulario y un
  comentario-guía.
- Nudge del validador del CLI — "falta verbo / verbo inválido" como advisory.
- Bump `fw-X.Y.Z` + actualización de docs (EN/es/zh-CN), por el flujo de release del framework.

## 8. Lo que esta ratificación NO decide

- **Forward-validation** sobre corpus variado (paso 3 de #332).
- **Declaración por task con frontmatter propio** — descartada por la regla de grano homogéneo (§3).
- **Pricing / ejecución real** de modelos (Fase 3 de Baton, diferida por la decisión #8).
- **Reintroducir un 5º valor** para no-trabajo — descartado (§2.4); reconsiderar sólo si la
  calibración futura muestra que `operate` funde señal económicamente relevante.
