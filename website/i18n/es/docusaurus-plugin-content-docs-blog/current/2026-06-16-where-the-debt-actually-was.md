---
slug: where-the-debt-actually-was
title: Dónde estaba realmente la deuda
authors:
  - jose
tags:
  - straymark
  - loom
  - architecture
  - technical-debt
  - visualization
  - rust
  - sentinel
date: 2026-06-16T00:00:00.000Z
description: La verdadera pregunta diaria del operador no es "muéstrame los documentos", es "¿dónde estamos?". Loom desarrolló una segunda superficie — la arquitectura como un edificio que puedes recorrer, en 2D y en 3D — y su overlay de deuda técnica salía vacío. No porque no hubiera deuda. Porque no sabía dónde ponerla.
---
*Loom empezó como un grafo de conocimiento de documentos. Luego el adopter de referencia dijo, más o menos: ese es un cuadro precioso de mi papeleo, pero mi pregunta diaria es "¿dónde estamos?" contra el **sistema** — los componentes, los layers, qué está construido y qué se sigue debiendo. Así que Loom desarrolló una segunda superficie: la arquitectura como un edificio, autorada una sola vez y renderizada como un plano 2D y como un modelo 3D explotado, con un overlay de estado que enciende los componentes. El overlay de deuda técnica salió vacío en la primera corrida. Había deuda de sobra. El overlay simplemente no tenía idea de qué componentes la cargaban — y acertar con eso tomó tres intentos, dos de los cuales parecían terminados.*

<!-- truncate -->

> *El overlay vacío fue el bug más honesto del mes: no se equivocaba sobre la deuda. Se equivocaba sobre dónde dibujarla.*

Este es el tercer post del arco de Loom — después de [extraer el parser compartido](what-the-second-reader-demanded) y [hacer que las aristas del grafo de documentos realmente resolvieran](what-the-graph-couldnt-draw-yet). Es aquel en el que Loom deja de ser una vista de los docs y se vuelve una vista del *sistema*, y donde un overlay engañosamente simple enseñó una lección sobre cada dashboard que alguna vez te mostró un cero confiado.

## El reencuadre

El pivote queda registrado en [`ADR-2026-06-02-002`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md). Loom fue acotado como una vista de grafo de conocimiento de documentos; el feedback del operador desde Sentinel lo reencuadró. Para un proyecto grande, la pregunta diaria del operador no es "muéstrame la red de documentos", es *"¿dónde estamos?"* contra el **mapa de implementación** — la arquitectura. La metáfora vino de la ingeniería civil y del BIM: **un modelo, muchas vistas**, con layers que se encienden para mostrar estado.

Ese reencuadre chocó contra un muro que la exploración ya había mapeado: **StrayMark no tiene concepto de `component`.** Mapea documentos a *archivos* — los "Files to modify" de un Charter, los "Modified Files" de un AILOG, `api_changes`. Así que una superficie de arquitectura necesita tres cosas nuevas: una forma de *modelar* componentes y layers, una forma de *proyectar* estado vivo de gobernanza sobre ellos, y un formato editable que cargue un layout autorado por humanos sin que una herramienta lo pisotee. Los hechos habilitantes ya estaban en mano — el estado de Charter (`declared` / `in-progress` / `closed`) y el drift declarado-vs-modificado se computan hoy, los ADRs ya embeben diagramas C4 y tablas de "Affected Components", y los stages 00–09 de `.straymark/` forman un layering natural.

## Estás aquí, desde la terminal

