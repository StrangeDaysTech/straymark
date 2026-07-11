---
slug: what-the-spec-path-only-proved-existed
title: Lo que el spec path solo probó que existía
authors:
  - jose
tags:
  - straymark
  - baton
  - speckit
  - coherence
  - cost-routing
  - rust
  - sentinel
date: 2026-06-25T00:00:00.000Z
description: Un cuarto experimento arrancó con un problema económico — cuando terminen los subsidios de las suscripciones, todo-frontier-todo-el-tiempo deja de ser costeable — y de inmediato chocó con uno estructural. No puedes enrutar trabajo a un modelo más barato si el trabajo está derivando de su propio plan; un router más barato sobre intención olvidada solo automatiza la deriva más rápido. Así que el primer movimiento de Baton no toca modelo alguno. Lee el plan que StrayMark solo había probado que *existía*.
---
*Un cuarto experimento, Baton, arrancó de un pronóstico económico incómodo: las suscripciones mensuales de IA están apuntaladas por un subsidio de consumo, y cuando ese subsidio termine — pago por millón de tokens, como ya facturan los proveedores chinos — correr cada tarea en un modelo frontier deja de ser costeable para un desarrollador independiente. El fix obvio es enrutar el trabajo barato a modelos baratos y reservar el frontier para diseño y adjudicación. El problema no-obvio es que un router más barato que opera sobre intención olvidada solo automatiza la deriva más rápido. Así que el primer entregable de Baton no enruta nada. Lee el plan de SpecKit que StrayMark, hasta ahora, solo había verificado que **existiera** — y lo reconcilia contra lo que realmente se construyó.*

<!-- truncate -->

> *Un router más barato que opera sobre intención olvidada solo automatiza la deriva más rápido. Entonces: coherencia primero, routing después.*

Este es el primer post sobre **Baton**, el experimento hermano más reciente — vive en `experiment-baton/`, junto a [Loom](where-the-debt-actually-was) y la comparación con OKF, con sus propios charters, sus propios tags y su propio graduation gate. Como Loom, está diseñado desde el día uno para graduar un core limpio y typed de vuelta a StrayMark si se lo gana. A diferencia de Loom, arranca del dinero.

## 1. Por qué ahora

El pronóstico es aburrido y difícil de discutir. Una suscripción tier MAX cuesta hoy unos \$100/mes, y ese precio se entiende ampliamente como asentado sobre \$400–600/mes de consumo que el proveedor está absorbiendo por ahora. Los labs chinos ya migraron a facturación medida, por millón de tokens; los occidentales parecen cuestión de tiempo. Cuando el subsidio se evapore, el costo real de la forma en que trabajamos hoy — cada commit, cada resumen de PR, cada clasificación de tarea atómica entregada al mismo modelo frontier que diseñó la arquitectura — se vuelve potencialmente incosteable para un operador en solitario. Y el operador, aquí, era yo.

Cambiar de proveedor de golpe es una apuesta de alta varianza. El movimiento accionable desde hoy es cambiar *cómo* se gasta el cómputo. No todo necesita un modelo frontier: commits, PRs, listas de tareas atómicas, clasificación — los modelos un tier abajo los hacen bien. Reservar Opus / GPT-5.5 para diseño, arquitectura, adjudicación y diagnóstico abierto. La tesis es simple: **apuntar el gasto de tokens al modelo apropiado para el tipo de tarea**, apoyándose en la disciplina de gobernanza que StrayMark ya produce para saber qué tipo de tarea es cada unidad.

Ese es todo el pitch, y sería un proyecto de fin de semana fácil si no fuera silenciosamente equivocado de una forma que hizo falta un bug de Sentinel para ver.

## 2. La trampa que reordenó el trabajo

Aquí está el modo de falla al que un router ingenuo camina directo. StrayMark gobierna la *implementación* — charters, AILOGs, follow-ups, TDEs — y Loom proyecta la arquitectura *emergente* desde esas señales más el código en disco. Pero la arquitectura *intencionada* — el plan global que vive en los artefactos de SpecKit — nunca entra al grafo. Verificado, no asumido: un Charter puede declarar `originating_spec: specs/004/spec.md`, y StrayMark valida solo que **el archivo exista**. `validate_spec_path()` nunca parsea su contenido. El vínculo entre un Charter y su spec es nominal. La projection de arquitectura de Loom lee charters, drift, AILOGs, TDEs, inventario en disco — y nunca una sola vez abre `spec.md`.

Así que la arquitectura intencionada y la emergente son dos planos que nadie reconcilia. La intención se diluye en la implementación, y la divergencia es invisible porque ningún artefacto los compara. Ahora imagina envolver eso con un router que alegremente manda "implementa esta tarea" a un modelo de tier económico. Si el plan de la propia tarea ya derivó, no ahorraste dinero — volviste la deriva más barata y rápida de producir. **Un router más barato sobre intención olvidada solo automatiza la deriva más rápido.** La coherencia tiene que ir primero, o el routing es activamente dañino.

### El villano, sobre datos reales

