---
slug: who-the-audit-thought-it-was
title: Quién creyó la auditoría que era
authors:
  - jose
tags:
  - straymark
  - audit
  - attribution
  - framework
  - governance
  - sentinel
date: 2026-07-08T00:00:00.000Z
description: La auditoría de StrayMark solo vale algo si familias de modelos independientes convergen en un finding por sí mismas — el acuerdo es la señal. Una serie de releases endureció esa garantía contra tres formas de fingirla. La más fresca es la más filosa — un router CLI inyecta su propio nombre de producto, así que un auditor firmó su reporte como "qwen-code" aun después de que el operador había cambiado el modelo de backend. A la matemática de convergencia le estaban dando de comer una mentira sobre quién hizo el trabajo.
---
*El valor de la auditoría externa de StrayMark no es que un modelo encontró un defecto. Es que **familias de modelos independientes convergieron** en él — un finding es señal solo cuando auditores de backends genuinamente distintos lo alcanzan por separado, y el paso de review dedupea y ratea sobre esa base. Lo cual significa que todo el aparato descansa en una suposición callada: que la identidad registrada de cada auditor es verdadera. Una serie de releases recientes encontró tres formas en que esa suposición se rompe, y las cerró. La más fresca es la más inquietante, porque nada se veía mal: un auditor firmó con confianza su propio reporte con el nombre del wrapper de CLI dentro del cual corría, no del modelo que en realidad hizo el razonamiento.*

<!-- truncate -->

> *Los router CLIs inyectan una identidad de producto por system prompt, así que los auditores escribían `auditor: qwen-code` aun después de que el operador confirmó que se había seleccionado un backend distinto vía `/model` — corrompiendo la atribución y fingiendo el acuerdo cross-family.*

Esta es una coda, desde un rincón del framework distinto al de la [trilogía de Baton](what-the-author-already-knew) que la precede — pero rima con ella. El arco de Baton fue sobre convergencia al servicio del *routing*; este es sobre convergencia al servicio de la *confianza*. Ambos se reducen a la misma disciplina: una señal solo vale lo que vale su procedencia. El [ciclo de auditoría externa](charters-and-the-external-audit-cycle) de StrayMark ha sido un comando de primera clase desde mayo; estos releases se tratan de volver su garantía central imposible de fingir.

## 1. Por qué la identidad es load-bearing

La auditoría funciona por desacuerdo y acuerdo. Corres el mismo charter frente a auditores de familias de modelos distintas; un defecto que solo una familia levanta es señal débil, un defecto que varias familias levantan independientemente es señal fuerte. El paso de consolidación (`straymark-audit-review`) dedupea findings y ratea auditores sobre esa convergencia. Para que algo de eso signifique algo, dos cosas deben ser verdad: un auditor no debe leer el reporte de otro auditor antes de escribir el suyo, y cada reporte debe estar **honestamente atribuido** a la familia que lo produjo.

Rompe cualquiera de las dos, y la matemática de convergencia no solo se degrada — *miente*. Dos corridas "independientes" que en secreto son el mismo modelo parecen acuerdo. Dos backends genuinamente distintos ambos firmados con el mismo nombre de wrapper colapsan en uno en el dedup. El rating premia o castiga a la familia equivocada. La auditoría sigue produciendo números confiados; solo que dejaron de significar lo que dicen.

Tres releases, tres formas distintas de fingir la convergencia.

## 2. Leer la tarea del hermano (fw-4.27.0, el endurecimiento v1)