La primera mitad salió *sin gráficos de ningún tipo* — [`cli-3.25.0` / `core-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.25.0), carril A1 de Loom. La apuesta: responder "¿dónde estamos?" en la terminal primero, probar la projection, luego dibujarla.

- **`straymark architecture generate`** mina la estructura del codebase en un primer borrador de `model.yml` más un `plan.drawio`, enriquecido con señal de los ADRs (diagramas C4, tablas de Affected-Components mejoran las etiquetas y los enlaces).
- **`straymark architecture sync`** es append-only — detecta nuevos directorios fuente o componentes de ADR y los agrega, **sin nunca pisotear ediciones humanas ni la geometría de DrawIO**. Dry-run por defecto.
- **`straymark architecture validate`** reporta señales de integridad modelo↔plano (`undrawn` / `unmodeled` / `empty`) y sale con código no-cero ante cualquiera de ellas, así que es gateable en CI.
- **`straymark status --where`** es el "estás aquí" textual: proyecta el estado de cada componente — `active` / `in-progress` / `implemented` / `has-debt` / `uncharted` — desde señales vivas de gobernanza e imprime un resumen.

La decisión load-bearing está en `straymark-core`: la projection es una función **pura** `(model + GovernanceState) -> Projection`, cero I/O, compartida por el `status --where` del CLI y el servidor Loom. Eso no es pulcritud por la pulcritud misma — es el mismo principio que impulsó todo el arco. La respuesta de la terminal y la respuesta visual se computan con *el mismo código*, así que **no pueden discrepar** (la spec lo llama NFR3). Un dashboard que contradice al CLI es peor que ningún dashboard; esto vuelve la contradicción irrepresentable.

## Un modelo, dos vistas

Luego las vistas, ambas leyendo esa única projection. [`loom-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.5.0) (A2) renderiza el `plan.drawio` autorado por humanos con **maxGraph**, preservando la geometría que dispusiste (NFR1) y superponiendo el estado de §4 como colores de celda no destructivos. [`loom-0.6.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.6.0) ([#268](https://github.com/StrangeDaysTech/straymark/issues/268)) agregó el north star de la Spec 002: un toggle `2D | 3D` que cambia el plano por una **escena axonométrica en Three.js** — cada layer un floor apilado y translúcido, cada componente una caja coloreada por la misma paleta de "estás aquí", aristas de dependencia dibujadas entre cajas, y un **explode slider** que despega los floors a lo largo del eje vertical mientras las líneas de dependencia se re-rutean a las nuevas posiciones de las cajas. La exploded view del BIM, literalmente: el CLI autora la estructura, Loom la renderiza como plano 2D *y* como modelo 3D, y el overlay de estado es siempre la única projection compartida. Cambiar de vista libera el contexto GL limpiamente; hacer clic en una caja abre el mismo panel de detalle de componente que usa el plano 2D.

<figure>
<img src="/img/blog/loom/arch-2d-thumb.png" data-zoom-src="/img/blog/loom/arch-2d.png" alt="Plano de Arquitectura 2D de Loom: cajas planas de componentes — cmd (binaries), Agents (AIOps), Audit Trail, CommsHub, Identity, MCP Server — alimentando a Core (eventbus / storage / ids), cada caja coloreada en rojo para has-debt o azul para implemented" loading="lazy" />
<figcaption><em>Un detalle del Plano de Arquitectura 2D — el <code>plan.drawio</code> autorado por humanos renderizado por maxGraph, superpuesto en vivo con la projection de estado compartida (rojo = <code>has-debt</code>, azul = <code>implemented</code>). Haz clic para ver el plano completo.</em></figcaption>
</figure>

<figure>
<img src="/img/blog/loom/arch-3d-thumb.png" data-zoom-src="/img/blog/loom/arch-3d.png" alt="Vista BIM axonométrica 3D de Loom: layers de arquitectura como floors apilados y translúcidos (CORE, MODULES, ENTRYPOINTS), componentes como cajas coloreadas en rojo (has-debt) o azul (implemented), líneas de dependencia dibujadas entre ellos" loading="lazy" />
<figcaption><em>El mismo modelo como una exploded view 3D — cada layer un floor translúcido, cada componente una caja en la misma paleta de estado, aristas de dependencia entre cajas; el explode slider despega los floors a lo largo del eje vertical. Haz clic para ver la escena completa.</em></figcaption>
</figure>

Es, genuinamente, un placer volar alrededor de tu propia arquitectura y ver brillar el componente activo. Y en la primera corrida real, el overlay que se suponía que era el punto — *¿dónde está la deuda técnica?* — estaba en blanco.

## Dónde estaba realmente la deuda

El overlay `has-debt` mintió tres veces antes de decir la verdad, y cada mentira parecía una feature terminada.

**Mentira #1 — siempre vacío.** Un TDE abierto (Transversal Debt Entry) declara sus documentos `related` en el frontmatter. Esos son *documentos de gobernanza* — AILOGs, audit reviews, Charters — **no paths fuente**. Así que cuando la projection intentó hacer match de la deuda de un TDE contra un componente (que está definido por file globs), no hacía match con nada, cada vez. El overlay no reportaba "no hay deuda"; reportaba "busqué deuda en un lugar donde la deuda nunca se registra". [#273](https://github.com/StrangeDaysTech/straymark/issues/273) ([`cli-3.26.0` / `core-0.6.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.26.0)) tendió el puente: resolver cada AILOG `related` a los archivos que *él* registró como modificados, y alimentar esos paths fuente al match. Con una sutileza que mordió de inmediato — los archivos modificados de un AILOG son la **unión** de su lista de frontmatter `files_modified` y su tabla `## Modified Files`, porque los logs más viejos a menudo cargan solo la lista, y leer únicamente la tabla los dejaba caer en silencio (encendió Identity/Core/Database en Sentinel pero perdió AuditTrail/CommsHub).

