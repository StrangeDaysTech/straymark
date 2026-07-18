---
slug: what-the-follow-up-took-for-granted
title: Lo que el follow-up dio por sentado
authors:
  - jose
tags:
  - straymark
  - governance
  - followups
  - charters
  - cli
draft: false
date: 2026-07-18T00:00:00.000Z
description: Un adopter greenfield drenó un backlog de follow-ups antes de publicar y descubrió que tres de ellos — los tres que eran trabajo real — cargaban cada uno una premisa que era falsa en el momento en que la chequeabas. No porque el código hubiera cambiado bajo ellos, sino porque un follow-up se escribe en el único momento en que estás menos equipado para verificarlo. La lección no es "escribe mejores follow-ups". Es que un follow-up es una hipótesis fechada, y el lugar barato para probarla es cuando la lees, no cuando la escribes — así que ahí es donde StrayMark ahora pone el chequeo.
---

*Un adopter drenó un backlog de follow-ups antes de publicar — de siete entradas a una. Cuatro eran ruido ya resuelto. De las tres que eran trabajo real, sin resolver, cada una cargaba una premisa falsa: un test a "replicar" que nunca existió, una gate a construir contra una referencia que no puede existir, una optimización para arreglar un coste que no estaba ahí. Ninguna de las tres estaba mal porque el código hubiera derivado bajo ella. Estaban mal en el momento en que se escribieron, y la falsedad solo afloró en un chequeo de treinta segundos hecho meses después. Ese timing es toda la historia. Un follow-up se escribe en el momento exacto en que menos puedes verificarlo — y se lee en un momento en que verificar es casi gratis. El registry de StrayMark solía tratar las entradas como instrucciones a ejecutar. Se entienden mejor como hipótesis fechadas a re-testear — y, a partir de este release, eso es lo que la herramienta dice y donde pone el chequeo.*

<!-- truncate -->

## El escenario

