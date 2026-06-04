---
slug: what-the-bash-script-said-was-in-sync
title: Lo que el script bash decía que estaba en sync
authors:
  - jose
tags:
  - straymark
  - follow-ups
  - gobernanza
  - adopters
  - sentinel
  - lnxdrive
  - cli
date: 2026-06-04T00:00:00.000Z
description: Los follow-ups se volvieron la segunda entidad de primera clase de StrayMark — y en un día, dos migraciones externas estresaron el diseño. El script bash deprecado del adopter de referencia llevaba reportando "in sync" mientras 29 entradas le eran invisibles.
---
*Al día siguiente de que los follow-ups se volvieran una entidad de primera clase — schema, namespace de CLI, directivas de agente que viajan, un skill invocable — dos adopters corrieron la migración de punta a punta y presentaron sus reportes. Uno encontró el loop que no habíamos cerrado. El otro encontró algo mejor: el script bash deprecado al que reemplazaba llevaba ciego en silencio quién sabe cuánto tiempo, reportando "in sync" mientras 8 AILOGs y 29 entradas seguían sin extraer. Ambos reportes se convirtieron en releases el mismo día.*

<!-- truncate -->

> *"…a pesar de que el script legacy reportaba **'in sync' tan recientemente como el día anterior**, incluyendo corridas con `--scan-all`."*

