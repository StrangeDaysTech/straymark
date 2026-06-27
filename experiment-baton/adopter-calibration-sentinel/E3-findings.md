# Baton E3 — realismo de costo con los costos reales de Sentinel

> **Para:** StrayMark / experimento Baton · **Fecha:** 2026-06-26 · Read-only
> (`--config` en la carpeta de calibración; el `config.yml` real no se tocó).
> Cierra el trío del plan de adoptante (§4) junto a [E1](E1-findings.md)/[E2](E2-findings.md).

## Costos y routing usados

`cost_per_mtok` en Baton es **un solo número por tier** → blended. Supuesto
declarado: **80% input / 20% output** (gobernanza/código es input-pesado). Precios
por Mtok (skill claude-api, cacheados 2026-06-04):

| Tier | Modelo | $ in / out | Blended 80/20 |
|---|---|---|---|
| frontier | Opus 4.6 | 5 / 25 | **9.0** |
| economic | Haiku 4.5 | 1 / 5 | **1.8** |
| local | Gemma 4 (self-hosted) | — | **0.0** |

Routing = política real del operador ([[project_routing_policy_model_tiers]]):
planner→frontier, **implementer→frontier** (trabajo interesante), auditor→economic,
**operator→local** (mecánico → Gemma). Config en [baton-config.yml](baton-config.yml).

## Resultado corpus-wide (`route --dry-run`, 762 unidades)

| | Valor |
|---|---|
| Costo todo-frontier | $776.16 |
| Costo ruteado | $544.54 |
| **Ahorro neto** | **$216.38 (28%)** |
| Overhead de clasificación | $15.24 |
| Sensibilidad | robusto a 2× overhead ✓ |
| **% del ahorro sobre routing low-confidence** | **11%** |

El 28% es modesto (vs el ~93% ilustrativo) **porque tu routing es conservador**:
implementer→frontier mantiene el grueso a precio completo; el ahorro real viene de
operator→local. Pero el **11% sobre low-confidence** (vs 57% con los defaults) es
mucho más fiable: tu política manda lo dudoso a frontier (caro/seguro), así que el
ahorro que ocurre está en clasificaciones de mayor confianza. Costo honesto, no
inflado.

## El cierre: el techo del ahorro lo pone el CLASIFICADOR, no el costo

Con tu routing, el ahorro grande depende de identificar bien el trabajo **operator**
(mecánico → Gemma gratis). Baton solo etiqueta 131/762 como operator — pero E2 mostró
que mucho del "implementer" es instrumentación mecánica. Medido sobre las 32 con
ground truth (100k tok/unidad uniforme, aísla el efecto de clasificación):

| Routing por… | Costo | Ahorro |
|---|---|---|
| Baseline todo-frontier | $28.80 | — |
| **Baton (clasificador roto)** | $17.46 | **39%** |
| **Ground truth (correcto)** | $13.14 | **54%** |

**Clasificar bien subiría el ahorro de 39% a 54%** — ~15 puntos que Baton deja sobre
la mesa por no distinguir lo mecánico. Y el desajuste tiene **dos signos**, ambos
malos:

- **Ahorro perdido (12 unidades):** trabajo mecánico (T001 scaffolding, CHARTER-01
  editorial, FU-011 índice, CHARTER-28 instrumentación…) ruteado a frontier/economic
  cuando debía ir a local. Dinero desperdiciado.
- **Barato de más — riesgo de calidad (3 unidades):** trabajo frontier ruteado barato
  por los falsos positivos de keyword de E1 — **T014** (diseño de interfaces →
  economic por el filename `audit.go`), **batch-2** (service layer → local por
  `(commit hash)`), **CHARTER-03** (auth/CI → local por "test-live"). Parte del "39%"
  de Baton es **ahorro falso**: sale de mandar trabajo crítico a un modelo incapaz.

## Conclusión del trío E1+E2+E3

- **E1:** Baton clasifica por substring del título → 4 errores hacia abajo, mecanismo
  inseguro.
- **E2:** un verbo declarado en autoría (costo ≈0) cierra esos errores y sube la
  confianza de 6→32 high.
- **E3:** con tus costos reales, ese defecto cuesta **~15 puntos de ahorro** (39%→54%)
  **y** mete riesgo de calidad en 3/32 unidades. La palanca de #328 (señal estructurada
  declarada) no es solo más precisión — es **más ahorro confiable**. El `work_verb`
  declarado es la inversión de mayor ROI: ≈0 tokens, y desbloquea el ahorro que el
  título-scan no puede.

## Caveats

- Blended 80/20 y modelos por tier (Haiku como economic) son supuestos — ajústalos en
  [baton-config.yml](baton-config.yml); la sensibilidad es robusta a 2×, así que el
  veredicto cualitativo aguanta.
- El gap 39%→54% se midió con tokens uniformes para aislar la clasificación; el net
  corpus ($216) usa el proxy effort-based de Baton.
- `route --dry-run` no ejecuta nada; `git status` intacto.
