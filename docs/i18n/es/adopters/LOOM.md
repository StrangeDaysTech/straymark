# StrayMark — Loom y el Mapa de Arquitectura (Experimental)

**Mira tu proyecto: la red de documentos y el mapa "estás aquí" de dónde está sucediendo el trabajo.**

> ⚠️ **EXPERIMENTAL — Loom v0 (N=1).** Loom es un experimento opt-in e inestable. Su API, superficie de CLI, formato del modelo en disco y su propia existencia pueden cambiar o eliminarse sin un ciclo de deprecación hasta que se gradúe. **No** forma parte del contrato soportado del Framework ni del CLI — no construyas automatización contra él todavía. El modelo `architecture` y `status --where` se envían en el CLI pero cargan la misma advertencia experimental.

---

## Qué es Loom

Loom es el tercer componente de StrayMark (junto al Framework y al CLI): un **dashboard de desarrollo** loopback-only y de solo lectura que hace legible un proyecto StrayMark al ojo humano. Tiene dos superficies sobre el mismo proyecto:

1. **Knowledge Graph** — la red de tus *documentos* StrayMark, renderizada como un grafo dirigido por fuerzas en vivo a partir de los enlaces de su frontmatter (`related`, `originating_ailogs`, …). Selecciona un nodo y todo su hilo de relaciones se ilumina. Esta vista es autoexplicativa y no necesita configuración.
2. **Mapa de Arquitectura** — el mapa de implementación del *sistema*: componentes y capas (tu arquitectura real), con un **overlay de estado "estás aquí"** en vivo computado a partir del estado de gobernanza (Charters activos, drift, trabajo cerrado, deuda técnica abierta). Responde visualmente al *"¿dónde estamos?"* diario. **Esta vista está dirigida por un modelo que tú redactas + refinas** — el resto de esta guía trata sobre eso.

Loom nunca escribe en tu proyecto — solo *visualiza* lo que el CLI computa. Los comandos del CLI (`validate`, `audit`, `charter drift`) siguen siendo la fuente de verdad y el gate.

---

## El modelo mental: un modelo, muchas vistas (BIM)

El Mapa de Arquitectura toma prestada la idea de **BIM "un modelo, muchas vistas"**. Hay **un** modelo, expresado como dos archivos enlazados bajo `.straymark/architecture/`:

| Archivo | Qué contiene | Quién lo posee |
|---|---|---|
| `model.yml` | La **semántica** — componentes → globs de archivos, capas, enlaces | Tú (sembrado por el CLI) |
| `plan.drawio` | El **layout** — el diagrama de cajas-y-flechas (DrawIO/mxGraph) | Tú (sembrado por el CLI) |

A partir de ese único modelo, StrayMark renderiza **tres proyecciones**, todas desde el *mismo* cómputo de estado para que no puedan contradecirse:

- **Textual** — `straymark status --where` (el "estás aquí" de la terminal).
- **Plan 2D** — la pestaña Architecture de Loom (tu `plan.drawio`, superpuesto con estado en vivo).
- **Axonométrico 3D** — el toggle `2D | 3D` de Loom (una vista explotada al estilo BIM).

El **overlay de estado se computa en vivo, nunca se mantiene a mano.** Tú redactas la *estructura* (qué archivos pertenecen a qué componente, cómo se relacionan los componentes); StrayMark lo colorea (`active` / `in-progress` / `implemented` / `has-debt` / `uncharted`) a partir de señales de gobernanza cada vez que lo miras.

---

## El flujo de trabajo recomendado

```
generate  →  refine (humano o IA)  →  validate  →  sync (a medida que crece el código)  →  loom serve
   draft        el modelo real          gate CI         append-only                          visualizar
```

### 1. Generate — una semilla de primer borrador

```bash
straymark architecture generate
```

Mina la estructura de tu codebase (un componente por directorio de fuente) y la enriquece a partir de tus ADRs (los diagramas C4 + las tablas "Affected Components" mejoran las etiquetas y añaden enlaces). Escribe `model.yml` + `plan.drawio` con cada componente en una capa placeholder `unassigned` y las etapas 00–09 de `.straymark` como capas placeholder.