El adopter es la misma librería CRDT greenfield en .NET 10 / Rust detrás de [#345](https://github.com/StrangeDaysTech/straymark/issues/345)/[#346](https://github.com/StrangeDaysTech/straymark/issues/346)/[#355](https://github.com/StrangeDaysTech/straymark/issues/355)/[#360](https://github.com/StrangeDaysTech/straymark/issues/360) — la discusión [Adopter] Weft. El Milestone 3 estaba code-complete salvo la publicación gateada por el operador. Antes de tirar de esa palanca irreversible, el plan era drenar el backlog de follow-ups: siete entradas abiertas, para que lo que se publicara no se publicara con brechas conocidas-pero-sin-cablear congeladas en un paquete público.

El triage las separó rápido. Cuatro no eran trabajo: dos eran riesgos ya resueltos que el extractor había raspado, una era una decisión diferida a un trigger que no se había disparado, una estaba bloqueada por un merge upstream fuera del control del proyecto. Eso dejó **tres follow-ups que eran trabajo genuino, sin resolver, accionable.** Los tres recibieron un Charter. Los tres, al inspeccionarlos, resultaron descansar sobre algo que no era cierto.

## Tres follow-ups, tres premisas falsas

**El test de paridad que no existía.** Un follow-up decía, en efecto, "replica para el shim de Loro el test de paridad header↔binding que el shim de yrs ya tiene". Razonable — salvo que el test de yrs no existía. Lo que existía eran dos *comentarios* — uno en el binding de C#, otro en el header de C — ambos afirmando "un test de CI valida que estas declaraciones coinciden con este header". Nunca se escribió tal test; la línea era aspiracional, rastreable hasta una nota de investigación que decía que el chequeo *podría* generarse. El autor del follow-up había leído un comentario como un hecho. El follow-up heredó la falsedad del comentario y añadió un salto de autoridad. Construirlo "tal cual" habría significado portar un test que no tenía original.

**La gate contra una referencia que no puede existir.** Otro pedía una gate de determinismo para el motor Loro, espejo de la gate de paridad yrs↔Yjs existente. La simetría es seductora: yrs tiene una implementación de referencia independiente (Yjs) contra la cual chequear byte-a-byte, así que seguramente Loro debería tener su equivalente. No puede. No hay una segunda implementación independiente del formato de Loro — el paquete npm es un build en WebAssembly del *mismo* core en Rust, así que compararse contra él es comparar el crate consigo mismo. El follow-up razonó por analogía, y la analogía falló en silencio. La gate realizable era una cosa distinta, más modesta (auto-determinismo entre corridas, un testigo de regresión en vez de una prueba de paridad) — que es lo que se construyó, pero solo porque la premisa se cazó primero.

**La optimización para un coste que no estaba ahí.** El tercero temía que reordenar el relay a persist-before-broadcast pusiera I/O en el hot path del actor y dañara el throughput. Trazar la cadena de `await` real mostró que la llamada de persistencia *ya* se await-eaba en el receive loop, antes de que la conexión leyera su siguiente frame — reordenar no añadía I/O a ningún hot path en el que no estuviera ya. Un harness de carga lo confirmó: el ordenamiento "seguro" costaba efectivamente cero en p50/p99. El follow-up había codificado un *modelo mental* de la arquitectura, no la arquitectura como estaba construida. Este es el más sutil de los tres: ningún comentario mentiroso, ninguna analogía rota — solo un mapa en la cabeza de alguien que había derivado un poco del territorio, y una nota que registró fielmente el mapa.

Tres follow-ups. Un comentario creído, una analogía sobre-confiada, un modelo mental ligeramente obsoleto. Modos de falla distintos, una forma compartida: **cada premisa era falsa en el momento de escribirla, y barata de falsificar en el momento de leerla.**

## Los dos momentos

Hay exactamente dos lugares donde podrías poner una regla de "verifica la premisa": cuando el follow-up se escribe, o cuando se lee.

Escribir ocurre en el peor momento posible para verificar. Un follow-up es una nota-para-uno-mismo anotada mientras *terminas otra cosa* — cerrando el Charter N, con la cabeza llena del subsistema actual, mirando de reojo uno distinto que estás por dejar. Verificar la mirada de reojo significa un cambio de contexto completo lejos del trabajo que intentas aterrizar. Es caro precisamente *entonces*.

Leer ocurre en el mejor momento posible. Cuando por fin actúas sobre el follow-up, ya estás dentro de ese subsistema, con el código abierto. Chequear "¿existe de verdad este test / existe de verdad esta referencia / existe de verdad este coste?" es un `grep`, la lectura de un archivo, una cadena de llamadas trazada — segundos. Las tres premisas falsas de arriba colapsaron exactamente en estos chequeos.

Así que la verificación no es solo más barata en tiempo de lectura — es *categóricamente* más barata, porque en tiempo de lectura ya pagaste el coste del cambio de contexto por otras razones. La economía apunta en una sola dirección.

## El reencuadre: un backlog es un buffer especulativo

Aquí está la parte que cambia cómo debe entenderse toda la feature. Es tentador concluir "los autores deberían verificar los follow-ups más fuerte antes de escribirlos". Esa es la lección equivocada, y empeoraría la herramienta.

Un backlog de follow-ups es un *buffer especulativo*. Su trabajo es capturar, barato, que algo *podría* valer la pena hacer — para que la señal no se pierda cuando la atención se mueve. Si exigieras verificación en el momento de captura, pasarías el cierre de cada Charter espeleando subsistemas que estás abandonando, y la respuesta racional sería dejar de escribir follow-ups del todo. **La verificación ansiosa anula el propósito del buffer.** La entrada sub-verificada no es un defecto del autor; es el *estatus epistémico esperado* de cualquier cosa en un buffer especulativo.

Lo que significa que las premisas falsas no eran bugs en cómo se escribieron los follow-ups. Eran el estado natural de una hipótesis que nunca se había probado — y un follow-up es una hipótesis. El único bug real habría sido *ejecutar una sin re-testearla*. Que es exactamente la trampa que tendía el encuadre del registry: presentaba las entradas como una lista de tareas, un set de instrucciones, un plan. Leídas como instrucciones, las premisas falsas se vuelven Charters desperdiciados. Leídas como **hipótesis fechadas**, se vuelven lo que son — apuestas baratas a re-chequear en el momento en que estás posicionado para hacerlo.

## Lo que enviamos

El reporte de campo llegó como [#365](https://github.com/StrangeDaysTech/straymark/issues/365). Se partió limpio en dos cambios y — apropiadamente — el reencuadre mismo se trató como una *hipótesis fechada*: se registró como una decisión ([`AIDEC-2026-07-18-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md)), la revisó un humano, y se firmó antes de que cambiara una línea de la doc enviada. Una afirmación sobre cómo tratar afirmaciones merecía la misma disciplina que argumentaba.

### Enmarcar las entradas como hipótesis, y mover la verificación a la ejecución

Liberado como [`fw-4.36.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.36.0) / [`cli-3.37.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.37.0) ([#369](https://github.com/StrangeDaysTech/straymark/pull/369)), en tres capas:

**Las palabras.** La doc del patrón de follow-ups gana una sección de primera clase, *"Estatus epistémico"*, que dice en voz alta la parte que se callaba: el registry es un buffer especulativo, una entrada es una hipótesis fechada y decadente en vez de una instrucción, una entrada sub-verificada es el estado *esperado* y no un defecto de autoría — y el único bug real es ejecutar una sin re-testear su premisa. Las directivas de agente (`AGENT-RULES.md §13`) ganan la regla correspondiente: **escribe barato en la captura; re-verifica la premisa cuando promuevas o actúes — nunca en la captura.** El encuadre fue el cambio que carga el peso. Todo lo demás solo le da dientes.

**Los campos.** Una entrada ahora puede cargar un `Premise` explícito — la suposición que la sostiene — y una fecha `Verified-at`. Ambos son opcionales y el schema sigue en `v1`, así que ningún registry existente cambia. Declarar la premisa es lo que convierte "re-verificar" de un empujón vago en un blanco concreto: *"el shim de yrs ya tiene un test de paridad"* es una oración que puedes falsificar en un `grep`. `Verified-at` ausente significa "nunca re-chequeada desde la captura" — el default honesto; su presencia es procedencia de que la hipótesis se probó contra la realidad antes de que alguien gastara un Charter en ella.

**El checkpoint.** Dos afordancias del CLI ponen el chequeo justo donde es barato:

- `straymark followups verify FU-NNN` superficie la premisa, opcionalmente la registra o actualiza (`--premise "..."`), y sella `Verified-at` cuando confirmas el re-chequeo (`--verified`). Sin flags es read-only — solo te muestra la suposición y pregunta si aún se sostiene. Este es el camino común: una entrada actuada como chore que nunca se vuelve un documento formal de deuda.
- `straymark followups promote FU-NNN --premise-verified` hace lo mismo en el momento en que un follow-up gradúa a un TDE: imprime la premisa con un recordatorio *"¿sigue siendo cierto? re-verifica contra el código"*, y sella `Verified-at` al confirmar.

La regla de diseño bajo ambas: **el CLI recuerda y registra; nunca bloquea.** La promoción procede con o sin el flag; `verify` nunca bloquea nada. No decidirá si tu premisa es cierta — ese es el trabajo del humano, parado en el único lugar donde el chequeo es casi gratis. Cualquier cosa más estricta recrearía el impuesto de tiempo-de-captura que el reencuadre existe para evitar.

### El título que escribió la máquina

El hallazgo secundario era más pequeño y más concreto, y se envió primero, en [`cli-3.36.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.36.2) ([#366](https://github.com/StrangeDaysTech/straymark/pull/366)). Cuando `followups drift --apply` auto-extrae una entrada de la sección `## Follow-ups` de un AILOG, tomaba la *primera línea física* del bullet como título. Los bullets de AILOG son prosa hard-wrapped, así que una oración de arranque quedaba cortada en la columna de wrap del autor — tres de las entradas de esta sesión quedaron literalmente tituladas como *"**Footgun de pack local contaminado con `test-hooks`** — el pack lee de"*, cortadas mid-thought. Una máquina agarrando un fragmento-de-línea pierde el matiz que carga un título a mano, y un follow-up cuyo *título* lo tergiversa arranca su vida ya un poco mal — lo que compone el mismísimo peligro de "leer como instrucción" del que trata el resto de este post.

El fix des-envuelve el bullet, prefiere un span en `**negrita**` de cabecera como título (la convención a la que los autores ya recurren), y si no toma la primera *oración*, con cap en un límite de palabra. La parte sutil fue mantenerlo **hash-neutral**: el registry deduplica entradas por un hash de contenido derivado de la línea cruda, así que un título más bonito tuvo que desacoplarse de la clave de dedup — de lo contrario cada entrada ya-extraída en el registry de cada adopter reaparecería como duplicado en el siguiente escaneo. Los títulos quedaron más nítidos; nada se re-duplicó.

## La versión portable

Si mantienes cualquier backlog de trabajo diferido — un registry de follow-ups, un `// TODO(later)`, un issue etiquetado `someday` — estás manteniendo un buffer de hipótesis, lo llames así o no. Las entradas son baratas de escribir y se escribieron cuando no podías chequearlas. El error no es escribirlas sueltas; eso es correcto, y exigir rigor en la captura solo haría que dejaras de capturar. El error es leerlas como un plan y ejecutar por fe. Re-testea la premisa cuando actúas — estás parado en el único lugar donde es casi gratis — y deja que una premisa falsa te cueste un `grep`, no un Charter.

---

*Base empírica: tres Charters drenando un backlog de follow-ups en el proyecto [Adopter] Weft, 2026-07-16 → 2026-07-18 (7 abiertos → 1). Enviado en StrayMark [`fw-4.36.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.36.0) / [`cli-3.37.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.37.0) (el reencuadre, [#365](https://github.com/StrangeDaysTech/straymark/issues/365)/[#369](https://github.com/StrangeDaysTech/straymark/pull/369), [`AIDEC-2026-07-18-001`](https://github.com/StrangeDaysTech/straymark/blob/main/.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md)) y [`cli-3.36.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.36.2) (fidelidad de título, [#366](https://github.com/StrangeDaysTech/straymark/pull/366)). Relacionados: [#360](https://github.com/StrangeDaysTech/straymark/issues/360), [#355](https://github.com/StrangeDaysTech/straymark/issues/355), [#346](https://github.com/StrangeDaysTech/straymark/issues/346).*

*Este documento se produjo con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad del contenido recae en el autor humano.*