**Mentira #2 — sobre-atribuido.** Ahora la deuda aparecía, y aparecía *en todas partes*. Un TDE sobre el módulo AuditTrail encendía `cmd`, `db` e `integration` también — porque un solo AILOG los había cableado a todos en el mismo cambio, y la projection heredaba el footprint entero de ese AILOG. El overlay pasó de en blanco a un manchón. Técnicamente estaba atribuyendo la deuda a "componentes tocados por los AILOGs que el TDE referencia", lo cual es una oración defendible y la respuesta equivocada. La deuda era sobre AuditTrail; el footprint era el subsistema entero.

**Mentira #3 — preciso.** [#276](https://github.com/StrangeDaysTech/straymark/issues/276) ([`fw-4.28.0` / `cli-3.28.0` / `core-0.8.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.28.0)) le dio a un TDE una forma de decir de qué se trata realmente: un campo **`affects`**, una lista de file globs (`["internal/modules/audittrail/**"]`). Cuando está presente, la projection atribuye la deuda a **exactamente** esos paths e ignora el footprint más amplio del AILOG. En ausencia de `affects`, la atribución por footprint queda como fallback — aditiva, sin migración, sin regresión para los TDEs que no se acotan a sí mismos. El framework entrega el campo en `TEMPLATE-TDE.md`; el [`loom-0.6.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.6.2) de Loom no necesitó cambio de código alguno — solo se recompiló contra `core-0.8.0`, porque la projection es compartida y el overlay visual viajó sobre el mismo fix que la terminal.

Hubo una cuarta corrección, más callada, que vale la pena nombrar. Aun cuando la deuda aterrizó en el componente correcto, un módulo que estaba *a la vez* implemented y en deuda pintaba un tranquilo azul "implemented" y la deuda nunca aparecía. Así que el overlay ganó una **prioridad**: `has-debt` y `wiring-gap` rankean por encima de `implemented`, en el plano 2D, la vista 3D y la leyenda. Un componente pinta el estado que necesita atención, no el estado que se siente bien.

<figure>
<img src="/img/blog/loom/arch-3d-explode-thumb.png" data-zoom-src="/img/blog/loom/arch-3d-explode.png" alt="Una órbita cercana de la vista 3D de Loom: una fila descendente de cajas de componentes, las rojas cargando deuda técnica (CommsHub, Identity) junto a las azules implemented (MCP Server, Policy Engine, Status Center)" loading="lazy" />
<figcaption><em>Orbitando el mismo edificio: las cajas rojas cargan deuda abierta, las azules están implemented — el overlay que tomó tres intentos para aterrizar el color en el componente correcto. Haz clic para ver la escena completa.</em></figcaption>
</figure>