Esa línea viene del Issue [#225](https://github.com/StrangeDaysTech/straymark/issues/225), el reporte de actualización de [Sentinel](https://github.com/StrangeDaysTech/sentinel), y es la frase que da nombre a este post. Para explicar por qué importa — y por qué es el argumento más fuerte a favor del carril de releases que cerró — hay que empezar por qué *es* un follow-up.

## El problema del trabajo pendiente

Todo AILOG en StrayMark puede terminar con una sección `## Follow-ups`: las cosas que este cambio difirió. Los riesgos que afloran a mitad de un Charter reciben el mismo trato — `R4 (new, not in Charter): …`. Es una convención de tiempo de escritura, y en tiempo de escritura funciona perfecto: quien implementa sabe exactamente qué está difiriendo, y lo anota junto al trabajo que lo difirió.

El modo de falla es el tiempo de lectura. *"¿Qué está pendiente en el proyecto?"* es una pregunta de operador, y la convención per-AILOG la responde con un barrido multi-archivo que empeora con cada Charter. El adopter de referencia chocó primero con el muro: para el CHARTER-12 de Sentinel la respuesta tenía 47 follow-ups de profundidad repartidos en docenas de AILOGs, y la respuesta del framework — [`fw-4.10.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.10.0), allá en mayo — fue una *convención*: un archivo de registry central, `.straymark/follow-ups-backlog.md`, cinco buckets organizados por **cuándo se vuelve accionable cada entrada**, y un script bash del lado del adopter (~296 líneas de awk y grep POSIX) para detectar AILOGs cuyos follow-ups aún no habían sido extraídos.

Ese carril v0 funcionó — y luego acumuló exactamente el tipo de deuda que esperarías de una convención mantenida a mano a N=91. El Issue [#214](https://github.com/StrangeDaysTech/straymark/issues/214) documentó las señales desde el asiento del operador. Los contadores del frontmatter, mantenidos manualmente, habían derivado a ficción: `total_open: 47` declarado contra 65 reales, cuatro semanas después de la última reconciliación. Y cada batch de extracción cargaba 20–75% de ruido — entradas que ya estaban *resueltas* cuando el script las agregaba, porque el texto del bullet lo decía (`closed in-Charter`, `fixed in batch 3`) y el script no sabía leer.

## Primera clase, con compuerta

La respuesta fue el [ADR-2026-06-03-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-03-followups-first-class.md): el backlog de follow-ups deja de ser una convención y se vuelve el segundo artefacto de primera clase de StrayMark, el mismo carril que tomó Charter — su propio path canónico, su propio schema, su propio namespace de CLI, su propio grupo en la TUI de `explore`.

[`fw-4.21.0` / `cli-3.19.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.19.0) entregó la sustancia:

- **Un schema JSON (v1, experimental)** para el frontmatter del registry, más las dimensiones v1 de entrada que el operador venía improvisando en prosa: `Severity: blocking` (canonicalizando una convención `PROD-BLOCKER` en las notas), `Origin-class` (artefacto de planeación vs realidad de ejecución), `Labels` libres, un vocabulario de `Destination`.
- **`straymark followups list / status / drift / promote`** — nativo, reemplazando el script bash. `drift --apply` extrae AILOGs no extraídos con granularidad per-AILOG (la que produjo 0 falsos positivos en 76 AILOGs del adopter de referencia), `promote` automatiza la elevación FU → TDE con trazabilidad, `status` es el pulso del registry.
- **Contadores propiedad del CLI.** Los campos `total_*` se recalculan desde los estados reales de las entradas en cada escritura. El modo de falla del counter-drift silencioso no queda "desaconsejado" — queda mecánicamente muerto.
- **Extracción anti-ruido.** Los bullets cuyo texto fuente carga un marcador de cierre aterrizan como `suspected-closed` en vez de contaminar el bucket ready como ruido TBD. El operador confirma o reabre en el triage.
- **Directivas de agente que viajan.** `AGENT-RULES.md §13` se entrega con el framework: mirar el registry al inicio de sesión, `drift --apply` en el mismo commit que el AILOG, triage al cierre de Charter. Los adopters dejaron de copiar un bloque a sus propias configuraciones de agente.

Y un freno deliberado, en el propio ADR: el schema sale como **v1 experimental**. La estabilización dura está gateada a un segundo adopter — la misma disciplina N=2 que [el post anterior](what-the-feature-flag-compiled-away) dedicó su sección central a defender. El workflow de un adopter, por bien documentado que esté, es la forma de un solo stack.

[`fw-4.22.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.22.0) completó la superficie con la parte que tenía que esperar un release de framework: el skill `/straymark-followups`, en las cuatro superficies de agente, envolviendo las directivas del §13. Un wrapper delgado por diseño — su `allowed-tools` omite `Write` deliberadamente, porque toda mutación del registry pasa por el CLI y los contadores siguen siendo propiedad del CLI. El skill conduce la disciplina; no posee lógica.

## Y entonces ambos adopters llegaron a la vez

Lo que pasó después es la parte que vale la pena dejar escrita, porque comprimió en un solo día el ciclo de feedback que la compuerta existe para esperar.

**[LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) adoptó el registry en frío** — primera adopción externa, 74 docs, Issue [#222](https://github.com/StrangeDaysTech/straymark/issues/222). La extracción en sí fue limpia (verificada contra un grep manual: cero AILOGs perdidos). Ambos hallazgos estuvieron en los bordes:

- **El loop que no habíamos cerrado.** El ciclo de vida sanciona explícitamente el triage *manual* — el operador cambia estados a mano. Pero los únicos comandos que recalculaban los contadores propiedad del CLI eran `drift --apply` y `promote`, y ambos son no-ops cuando no hay nada que extraer ni promover. Tras una sesión de triage puro, el registry quedaba con contadores conscientemente obsoletos y sin vía conforme para arreglarlos: editarlos a mano viola el §13, y la promesa del warning de `status` ("la siguiente escritura los recalcula") solo era verdad si una escritura futura llegaba a ocurrir. El workflow sancionado y el invariante no se encontraban.
- **Un modismo de cierre que no hablábamos.** Las dos entradas extraídas nacieron resueltas — su texto fuente decía `Charter row updated atomically in this PR`. El vocabulario anti-ruido conocía `closed in-Charter`, `fixed in batch N` y hashes de commit; no conocía esta fórmula, así que ambas entradas aterrizaron como ruido TBD `open`. La regresión exacta que el refinamiento existía para matar, a través de un modismo no reconocido.

[`fw-4.23.0` / `cli-3.20.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.20.0) salió el mismo día: un verbo dedicado **`followups recount`** (recalcula contadores, no toca nada más, idempotente), `drift --apply` reconciliando contadores incluso con cero extracciones, la **familia de modismos born-resolved** en el vocabulario de cierre — y, más duradero, la tabla de modismos canónicos documentada en el pattern doc, para que los autores de AILOGs converjan en fórmulas reconocibles al escribir, en vez de descubrir las no reconocidas al extraer.

**Sentinel corrió la migración oficial** — Issue [#225](https://github.com/StrangeDaysTech/straymark/issues/225): borrar el script, `drift --scan-all --apply`, reapuntar el hook de pre-commit. Y el parser nativo extrajo de inmediato 29 entradas de 8 AILOGs que el script bash había reportado como completamente en sync — incluyendo corridas con `--scan-all` — el día anterior.

La causa raíz, verificada contra el script borrado en el historial de git, es del tipo de cosa que hace que "deprecado" se quede corto como palabra. El extractor awk exigía un heading de sección `## Risk` *y* la forma exacta de bullet `- **R<N> (new`. Esos 8 AILOGs escribían sus riesgos como párrafos planos — `R4 (new, not in Charter): …` en columna 0, sin bullet, sin heading. Matching sensible al formato, cero matches, ningún error. Los AILOGs nunca llegaron siquiera a registrar que *tenían* contenido de follow-ups. Al script no le faltaban features. Producía **falsos negativos silenciosos en la propia detección de drift — la única cosa que existía para prevenir.**

El parser leniente los capturó todos porque la leniencia fue una decisión de diseño, no un accidente. `cli-3.19.1` ya había ampliado el parsing de estados por la misma razón: los operadores anotan en línea (`open — **OVERDUE** (…)`) y un parser de match exacto habría degradado 4 de las 65 entradas de Sentinel a `unknown` y escrito contadores incorrectos en la primera migración. Parsear lo que los humanos realmente escriben, y dejar que la validación de schema aconseje. Cada variante de formato que las herramientas estrictas tiraban al piso resultó ser estructural.

[`fw-4.23.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.23.1) cerró el reporte: el párrafo de migración del pattern doc ahora manda `--scan-all` en el primer barrido post-migración *aunque el script legacy dijera "in sync"* — citando el dato 8/29 como la razón. (El otro hallazgo de Sentinel, el upgrade v0 → v1 que no disparaba en un `--apply` no-op, había sido arreglado horas antes por el carril de LNXDrive — los dos reportes se resolvieron los bordes mutuamente sin saberlo.)

Hubo también una confirmación silenciosa enterrada en el reporte de Sentinel, y merece el reflector por una frase. A mitad del triage, con 29 entradas siendo reclasificadas a mano, `status` imprimió *"frontmatter says total_open: 91 but the real count is 77 — run `straymark followups recount`"* — en el momento exacto — y un comando reconcilió todo. El modo de falla de counter-drift que abrió el issue #214 está muerto, y el adopter que lo reportó lo vio morir.

## Lo que seguimos sin hacer deliberadamente

La sección de contención, porque todos los posts de esta serie tienen una y la disciplina solo cuenta si sobrevive a las buenas noticias.

El reporte de Sentinel ofreció dos modismos de cierre más (`Validated: … No vulnerabilities found`, `Bundled into CHARTER-19`) — una ocurrencia cada uno. Quedan *anotados*, no entregados. La nueva sección de vocabulario del pattern doc enuncia la regla a la que nos atenemos: proponer un modismo upstream cuando **recurre**. Un vocabulario que absorbe cada fórmula de una sola vez deja de ser un vocabulario.

El schema sigue siendo **v1 experimental**. Dos migraciones productivas con cero hallazgos a nivel de schema es exactamente la señal favorable que la compuerta se construyó para medir — ambos reportes fueron ergonomía y vocabulario, nada estructural — pero la decisión de estabilización dura es una decisión que se toma con la cabeza fría, no un reflejo que se entrega la misma semana que la feature. Y la integración suave con `charter close` (auto-correr `drift --apply` al cierre de Charter) sigue gateada tras una señal de fricción que ningún adopter ha enviado.

## Si llegaste hasta aquí

El ejercicio portátil esta vez es casi vergonzosamente pequeño. Elige el archivo — o la etiqueta de issues, o la convención de TODOs — que responde *"¿qué está pendiente?"* en tu proyecto. Ahora revisa dos cosas. Primera: ¿quién mantiene sus números de resumen, un humano o una herramienta? Si es un humano, están mal; la única pregunta es por cuánto (nosotros lo medimos: 18, después de cuatro semanas). Segunda: si tu tooling de extracción reportó hoy "nada que hacer", ¿sabrías si eso es *verdad* o si tus últimos tres pendientes diferidos simplemente no matchearon su regex? La diferencia entre esos dos estados es invisible por construcción — eso es lo que significa *silencioso* — y la única salida es tooling leniente con lo que lee y exclusivo con lo que posee.

Y si mantienes una convención de trabajo diferido que no hemos visto — otro vocabulario de buckets, otro modismo de cierre, otra respuesta a quién posee los contadores — el canal es el mismo por el que pasaron #214, #222 y #225: [abre un issue](https://github.com/StrangeDaysTech/straymark/issues). El schema v1 es experimental precisamente porque tu registry es el que todavía no conoce.

---

*StrayMark `fw-4.21.0` → `fw-4.23.1`, `cli-3.19.0` → `cli-3.20.0` — ADR [2026-06-03-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-03-followups-first-class.md) · Issues [#214](https://github.com/StrangeDaysTech/straymark/issues/214) · [#220](https://github.com/StrangeDaysTech/straymark/issues/220) · [#222](https://github.com/StrangeDaysTech/straymark/issues/222) · [#225](https://github.com/StrangeDaysTech/straymark/issues/225) · PRs [#217](https://github.com/StrangeDaysTech/straymark/pull/217) · [#218](https://github.com/StrangeDaysTech/straymark/pull/218) · [#221](https://github.com/StrangeDaysTech/straymark/pull/221) · [#223](https://github.com/StrangeDaysTech/straymark/pull/223) · [#227](https://github.com/StrangeDaysTech/straymark/pull/227). Patrón: [`FOLLOW-UPS-BACKLOG-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md) (v1, experimental). Predecesores: [Lo que el binario no pudo esconder](what-the-binary-couldnt-hide) · [Lo que el feature flag compiló fuera](what-the-feature-flag-compiled-away).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*