Esto no es hipotético. El caso canónico es el [issue #304](https://github.com/StrangeDaysTech/straymark/issues/304), un bug real en el adopter Sentinel. Una decisión post-MVP (PM-002, en el backlog de un spec) extendió un contrato de salud per-componente. El spec del frontend — un spec *distinto* — nunca referenció esa decisión. Implementó un contrato **asumido**: campos equivocados, enums equivocados, métricas que no existían. Los mocks codificaron la suposición. Los tests pasaron, en verde. Y solo detonó en staging, como `TypeError: t.find is not a function`.

El mismo mecanismo produjo dos cicatrices más de Sentinel. La **telemetría mockup**: un agente, necesitando una fuente de datos que un módulo planeado-pero-no-construido se suponía debía proveer, emitió mockups en silencio donde los necesitó — así que toda la superficie de telemetría salió falsa, construida sobre un cimiento que no existía. Y la **dispersión de PolicyEngine**: un módulo documentado en `.specify/memory/` como un componente limpio y dedicado cuyas funciones en cambio se dispersaron por otros módulos, porque el agente que tomaba decisiones locales nunca consultó el plan global que decía otra cosa. Ninguno es un descuido humano aislado. Son un solo hueco estructural: *un agente que nunca leyó el plan global de SpecKit toma decisiones locales que contradicen el diseño, y nada se lo dice.*

## 3. Fase 1 — la costura de lectura

Hay tres costuras entre la intención de SpecKit y la gobernanza de StrayMark. El primer Charter de Baton — [`CHARTER-01-coherence-bridge`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-01-coherence-bridge.md), effort L, cerrado el mismo día que se reportó — construye solo la costura de **lectura**. Ingerir la intención, reconciliarla, emitir un diagnóstico. Read-only: diagnostica, no muta SpecKit y no corre agentes. No toca modelos. Esa última restricción es el punto — el driver económico es real, pero el primer entregable le presta exactamente cero atención, porque el problema de coherencia tiene que resolverse antes de que un router sea seguro de construir.

Cuatro piezas, construidas en orden a lo largo de cinco batches:

- **Un adaptador SpecKit versionado, read-only** ([#308](https://github.com/StrangeDaysTech/straymark/issues/308)). Anclado a `speckit_version` (0.11.3, el que corre Sentinel), parsea `specs/**/{spec,plan,tasks}.md`, backlogs post-MVP, `contracts/**`, y el `.specify/memory/` de formato libre. Los archivos de memory se tratan como *pista*, nunca como contrato duro — parsing tolerante, findings de baja confianza, para que la prosa humana no produzca falsas alarmas.
- **Un IntentModel — el tercer plano** ([#309](https://github.com/StrangeDaysTech/straymark/issues/309)). Una representación typed de los componentes, módulos y contratos que SpecKit *declara*, incluidas las decisiones de backlog post-MVP. Reusa los tipos del modelo de arquitectura de Loom donde puede y **no** re-implementa `glob_match` ni drift; todo el principio de diseño es *no dupliques el matcher, compártelo*.
- **Edges de procedencia de contrato cross-spec** — el corazón de #304. Vinculan un consumidor (un spec, un FR) con el productor o decisión que define el contrato del que depende. Inferidos conservadoramente en esta fase; se prefiere un falso negativo silencioso a un falso positivo ruidoso.
- **Un motor de coherencia, C1–C4** ([#310](https://github.com/StrangeDaysTech/straymark/issues/310)), más un overlay de intención consumible por Loom ([#311](https://github.com/StrangeDaysTech/straymark/issues/311)) que extiende la projection a un tercer plano: *intención vs. gobernanza vs. código*. Clases de finding de alta confianza primero: un componente intencionado en memory sin archivos que lo implementen; un campo de contrato que un consumidor requiere sin fuente productora; un consumidor construido contra una forma que una decisión posterior cambió. El CLI espeja `architecture validate` — `text|json|markdown`, exit no-cero ante findings, gateable en CI.

El prototipo vive en un crate `straymark-baton`, espejo de cómo `straymark-loom` se sienta aparte de `core`. La lógica pura y typed queda expuesta para poder graduarla a `straymark-core` después — pero no aún, para que un experimento no acople el core compartido a código que podría desecharse.

## 4. Lo que el dogfood realmente cazó

El graduation gate fue deliberadamente concreto: *la Fase 1 tiene éxito si el diagnóstico, corrido read-only contra Sentinel, caza al menos una deriva arquitectónica real que la revisión humana había dejado pasar.*

Corrido contra Sentinel en HEAD `24d5a66`, con `git status --porcelain` vacío antes y después de ambos comandos — el bridge no muta nada en el repo objetivo — la primera corrida cruda emitió **90 findings**, mayormente ruido. Esa es la forma esperada de un primer dogfood: los datos reales sacan a la luz bugs de precisión que ningún fixture haría. Cuatro fixes de calibración dirigidos lo bajaron a **6 findings, 0 blocking**, cada fix un bug genuino cazado solo por la realidad — una decisión de backlog estaba reclamando cada endpoint que su spec mencionaba en vez de solo los que su propia sección citaba (eso solo colapsó una clase de finding de 84 a 1); archivos de test estaban siendo contados como productores de contrato; un match derivado de memory disparaba sobre un substring.

El titular que sobrevivió es un C4 preciso:

```
[C4] spec '005-frontend-dashboard' consumes contract 'services.public-visibility'
     but never references its defining decision(s): PM-001, AILOG-2026-04-21-002
```

Ese es el patrón #304, estructuralmente, sobre datos vivos: un spec consumidor depende de un contrato que una decisión en *otro* spec definió, sin reconocerlo — la exacta deriva de propagación de decisión cross-spec que nada había sacado nunca a la luz, y que la revisión humana no había señalado. Cazado read-only, sobre código real, en una línea legible.

El overlay de intención agregó el resto del cuadro legible — `DevPortal` y `UsageGuard` diseñados pero sin módulo (gaps reales); `SentinelAgent` en los docs de memory versus un directorio `agents/` en disco (una deriva de nombres). Y un detalle para el que le he tomado confianza a la herramienta: **PolicyEngine ahora muestra `intended & implemented`.** El módulo que el equipo alguna vez olvidó y dispersó fue construido desde entonces, y el bridge correctamente *no* lo señala. Refleja la realidad actual, no una anécdota rancia — que es exactamente lo que necesitas de algo que vas a cablear a un flujo de autoría.

## 5. Lo que deliberadamente no hicimos

La sección de contención, que Baton necesita más que la mayoría, porque el pitch económico es seductor.

No tocamos **modelos.** Ni tier, ni token, ni costo, ni routing — la Fase 1 no importa nada de eso, a propósito. Todo el punto era probar que la coherencia es un prerrequisito, y construir el router primero habría sido el error que el experimento existe para evitar.

Nos quedamos **read-only.** El bridge diagnostica; nunca escribe un patch de vuelta a SpecKit. La autocorrección silenciosa de un contrato es precisamente la clase de confianza-en-lo-equivocado que metió a Sentinel en esto.

Y shippeamos una **limitación honesta**, no una victoria limpia. El keying de contrato sobre archivos de tipos generados (`types.gen.ts` cargaba cada tipo de API en un archivo) funde contratos distintos, así que los findings de mismatch de campo/enum (C2/C3) todavía no se aíslan en Sentinel — la corrida es 0 blocking *por diseño*, no porque la deriva esté completamente cazada. El lado consumidor de eso recibió un fix de binding en call-site ([#313](https://github.com/StrangeDaysTech/straymark/issues/313)); el lado productor simétrico es un follow-up hermano rastreado ([#319](https://github.com/StrangeDaysTech/straymark/issues/319)). Todo esto es **EXPERIMENTAL (Baton v0 / N=1)** y puede cambiar sin ciclo de deprecación. La propuesta de valor de una herramienta de coherencia es confianza, y la confianza se pierde mucho más rápido por un falso positivo confiado de lo que se gana por diez verdaderos — así que las clases de finding arrancan angostas y la severidad arranca baja.

## 6. Si llegaste hasta aquí

La pregunta portátil es la que nos entregó el check nominal del spec-path. Encuentra un lugar en tu propio sistema donde un artefacto *referencia* a otro — un config que nombra un schema, un servicio que declara una dependencia, un ticket que enlaza un doc de diseño — y pregunta qué verifica realmente la referencia. ¿El check confirma que la cosa enlazada *existe*, o confirma que las dos cosas todavía *concuerdan*? StrayMark validó que `originating_spec` apuntara a un archivo real durante meses, y se sintió como un vínculo todo ese tiempo. Estaba probando existencia e implicando coherencia, y la brecha entre esas dos es donde un frontend se construye contra un contrato que nadie actualizó. Cada "está enlazado, así que está bien" que no reconciliaste personalmente merece la misma sospecha — no *si el vínculo está ahí*, sino *si el vínculo verifica algo más allá del nombre de archivo*.

El próximo post es donde Baton por fin mira el dinero — y descubre que aquello que asumió que sería la palanca no lo es.

---

*Baton Fase 1 — [`CHARTER-01-coherence-bridge`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/CHARTER-01-coherence-bridge.md) · concepto [`01-baton-concept.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/01-baton-concept.md) · dogfood [`03-sentinel-dogfood-report.md`](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/03-sentinel-dogfood-report.md). Issues [#304](https://github.com/StrangeDaysTech/straymark/issues/304) · [#308](https://github.com/StrangeDaysTech/straymark/issues/308) · [#309](https://github.com/StrangeDaysTech/straymark/issues/309) · [#310](https://github.com/StrangeDaysTech/straymark/issues/310) · [#311](https://github.com/StrangeDaysTech/straymark/issues/311) · [#313](https://github.com/StrangeDaysTech/straymark/issues/313). Arco hermano: [Dónde estaba realmente la deuda](where-the-debt-actually-was).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*