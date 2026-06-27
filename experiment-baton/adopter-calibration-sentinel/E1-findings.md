# Baton E1 — corrección sobre el corpus de Sentinel

> **Procedencia del ground truth:** agente (Claude) etiquetando contra el código/
> artefacto real producido, **ciego** a la predicción de Baton, **corregido con la
> política de routing declarada por el operador**. NO "humano que hizo el trabajo"
> (ver [README](README.md)). Muestra: 32 unidades estratificadas (de 762).
> Fecha: 2026-06-26. **Estado: final.** T012/T014 adjudicados = `implementer` (definir
> contratos fundacionales es diseño). Residual hacia abajo = **4**. Forward:
> [proposal-declared-verb.md](proposal-declared-verb.md).

## Adjudicación del operador (2026-06-26)

Opus 4.6 para todo fue **comodidad de no decidir por unidad, no intención de routing**.
La política real, en dos capas:

1. **Por categoría:** tests / test-infra / migraciones **mecánicas** → compacto
   especializado (Gemma-4-class) = `operator`. Trabajo "interesante" → frontier =
   `implementer`. Diseño/patrones/escalabilidad → frontier = `planner`.
2. **Por carga cognitiva residual (el principio profundo):** el tier NO lo fija la
   superficie del artefacto sino **si el pensamiento difícil ya se gastó aguas
   arriba**. Una query compleja que solo *instrumenta* un diseño hecho en otra etapa
   (el modelo solo crea el archivo y la coloca) es **mecánica → operator**, aunque el
   output parezca sofisticado. Lo que paga frontier es la *decisión nueva*
   (optimización, diagnóstico, diseño), no transcribir una ya tomada.

Reetiquetadas 8 unidades `implementer`→`operator` (nota `[POLÍTICA OPERADOR]` en
`labels-blind.yml`): T008, #batch-4-041, CHARTER-10, CHARTER-11, FU-011, FU-020 (capa 1);
#batch-1-041, CHARTER-28 (capa 2 — instrumentación de diseño upstream).

## Resultado

| Métrica | Naive | **Tras adjudicación** | Objetivo |
|---|---|---|---|
| Exactitud global | 0.34 | **0.44** | — |
| **Precisión high+medium (decisiva)** | 0.39 | **0.57** | ≥ 0.80 |
| Precisión solo-high | 0.67 | **0.83** | — |
| Errores **hacia abajo** | 10 | **4** (todos medium) | ~0 |

Por clase predicha: operator 5/7 (0.71), implementer 7/12 (0.58), auditor 2/7 (0.29),
planner 0/6 (0.00).

## El hallazgo central: precisión ≠ mecanismo fiable

`signals::scan_cues` lee **solo `unit.title`** por substring. La mejora a 0.57/0.83
**es en parte azar**: Baton "acierta" cuando el keyword incidental coincide con el
tier correcto, y falla con el mismo mecanismo cuando no.

**Aciertos right-for-wrong-reason:** CHARTER-10 (acierta operator por "**coverage**",
que aquí es cobertura de *fechas* de partición); CHARTER-11 (por "**test**").

**El otro filo del mismo mecanismo** — CHARTER-28: Baton predijo `implementer [high]`
por la keyword "**Wire**", cuando es instrumentación mecánica de métricas ya
declaradas (→ operator). Ruteó *hacia arriba* (caro, seguro) por accidente.

**4 errores residuales hacia abajo** (frontier → barato; el error inaceptable del §2):

| Unidad | pred | verdadero | Token que disparó | Por qué es frontier |
|---|---|---|---|---|
| "**Audit** remediation" (#batch-7.4) | auditor | implementer | "audit" = el *objeto* remediado | diagnostica+arregla un race READ COMMITTED (decisión nueva) |
| AILOG-041 #batch-2 | operator | implementer | "**commit**" del sufijo `(commit hash)` | dominante = service layer (lógica de negocio) |
| "make **test**-live" (CHARTER-03) | operator | implementer | "test" del *nombre de make-target* | auth/OIDC en CI (decisión nueva) |
| `interfaces/audit.go` (T014) | auditor | implementer | "audit" en el *filename* | def. de contratos — **ver §Abierto** |

## La implicación grande (positiva para el routing)

Si el tier lo fija la **carga cognitiva residual**, entonces en un proyecto
rigurosamente spec-driven (SpecKit) **la mayoría de la "implementación" es
instrumentación mecánica de diseño upstream** → rutea legítimamente a compacto. El
trabajo que paga frontier es **escaso y concentrado**: autoría de spec/arquitectura
(planner), queries complejas/optimización, debugging/diagnóstico duro, y diseño de
tooling/abstracciones nuevas. **El ahorro potencial es grande** — pero solo es
*confiable* si el clasificador distingue "instrumentar diseño previo" de "decisión
nueva", y **eso requiere la provenance del diseño, no el keyword del título**.

Baton **ya construye** un modelo de intent/provenance (módulo B2: `provenance.rs`,
`intent.rs`). El defecto no es que le falte la señal — es que el **clasificador la
ignora** y rutea por substring. Esa es la fix de mayor palanca. Confirma #328 en su
forma más fuerte: la palanca es la señal estructurada (incl. provenance), no el texto.

## Resuelto: definir contratos fundacionales = implementer (diseño)

**T012** (`comms.go` 4 ports per Arquitectura §4.1) y **T014** (`audit.go` interfaces
+ tipos) se adjudicaron **`implementer`**: aunque la arquitectura esboza la forma,
elegir firmas/tipos de los contratos fundacionales de los que todo depende **es
diseño**, no transcripción. T014 queda confirmado como **error residual hacia abajo**
(Baton → auditor por el filename `audit.go`). Residual final: **4**.

## Recomendaciones a Baton (StrayMark)

1. **No leer tokens incidentales**: paths/filenames (`*.go`), sufijos `(commit
   <hash>)`/fechas, nombres propios de make-targets. El verbo del trabajo, no los
   sustantivos del objeto.
2. **Verbo, no objeto.** "Audit **remediation**", "**fix** the audit finding" =
   implement. "**Wire** the already-declared metric" = operator. La keyword sola
   miente; pesa el verbo + el complemento.
3. **Usa tu propia provenance (B2).** Distinguir "instrumenta diseño upstream" de
   "decisión nueva" es lo que separa operator de implementer/planner. El clasificador
   debe consumir las aristas de intent-provenance, no el título.
4. **Polisemia "coverage"/"test".** Dieron resultados correctos por azar (frágil).
5. **Unidades-nota no son trabajo.** FU-136 (meta-cierre) y FU-174 (drift-doc) → filtro
   de "unidad sin trabajo".

## Caveats de validez

- Ground truth de **agente**, corregido con política del operador (no humano-puro).
  Riesgo asumido: las reclasificaciones a operator se hicieron solo con evidencia de
  diseño-upstream en el artefacto (batch-1: "CHARTER-07 ya generó las queries";
  CHARTER-28: instrumentos declarados en CHARTER-18). Donde la evidencia es débil
  (T012/T014) NO se flipó: se dejó abierto.
- n=32; planner: 0 verdaderos en la muestra → Baton sobre-predice planner 6 veces,
  todas error "hacia arriba" (seguro).
- **Robusto:** la clase de defecto cue-substring (filename/metadata/polisemia/verbo-
  vs-objeto) es indiscutible y es el determinante de la (in)fiabilidad — no el número
  agregado de precisión.
