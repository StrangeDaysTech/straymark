# Baton — plan de prueba de calibración para adoptantes (read-only)

> ⚠️ **Post-graduación (fw 4.38.0):** la terna E1/E2/E3 de este plan fue reemplazada
> por la E1 simplificada de [`07-track-c-adopter-kit.md`](07-track-c-adopter-kit.md).
> E2 quedó obsoleto por diseño (la señal ahora es `work_verb` declarado, decisión #332)
> y E3 no es necesaria para forward-validation. Este documento se conserva como
> registro histórico de la calibración pre-graduación.

> **Para:** el equipo de un repo adoptante (referencia: Sentinel). **Desde:** StrayMark / experimento Baton.
> **Naturaleza:** read-only, **recommend-only**. Nada de esto ejecuta un modelo, dispatcha un agente, abre red, ni muta tu repo. `straymark-baton route` exige `--dry-run`.
> **Motiva:** el hallazgo [#328 "La granularidad NO es la palanca"](https://github.com/StrangeDaysTech/straymark/issues/328) y `experiment-baton/04-phase2-dry-run-dogfood.md`.

## 0. Por qué te pasamos esto

Baton clasifica el trabajo que StrayMark **ya registró** en tu repo (charters / batches / follow-ups / tasks), recomienda un tier de modelo (frontier / económico / local) y estima el ahorro vs. cargar todo a frontier — **sin ejecutar nada**. En el primer dogfood sobre tu corpus medimos *cobertura* (qué fracción clasificamos con confianza) pero **no corrección** (si esas clasificaciones son acertadas). El único oráculo de corrección eres **tú**, que hiciste el trabajo.

Este plan cierra ese hueco con tres experimentos baratos y te deja, de paso, una mejora de higiene de gobernanza que perdura. La secuencia que des-arriesga toda la línea de routing consciente de costo es:

```
validar corrección (tú)  →  subir confianza (más señal)  →  ejecución real (Fase 3)
```

## 1. Precondiciones

- El binario `straymark-baton` (te lo pasamos, o compílalo desde `experiment-baton/`).
- Tu repo con su `.straymark/` habitual (charters, follow-ups, AILOGs, specs).
- `python3` o `jq` para muestrear el JSON (opcional; cualquier herramienta sirve).

Comprobación rápida (no muta nada):

```bash
straymark-baton classify <tu-repo> --out json | head
straymark-baton route    <tu-repo> --dry-run
git -C <tu-repo> status   # debe quedar intacto
```

---

## 2. Experimento E1 — corrección (ground truth)

**Pregunta:** ¿son correctas las clasificaciones? Y sobre todo, **¿son correctas las de *alta confianza*?** (Esa es la métrica que decide la estrategia: si "high+medium" es fiable, se puede enrutar solo eso y mandar el resto a frontier por defecto seguro — ahorro real sin riesgo, aun con 43% de cobertura.)

**Pasos:**

1. Genera las clasificaciones:
   ```bash
   straymark-baton classify <tu-repo> --out json > baton-classify.json
   ```
2. Toma una **muestra estratificada** de ~30 unidades, repartidas por granularidad × clase predicha (incluye deliberadamente unidades de *alta* y de *baja* confianza). 
3. Para cada unidad de la muestra, registra la clase **verdadera** según tu juicio (quien hizo el trabajo): ¿fue principalmente diseño/arquitectura (planner), implementación acotada (implementer), contraste/revisión (auditor), o mecánico — commits/docs/limpieza (operator)?
4. Compara y computa, **por clase** y **separando por confianza**:
   - precisión = aciertos / predicciones de esa clase,
   - **precisión del subconjunto high+medium** (la cifra decisiva),
   - matriz de confusión (a dónde se va lo que erramos — ¿erramos hacia *abajo*, que es lo peligroso, o hacia *arriba*?).

**Plantilla de etiquetado** (`baton-calibration/labels.yml`):

```yaml
# Una entrada por unidad muestreada. Llena `true_class` y `notes`.
- id: 005-frontend-dashboard:T012
  granularity: task
  predicted_class: operator
  confidence: low
  true_class:        # planner | implementer | auditor | operator
  notes:
- id: CHARTER-43-...
  granularity: charter
  predicted_class: implementer
  confidence: medium
  true_class:
  notes:
```

**Qué buscamos:** que la precisión de high+medium sea alta (≥~0.8). Si lo es, el ahorro confiable es real ya. Si lo erramos sobre todo *hacia abajo* (clasificar como mecánico algo que era diseño), eso es la señal de alarma — significaría endurecer el sesgo conservador.

---

## 3. Experimento E2 — cobertura de señal (la prueba de #328)

**Pregunta:** ¿enriquecer la señal sube la confianza? (Nuestra tesis: la palanca es la señal, no la granularidad.)

**Pasos:**

1. Mide la línea base: corre `route --dry-run --out json` y anota la fracción de unidades en confianza Low por granularidad.
2. Elige un subconjunto acotado (p. ej. 10–15 follow-ups y 5 charters) y **enriquece su señal de gobernanza**:
   - en charters: confirma que llevan `effort_estimate` y, si tu plantilla lo permite, un nivel de riesgo y un scope de archivos real;
   - en follow-ups: confirma `bucket` y `**Severity**`;
   - en general: títulos que nombren el *tipo* de trabajo (diseñar / implementar / auditar / limpiar), no solo el objeto.
3. Re-corre y mide el desplazamiento Low → Medium/High en ese subconjunto.

**Qué buscamos:** un desplazamiento medible hacia mayor confianza al añadir señal — confirmaría #328 empíricamente sobre tu repo. Importante: **el enriquecimiento del paso 2 no es trabajo desechable** — es mejor higiene de gobernanza que te sirve más allá de Baton (ver §5).

---

## 4. Experimento E3 — realismo de costo

**Pregunta:** ¿cuánto es el ahorro con *tus* costos reales, no los ilustrativos?

**Pasos:**

1. Añade un bloque `baton:` a tu `.straymark/config.yml` con tus costos por Mtok reales por tier (los demás campos son opcionales; heredan defaults):
   ```yaml
   baton:
     cost_per_mtok: { frontier: 15.0, economic: 1.0, local: 0.0 }   # pon los tuyos
     # opcional: work_size, routing, escalate_high_risk, overhead_per_unit
   ```
2. Re-corre `route --dry-run`. Lee el ahorro **neto** y la **sensibilidad** (breakeven de overhead, robusto a 2×).
3. Anota cuánto del ahorro descansa en routing de baja confianza (la columna de caveat). Esa es la cifra honesta.

**Qué buscamos:** una estimación de ahorro creíble para tu stack, con su fragilidad explícita al lado.

---

## 5. Cómo incrustar esto en tu flujo StrayMark (no lo corras como one-off)

La lección de #328 es justamente que **la gobernanza debería registrar señal estructurada de primera clase**. Así que conviene que esta calibración *viva dentro de StrayMark*, no en un cuaderno aparte. Sugerencias de inserción, de menos a más permanente:

1. **Córrelo como un Charter de Sentinel.** La calibración es una unidad de trabajo acotada: declara un Charter (`effort_estimate`, scope = la muestra + el subconjunto enriquecido, verificación = E1/E2/E3 corridos), y ciérralo con un AILOG que registre los resultados. Es el encuadre nativo de StrayMark — y de paso, ese Charter se vuelve una unidad enrutable más que Baton puede clasificar (dogfood recursivo).
2. **Versiona el ground truth como artefacto de gobernanza.** Guarda `baton-calibration/labels.yml` dentro de `.straymark/` (p. ej. `.straymark/baton-calibration/`), no como archivo suelto. Queda bajo control de versiones, auditable, y re-ejecutable en cada revisión. Es la lección de #328 comiéndose a sí misma: registrar señal estructurada.
3. **Trata E2 como mejora permanente, no como prueba.** Enriquecer charters/follow-ups con esfuerzo/riesgo/scope es **higiene de gobernanza que perdura** — mejora tus auditorías, tu `metrics`, tu triage de follow-ups, *y* sube la confianza de Baton. No deshagas el enriquecimiento al terminar.
4. **(Opcional, recurrente) Un "pulse económico" en tu revisión periódica.** Igual que `straymark-baton coherence` es CI-gateable, `route --dry-run` puede ser un paso de revisión de gobernanza que corres cada cierto tiempo (p. ej. al cerrar un Charter grande) para ver cómo evoluciona tu perfil de costo y tu cobertura de señal. Read-only, no bloquea nada.
5. **(Más adelante) Realimentar el triage.** Cuando un Charter o follow-up se clasifica con baja confianza, eso es en sí una señal de que le falta metadata — podría volverse un nudge en tu flujo de triage ("este follow-up no es clasificable: añade severidad/scope"). No lo construimos aún; lo dejamos anotado como dirección.

---

## 6. Qué reportarnos de vuelta

Un resumen corto (o el `labels.yml` + las tres corridas JSON):

- **E1:** precisión por clase y, sobre todo, **precisión del subconjunto high+medium**; dirección de los errores (¿hacia abajo?).
- **E2:** desplazamiento de confianza Low→Medium/High al enriquecer; tu impresión cualitativa de si la señal fue la palanca.
- **E3:** ahorro neto con tus costos reales y qué fracción descansa en baja confianza.
- **Fricciones:** cualquier cosa del flujo de inserción (§5) que no encajó.

Con eso cerramos el bucle de #328 con datos de tu repo y sabremos si el camino a un ahorro *confiable* es el que creemos (más señal), antes de diseñar la ejecución real (Fase 3).

## 7. Garantías

- **Read-only / recommend-only.** Ningún subcomando muta tu repo, ejecuta un modelo, dispatcha un agente ni abre red. `route` exige `--dry-run`. Verificable: `git status` intacto tras cualquier corrida.
- **Sin datos personales, sin inferencia de modelo.** Es lectura de tus propios artefactos de gobernanza + aritmética de costo ilustrativa/declarada. (EU AI Act: no aplica — herramienta local de desarrollo.)
- **Tus costos no salen del repo.** El bloque `baton:` vive en tu `.straymark/config.yml`; tú decides si nos compartes las cifras o solo los porcentajes relativos.
