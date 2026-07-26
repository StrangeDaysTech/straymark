---
slug: what-the-graph-couldnt-draw-yet
title: Lo que el grafo todavía no podía dibujar
authors:
  - jose
tags:
  - straymark
  - loom
  - knowledge-graph
  - visualization
  - rust
  - sentinel
date: 2026-06-14T00:00:00.000Z
description: El primer release de Loom renderizó el grafo de documentos en vivo en el navegador en menos de un segundo. También dibujó 330 edges hacia la nada. La feature visible era la parte fácil — hacer que las referencias realmente resolvieran fue el trabajo, y nos enseñó cuándo un dangling edge es un bug y cuándo es una señal.
---
*El walking skeleton salió rápido y se veía estupendo: un grafo force-directed de cada documento de StrayMark, coloreado por tipo, reconstruyéndose en el navegador en el segundo siguiente a guardar un archivo. Después lo apuntamos a un corpus real — [Sentinel](https://github.com/StrangeDaysTech/sentinel), 395 referencias — y 330 de ellas quedaron dangling. No porque los documentos estuvieran rotos. Porque se referenciaban entre sí del modo en que los humanos escriben, no del modo en que un graph builder ingenuo hace match. El render fue un fin de semana. Hacer que los edges aterrizaran tomó el resto de los follow-ups de Loom M1, y cambió lo que un "enlace roto" siquiera significa.*

<!-- truncate -->

> *330 de 395 referencias dangling. El grafo era técnicamente correcto y prácticamente inútil.*

El primer post de este arco fue sobre la plomería — extraer [`straymark-core`](what-the-second-reader-demanded) para que el CLI y Loom parseen documentos con el mismo código. Este es sobre lo que pasó cuando ese graph builder compartido se topó con un corpus contra el que no había sido afinado, y descubrió que dibujar un grafo es la parte por la que nadie debería impresionarse.

## El walking skeleton

[`loom-0.1.0` / `cli-3.24.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.24.0) ([#240](https://github.com/StrangeDaysTech/straymark/issues/240)) es Loom M1: un dashboard web loopback-only, read-only, que renderiza el grafo de documentos del proyecto en vivo.

El stack, registrado en [`ADR-2026-06-02-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-02-001-loom-stack.md): un servidor **axum + tokio** que construye el grafo vía `straymark-core` y sirve una pequeña API de lectura más un WebSocket; un watcher de sistema de archivos **`notify`** (debounce de 250 ms) que reparsea ante cambios `.md` asentados y empuja un rebuild por el socket — un navegador abierto refleja una edición en bastante menos de un segundo (medido ~255 ms); un frontend **Sigma.js + graphology**, force-directed, coloreado por tipo de documento, donde seleccionar un node ilumina su thread transitivo completo y atenúa el resto. Toda la UI web está **embebida vía rust-embed** — los adopters nunca corren `npm`; descargan un binario.

Dos innegociables vinieron de la postura de seguridad del spec, y vale la pena nombrarlos porque moldearon todo lo que siguió: el servidor **bindea `127.0.0.1` exclusivamente** y **rechaza `Host` headers no-loopback** (anti DNS-rebinding), y es **read-only por construcción**. No hay un write path que agregar después y olvidar asegurar, porque no hay write path en absoluto. El `straymark loom serve` del CLI es un launcher de descarga-bajo-demanda — la compuerta de descarga *es* la frontera del opt-in experimental — y cae de vuelta al binario cacheado cuando está offline.

## De la imagen al instrumento

M2 y M3 convirtieron la imagen en algo que de verdad alcanzarías. [`loom-0.2.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.2.0) ([#241](https://github.com/StrangeDaysTech/straymark/issues/241)) agregó **detección de community Louvain** con coloreo por cluster, un panel de stats del corpus (conteos por tipo/status/riesgo, orphans navegables, referencias dangling) y filtros del lado del servidor por tipo, status, riesgo, tag y rango de fechas. [`loom-0.3.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.3.0) ([#243](https://github.com/StrangeDaysTech/straymark/issues/243)) agregó **rebuilds incrementales** — un parse cache reparsea solo los archivos cambiados y la SPA parchea el grafo en sitio, preservando el layout de todo lo que no se movió en vez de re-barajar el canvas entero en cada tecla — más **reporte de ciclos de dependencias (SCC)** sobre los edges semánticos dirigidos, **tamaño de node basado en centrality** (Betweenness por defecto, de modo que los documentos puente crecen), búsqueda-con-enfoque-de-cámara, "pin subgraph" para aislar un thread, deep-links de abrir-en-editor e internacionalización completa de la UI (`en` / `es` / `zh-CN`) guiada por el idioma configurado del proyecto.

<figure>
<img src="/img/blog/loom/kg-graph-thumb.png" data-zoom-src="/img/blog/loom/kg-graph.png" alt="Vista Knowledge Graph de Loom: un grafo force-directed de documentos StrayMark, nodes coloreados por community y dimensionados por centrality, con etiquetas como 'Decommissioning Saga step 3' y 'Observability bundle: 7 instruments'" loading="lazy" />
<figcaption><em>Un detalle de la vista Knowledge Graph de Loom — documentos como grafo force-directed, nodes coloreados por community Louvain y dimensionados por betweenness centrality, de modo que los documentos puente crecen. Haz clic para ver el corpus completo.</em></figcaption>
</figure>

Todo eso es genuino, y todo eso estaba montado encima de un grafo donde la mayoría de los edges no iban a ninguna parte.

## 330 de 395

Acá está el número que reencuadró el proyecto. Haciendo dogfooding de `loom-0.1.0` contra Sentinel, **330 de 395 referencias estaban dangling** — dibujadas como edges hacia nodes que no existían. Un knowledge graph que no puede conectar cinco sextos de su propio corpus no es un knowledge graph; es un scatter plot con delirios.

La causa no fue corrupción. Fue que los documentos de StrayMark se referencian entre sí del modo en que una persona escribe una cita, y el graph builder hacía match como una base de datos que espera primary keys. Un AILOG dice `CHARTER-12`, o nombra un archivo por su basename, o da un sufijo de path relativo — y el builder, buscando un id de node exacto, no encontraba nada. Peor: categorías enteras de target *ni siquiera eran nodes*: el grafo descubría AILOGs y TDEs y ADRs, pero los Charters, la telemetría de PLAN y los reviews de audit vivían en `.straymark/` y nunca habían sido inyectados, así que cada referencia a ellos quedaba dangling por definición.

[`loom-0.4.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.0) ([#246](https://github.com/StrangeDaysTech/straymark/issues/246)) arregló ambas mitades, y — porque el fix vive en el graph builder compartido de `straymark-core` — `straymark audit` ganó la misma conectividad gratis:

- **Normalización de referencias (R1).** El builder ahora resuelve el target de un edge por id exacto y, si eso falla, por basename de archivo único, sufijo de path relativo único, prefijo `CHARTER-NN`, o prefijo de id datado inicial — **nunca resolviendo un match ambiguo** (dos candidatos significan que se queda dangling, porque un edge equivocado es peor que uno faltante). Los targets resueltos se canonicalizan al id del node.
- **Nodes de entidad (R2).** Un nuevo módulo `core::entities` descubre `.straymark/charters/*.md`, `plans/PLAN-*.telemetry.yaml` y `audits/*/review.md` y los inyecta como nodes `CHARTER` / `PLAN` / `AUDIT`, para que las referencias a ellos tengan algo donde aterrizar.

Medido en Sentinel: referencias dangling **330 → 87**, nodes **131 → 193** (+41 charters, +5 plans, +16 audits), orphans **2 → 0**. Y las 87 restantes *correctamente* se quedan dangling — apuntan a archivos fuera del corpus de gobernanza (`.specify/memory/…`, `constitution.md`). Lo cual planteó la pregunta de verdad.

## Cuándo un dangling edge es una feature

Si 87 referencias no resueltas son *correctas*, entonces "dangling" no puede ser sinónimo de "roto". Un panel que grita por las 87 está crying wolf, y un panel que las esconde está escondiendo enlaces realmente rotos en el mismo bucket. La categoría misma estaba mal.

[#262](https://github.com/StrangeDaysTech/straymark/issues/262) (entregado junto con la vista Architecture en [`loom-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.5.0)) dividió los targets no resueltos en tres clases: un **enlace de gobernanza roto** (un id que *debería* resolver y no lo hace — el caso accionable), una **referencia a archivo** (un path a código, a un spec, a un sidecar — no es un documento, correctamente ausente del grafo de docs) y un **enlace externo** (una URL). En el corpus de Sentinel el panel de referencias dangling cayó de **92 falsas alarmas a 0** enlaces realmente rotos. El dangling edge no fue suprimido; fue *clasificado*. Se volvió una señal con un tipo, que es la única forma en la que un warning vale la pena mostrarse.

Esta es la misma lección que el [arco anterior](what-the-bash-script-said-was-in-sync) aprendió sobre un script bash deprecado que reportaba "in sync": sé leniente con lo que lees, exclusivo con lo que afirmas. R1 es leniencia (parsear la cita que un humano realmente escribió); la regla de nunca-resolver-ambiguo y la clasificación de tres vías son exclusividad (no afirmes un edge que no puedas justificar, no grites roto cuando no lo está). La misma disciplina, distinta superficie.

## El bug que se comía los clicks

Un fix de este tramo merece su propio párrafo porque el síntoma era enloquecedor y la causa era elegante. Los operadores reportaban que los botones del panel lateral de Loom — enlaces de dangling-ref, toggles de community, acciones de stats — se sentían *muertos*, los clicks aterrizando en ninguna parte, de forma intermitente. [`loom-0.4.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.1) ([#247](https://github.com/StrangeDaysTech/straymark/issues/247)) lo encontró: el watcher estaba broadcasteando un "rebuild" por WebSocket cada vez que el tiempo de modificación de un archivo se movía **incluso cuando su contenido no había cambiado** — un save-sin-ediciones del editor, un formatter, un `touch`, una reescritura de cloud-sync. Cada broadcast no-op re-renderizaba los paneles de cada cliente abierto vía `innerHTML`, destruyendo los click handlers recién bindeados una fracción de segundo después de adjuntarlos. El grafo se veía bien; los paneles estaban siendo lobotomizados con un temporizador. Dos fixes: el watcher ahora diffea el grafo nuevo contra el viejo y se queda en silencio cuando son idénticos, y los paneles usan **event delegation** sobre contenedores estables para que los clicks sobrevivan a un re-render legítimo. [`loom-0.4.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.2) ([#248](https://github.com/StrangeDaysTech/straymark/issues/248)) luego hizo legibles los grafos de 100+ nodes — densidad de etiquetas acotada al node más central por región de pantalla, un toggle "hide isolated", controles de zoom/fit en pantalla.

## Lo que deliberadamente no hicimos

El servidor se mantiene **estrictamente read-only**. Abrir-en-editor es un deep-link del lado del cliente (una URL `vscode://` que el navegador entrega) y copiar-path es una escritura al clipboard en la página — el servidor nunca toca tu editor, tus archivos, ni nada fuera del socket loopback. No hay una feature de "y Loom también puede editar el doc por ti" en el roadmap, porque la postura read-only-por-construcción vale más que la conveniencia, y no puedes agregar la conveniencia sin tirar la postura.

## Si llegaste hasta aquí

El ejercicio portátil: toma las referencias cruzadas en tu propio proyecto — menciones de issues, enlaces de docs `@see`, notas de "depends on" en tickets, el grafo de imports si quieres ser literal — y pregúntate qué fracción realmente resolvería si una herramienta intentara seguirlas. Después hazte la pregunta más difícil que el momento del 330-de-395 nos forzó: de las que *no* resuelven, ¿puede tu tooling distinguir un enlace genuinamente roto de una referencia que correctamente apunta fuera de su propio mundo? Hasta que pueda, cada número de "N referencias rotas" que te muestra es una mezcla desconocida de incendios y falsas alarmas — y un warning que aprendiste a ignorar es peor que ningún warning.

---

*StrayMark `loom-0.1.0` → `loom-0.5.0`, `cli-3.24.0` — ADR [2026-06-02-001](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/02-design/decisions/ADR-2026-06-02-001-loom-stack.md) · Spec [001-loom-server](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-loom/specs/001-loom-server/spec.md) · Issues [#240](https://github.com/StrangeDaysTech/straymark/issues/240) · [#241](https://github.com/StrangeDaysTech/straymark/issues/241) · [#243](https://github.com/StrangeDaysTech/straymark/issues/243) · [#246](https://github.com/StrangeDaysTech/straymark/issues/246) · [#247](https://github.com/StrangeDaysTech/straymark/issues/247) · [#262](https://github.com/StrangeDaysTech/straymark/issues/262). Predecesor: [Lo que el segundo lector exigió](what-the-second-reader-demanded). Siguiente: [Dónde estaba realmente la deuda](where-the-debt-actually-was).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*
