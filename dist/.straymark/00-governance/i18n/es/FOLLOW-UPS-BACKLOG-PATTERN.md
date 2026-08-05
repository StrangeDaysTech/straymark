# Patrón de Backlog de Follow-ups - StrayMark

> Registry de primera clase para gestionar entradas acumuladas de `§Follow-ups` y `R<N> (new, not in Charter)` a lo largo de muchos AILOGs y Charters.

**Idiomas**: [English](../../FOLLOW-UPS-BACKLOG-PATTERN.md) | Español | [简体中文](../zh-CN/FOLLOW-UPS-BACKLOG-PATTERN.md)

---

## Estado

**v1 — entidad de primera clase desde fw-4.21.0 / cli-3.19.0** (experimental; la estabilización dura está condicionada a un segundo adopter, según el principio de diseño #12 y ADR-2026-06-03-001).

Cronología de maduración, espejando el carril del Charter:

| Etapa | Release | Qué aterrizó |
|-------|---------|--------------|
| Convención (v0) | fw-4.10.0 | Documento del patrón + script bash del lado del adopter (Sentinel CHARTER-12, N=47) |
| Refinamiento (v0.1) | fw-4.13.1 | Ruta de promoción FU → TDE (2 formas), contador `total_promoted` |
| **Primera clase (v1)** | **fw-4.21.0 / cli-3.19.0** | Schema JSON, CLI nativo `straymark followups`, integración con `explore`/`status`, directivas de agente shippeadas en `AGENT-RULES.md §13`, template del registry |

El registry es un **artefacto de primera clase** como el Charter — no es uno de los 16 tipos de documento. Tiene su propia ruta canónica, su propio schema, su propio namespace de CLI y su propio grupo sintético en la TUI de `explore`.

---

## Cuándo aplica este patrón

La convención per-AILOG `§Follow-ups` de StrayMark funciona en tiempo de escritura — cuando se crea un AILOG, el implementador documenta lo que se difiere a Charters subsiguientes o triggers operativos. Eso funciona bien hasta que la lista acumulada crece más allá de lo que un operador puede escanear de memoria.

Adopta este patrón cuando se cumpla **cualquiera** de estas condiciones:

- El proyecto ha acumulado **~20 o más AILOGs** con secciones `§Follow-ups` no triviales.
- Los operadores piden repetidamente a los agentes "lista qué está pendiente en el proyecto" y la respuesta requiere un escaneo multi-archivo.
- Un follow-up de tipo "haz esto cuando llegue X" estuvo a punto de perderse porque el AILOG originador nunca fue releído después de que llegó X.
- Una retrospectiva de Charter aflora follow-ups que deberían haber sido clasificados como `closed` semanas antes pero nunca fueron indexados.

Por debajo de ese volumen, la convención per-AILOG por sí sola es suficiente — adoptar este patrón temprano agrega overhead de mantenimiento sin retorno.

### El registry como input de planificación

Lección empírica del adopter de referencia (issue #214, N=91 entradas): el backlog es más que una lista de chores diferidos. Los follow-ups se originan no solo de la planificación (ex-ante) sino de la **realidad de ejecución** — corridas de tests, lecturas de telemetría, incidentes de staging, bugs observados en entornos reales (no simulados) — y retroalimentan la planificación: se vuelven chores, mini-charters, o incluso reconfiguran Charters que ya estaban planeados. El registry es el **contraparte ex-post de SpecKit**: SpecKit alimenta la planificación desde la intención; el backlog la alimenta desde la ejecución. Las dimensiones de v1 (`Origin-class`, `Severity`, `Labels`, el vocabulario de `Destination`) existen para hacer ese bucle de planificación consultable.

*Alimenta* la planificación — pero no lo leas *como* un plan. Cada entrada es una apuesta a re-chequear cuando se toma, no una tarea comprometida (ver "Estatus epistémico" a continuación).

---

## Estatus epistémico — las entradas son hipótesis fechadas

*(AIDEC-2026-07-18-001, del reporte de campo de Weft #365.)*

Un backlog de follow-ups es un **buffer especulativo**. Su valor es la *captura barata* para no perder una señal mientras terminas otra cosa. Esa baratura es justamente el punto — y dicta cómo debe leerse una entrada y cuándo debe verificarse su premisa.

**Una entrada es una hipótesis fechada y decadente, no una instrucción.** Registra lo que parecía cierto y valía la pena hacer en el momento de captura — una nota anotada con la atención puesta en otro trabajo. Esa nota puede ser falsa en el instante en que se escribe: un comentario creído, una analogía sobre-confiada, un modelo mental ya volviéndose obsoleto. (En el reporte fundacional, tres follow-ups genuinos descansaban cada uno sobre una premisa que era falsa *al momento de escribirse* y costó un chequeo de ~30 segundos falsificar meses después.) Por lo tanto:

- **Una entrada sub-verificada no es un defecto de autoría** — es el estatus epistémico *esperado* de cualquier cosa en un buffer especulativo. Exigir verificación en la captura anularía el propósito del buffer; la respuesta racional sería dejar de escribir follow-ups y perder la señal.
- **El único bug real es ejecutar una entrada sin re-testear su premisa.** Leídas como instrucciones, las premisas falsas se vuelven Charters desperdiciados. Leídas como hipótesis fechadas a re-chequear en la ejecución, se vuelven apuestas baratas.

**Disciplina — escribe barato en la captura; re-verifica la premisa cuando promuevas/actúes:**

- En la captura, anota el follow-up libremente. Opcionalmente declara su **`Premise`** — la suposición que lo sostiene — para que el re-chequeo posterior tenga un blanco concreto.
- En la ejecución (estás por construir sobre ella), **re-verifica la premisa contra el código primero** — un `grep`, la lectura de un archivo, una cadena de llamadas trazada. Ya estás dentro de ese subsistema, así que el chequeo son segundos. Regístralo con `straymark followups verify FU-NNN --verified` (o `followups promote FU-NNN --premise-verified`), que sella **`Verified-at`**.
- `Verified-at` ausente = nunca re-chequeada desde la captura (el default honesto). Su presencia es la procedencia de que la hipótesis se probó contra la realidad antes de gastar esfuerzo en ella.

---

## Forma

### Archivo del registry

Único archivo markdown en la ruta canónica:

```
.straymark/follow-ups-backlog.md
```

Un template con frontmatter vacío y los cinco headers de bucket se shippea en `.straymark/templates/follow-ups-backlog.md`.

### Frontmatter (YAML)

```yaml
---
last_scan: 2026-06-03
last_scan_range: AILOG-NNNN-NN-NN-NNN..AILOG-NNNN-NN-NN-NNN  # opcional — primer..último AILOG cubierto
schema_version: v1
total_open: 0                # CLI-owned — recalculado en cada escritura
total_promoted: 0            # CLI-owned
total_closed_in_session: 0   # CLI-owned
total_phase_blocked: 0       # CLI-owned
total_suspected_closed: 0    # CLI-owned (nuevo en v1)
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-04-11-001
  - AILOG-2026-04-12-001
  # ... una entrada por cada AILOG cuyos follow-ups han sido procesados
---
```

**Los contadores `total_*` son CLI-owned desde v1.** Cada comando de escritura (`straymark followups drift --apply`, `straymark followups promote`) los recalcula desde los estados reales de las entradas. No los mantengas a mano — los valores rancios editados a mano se corrigen en la siguiente escritura. Esto cierra el modo de fallo de drift silencioso de contadores observado en N=91 (declarado `total_open: 47` vs 65 real tras 4 semanas — issue #214 Señal 2). `straymark followups status` siempre muestra conteos recalculados al vuelo, así que el pulso es confiable incluso si el archivo está rancio.

La lista `fully_extracted_ailogs` registra todo AILOG cuyas entradas de `§Follow-ups` y `R<N>` han sido transferidas al registry (o explícitamente clasificadas como superseded). Desde cli-3.21.0 es **informativa** (la muestra `followups status`); la detección de drift deduplica por hash de contenido por follow-up, no por esta lista — ver "Dedup por hash de contenido por follow-up" abajo.

El schema formal del frontmatter es `.straymark/schemas/follow-ups-backlog.schema.v1.json` (v1 experimental — ver Estado arriba).

### Buckets

Cinco buckets organizan las entradas por tipo de trigger — *cuándo son accionables*:

- `ready` — accionable ahora, sin dependencia de trigger externo.
- `time-triggered` — trigger basado en calendario (ciclo de auditoría, revisión periódica).
- `charter-triggered` — bloqueado por un Charter futuro que toque el área relevante.
- `phase-blocked` — bloqueado por un componente o fase futura que aún no existe.
- `operational` — decisión manual del operador o acción de sistema externo.

El vocabulario es estable en N=91 entradas en el adopter de referencia — no se ha necesitado un sexto bucket. La Severity (*cuánto duele saltársela*) intencionalmente **no** es un bucket: es un campo per-entry ortogonal (ver abajo).

### Esquema de entrada (v1)

Cada entrada dentro de un bucket sigue esta forma (campos v1 marcados; todos opcionales — las entradas v0 siguen siendo válidas):

```markdown
### FU-NNN — <descripción corta>
- **Origin**: AILOG-NNNN-NN-NN-NNN <pointer a la sección fuente>
- **Source-hash**: <12 hex>                                                           (cli-3.21.0+, auto-gestionado — la clave de dedup de drift; no editar a mano)
- **Origin-class**: ex-ante-planning | testing | telemetry | staging | real-env-bug   (v1, opcional)
- **Status**: open | in-progress | suspected-closed | closed | superseded | promoted
- **Severity**: normal | blocking                                                     (v1, opcional; default normal)
- **Work verb**: design | implement | audit | operate                                 (opcional; clasificación de trabajo declarada, Baton #332)
- **Design provenance**: new | upstream                                               (opcional; sólo para implement — upstream degrada a operator)
- **Trigger**: ready | <fecha calendario> | when <X> | <otro>
- **Destination**: chore | mini-charter | charter-replanning | operations | <charter-id> | <TDE id>
- **Cost**: <estimación de esfuerzo>
- **Labels**: <tags libres, separados por comas>                                       (v1, opcional)
- **Premise**: <la suposición que sostiene esta entrada>                               (v1, opcional)
- **Verified-at**: <YYYY-MM-DD en que la premisa se re-chequeó contra el código>       (v1, opcional)
- **Notes**: <contexto libre>
- **Promoted to**: <id de TDE, cuando Status: promoted — ver "Promoción a TDE" abajo>
```

`FU-NNN` es monotónicamente creciente a lo largo de la vida del registry; no se renumera cuando las entradas se cierran.

**Las dimensiones de v1**, cada una canonicalizando una necesidad observada empíricamente (issue #214):

- **`Origin-class`** — dónde nació la entrada: artefactos de planificación (ex-ante) vs realidad de ejecución (testing, telemetry, staging, bugs de entorno real). Hace consultable el bucle de planificación ex-post.
- **`Severity`** — `blocking` marca issues de clase fiabilidad que deben aterrizar antes de un cutover a producción. Canonicaliza la convención en prosa `PROD-BLOCKER` que emergió en el campo `Notes` del adopter de referencia (Señal 3). Ortogonal al bucket: una entrada `charter-triggered` puede ser `blocking`.
- **`Labels`** — tags libres para agrupar entradas en Charters / mini-charters / chores planeados durante el triage. Consultable vía `straymark followups list --label <tag>`.
- **Vocabulario de `Destination`** — formaliza dónde aterriza el trabajo cuando se dispara: `chore`, `mini-charter`, `charter-replanning` (la entrada reconfigura un Charter ya planeado en vez de agregarle una tarea), `operations`, un id de Charter específico, o un id de TDE. Los valores free-form siguen siendo aceptados (parsing tolerante).
- **`Premise` / `Verified-at`** *(AIDEC-2026-07-18-001)* — la suposición que sostiene una entrada, y la fecha en que se re-chequeó por última vez contra el código. Una entrada es una *hipótesis fechada* (ver "Estatus epistémico"); estos campos le dan a la disciplina de re-verificar-en-ejecución un blanco concreto y un sello de auditoría. Ambos opcionales; `Verified-at` ausente = nunca re-chequeada desde la captura.

### Vocabulario de status

- `open` — pendiente, sin acción aún.
- `in-progress` — un Charter ha sido declarado o está en ejecución para atender esta entrada.
- `suspected-closed` *(nuevo en v1)* — auto-extraído por `drift --apply` desde un AILOG cuyo texto carga un marcador de cierre explícito (`closed in-Charter`, `fixed in batch N`, un hash de commit, o un modismo born-resolved como `updated atomically in this PR` — ver "Modismos canónicos de marcador de cierre" abajo). El operador confirma (→ `closed`) o reabre (→ `open`) en el siguiente triage. Ver "Detección de drift" abajo.
- `closed` — entrada resuelta (Charter mergeado, tarea operativa hecha, tiempo transcurrido y revisado).
- `superseded` — atendida por otro trabajo que no referenció esta entrada directamente.
- `promoted` — la entrada fue elevada a un documento TDE porque cumple los criterios de deuda transversal (ver "Promoción a TDE" abajo). El campo `Promoted to:` carga el id del TDE.

Las entradas closed, superseded y promoted permanecen en el archivo (historia auditable). Los operadores pueden moverlas a una sección `## Bucket: closed` al final para limpieza visual, pero nunca se eliminan.

---

## Promoción a TDE

Algunas entradas FU no son solo tareas diferidas — describen **deuda técnica transversal** que merece su propio documento de gobernanza (TDE). Los criterios para promoción reflejan la desambiguación TDE-vs-`R<N>` en `AGENT-RULES.md §3`:

- La entrada es *herencia de un Charter previo* (ya vivió ≥1 cierre de Charter sin remediación).
- La entrada *aplica a múltiples módulos o múltiples Charters* — el registry central la ha fragmentado en bullets que comparten una causa raíz.
- La entrada *requiere un Charter dedicado fuera del envelope de scope actual* para remediarse.
- La entrada *requiere priorización o asignación humana* que la revisión periódica del operador no puede decidir desde el bullet solo (matriz impact × effort, ownership).

Cuando cualquiera de estos se cumple, promueve la entrada FU a un documento TDE bajo `.straymark/06-evolution/technical-debt/`:

```bash
straymark followups promote FU-NNN
```

El comando automatiza el flujo de tres pasos que era manual en v0:

1. Crea el documento TDE (la misma maquinaria que `straymark new --type tde`), pre-llenando `impact`, `effort`, `type`, y el contexto del body desde la entrada FU.
2. Agrega `promoted_from_followup: FU-NNN` al frontmatter del TDE para trazabilidad.
3. En la entrada FU, establece `Status: promoted`, `Destination: TDE-YYYY-MM-DD-NNN`, y `Promoted to: TDE-YYYY-MM-DD-NNN`; recalcula los contadores del frontmatter.

La entrada FU **no se elimina** tras la promoción — su presencia en el registry es el rastro auditable que muestra de dónde vino el TDE.

### Dos formas de promoción — promoción-de-existente vs retroactiva-en-la-creación

El flujo anterior cubre el **caso estándar**: una entrada FU `open` ya existe en el registry y se eleva a un TDE durante revisión periódica. Existe un segundo caso igualmente válido que emergió empíricamente del retrospectivo de Sentinel CHARTER-13:

- **Promoción de entrada existente** — un FU fue registrado (típicamente vía `drift --apply`) como `open` semanas o Charters atrás, vivió ≥1 cierre de Charter sin resolución, y cumple los cuatro criterios anteriores. Flujo estándar.
- **Promoción retroactiva en la creación** — la deuda se reconoce como TDE-worthy *durante* un retrospectivo (ceremonia de cierre de Charter, ciclo de auditoría, redacción de RFC) y nunca existió como FU `open`. Se crea primero el TDE; se agrega una entrada FU al registry *con `Status: promoted`* desde el nacimiento, proporcionando el rastro auditable desde el TDE hacia el contexto originador (un `R<N>` en un AILOG, un finding del calibrador, una clasificación diferida).

Ambas formas producen el mismo estado final en el registry: una entrada con `Status: promoted` y un puntero `Promoted to: TDE-YYYY-MM-DD-NNN`. La diferencia es si la entrada pre-existía como `open` o nació `promoted`. La detección de drift las trata idénticamente, y las analíticas que cuentan `total_promoted` obtienen el mismo número en ambos casos.

Ante la duda, prefiere crear la entrada FU — aunque sea retroactivamente — porque cross-referencia el TDE de vuelta al AILOG / número-R / contexto fuente que disparó el reconocimiento. Un TDE con `promoted_from_followup: FU-NNN` apuntando a una entrada que existe en el backlog es más navegable que uno apuntando a un FU ficticio.

### Cuándo promover

- **Revisión periódica** — cuando el operador hace el pase manual de reclasificación, promueve cualquier entrada que haya vivido ≥2 cierres de Charter sin resolución y cumpla los criterios anteriores.
- **Cierre de Charter** — al revisar entradas que el Charter recién cerrado resolvió, si encuentras entradas que *no* fueron resueltas y cumplen los criterios anteriores, promuévelas en vez de dejarlas como `open`.
- **Pre-declaración de Charter** — si estás a punto de declarar un Charter y notas que el registry contiene entradas que este Charter *parcialmente* atendería, la porción no atendida puede pertenecer como TDE en vez de como otro FU diferido.

---

## Detección de drift — nativa desde cli-3.19.0

La detección de drift mantiene el registry sincronizado con nuevos AILOGs. Desde cli-3.19.0 es un **comando nativo del CLI** — sin script externo:

```bash
straymark followups drift              # escanea AILOGs modificados en git diff origin/main..HEAD (fallback HEAD~1..HEAD) UNIDO con el working tree (git status --porcelain); sale con 1 si hay drift
straymark followups drift --apply      # mismo escaneo + extrae nuevas entradas al registry
straymark followups drift --scan-all   # barrido completo periódico sobre cada AILOG
```

Desde cli-3.21.0 el escaneo por defecto une el rango git commiteado con el working tree (`git status --porcelain`), así un AILOG sin commitear/untracked es visible para el flujo pre-commit documentado — ya no necesitas `--scan-all` para ver un AILOG recién escrito antes de commitearlo (issue #229).

### Qué hace `--apply`

1. Extrae cada bullet `§Follow-ups` y riesgo `R<N> (new, not in Charter)` **cuyo hash de contenido aún no está en el registry**, agregándolos bajo `## Bucket: ready` con ids `FU-NNN` auto-numerados y un `Source-hash` almacenado. El operador reclasifica bucket/trigger/destination en el siguiente triage. (Los AILOGs ya extraídos se re-escanean y se deduplican por follow-up — ver "Dedup por hash de contenido por follow-up" abajo.)
2. **Refinamiento anti-ruido** *(v1 — resuelve issue #214 Señal 1)*: los bullets cuyo texto del AILOG carga un marcador de cierre explícito (`closed in-Charter`, `fixed in batch N`, una referencia de hash de commit) se extraen con `Status: suspected-closed` en vez de `open`, en lugar de contaminar el bucket `ready` como ruido TBD. A lo largo de ambas ocurrencias documentadas en el adopter de referencia, 20–75% de las entradas auto-agregadas por batch ya estaban resueltas in-Charter — este refinamiento elimina el único costo recurrente del workflow v0.
3. Agrega el id del AILOG a `fully_extracted_ailogs`.
4. **Recalcula todos los contadores `total_*`** desde los estados reales de las entradas (Señal 2).
5. Si el registry es `schema_version: v0`, lo actualiza a `v1` in situ — de forma no destructiva e idempotente (todos los campos v1 son opcionales; no se reescribe nada excepto el marcador de versión y los contadores).

Desde cli-3.20.0, `--apply` recalcula los contadores **incluso cuando no hay nada que extraer** — así un `drift --apply` pre-commit también reconcilia contadores que una sesión de triage manual dejó obsoletos (feedback del primer adopter externo, issue #222 Finding 1).

### Modismos canónicos de marcador de cierre

El refinamiento anti-ruido reconoce un vocabulario fijo, sin distinguir mayúsculas. Los autores de AILOGs deben converger en estas fórmulas al escribir, para que las entradas born-resolved aterricen como `suspected-closed` en vez de ruido TBD:

| Familia de modismo | Ejemplos |
|---|---|
| Cierre in-Charter | `closed in-Charter`, `closed in Charter`, `resolved in-Charter`, `resolved in Charter` |
| Fix por batch | `fixed in batch 3` (requiere el número) |
| Referencia a commit | un hash de commit entre backticks: `` `ab12cd34ef` `` (7–40 chars hex, al menos un dígito) |
| Born-resolved *(cli-3.20.0+, #222 Finding 2)* | un verbo de cierre — `updated` / `corrected` / `remediated` / `resolved` / `fixed` / `closed` — seguido de `in this PR` o `in this commit`, p.ej. `Charter row updated atomically in this PR` |

Las fórmulas fuera de este vocabulario (p.ej. `done earlier`, `no longer relevant`) se extraen como `open`; el operador las cambia en el triage. Cuando un nuevo modismo de cierre se repita en tus AILOGs, proponlo upstream en vez de editar extracciones a mano.

### Dedup por hash de contenido por follow-up

Desde cli-3.21.0, drift deduplica **por follow-up mediante un hash de contenido estable** (`fu_content_hash` del id del AILOG fuente + sección de origen + descripción), almacenado como el `Source-hash` de cada entrada. Los AILOGs ya extraídos se re-escanean y cada follow-up se deduplica contra el registry — así un follow-up **agregado a un AILOG ya extraído** (el caso del Charter multi-batch, donde el `§Follow-ups` de un AILOG crece a lo largo de batches) se detecta en vez de perderse en silencio (issue #231).

La objeción original al matching per-bullet eran los falsos positivos por paráfrasis: las entradas curadas del registry reescriben el bullet del AILOG, así que recalcular un hash desde el texto *del registry* re-marcaría contenido ya extraído. El `Source-hash` almacenado resuelve esto — se captura en el momento de extracción desde el texto original del AILOG y nunca se recalcula desde el heading (luego parafraseado) del registry. La propiedad de cero falsos positivos se preserva para toda entrada que lleve hash.

Las entradas legacy creadas antes de cli-3.21.0 no tienen `Source-hash`; para ellas drift recae en recalcular el hash desde `Origin` + `description` — best-effort, y el único vector residual de vulnerabilidad por paráfrasis, decreciente a medida que las entradas legacy se cierran. `fully_extracted_ailogs` se conserva (registra qué AILOGs han sido escaneados y lo muestra `followups status`) pero **ya no es el gate de skip** — la dedup es por hash de contenido, no por id de AILOG completo.

### Script bash legacy (deprecado)

La implementación de referencia v0 (`scripts/check-followups-drift.sh`, ~296 líneas de bash POSIX en el repo del adopter Sentinel) está **deprecada a partir de cli-3.19.0**. Sigue funcionando para registries v0 pero ya no se mantiene y carece del refinamiento anti-ruido y del recálculo de contadores. Ruta de migración: borra el script, ejecuta `straymark followups drift --scan-all --apply` una vez (esto también actualiza el registry a v1), y actualiza cualquier pre-commit hook para que llame al CLI en su lugar.

**Corre ese primer barrido post-migración con `--scan-all` aunque el script reportara "in sync"**: el extractor bash era sensible al formato (exigía un heading `## Risk` y la forma exacta de bullet `- **R<N> (new`) y producía **falsos negativos silenciosos** ante variantes de formato — los AILOGs que escribían riesgos como párrafos planos nunca registraban como portadores de follow-ups. En la migración del adopter de referencia ([issue #225](https://github.com/StrangeDaysTech/straymark/issues/225)), el parser nativo leniente capturó **8 AILOGs / 29 entradas** que el script había reportado como "in sync" el día anterior. Los falsos negativos silenciosos en la detección de drift son exactamente el modo de falla que la herramienta existe para prevenir — por eso el script está deprecado en vez de mantenido.

---

## Superficie del CLI

```bash
straymark followups list                  # enumera entradas: id FU, status, severity, bucket, destination
straymark followups list --bucket ready --status open --severity blocking --label <tag>
straymark followups status                # pulso del registry: contadores (recalculados al vuelo), desglose por bucket/severity
straymark followups status FU-NNN         # vista de detalle de una entrada
straymark followups drift [--apply|--scan-all]   # detección de drift (ver arriba)
straymark followups recount               # recalcula los contadores CLI-owned tras una sesión de triage manual (cli-3.20.0+)
straymark followups promote FU-NNN [--premise-verified]   # automatiza la promoción FU → TDE; superficie el re-chequeo de la premisa, sella Verified-at con el flag (cli-3.37.0+)
straymark followups verify FU-NNN [--premise "..."] [--verified] [--at FECHA]   # re-verifica una hipótesis fechada en la ejecución: superficie/registra la premisa, sella Verified-at (cli-3.37.0+)
straymark followups note FU-NNN "<texto>" [--source CHARTER-NN|AILOG-…]   # anexa una anotación fechada a Notes (cli-3.39.0+)
straymark followups set-status FU-NNN <status>   # cambia el status Y recomputa los contadores en un solo paso (cli-3.39.0+)
straymark followups new --title "..." --origin "CHARTER-NN §Scope" [--bucket …] [--cost …] [--trigger …] [--premise …]   # crea una entrada declarada ex-ante (cli-3.39.0+)
```

`verify` y `promote --premise-verified` son las afordancias en tiempo de ejecución de la disciplina de "Estatus epistémico": ponen la premisa frente al operador en el momento del gasto y registran que el re-chequeo ocurrió. El juicio humano queda fuera del CLI — superficie y sella, nunca decide la verdad.

### Mutar y crear entradas (cli-3.39.0+)

`note`, `set-status` y `new` reemplazan los últimos flujos que exigían editar a mano este archivo parseado por el CLI. Ahí fallaban dos cosas: una edición a mano puede malformar una entrada y romper `list`/`status`/`drift`, y el cambio de status era un **dos-pasos** — editar el bullet y luego acordarse de `recount` — que se desincroniza en cuanto olvidas la segunda mitad, dejando a los contadores mintiendo en silencio sobre el backlog.

- **`note`** anexa a `Notes` (campo de una sola línea, así que las anotaciones se componen en vez de apilarse), con la fecha sellada y, con `--source`, con el Charter o AILOG que la motivó. Sirve para el caso común: registrar una mitigación *parcial* sin cambiar el status.
- **`set-status`** escribe el status y recomputa los contadores en el mismo paso. Rechaza un status fuera del vocabulario — el parser es indulgente, así que un typo no fallaría: sacaría la entrada de todos los contadores en silencio — y redirige `promoted` a `followups promote`, que además escribe el TDE que le da a ese status algo a lo que apuntar.
- **`new`** crea una entrada cuyo origen es una **declaración de Charter** (`Origin-class: ex-ante-planning`). Las dos rutas anteriores asumen origen ex-post: `drift --apply` extrae de AILOGs, y un diferimiento decidido en tiempo de declaración precede a cualquier AILOG por diseño. `new` asigna el id de forma atómica y lo imprime, así que el cuerpo del Charter cita una entrada que **existe** en vez de una reserva conjeturada que el siguiente `drift --apply` podría entregarle a otra entrada. Una entrada ex-ante no lleva **`Source-hash`**: no hay AILOG que hashear, e inventar uno haría que un `drift --apply` posterior creyera haber extraído algo que nunca vio.

Los tres se niegan a escribir si el registro tiene avisos de parseo: una edición quirúrgica contra una estructura mal leída puede corromper entradas vecinas. `recount` sigue siendo la vía de escape para una sesión de triage manual masivo — y el chequeo idempotente de que estos verbos hicieron bien la aritmética.

El registry también aparece como un grupo sintético **Follow-ups** en la TUI de `straymark explore` (sub-nodos por bucket) y como un bloque de conteos en `straymark status`.

---

## Integración con agentes

Desde fw-4.21.0 las directivas de agente **se shippean con el framework** en [`AGENT-RULES.md §13`](AGENT-RULES.md) — los adopters ya no copian un bloque a su propio `CLAUDE.md` / `AGENT.md`. En resumen:

- **Session start**: echa un vistazo a `.straymark/follow-ups-backlog.md` (o ejecuta `straymark followups status`) para saber qué está pendiente en el proyecto.
- **Pre-commit**: ¿creaste o modificaste algún AILOG con entradas `## Follow-ups` o `R<N> (new, not in Charter)`? → ejecuta `straymark followups drift --apply` en el mismo commit.
- **Post-Charter close**: revisa las entradas que el Charter resolvió; márcalas `closed` (con el id del Charter de cierre en `Notes`) o `superseded`; confirma o reabre cualquier entrada `suspected-closed`; luego corre `straymark followups recount` para que los contadores CLI-owned viajen en el mismo commit que el triage; promueve las entradas no resueltas que cumplen los criterios de TDE vía `straymark followups promote`.

Esto hace al agente el mantenedor primario del registry, al CLI la capa de verificación, y al operador el revisor periódico (re-bucketing, confirmar suspected-closed, podar superseded, promover a TDE cuando los criterios aplican).

---

## Walkthrough de adopción

Para un adopter empezando desde cero:

1. Copia `.straymark/templates/follow-ups-backlog.md` a `.straymark/follow-ups-backlog.md` (lista `fully_extracted_ailogs:` vacía, cinco headers `## Bucket:`).
2. Ejecuta `straymark followups drift --scan-all --apply` para sembrar el registry desde AILOGs existentes.
3. Reclasifica manualmente las entradas auto-generadas en `## Bucket: ready` a los buckets correctos; llena `Origin-class`/`Severity`/`Labels` donde agreguen señal. Esto es triage one-time, típicamente 30-60 min para un backlog de ~50 entradas.
4. Listo — las directivas de agente en `AGENT-RULES.md §13` ya están activas; no se necesitan ediciones a `CLAUDE.md`.

Para un adopter migrando desde la convención v0: ejecuta `straymark followups drift --apply` una vez (auto-actualiza el registry a v1), borra el script bash local, y actualiza cualquier pre-commit hook para que llame al CLI.

---

## Implementación de referencia

`StrangeDaysTech/sentinel` — el adopter originador:

- Patrón v0: CHARTER-12, mergeado 2026-05-06 ([sentinel#53](https://github.com/StrangeDaysTech/sentinel/pull/53), [sentinel#54](https://github.com/StrangeDaysTech/sentinel/pull/54)). 47 entradas sembradas desde la retrospectiva CHARTER-08 → CHARTER-11.
- Validación de escala: triage post-Etapa-3 en **N=91 FUs / 76 AILOGs extraídos / 65 open** ([issue #214](https://github.com/StrangeDaysTech/straymark/issues/214)) — el input empírico que impulsó el schema v1 y el CLI nativo (ADR-2026-06-03-001).

---

## Preguntas abiertas

Resueltas en v1:

- ~~**Validación de schema.**~~ → `.straymark/schemas/follow-ups-backlog.schema.v1.json` (frontmatter), validación de la forma de entrada en el parser del CLI.
- ~~**Cristalización como CLI `straymark followups`.**~~ → `list / status / drift / promote` nativos desde cli-3.19.0.
- ~~**Heurística de clasificación de bucket** (parcialmente).~~ → `suspected-closed` elimina la clase de ruido dominante; la sugerencia completa de bucket (usando `tags` del AILOG / `effort_estimate` del Charter) sigue abierta.

Aún abiertas para revisiones futuras:

- **Integración con el ciclo de auditoría.** Cuando `straymark charter audit --merge-reports` produce findings de deuda real que no son remediados atómicamente pre-cierre, esos findings viven solo en `.straymark/audits/<id>/review.md`. No fluyen automáticamente al registry central. Aflorarlos automáticamente cerraría un gap conocido.
- **Semántica `closed` vs `superseded`.** Hoy la diferencia es si el trabajo de resolución referenció explícitamente la entrada. Una convención más estricta puede emerger.
- **Integración suave con `charter close`** (issue #135 Tier 3): auto-invocar `followups drift --apply` tras un cierre de Charter, con un prompt interactivo de promoción. Condicionada a una señal de fricción de un segundo adopter.
- **Estabilización dura del schema (v1.0).** Condicionada a la validación por un segundo adopter en otro dominio, según el principio de diseño #12.

---

## Créditos

Contribuido vía [issue #111](https://github.com/StrangeDaysTech/straymark/issues/111) por el adopter Sentinel; madurado a primera clase vía [issue #214](https://github.com/StrangeDaysTech/straymark/issues/214) y ADR-2026-06-03-001. Fundamento empírico: cadena CHARTER-08 → CHARTER-11 y el triage post-Etapa-3 en N=91 en `StrangeDaysTech/sentinel`. Autor: José Villaseñor Montfort.

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7 / Opus 4.8); toda la responsabilidad del contenido recae en el autor humano.*

---

## Relacionado

- [EMERGENT-OBSERVATION-DESIGN.md](EMERGENT-OBSERVATION-DESIGN.md) — el meta-patrón que esta convención de detección de drift instancia en la superficie per-AILOG ↔ registry.
- [CHARTER-CHAIN-EVOLUTION.md](CHARTER-CHAIN-EVOLUTION.md) — patrón hermano que opera a nivel de cadena (Pattern 1) y a nivel de ciclo (Pattern 2).
- [AGENT-RULES.md §3](AGENT-RULES.md) — criterios de escalación TDE-vs-`R<N>` que pueden promover follow-ups a entradas de deuda dedicadas; §13 — las directivas de agente shippeadas para el mantenimiento del registry.
- `STRAYMARK.md §16` — el resumen a nivel onboarding del registry como artefacto de primera clase.

---

*StrayMark fw-4.40.0 | [Strange Days Tech](https://strangedays.tech)*