La semilla es **consciente del lenguaje y la estructura** (desciende a través de `internal/`, `src/`, `pkg/`, … y omite el scaffolding de Maven/Gradle para que un módulo Java no sea una sola caja `main`). Para stacks no predeterminados puedes extenderla con una sección `architecture:` en `.straymark/config.yml` — ver [CLI-REFERENCE](./CLI-REFERENCE.md#straymark-architecture-generatesyncvalidate-cli-3250-experimental).

**La semilla es un borrador, no la respuesta.** Captura la forma; tú la haces significativa en el siguiente paso.

### 2. Refine — la fase que importa (humano *o* IA)

Aquí es donde un mapa generado se vuelve uno real. `model.yml` es YAML plano y `plan.drawio` es un archivo DrawIO estándar, así que el refinamiento puede hacerlo **una persona, un agente IA, o ambos**:

- **Renombra las capas** de las etapas placeholder a tu arquitectura real (p. ej. `entrypoints`, `domain`, `persistence`, `web`).
- **Reasigna componentes** fuera de `unassigned` hacia esas capas.
- **Corrige las etiquetas** a nombres humanos (`internal-modules-commshub` → "CommsHub").
- **Ajusta los globs** para que cada componente posea exactamente sus archivos.
- **Añade `links`** entre componentes para capturar dependencias reales.
- **Distribuye el diagrama** en DrawIO (abre `plan.drawio`) para que el plan 2D se lea bien.

> **Refinamiento asistido por IA.** Como el modelo es solo YAML + XML de DrawIO, un agente IA de codificación puede hacerlo directamente: apúntalo a `model.yml` y a tu codebase y pídele que asigne capas, corrija etiquetas y añada enlaces de dependencia. Opera el mismo CLI (`generate` / `validate` / `sync`) y edita los mismos archivos que tú. El refinamiento es un ciclo normal de revisar-y-editar — conserva lo que está bien, corrige lo que está mal.

### 3. Validate — atrapa el drift modelo↔plan

```bash
straymark architecture validate            # texto; sale 1 ante cualquier señal (gateable en CI)
straymark architecture validate --output json
```

Reporta señales de integridad: **undrawn** (un componente sin celda en el diagrama), **unmodeled** (una celda del diagrama ausente del modelo), **empty** (globs que no coinciden con ningún archivo). Los errores ahora son legibles (nombran la causa exacta — p. ej. un id de componente que colisiona con un id de capa), así que un modelo malformado te dice qué corregir.

### 4. Sync — mantente al día a medida que crece el código (append-only)

```bash
straymark architecture sync            # dry-run: muestra lo nuevo
straymark architecture sync --apply    # añade nuevos dirs / componentes de ADR
```

Detecta nuevos directorios de fuente y componentes de ADR que aún no están en el modelo y los **añade** — **nunca** pisa tus ediciones ni tu geometría de DrawIO. Ejecútalo cuando añadas un módulo; refina las nuevas entradas de la misma manera.

### 5. Serve — míralo en vivo

```bash
straymark loom serve            # descarga el binario loom-* en el primer uso, abre el navegador
```

Abre ambas vistas en `localhost`. La pestaña Architecture muestra tu plan refinado con el overlay en vivo; el toggle `2D | 3D` da la vista axonométrica. Las ediciones en `.straymark/` (o en el modelo) actualizan el navegador abierto en menos de un segundo. También hay una respuesta solo-terminal que no necesita servidor:

```bash
straymark status --where        # el "estás aquí" textual
```

---

## Limitaciones honestas (es una semilla, no salida autoritativa)

- La **semilla generada es un punto de partida.** No inferirá tus capas o fronteras de componentes intencionadas — esa es la fase de refinamiento, por diseño.
- **La cobertura depende de tu stack.** El escaneo reconoce ~25 lenguajes por defecto y maneja layouts comunes (Go `internal/`, JS `src/`, Maven/Gradle multi-módulo). Un lenguaje no predeterminado o un layout inusual necesita una entrada de config `architecture:` y más refinamiento a mano — pero la semilla nunca *engaña*; solo da menos ventaja inicial.
- El **overlay `has-debt` es tan preciso como tus registros de deuda.** Por defecto, la deuda se atribuye a los componentes que los AILOGs referenciados de un TDE abierto realmente modificaron; las listas `files_modified` gruesas la esparcen más ampliamente que la deuda misma. Para acotarla con exactitud, declara un campo **`affects`** en el TDE (globs de archivos, p. ej. `affects: [internal/modules/audittrail/**]`) — cuando está presente, atribuye la deuda solo a esos paths, ignorando el footprint del AILOG.
- **Loom es N=1.** Ha sido validado por dogfooding en este proyecto y en la referencia Sentinel. Espera asperezas; repórtalas.

---

## Ver también

- [CLI-REFERENCE](./CLI-REFERENCE.md) — `straymark architecture <generate|sync|validate>`, `status --where`, y los flags de `loom serve` + la sección de config `architecture:`.
- [WORKFLOWS](./WORKFLOWS.md) — cómo encaja el mapa de arquitectura en la cadencia diaria de StrayMark.
- [ADOPTION-GUIDE](./ADOPTION-GUIDE.md) — meter StrayMark en un proyecto en primer lugar.
