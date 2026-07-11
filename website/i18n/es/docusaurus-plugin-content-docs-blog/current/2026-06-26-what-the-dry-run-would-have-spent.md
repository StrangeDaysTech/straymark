---
slug: what-the-dry-run-would-have-spent
title: Lo que el dry run habría gastado
authors:
  - jose
tags:
  - straymark
  - baton
  - cost-routing
  - speckit
  - telemetry
  - rust
  - sentinel
date: 2026-06-26T00:00:00.000Z
description: La segunda fase de Baton por fin miró el dinero — pero no enrutó nada. Clasificó las 762 unidades de trabajo que StrayMark ya había registrado en Sentinel, recomendó un tier para cada una, e imprimió lo que una política de routing habría gastado versus todo-frontier. El titular fue un techo ilustrativo de ~93%. La lectura honesta invirtió la propia hipótesis del experimento — la granularidad nunca fue la palanca. La cobertura de señal sí.
---
*El coherence bridge no tocó modelos a propósito. La segunda fase de Baton por fin hace la cosa económica — y aun así no corre nada. Lee las 762 unidades de trabajo que StrayMark ya registró en el adopter Sentinel (charters, batches, follow-ups, tasks), clasifica cada una con señales deterministas baratas, recomienda un tier de modelo, y emite **telemetría económica**: qué habría ido a dónde, y cuánto habría costado bajo una política de routing versus mandar todo a un modelo frontier. No se despacha ningún agente; no se invoca ningún modelo. El punto no es ahorrar todavía — es hacer visible el ahorro potencial y probar una regla dura: el costo de decidir no debe comerse el ahorro. No lo hizo. Pero los datos respondieron una pregunta distinta de la que hicimos.*

<!-- truncate -->

> *Si el overhead borra el ahorro a cierta granularidad, ese es un resultado válido, no un fallo a esconder.*

Este es el segundo post sobre **Baton** (después de [el coherence bridge](what-the-spec-path-only-proved-existed)). La Fase 1 estableció que no puedes enrutar con seguridad sobre intención que deriva. La Fase 2, [`CHARTER-03-dry-run-router`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-03-dry-run-router.md), construye la capa de clasificación-y-recomendación como un **dry run retrospectivo** — y, un Charter antes, cierra el lazo sobre *cuándo* debe dispararse el check de coherencia.

## 1. La costura de activación — detección, movida más temprano

La Fase 1 shippeó `coherence` y `overlay` como comandos on-demand, read-only. Pero la deriva del #304 — el frontend construido contra un contrato asumido — nace en el momento en que un agente **codifica antes** de que nadie corra el diagnóstico. La detección on-demand llega tarde.

