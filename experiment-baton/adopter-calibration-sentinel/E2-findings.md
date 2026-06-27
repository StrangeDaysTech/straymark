# Baton E2 — piloto `work_verb` (la palanca de #328) sobre el sample E1

> **Para:** StrayMark / experimento Baton · **Fecha:** 2026-06-26 ·
> Read-only (`verb_pilot.py` simula el cambio mínimo; no patchea Baton).
> Complementa [E1-findings.md](E1-findings.md) y [proposal-declared-verb.md](proposal-declared-verb.md).

## Qué mide

E2 prueba la tesis de #328 ("la palanca es la señal, no la granularidad") con la
forma más barata de señal: un **verbo declarado en autoría**. Sobre las 32 unidades
del sample E1 (con ground truth + predicciones reales de Baton), declaré
`work_verb` + `design_provenance` como lo haría el autor ([verbs.yml](verbs.yml)) y
simulé el clasificador verb-aware. Baseline del corpus (`route --dry-run`): **57%
low-confidence, y 57% del ahorro descansa sobre routing low-confidence** — y E1 mostró
que la precisión solo-low es 0.11. Ahí vive el riesgo.

## Resultado

| | Baton (title-scan) | **Verb declarado** |
|---|---|---|
| Exactitud global | 0.44 | **1.00** |
| Precisión high+medium | 0.57 | **1.00** |
| Confianza (high · med · low) | 6 · 17 · 9 | **32 · 0 · 0** |
| Errores hacia abajo | 4 | **0** |

## El 1.00 es POR CONSTRUCCIÓN — no es el hallazgo

Declarar el verbo ≈ declarar la clase (el mapa verb→tier es casi la identidad), así
que la exactitud perfecta es esperada y **no demuestra nada por sí sola**. Lo que el
piloto sí demuestra, y es lo que importa:

1. **La baja confianza de Baton era un ARTEFACTO de inferencia, no incertidumbre real.**
   Title-scan marcó 9/32 low + 17 medium porque *adivina* el verbo desde un substring
   ambiguo. El autor **conocía el verbo con certeza** al crear la unidad. Declararlo
   convierte 6 high → **32 high a costo ≈ 0 tokens**. Esa es la palanca de #328
   cuantificada: el corpus pasaría de 57% low-confidence hacia ~0%, moviendo el ahorro
   desde routing no-fiable (0.11 de precisión) a routing declarado.

2. **Elimina la CLASE de error de E1.** Los 4 errores hacia abajo (todos falsos
   positivos de keyword: `audit.go` filename, `(commit hash)`, "Audit remediation",
   "make test-live") **se corrigen los 4** — porque el verbo se declara, no se infiere
   de un token contaminado.

3. **`design_provenance` es provablemente NECESARIO** (no redundante con el verbo).
   En 2 casos el verbo solo habría fallado:
   - `#batch-1` (041): verb=implement → *implementer*, pero `provenance=upstream`
     (queries que CHARTER-07 ya diseñó) → degrada a **operator** ✓.
   - `CHARTER-28`: verb=implement ("wire") → *implementer*, pero instrumenta métricas
     declaradas en CHARTER-18 → **operator** ✓.
   Un único campo de verbo no captura "instrumenta diseño previo". La segunda
   dimensión —tu insight de carga cognitiva residual— es indispensable.

4. **Refinamiento de vocabulario que surgió del piloto.** Una taxonomía de 4 verbos
   conflaba `design`→planner con "definir contrato fundacional"→implementer. Se
   resolvió tratando la **definición de contratos acotados como `implement`** y
   reservando `design` para arquitectura abierta. (En el sample: 0 `design` — coherente
   con E1, planner=0 real. El trabajo de diseño abierto es escaso y vive en specs/ADRs,
   no en charters de ejecución.)

## Lo que el piloto NO demuestra (límites honestos)

- **No prueba que los verbos del mundo real sean correctos.** El autor podría
  mis-declarar o derivar hacia el verbo cómodo. Eso solo lo mide un ensayo *forward*
  (declarar el verbo en autoría y recalibrar a ciegas) — pero **ese ensayo le toca a
  StrayMark, no a Sentinel**: es el dueño del campo quien debe validar su fiabilidad
  post-adopción, sobre un corpus variado de muchos repos. Correrlo en Sentinel sería
  mala inversión: (a) se instrumentaría contra un esquema aún no ratificado (riesgo de
  retrabajo si la forma final difiere); (b) el corpus restante de Sentinel es casi todo
  frontend → baja variabilidad de clase (no ejercita planner ni el caso difícil de
  `design_provenance`), así que mediría poco y sesgado.
- n=32, un solo autor (el agente). El valor es de *mecanismo*, no de tamaño de muestra.

## Conclusión para la propuesta

#328 queda confirmado por la vía más barata: la señal estructurada declarada (verbo +
provenance) recupera el tier correcto a confianza alta y costo ≈ 0, donde el title-scan
determinista da 0.57 con 57% de low-confidence. **La recomendación a StrayMark es
firme:** promover el verbo de *inferido* a *declarado* (campo de gobernanza), con
title-scan como fallback para legacy, y `design_provenance` como segunda dimensión.
La validación *forward* (¿declaran bien los autores en el mundo real?) es responsabilidad
de StrayMark post-adopción, sobre un corpus variado — no de un adoptante con corpus de
baja variabilidad. Detalle del diseño en [proposal-declared-verb.md](proposal-declared-verb.md).