La primera y más obvia: un auditor espiando los findings de otro auditor antes de escribir los suyos. Eso se cerró antes, en el endurecimiento v1 del audit-prompt ([#261](https://github.com/StrangeDaysTech/straymark/issues/261)) — independencia de auditores más un contamination guard. Es la línea base: la convergencia solo es evidencia si se alcanza a ciegas. Vale nombrarla aquí porque los dos fixes más nuevos son el mismo principio aplicado a fugas más sutiles.

## 3. Confundir el mock con el territorio (fw-4.32.0)

El pase v1.1 del audit-prompt ([`fw-4.32.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.32.0), [#303](https://github.com/StrangeDaysTech/straymark/issues/303)/[#306](https://github.com/StrangeDaysTech/straymark/issues/306)) apretó *qué* se le permite a un auditor confiar como realidad. Dos reglas aditivas:

- **Objeto de auditoría vs. oráculo de verdad.** El prompt ahora separa *dónde reportas defectos* (el objeto de auditoría — el código en el git range) de *qué puedes leer para validarlos* (un oráculo de verdad — incluidas cosas fuera del range). Una corrida que audita un cliente debe cruzar las llamadas de API/IPC/contrato contra la **definición real del lado servidor**, aun cuando el servidor esté fuera del diff. Un mismatch de contrato cliente↔servidor es un defecto auditable del cliente, y **los tests verdes del lado cliente no lo absuelven** — los mocks codifican la propia suposición del cliente, así que un test que pasa es solo la suposición concordando consigo misma. (Si ese modo de falla suena familiar, es exactamente la deriva #304 que el [coherence bridge de Baton](what-the-spec-path-only-proved-existed) se construyó para cazar, vista desde el lado de la auditoría.)
- **Fidelidad de verificación.** Para cada afirmación de "verificado / resuelto / hecho", el auditor ahora pregunta *contra qué realidad* se chequeó — la condición que de verdad importa, versus un proxy conveniente (un test local, un mock, la propia aserción del documento) — y abre el artefacto en vez de confiar en un resumen aguas abajo. La categoría de finding `real_debt` también se reapuntó al registry de follow-ups de primera clase con promoción a TDE, para que un defecto confirmado aterrice en algún lugar durable en vez de una nota post-auditoría suelta.

Esto es la auditoría aprendiendo a desconfiar de la evidencia conveniente. Los dos releases siguientes la hacen desconfiar de algo que nunca pensó en cuestionar: de sí misma.

## 4. Quién hizo la verificación (fw-4.33.0 y fw-4.34.0)

Aquí está el bug, y es uno bueno porque todo en él se veía correcto.

Los router CLIs — Qwen Code, Gemini CLI y sus parientes — inyectan una **identidad de producto** a través del system prompt. Así que cuando un auditor llenaba el frontmatter de su reporte, se autodetectaba y escribía `auditor: qwen-code` — el nombre del *wrapper* — **aun después de que el operador había cambiado el modelo de backend real vía `/model`.** El reporte estaba bien formado. El campo estaba poblado. Simplemente nombraba al CLI, no a la mente.

Eso hace dos cosas malas a la vez. **Corrompe la atribución** — ya no puedes decir qué familia de modelos produjo realmente el finding. Y puede **fingir el acuerdo cross-family**: dos corridas sobre backends genuinamente distintos ambas se firman con el mismo nombre de producto de CLI, o una corrida firma la familia equivocada, y la matemática de convergencia-y-dedup sobre la que descansa toda la auditoría se envenena en la fuente. El rating de auditores — la cosa que se supone te dice qué familias son confiables — ratea un fantasma.

[`fw-4.33.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.33.0) vuelve la **identidad provista por el operador** autoritativa para el auditor. El Paso 2 de `straymark-audit-execute` se reescribió para tomar la identidad de un segundo argumento opcional — `/straymark-audit-execute <CHARTER-ID> <AUDITOR-SLUG>` — o una declaración en el chat, para **prohibir sustituir el nombre de producto del CLI**, y para agregar un **guard post-escritura** obligatorio que verifica que el frontmatter `auditor:` y el header del reporte ambos coincidan con el slug provisto. La autodetección sobrevive solo como fallback de último recurso cuando el operador no provee nada.

[`fw-4.34.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.34.0) aplica el fix idéntico al **calibrador** — la identidad en el paso de consolidación (`straymark-audit-review`), los campos `calibrator:` / `**Reviewer:**` — con el mismo segundo argumento opcional (`/straymark-audit-review <CHARTER-ID> <CALIBRATOR-SLUG>`) y el mismo guard post-escritura. Porque el reviewer que *juzga* la convergencia tiene que estar tan honestamente atribuido como los auditores que la produjeron; un árbitro mal etiquetado no es mejor que un jugador mal etiquetado.

Ambos fixes shippearon a través de las cuatro copias runtime — el workflow `.agent` y los skills `.claude` / `.gemini` / `.codex` — porque el bug vive dondequiera que el prompt corra, y un fix en una copia es un fix que falla en el momento en que el operador cambia de CLI.

## 5. Lo que esto deliberadamente no es

No es un **detector.** El framework no puede meter la mano en un CLI y leer qué backend está realmente seleccionado; el operador sí, y el fix vuelve al operador la autoridad en vez de fingir que la herramienta puede decirlo. El guard verifica *consistencia* — que lo que se escribió coincida con lo que el operador declaró — no verdad de fondo sobre el silicio.

No es **autodetección disfrazada.** La autodetección se queda solo como fallback, y es explícitamente la ruta menos confiable. Todo el movimiento es degradar la conjetura de la herramienta sobre sí misma por debajo de la declaración del humano, exactamente como la [graduación de work-verb de Baton](what-the-author-already-knew) degradó un escaneo de título por debajo de un verbo declarado. La misma lección, otro rincón: la señal autoritativa es la que el operador declara, no la que la máquina infiere sobre su propia identidad.

## 6. Si llegaste hasta aquí

La pregunta portátil es sobre procedencia en cualquier sistema que agrega juicios independientes — sign-offs de code review, findings de red-team, votos de ensemble de modelos, segundas opiniones. El agregado solo es tan confiable como la *identidad* atada a cada input, y la identidad es exactamente el campo que nadie recontrola, porque se puebla automáticamente y siempre se ve plausible. `auditor: qwen-code` es un valor perfectamente bien formado. Solo que responde "qué herramienta corrió esto" cuando la matemática necesitaba "qué mente produjo esto", y esas dos divergieron en el instante en que un wrapper te dejó cambiar el backend por debajo. El audit-prompt pasó releases aprendiendo a confiar en *qué* se verificó. Estos dos lo hacen confiar en *quién* hizo la verificación. Ve a encontrar el campo de identidad en tu propia tubería que se llena gratis — y pregunta si nombra la cosa de la que en realidad dependes, o solo la cosa que dio la casualidad de estar sosteniendo la pluma.

---

*StrayMark Framework [`fw-4.32.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.32.0) → [`fw-4.33.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.33.0) → [`fw-4.34.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.34.0), sobre el endurecimiento v1 de fw-4.27.0. Issues [#261](https://github.com/StrangeDaysTech/straymark/issues/261) · [#303](https://github.com/StrangeDaysTech/straymark/issues/303) · [#306](https://github.com/StrangeDaysTech/straymark/issues/306). Relacionados: [Charters como entidad de primera clase, y el audit cycle externo](charters-and-the-external-audit-cycle) · [Lo que el autor ya sabía](what-the-author-already-knew).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*