Así que un Charter pequeño en medio, [`CHARTER-02-activation-seam`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-02-activation-seam.md) ([#316](https://github.com/StrangeDaysTech/straymark/issues/316)), mueve la señal a tiempo de autoría. Es una **extensión SpecKit** que engancha el evento de ciclo de vida `before_implement` y corre el motor de la Fase 1 — acotado al feature activo vía un flag `--spec`, por velocidad — surfaceando los findings de alta confianza al agente *antes* de que escriba código. El precedente ya existía y estaba verificado en Sentinel: la extensión `agent-context` de SpecKit corre `after_specify`/`after_plan` para refrescar contexto. Baton envía una extensión análoga que, en `before_implement`, verifica coherencia en vez de solo refrescar.

El sesgo de diseño es el que la Fase 1 se ganó: **advisory por defecto**, no un gate duro. Un falso positivo que interrumpe la autoría cuesta más confianza que uno que aparece en un reporte on-demand, así que el hook surfacea findings blocking y de alta confianza y deja decidir al humano, con un gate duro disponible solo por config. Y degrada con gracia — si el binario `straymark-baton` no se encuentra, el hook avisa y se hace a un lado; nunca rompe el flujo de SpecKit. Dogfoodeado en Sentinel, `before_implement` sobre el feature `005-frontend-dashboard` surfaceó el C4 real (`services.public-visibility` consumido sin referenciar su decisión definitoria) en modo advisory, repo intacto. Ese es el tercer pedido del issue #304 — *"señal de drift en tiempo de autoría, no solo en tiempo de auditoría"* — en su forma más fuerte.

## 2. El dry-run router — tres decisiones de encuadre primero

Luego la capa económica. Tres decisiones de framing, resueltas con el operador antes de escribir una línea, porque el documento de concepto las había dejado deliberadamente abiertas:

1. **Unidad enrutable: instrumentar lo que existe, y medir.** Sin vocabulario inventado. Un borrador previo de consultor había propuesto un concepto "Work Unit / WU-NNN" que no existe en el repo; tiramos el *nombre* pero conservamos el problema real que señalaba. Baton clasifica a la granularidad que StrayMark **ya registra** — Charter, Batch, Follow-up, Task — mide cuán heterogéneo es cada nivel, y deja que la telemetría revele qué granularidad "paga". La sub-decisión abierta se responde con datos, no por decreto.
2. **Modelo de costo: tiers ilustrativos en config.** Un bloque `baton:` en `config.yml` (el mismo patrón config-driven que [#279](https://github.com/StrangeDaysTech/straymark/issues/279)) declara un costo-por-millón-de-tokens *ilustrativo* por tier. Eso desbloquea el dry run sin resolver la identidad de costo real de proveedores (diferida a la Fase 3). Medimos ahorro **relativo**, no una factura.
3. **Clasificador: solo señales baratas, sin LLM.** Reglas deterministas sobre señales que StrayMark ya computa. Esto satisface el corolario económico duro directamente: clasificar debe ser barato, o el clasificador se vuelve el costo que debía evitar. Un clasificador LLM sigue siendo un escalamiento futuro, usado solo cuando las señales baratas sean ambiguas *y* el ahorro lo justifique.

Las unidades enrutables son el trabajo que StrayMark ya registró: 45 Charters, 82 Batches, 135 Follow-ups, 500 Tasks — **762 unidades**. Cada una se clasifica en una clase de tarea — Planner/Architect, Implementer, Auditor, Operator — a partir de señales baratas: `effort_estimate`, complejidad de `analyze`, el `risk_level` del Charter, el bucket y severidad de un follow-up, el estado del componente en la projection de Loom, los findings de coherencia de la Fase 1, la superficie de archivos tocada. Typed, puro, sin I/O. Una política de tiers mapea clase → tier con un fallback conservador, y `route --dry-run` imprime la economía. Recomienda. Nunca ejecuta.

### La regla dura, dicha de entrada

La restricción que gobierna todo el diseño:

> El costo de clasificar, atomizar y enrutar no debe igualar ni superar el ahorro de no cargar todo a un modelo frontier. Si la complejidad de la solución empareja el costo de no tenerla, la solución no existe.

La telemetría mide esto **explícitamente** — overhead versus ahorro, por granularidad. Si el overhead de una granularidad iguala o supera su ahorro, se reporta como *no enrutable*, no se fuerza. Ese es el guard de honestidad: un resultado negativo es un resultado.

## 3. La economía titular (ilustrativa)

Corrido read-only contra el corpus de gobernanza real de Sentinel, `git status` intacto:

```
ALL (762 units) — routable
  tiers: economic 617 · frontier 14 · local 131
  cost: all-frontier 1293.60 → routed 93.68  (gross saving 1199.92 ≈ 93%)
  overhead 15.24 → net saving 1184.68
  caveats: 57% low-confidence · 57% of saving on low-confidence routing · 12% conflicted
  sensitivity: breakeven overhead/unit 1.575 · robust at 2× overhead: true
```

Cada granularidad sale net-positive y robusta al doble del overhead ilustrativo. El overhead de clasificación es el **1.2% del ahorro bruto** — así que la regla dura no se viola por el costo de decidir. Por la letra del gate, el routing paga en todas partes. Si me hubiera detenido a leer ahí, habría shippeado un número triunfal.

## 4. La lectura honesta — confianza, no granularidad

El veredicto §4.2 dice "enrutable en todas partes". Los guards de honestidad dicen que el ahorro es **frágil**, y lo dicen de una forma que contradice la conjetura fundacional del experimento.

| Granularidad | conflicto % | confianza high+med % |
|---|---|---|
| Charter | 6% | 46% |
| Batch | 8% | 39% |
| Follow-up | 5% | 37% |
| Task | 15% | 44% |

Dos hallazgos, ambos contra la hipótesis:

**La heterogeneidad *no* es el bloqueador.** La conjetura era que una unidad gruesa — un Charter entero — empaqueta trabajo mezclado (diseño + código + auditoría + limpieza) y por eso resiste el routing limpio, y que una unidad más fina enrutaría mejor. Los datos la invierten: **Task** tiene el conflicto *más alto* (15%), Charter de los más bajos (6%). La razón es deflacionaria — la métrica de conflicto está confundida por la **verbosidad del título**. Los títulos de task son descriptivos, así que surfacean más cues, así que detectan más "conflicto"; los títulos de charter son escuetos. El conflicto era un proxy débil de heterogeneidad, y no hay ninguna granularidad que sea significativamente más homogénea para preferir.

**La cobertura de señal es la restricción vinculante.** La confianza high+medium se sienta en **37–46% a través de *todas* las granularidades** — plana. **El 57% de cada nivel es de baja confianza**, y no por conflictos (5–15%) sino por el *default sin-cue*: el 45% de las unidades no surfacea ningún cue de clasificación. La confianza alta necesita `effort_estimate`, que solo los Charters cargan; la mayoría de las unidades se clasifican con un título escueto y nada más. Así que **el 57% de ese 93% de ahorro bruto descansa en conjeturas de baja confianza.**

La respuesta empírica a la pregunta de la unidad enrutable: *¿qué granularidad es enrutable?* Por ahorro neto, todas; por confianza, ninguna es más limpia que otra. **La granularidad no es la palanca en este corpus. La cobertura de señal sí.** Introducir una sub-unidad más fina — aquello a lo que el vocabulario descartado "Work Unit" apuntaba — no ayudaría: Task ya es la más fina, y no es más confiable que un Charter.

## 5. Graduar conocimiento, no un ahorro

El gate permitía exactamente esto: *ahorro relativo net-positive después del overhead a alguna granularidad, nombrando cuál; y un resultado net-negativo con evidencia igual gradúa el conocimiento.*

Está CUMPLIDO — y lo que gradúa es el conocimiento, que resultó ser el desenlace más valioso. El dry run establece el **techo** del routing (~93% ilustrativo, sobreviviendo 2× overhead) y su **piso de confianza** actual (~43% de las unidades enrutan con confianza high+medium, así que ~57% del ahorro es una conjetura). Y **reencuadra la Fase 3**, sobre datos y no sobre aserción: el camino a un ahorro *confiable* no es una unidad enrutable distinta. Es **cablear las señales diferidas** — complejidad por función (que necesita graduar `analyze` del crate `cli` a `core`), estado de arquitectura de la projection de Loom, findings de coherencia de la Fase 1 — para subir la confianza *antes* de que algo se ejecute. B2 difirió esas señales pesadas bajo la regla cheap-first; el dogfood acaba de justificarlas retroactivamente como el siguiente paso. Ese es el loop empírico funcionando exactamente como se pretendía.

## 6. Lo que deliberadamente no hicimos

Sigue siendo **recomienda-only.** `route` requiere `--dry-run`; ningún cliente de modelo está enlazado, ningún agente se despacha, ninguna llamada de red se hace. La ejecución real es la Fase 3.

Los costos son **ilustrativos**, y etiquetados como tales en todas partes. El 93% es *forma*, no dinero — la identidad de costo real de proveedor queda diferida hasta que haya un contrato unificado que modelar, que aún no lo hay. Reportar una cifra en dólares falsa sería su propia clase de telemetría mockup.

Y resistimos el patch tentador. Cuando las señales baratas sub-cubrieron el 45% de las unidades, el movimiento obvio era echar mano de un clasificador LLM para llenar el hueco. No lo hicimos — eso violaría la regla cheap-first y enterraría el hallazgo real (cobertura de señal) bajo una conjetura más cara. La sub-cobertura queda archivada como follow-up, no escondida. Todo esto sigue siendo **EXPERIMENTAL (Baton v0 / N=1)**.

## 7. Si llegaste hasta aquí

La lección portátil es lo que los guards de honestidad le hicieron a un número hermoso. Baton pudo haber shippeado "93% más barato" y estar diciendo la verdad — el ahorro bruto es real. También habría sido profundamente engañoso, porque más de la mitad de él cabalga sobre la herramienta conjeturando cuando no tenía de dónde agarrarse. El número que importaba no era el ahorro; era la *distribución de confianza detrás* del ahorro, y los dos apuntan en direcciones opuestas. Encuentra una métrica titular en tu mundo que cargue buenas noticias — un lift de conversión, una ganancia de latencia, una reducción de costo — y pregunta qué fracción de ella descansa en casos de los que la medición estaba realmente segura. Un promedio verdadero sobre una pila de conjeturas sigue siendo una pila de conjeturas. El output más útil del dry run no fue el 93%. Fue el 57% al lado.

El próximo post es donde el fix para ese 57% resulta costar casi nada — porque el autor sabía la respuesta desde el principio.

---

*Baton Fase 2 — [`CHARTER-03-dry-run-router`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-03-dry-run-router.md) · [`CHARTER-02-activation-seam`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-02-activation-seam.md) · dogfood [`04-phase2-dry-run-dogfood.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/04-phase2-dry-run-dogfood.md). Issues [#316](https://github.com/StrangeDaysTech/straymark/issues/316) · [#323](https://github.com/StrangeDaysTech/straymark/issues/323) · [#324](https://github.com/StrangeDaysTech/straymark/issues/324) · [#325](https://github.com/StrangeDaysTech/straymark/issues/325) · [#326](https://github.com/StrangeDaysTech/straymark/issues/326). Predecesor: [Lo que el spec path solo probó que existía](what-the-spec-path-only-proved-existed).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*