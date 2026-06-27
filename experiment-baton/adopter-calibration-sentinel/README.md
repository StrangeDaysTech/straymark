# Sentinel — calibración del clasificador de Baton (feedback de adoptante)

> **De:** el repo adoptante **Sentinel** (StrangeDaysTech/sentinel). **Para:** StrayMark / experimento Baton.
> **Motiva:** [`../05-adopter-test-plan.md`](../05-adopter-test-plan.md) §2–§4 + hallazgo [#328](https://github.com/StrangeDaysTech/straymark/issues/328).
> **Naturaleza:** read-only / recommend-only — ningún subcomando ejecutó un modelo, abrió red ni mutó el repo de Sentinel.

Corrimos los tres experimentos del plan de adoptante contra el corpus de gobernanza
real de Sentinel (**762 unidades**): E1 valida **corrección** (Baton midió cobertura,
no corrección), E2 prueba empíricamente la tesis de #328, y E3 le pone costo real.
Cierra con una propuesta concreta: **promover el verbo del trabajo de *inferido* a
*declarado*** ([`proposal-declared-verb.md`](proposal-declared-verb.md)).

## Procedencia del ground truth (declarada por honestidad)

El plan asumía como oráculo "el humano que hizo el trabajo", pero el implementador fue
el agente bajo supervisión del operador, que no tiene memoria unitaria de 762 unidades.
El oráculo se cambió a **el agente etiquetando contra el código/artefacto real
producido, ciego a la predicción de Baton, para supervisión y override del operador**.
Es más débil que ground truth humano puro (sigue siendo juicio de un agente), pero
defendible: el agente lee lo que Baton *nunca* lee (el diff real), no solo el título.

## Resultado de cabecera

| Experimento | Hallazgo |
|---|---|
| **E1 — corrección** | `signals::scan_cues` clasifica por **substring del título** → falsos positivos de keyword (`audit.go`→auditor, `(commit hash)`→operator, "coverage" de fechas→operator, "Audit **remediation**"→auditor). Tras adjudicar la política de tiers del operador: **high+medium 0.57**, **4 errores hacia abajo** residuales. |
| **E2 — señal (`work_verb`)** | Un verbo declarado en autoría (costo ≈0 tokens) sube la confianza **6→32 high** y cierra los 4 errores. `design_provenance` es **provablemente necesario** (instrumentar diseño upstream ≠ decisión nueva). El 1.00 de exactitud es POR CONSTRUCCIÓN — el valor es que la baja confianza de Baton era artefacto de inferencia, no incertidumbre real. |
| **E3 — costo real** | Con costos reales (Opus 4.6 / Haiku 4.5 / Gemma), el clasificador roto rutea con **39% de ahorro** vs **54%** del ground-truth correcto → deja ~15 puntos sobre la mesa **y** mete riesgo de calidad en 3/32 (trabajo frontier ruteado barato por falsos positivos de keyword). |

**Síntesis:** la palanca de #328 (señal estructurada declarada, no el título) no es solo
más precisión — es **más ahorro confiable** y menos riesgo de calidad. El `work_verb`
declarado es la inversión de mayor ROI. La fix de mayor palanca: que el clasificador
consuma la **provenance del diseño que Baton ya construye** (módulo B2), en vez de
rutear por substring del título.

## Archivos

| Archivo | Qué es |
|---|---|
| [`proposal-declared-verb.md`](proposal-declared-verb.md) | **La propuesta.** Tercera vía entre determinista-por-título y score-IA-por-unidad: campo de verbo declarado en autoría. |
| [`E1-findings.md`](E1-findings.md) | Corrección + causa raíz cue-substring + recomendaciones. |
| [`E2-findings.md`](E2-findings.md) | Piloto `work_verb` (la palanca de #328) + límites honestos. |
| [`E3-findings.md`](E3-findings.md) | Realismo de costo + el techo que pone el clasificador. |
| `labels-blind.yml` | Las 32 etiquetas de ground truth con evidencia citada (hoja ciega rellenada). |
| `.predictions-key.json` | Clave (id → predicción de Baton) para el join post-etiquetado. |
| `verbs.yml` | Declaración del autor de `work_verb` + `design_provenance` (E2). |
| `baton-config.yml` | Bloque `baton:` con costos reales + routing del operador (E3). |
| `score.py` / `verb_pilot.py` | Scorer E1 + simulación del clasificador verb-aware. |

## Reproducir los números reportados

```bash
python3 score.py        # E1: high+medium 0.57, 4 errores hacia abajo
python3 verb_pilot.py   # E2: 0 errores hacia abajo con verbo declarado
# E3 (requiere el binario straymark-baton + un checkout del repo Sentinel):
#   straymark-baton route <sentinel-repo> --dry-run --config baton-config.yml
```

> **Omitido a propósito:** `baton-classify.json` (volcado crudo de las 762 unidades del
> corpus de Sentinel, ~287 KB). Las cifras reportadas se reproducen desde
> `labels-blind.yml` + `.predictions-key.json` (las 32 muestreadas); el volcado completo
> es regenerable con `straymark-baton classify <sentinel-repo> --out json` y se
> comparte a pedido. Así esta contribución expone solo las unidades muestreadas (ya
> citadas como evidencia), no el corpus interno completo.

## Validación *forward* (no incluida — es de StrayMark)

E2 prueba el **mecanismo** (verbo declarado → tier correcto). Lo que falta validar es la
**fiabilidad humana/agente** de la señal en el flujo real (¿declaran bien los autores en
autoría?). Ese ensayo *forward* le toca a **StrayMark** post-adopción, sobre un corpus
variado de muchos repos — no a un adoptante con corpus de baja variabilidad y contra un
esquema aún no ratificado. Se deja anotado como la dirección, no ejecutado aquí.