## Abriéndolo

Dos releases hicieron la superficie de arquitectura usable más allá de los dos repos que la engendraron. [#279](https://github.com/StrangeDaysTech/straymark/issues/279) ([`cli-3.27.0` / `core-0.7.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.27.0)) volvió el seed scanner **agnóstico de lenguaje y de estructura**: un `ScanConfig`, resuelto desde una sección opcional `architecture:` en `.straymark/config.yml`, que reconoce ~30 lenguajes de fábrica y omite scaffolding de build como `src/main/java`, de modo que un proyecto multi-module Maven/Gradle obtiene un componente por módulo en vez de colapsar en una sola caja `main`. Un proyecto en un lenguaje no-default ya no es un mapa vacío. Y [#280](https://github.com/StrangeDaysTech/straymark/issues/280) entregó la guía de adopter — [`docs/adopters/LOOM.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/LOOM.md) en EN, ES y zh-CN, en vivo en [straymark.dev](https://straymark.dev) — con el workflow recomendado `generate → refine → sync → validate`.

## Lo que deliberadamente no hicimos

La sección de contención. Todo esto sigue siendo **EXPERIMENTAL (Loom v0 / N=1)** y puede cambiar sin ciclo de deprecación — el modelo de arquitectura, los verbos del CLI, la semántica del overlay, la vista 3D. Los defaults del scanner agnóstico no cambian para los layouts Go/Rust/JS existentes (Sentinel sigue sembrando los mismos 14 componentes) — `#279` solo *agrega* alcance, no reconfigura lo que ya funcionaba. Y `affects` es aditivo: el fallback por footprint queda, así que el campo es un opt-in para los TDEs que quieren precisión, no una nueva obligación para los que no. La tentación con una feature north-star es estabilizarla porque por fin se ve impresionante. Se ve impresionante *y* sigue siendo v0 hasta que un segundo proyecto más allá de Sentinel haya recorrido su propio edificio.

## Si llegaste hasta aquí

El ejercicio portátil es el que nos entregó el overlay vacío. Encuentra un dashboard, reporte o métrica en tu mundo que muestre un número tranquilizador — cero incidentes abiertos, cero checks fallando, cero deuda en este servicio. Ahora pregunta si ese cero significa "miré y no hay ninguno" o "miré en un lugar donde esta cosa nunca se registra". El overlay `has-debt` no mentía sobre la cantidad de deuda; mentía sobre su propia competencia para encontrarla, y lo hacía con un azul perfectamente tranquilo. Las tres iteraciones — vacío, manchado, acotado — son lo que tomó hacer que el color signifique lo que una persona que lo lee asume que significa. Cada cero confiado que no rastreaste personalmente merece la misma sospecha: no *si el número es correcto*, sino *si la cosa que lo produce siquiera sabe dónde mirar*.

---

*StrayMark `cli-3.25.0` → `cli-3.28.0`, `core-0.5.0` → `core-0.8.0`, `fw-4.27.0` → `fw-4.28.0`, `loom-0.5.0` → `loom-0.6.2` — ADR [2026-06-02-002](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md) · Spec [002-architecture-plan](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-loom/specs/002-architecture-plan/spec.md) · Guía de adopter [LOOM.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/LOOM.md) · Issues [#268](https://github.com/StrangeDaysTech/straymark/issues/268) · [#273](https://github.com/StrangeDaysTech/straymark/issues/273) · [#276](https://github.com/StrangeDaysTech/straymark/issues/276) · [#279](https://github.com/StrangeDaysTech/straymark/issues/279) · [#280](https://github.com/StrangeDaysTech/straymark/issues/280). Predecesores: [Lo que el segundo lector exigió](what-the-second-reader-demanded) · [Lo que el grafo todavía no podía dibujar](what-the-graph-couldnt-draw-yet).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*
