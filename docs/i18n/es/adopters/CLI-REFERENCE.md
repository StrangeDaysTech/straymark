# StrayMark - Referencia CLI

**Referencia completa de la herramienta de línea de comandos `straymark`.**


---

## Tabla de Contenidos

1. [Instalación](#instalación)
2. [Versionado](#versionado)
3. [Comandos](#comandos) — init, update, remove, status, repair, validate, new, charter, followups, compliance, metrics, analyze, audit, explore, about
4. [Variables de Entorno](#variables-de-entorno)
5. [Códigos de Salida](#códigos-de-salida)

---

## Instalación

Instala el CLI de StrayMark usando uno de los métodos a continuación. Para instrucciones completas de configuración, consulta el [README](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/es/README.md#inicio-rápido).

**Instalación rápida (binario precompilado):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
```

**Desde el código fuente:**

```bash
cargo install straymark-cli
```

---

## Versionado

StrayMark usa **tags de versión independientes** para cada componente:

| Componente | Prefijo de tag | Ejemplo | Qué incluye |
|------------|---------------|---------|-------------|
| Framework | `fw-` | `fw-4.43.0` | Plantillas (12 tipos), docs de gobernanza, directivas |
| CLI | `cli-` | `cli-3.45.0` | El binario `straymark` |
| Loom (EXPERIMENTAL) | `loom-` | `loom-0.4.2` | El servidor de visualización `straymark-loom`, descargado bajo demanda por `straymark loom serve` |

Framework y CLI se publican de forma independiente. Una actualización del framework no requiere actualización del CLI, y viceversa.

**Verificar versiones instaladas:**

```bash
straymark about    # Muestra versión CLI + versión framework (si está instalado)
straymark status   # Muestra estado completo de la instalación incluyendo versiones
```

---

## Comandos

### `straymark init [path] [--hooks]`

Inicializa StrayMark en un directorio de proyecto.

**Argumentos y flags:**

| Argumento/Flag | Por defecto | Descripción |
|----------------|-------------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto destino |
| `--hooks` *(cli-3.7.0+)* | off | Tras `init`, instala el hook pre-PR del framework (`.straymark/hooks/pre-pr.sh`) como `.git/hooks/pre-push`. Ejecuta `straymark charter drift` automáticamente antes de cada push. Opt-in por principio #6 (disciplina cognitiva > productividad cruda). Se rehúsa a sobreescribir un `pre-push` ya existente; se omite silenciosamente si no es un repositorio git. |

**Qué hace:**

1. Descarga el último release del framework (`fw-*`) desde GitHub
2. Crea la estructura de directorios `.straymark/`
3. Crea `STRAYMARK.md` con las reglas de gobernanza
4. Configura archivos de directivas de agentes IA (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `QWEN.md`, `.cursorrules`, etc.)
5. Copia workflows de CI/CD
6. *(`--hooks`)* instala el hook pre-PR

**Ejemplo:**

```bash
$ straymark init .
✔ Downloaded StrayMark fw-4.15.0
✔ Created .straymark/ directory structure
✔ Created STRAYMARK.md
✔ Configured AI agent directives
StrayMark initialized successfully!
Next: git add .straymark/ STRAYMARK.md && git commit -m "chore: adopt StrayMark"
```

---

### `straymark update`

Actualiza **ambos** framework y CLI a sus últimas versiones. Equivale a ejecutar `update-framework` seguido de `update-cli`.

Si `.straymark/` no existe en el directorio actual, la actualización del framework se omite con una advertencia.

**Ejemplo:**

```bash
$ straymark update
Updating framework...
✔ Framework updated to fw-4.15.0
Updating CLI...
✔ CLI updated to cli-3.5.2
```

---

### `straymark update-framework`

Actualiza solo los archivos del framework. Busca el último release `fw-*` en GitHub.

**Manejo de conflictos:** Si has modificado archivos del framework (ej. docs de gobernanza o plantillas), la actualización preserva tus cambios y reporta conflictos para resolución manual.

**Ejemplo:**

```bash
$ straymark update-framework
✔ Framework updated to fw-4.15.0
```

---

### `straymark update-cli`

Auto-actualiza el binario `straymark`. Detecta automáticamente el método de instalación y usa el mecanismo de actualización apropiado:

- **Binario precompilado** (instalado via `install.sh` / `install.ps1`): Descarga el último binario de GitHub Releases
- **Cargo** (instalado via `cargo install`): Ejecuta `cargo install --force straymark-cli`

Usa `--method` para forzar el método: `--method=github` o `--method=cargo`.

**Ejemplo:**

```bash
$ straymark update-cli
✔ CLI updated to cli-3.5.2

$ straymark update-cli --method=cargo
Compiling from source, this may take a few minutes...
✔ CLI updated to cli-3.5.2
```

---

### `straymark remove [--full]`

Elimina StrayMark del proyecto actual.

**Flags:**

| Flag | Descripción |
|------|-------------|
| `--full` | Elimina todo, incluyendo documentos creados por el usuario en `.straymark/`. Pide confirmación. |

**Comportamiento por defecto** (sin `--full`): elimina la estructura del framework pero preserva los documentos que creaste dentro de `.straymark/`.

**Ejemplo:**

```bash
$ straymark remove
✔ StrayMark framework removed. User documents preserved in .straymark/.

$ straymark remove --full
⚠ This will delete all StrayMark files including your documents.
Continue? [y/N]: y
✔ StrayMark completely removed.
```

---

### `straymark status [path]`

Muestra el estado de la instalación y estadísticas de documentación.

**Argumentos:**

| Argumento | Por defecto | Descripción |
|-----------|-------------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto destino |

**La salida incluye:**

- Ruta del proyecto
- Versión del framework
- Versión del CLI
- Idioma configurado
- Integridad de la estructura de directorios
- Estadísticas de documentos (conteo por tipo)

**Ejemplo:**

```bash
$ straymark status
StrayMark Status
───────────────
Path:              /home/user/my-project
Framework version: fw-4.15.0
CLI version:       cli-3.5.2
Language:          en
Structure:         ✔ Complete

Documents:
  AILOG:  12
  AIDEC:   4
  ADR:     7
  REQ:     3
  TES:     2
  TDE:     1
  INC:     0
  ETH:     1
  Total:  30
```

---

#### `straymark status --where [path] [--out <dir>]` *(cli-3.25.0+, EXPERIMENTAL)*

El acompañante textual del **"estás aquí"** de la vista del Plan de Arquitectura de Loom (la mitad en terminal del dashboard). Carga `architecture/model.yml` y proyecta el estado de cada componente a partir de tus señales de gobernanza en vivo, de modo que puedas responder *"¿dónde estamos?"* sin abrir un navegador.

> ⚠️ **EXPERIMENTAL (Loom v0).** Requiere un `architecture/model.yml` (ver `straymark architecture generate`). La forma de la salida puede cambiar sin ciclo de deprecación.

| Flag | Default | Descripción |
|---|---|---|
| `--where` | off | Cambia `status` a la vista de proyección de arquitectura |
| `--out <dir>` | `.straymark/architecture` | Directorio que contiene `model.yml` (apúntalo a otro sitio para proyectar un modelo mantenido fuera de `.straymark/`) |

Cada componente se etiqueta con uno o más estados, derivados puramente de la gobernanza (sin nuevo frontmatter):

- **`active`** — un Charter `in-progress` declara archivos que coinciden con el componente (← *estás aquí*);
- **`in-progress`** — dentro de un componente activo, archivos declarados ya tocados (declarados ∩ modificados-en-git);
- **`implemented`** — un Charter cerrado (y sus AILOGs) tocó el componente;
- **`has-debt`** — un `TDE` abierto se relaciona con el componente;
- **`uncharted`** — archivos en disco que ningún Charter o documento referencia.

La vista termina con un **resumen "Where are we"**: el Charter activo, el progreso declarados-vs-modificados, los AILOGs recientes y la deuda abierta. Los flags `active`/`implemented` se alinean con `straymark charter list --status in-progress`/`--status closed` + `charter drift` por construcción (una sola proyección compartida).

```
$ straymark status --where

  Where are we

  Tooling (Rust workspace)
    ▸ straymark-cli   [active] [in-progress]  ← you are here
    · straymark-core  [implemented]

  Visualization
    · Loom            [has-debt]

  Summary
    Active Charter: A1.4 — status --where
    Progress: 1/2 declared files touched (50%)
    Debt: 1 component with open debt, 0 uncharted
```

---

### `straymark repair [path]`

Repara una instalación de StrayMark rota restaurando directorios y archivos del framework faltantes.

**Argumentos:**

| Argumento | Default | Descripción |
|-----------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |

**Qué hace:**

1. Verifica directorios faltantes y los restaura con `.gitkeep`
2. Descarga el release del framework **una sola vez** si se necesitan archivos (templates, governance, config)
3. Re-inyecta directivas si falta `STRAYMARK.md`
4. Recalcula checksums después de la reparación
5. Nunca modifica ni elimina documentos generados por el usuario

**Ejemplo:**

```bash
$ straymark repair
Repairing StrayMark in /home/user/mi-proyecto
  → Found 1 issue(s) to repair
→ Restoring 1 missing directory...
✓ Restored .straymark/templates/
→ Downloading framework to restore missing files...
✓ Restored 16 file(s) from framework

✓ StrayMark repaired successfully!
```

---

### `straymark install-skills --agent <codex|qoder|qwen|claude|agy> [--path .] [--dry-run] [--symlink]` *(cli-3.16.0+)*

Instala skills de StrayMark en el directorio **a nivel de usuario** de skills del agente IA. `--agent codex` copia cada skill `straymark-*` desde `<path>/.codex/skills/` (materializado por `straymark init` o `straymark update`) hacia `$CODEX_HOME/skills/` (o `$HOME/.codex/skills/` si `CODEX_HOME` no está definida). `--agent qoder` hace lo mismo desde `<path>/.qoder/skills/` hacia `$QODER_CONFIG_DIR/skills/` (o `$HOME/.qoder/skills/` si `QODER_CONFIG_DIR` no está definida). `--agent qwen` *(cli-3.42.0+)* hace lo mismo desde `<path>/.qwen/skills/` hacia `$QWEN_HOME/skills/` (o `$HOME/.qwen/skills/` si `QWEN_HOME` no está definida) — el directorio que resuelve el propio `Storage.getGlobalQwenDir()` de Qwen Code.

**Solo Codex requiere este paso.** Qoder y Qwen Code también resuelven un directorio de skills a nivel de proyecto (`<proyecto>/.qoder/skills/`, `<proyecto>/.qwen/skills/`), así que para esos dos la instalación a nivel de usuario es una conveniencia: deja los skills de StrayMark disponibles en cualquier proyecto, no solo en los que tienen el framework instalado.

Para `--agent claude` y `--agent agy` el comando termina con un error explicativo: esos agentes leen los skills exclusivamente del árbol del proyecto (`.claude/skills/`, `.agent/skills/`), por lo que no admiten instalación a nivel de usuario.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `--agent` | requerido | Uno de `codex`, `qoder`, `qwen`, `claude`, `agy`. Solo `codex`, `qoder` y `qwen` ejecutan trabajo; los demás muestran guía y salen. |
| `--path` | `.` | Directorio del proyecto cuyo `.codex/skills/` (o `.qoder/skills/`, `.qwen/skills/`) es el origen. |
| `--dry-run` | off | Imprime qué se instalaría sin escribir nada. |
| `--symlink` | off | Enlaza simbólicamente cada skill en lugar de copiarlo (solo Unix; útil para devs del framework iterando sobre los skills). |

Re-ejecutar el comando reemplaza cualquier directorio `straymark-*` existente en el destino; skills que no comienzan con `straymark-` (por ejemplo el bundle `.system/` de Codex) no se tocan.

---

### `straymark validate [path] [--fix] [--staged] [--agent <codex>] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

Valida documentos StrayMark verificando cumplimiento y corrección.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--fix` | — | Corregir automáticamente problemas simples |
| `--staged` | — | Validar solo archivos staged en Git (ideal para hooks pre-commit) |
| `--agent` *(cli-3.16.0+)* | — | Cambia a modo agente y revisa una instalación a nivel de usuario de skills en lugar de documentos del proyecto. Uno de `codex`, `qoder`, `qwen` *(estos dos últimos desde cli-3.42.0)* — verifica `~/.codex/skills/straymark-*`, `~/.qoder/skills/straymark-*` o `~/.qwen/skills/straymark-*` para presencia, frontmatter YAML parseable y `name`/`description` requeridos. Para `codex` además señala claves Claude-only como `allowed-tools` (cuya presencia indica que alguien copió skills desde `.claude/` por error); Qoder y Qwen Code parsean el frontmatter completo de Claude, así que ahí `allowed-tools` es lo esperado. |
| `--include-charters` | — | Validar también los Charters en `.straymark/charters/` contra el JSON Schema y la integridad referencial (los IDs en `originating_ailogs` resuelven; el path en `originating_spec` existe). Incluye **`CHARTER-FILES-EXIST`** *(cli-3.17.0+)*: avisa cuando una fila de `## Archivos a modificar` nombra una ruta que no existe en disco y no está marcada como "Nuevo" — atrapa Charters redactados contra código asumido, no leído (hallazgo #210). Solo-aviso; distinto de `charter drift` (que compara lo declarado contra los archivos modificados en git). Opt-in, default `false` para no afectar a proyectos que no usan el patrón. Por ahora solo se honra fuera de `--staged`; la validación de Charters en modo staged llega en cli-3.10.0. |
| `--check-pending-reviews` *(cli-3.7.0+)* | off | Lista documentos con `review_required: true` y sin `review_outcome` cuya antigüedad supere `--max-pending-days`. **Solo warn** — nunca falla el exit code de validate; útil para dashboards de CI sobre el backlog de aprobaciones. |
| `--max-pending-days` *(cli-3.7.0+)* | `14` | Umbral en días para `--check-pending-reviews`. |

**Reglas de validación:**

- `NAMING-001`: Convención de nombres de archivo
- `META-001/002/003`: Campos obligatorios, id vs nombre de archivo, valores válidos
- `CROSS-001/002/003`: Riesgo alto requiere revisión, EU AI Act, tipos SEC/MCARD/DPIA
- `TYPE-001/002`: INC necesita severidad, ETH necesita base legal GDPR
- `REF-001`: Documentos referenciados existen
- `SEC-001`: No contiene información sensible
- `OBS-001`: Tag observabilidad requiere sección de alcance
- Vocabulario de clasificación declarada de trabajo *(fw-4.38.0+)*: el frontmatter de Charter (`work_verb` / `design_provenance`) y las entradas del backlog de follow-ups (`**Work verb**:` / `**Design provenance**:`) se verifican contra el vocabulario controlado — `design | implement | audit | operate` y `new | upstream`. **Solo advisory** (Baton #332): campos ausentes no emiten nada — no declarado es un estado honesto, nunca un error — y valores fuera del vocabulario emiten un warning que nunca bloquea.
- Ids de follow-up sin registrar *(cli-3.41.0+, #392)*: un AILOG cuyo cuerpo menciona un id `FU-NNN` / `FU-NNN-NNN` fuera de su propia sección `## Follow-ups` — donde el extractor no puede verlo — y ese id **no aparece en ninguna parte del registro**, emite un warning `FOLLOWUP-UNTRACKED-ID`. La pregunta es «¿pudo el extractor haberlo visto alguna vez?», así que tres formas no emiten nada *(cli-3.41.1+)*: ids que el registro conoce como entradas; ids que el registro menciona de cualquier otra forma — un alias de id de autor dentro del título de una entrada (`### FU-335 — FU-058-022 — …`), una referencia en `Notes`, una entrada cerrada y podada por triage; e ids que el propio documento declara en su `## Follow-ups`, esté donde esté la mención en prosa. **Solo warning**; se omite por completo si el proyecto no tiene registro.

Cuando `regional_scope` incluye `china`, se activan doce reglas adicionales (`CROSS-004` a `CROSS-011`, `TYPE-003` a `TYPE-006`) que cubren escalado de revisión TC260, vínculo PIPIA desde documentos con datos sensibles, cross-references de CACFILE / AILABEL, coherencia severidad-deadline CSL, y retención de 3 años de PIPIA. Sin `china` en scope, estas reglas se omiten — sin falsos positivos.

**Código de salida:** 0 si no hay errores (warnings OK), 1 si hay errores.

---

### `straymark new [path] [-t <tipo>] [--title <titulo>]`

Crea un nuevo documento StrayMark a partir de una plantilla.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--doc-type`, `-t` | — | Tipo de documento. Core (12): `ailog`, `aidec`, `adr`, `eth`, `req`, `tes`, `inc`, `tde`, `sec`, `mcard`, `sbom`, `dpia`. China (4, opt-in): `pipia`, `cacfile`, `tc260ra`, `ailabel`. |
| `--title` | — | Título del documento |

Si no se especifica `--doc-type` o `--title`, se solicitan de forma interactiva. Los tipos chinos se filtran del prompt (y se rechazan en `-t`) cuando `regional_scope` no incluye `china`.

**Ejemplos:**

```bash
# Creación interactiva
$ straymark new

# Crear un AILOG con título (no-interactivo)
$ straymark new -t ailog --title "Implementar autenticación JWT"

# Crear un ADR
$ straymark new --doc-type adr --title "Elegir PostgreSQL como base de datos"
```

**Ejemplo de salida:**

```
$ straymark new -t ailog --title "Refactorizar módulo de pagos"

  ✔ Created: .straymark/07-ai-audit/agent-logs/AILOG-2026-04-01-001-refactorizar-modulo-de-pagos.md

  Next steps:
    1. Edit the document to fill in details
    2. Commit: git add .straymark/07-ai-audit/agent-logs/AILOG-2026-04-01-001-refactorizar-modulo-de-pagos.md
```

---

### `straymark approve <doc-id> --outcome <outcome> --reviewer <id> [--at YYYY-MM-DD] [--notes "..."] [--path <dir>]`

*Disponible desde **cli-3.7.0** + **fw-4.6.0**. `--quiet` y warning de alto riesgo añadidos en cli-3.8.0.*

Registra una aprobación humana formal sobre un documento con `review_required: true`. Escribe los tres campos de aprobación en el frontmatter (`reviewed_by`, `reviewed_at`, `review_outcome`) **y** añade la sección canónica `## Approval` al cuerpo en una edición atómica. Implementa la señal de cierre canonizada en `DOCUMENTATION-POLICY.md §3.5`.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `<doc-id>` | — | ID del documento. Acepta el prefijo (`AIDEC-2026-05-02-001`) o el ID completo con slug (`AIDEC-2026-05-02-001-foo`). |
| `--outcome` | — | Uno de `approved`, `revisions_requested`, `rejected`. Solicita prompt en TTY si está ausente. |
| `--reviewer` | — | Identidad del revisor: email, GitHub handle, o DID. Solicita prompt en TTY si está ausente. |
| `--at` | hoy | Fecha de aprobación (`YYYY-MM-DD`). |
| `--notes` | — | Notas opcionales del revisor (anexadas en la sección del cuerpo). |
| `--path` | `.` | Directorio del proyecto. |

**Comportamiento:**

- Advierte (no falla) si el documento no tiene `review_required: true` — el sign-off retroactivo es un caso real.
- **Mutación de frontmatter** (latest-wins): reemplaza los `reviewed_by/_at/outcome` existentes; si no existen, los inserta tras `review_required:`. Implementa la convención multi-revisor de §3.5: el frontmatter mantiene la *última* aprobación.
- **Mutación del cuerpo** (cronológica): añade un nuevo bloque `## Approval` antes de cualquier firma de plantilla final. Re-ejecutar `approve` preserva los bloques anteriores, así el cuerpo muestra el historial completo de revisiones.
- `review_required: true` **no** se cambia a `false` tras la aprobación — permanece como registro histórico de por qué fue necesario revisar.

**Ejemplos:**

```bash
# Driven por flags (CI / scripts)
$ straymark approve AIDEC-2026-05-02-001 \
    --outcome approved \
    --reviewer pepe@example.com \
    --notes "Revisado contra ADR-007. LGTM."

  ✔ AIDEC-2026-05-02-001 marked as approved.
    Reviewer: pepe@example.com
    Date:     2026-05-02
    File:     .straymark/07-ai-audit/decisions/AIDEC-2026-05-02-001-foo.md

# Ciclo iterativo: revisions_requested → re-aprobación
$ straymark approve AIDEC-... --outcome revisions_requested --reviewer reviewer@x.io
# (autor itera)
$ straymark approve AIDEC-... --outcome approved --reviewer reviewer@x.io
# Frontmatter muestra la última (approved); el cuerpo conserva AMBOS bloques cronológicamente.

# Visibilidad del backlog
$ straymark validate --check-pending-reviews --max-pending-days 14
```

> Ver `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md` §3.5 "Recording Approval" para la definición canónica del flujo (semántica de cierre, formato del cuerpo, convención multi-revisor).

---

### `straymark charter <subcomando>`

Gestiona **Charters**: unidades acotadas y auditables de trabajo, declaradas ex-ante y validadas ex-post. Un Charter empareja scope declarativo (archivos a tocar, riesgos, comandos de verificación ejecutables) con el ancla de auditoría ex-post (drift detection, auditoría multi-modelo). Los Charters viven en `.straymark/charters/NN-slug.md` (a nivel del project root, **no** bajo `.straymark/`).

> **Nota histórica.** En el experimento Sentinel `/plan-audit` que cristalizó este patrón (abril 2026, 6 ciclos), los Charters se llamaban *Plans*. El CLI StrayMark usa **Charter** going-forward para evitar la colisión nominal con el `plan.md` de GitHub SpecKit. Los archivos históricos de Sentinel preservan "Plan" deliberadamente. El alcance conceptual completo y la justificación del rename viven en `Propuesta/que-es-un-charter.md`.

**Subcomandos:**

- `straymark charter new` — crea un nuevo Charter desde el template del framework
- `straymark charter list` — enumera Charters con filtros opcionales
- `straymark charter status` — muestra detalle de un Charter, o los 5 más recientes
- `straymark charter close` *(cli-3.7.0+)* — registra la telemetría post-ejecución y mueve el Charter a `closed`
- `straymark charter drift` *(cli-3.7.0+)* — chequea drift archivo-vs-commit con supresión AILOG-aware y gate de Batch Ledger *(gate añadido en cli-3.13.2)*
- `straymark charter batch-complete` *(cli-3.13.0+, fw-4.14.0+)* — marca un batch del Charter como completado en la `## Batch Ledger` del AILOG
- `straymark charter audit` *(cli-3.8.0+)* — orquesta una revisión externa multi-modelo (orchestration-only, ver más abajo)
- `straymark charter refresh-suggest` *(cli-3.14.0+, fw-4.16.0+)* — recomendación heurística para refresh SpecKit pre-declare en módulos multi-Charter (ver [CHARTER-CHAIN-EVOLUTION.md](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/i18n/es/CHARTER-CHAIN-EVOLUTION.md) Patrón 1). Umbral por defecto `r_n_plus_one_emergent_count` > 6; override solo vía `--threshold N` (no hay campo equivalente en `config.yml` en v0.2).
- `straymark charter amend` *(cli-3.14.0+, fw-4.16.0+)* — scaffolding para enmienda post-close Batch N.4 (remediación dirigida por auditoría sobre la misma rama de execute, sin abrir un Charter nuevo — ver [CHARTER-CHAIN-EVOLUTION.md](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/i18n/es/CHARTER-CHAIN-EVOLUTION.md) Patrón 2)

#### `straymark charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <titulo>] [path]`

Crea un Charter desde el template del framework en `.straymark/charters/NN-slug.md`. Si no se pasa `--title`, se solicita interactivamente. Los dos flags de origen son mutuamente excluyentes a nivel de clap.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--type`, `-t` | `M` | Estimación de esfuerzo. Uno de `XS`, `S`, `M`, `L`. |
| `--title` | — | Título del Charter. Se usa para construir el slug y el nombre de archivo. Solicita prompt si está ausente. |
| `--from-ailog` | — | ID del AILOG origen (p.ej. `AILOG-2026-04-28-021`). Pre-popula `originating_ailogs` en el frontmatter. **Mutuamente excluyente con `--from-spec`.** |
| `--from-spec` | — | Path a un spec.md de SpecKit (p.ej. `specs/001-feature/spec.md`). Pre-popula `originating_spec` en el frontmatter. El path se verifica al crear. **Mutuamente excluyente con `--from-ailog`.** |

Cuando ningún flag de origen se pasa, ambos `originating_ailogs` y `originating_spec` quedan comentados en el frontmatter generado — el Charter se crea "sin origen explícito" y el usuario lo llena antes de mover el status a `in-progress`.

El frontmatter generado también incluye dos campos opcionales de **clasificación declarada de trabajo** *(fw-4.38.0+, Baton #332)*: `work_verb: design | implement | audit | operate` y `design_provenance: new | upstream`. Declararlos cuesta ≈ 0 en autoría y son la señal autoritativa para el routing de modelos consciente de costo; un Charter sin declarar es inclasificable y se rutea conservadoramente al tier frontier. Dos reglas de determinación: definir un contrato fundacional acotado es `implement`, NO `design`; e `implement` + `design_provenance: upstream` (solo instrumenta diseño previo) degrada a trabajo mecánico.

**Ejemplos:**

```bash
# Standalone (sin origen) — prompt interactivo de título
$ straymark charter new --type M

# Modo mantenimiento / post-MVP — Charter rooteado en un AILOG existente
$ straymark charter new -t S --from-ailog AILOG-2026-04-28-021 --title "thresholds por servicio"

# Modo greenfield — Charter implementando un spec de SpecKit
$ straymark charter new -t L --from-spec specs/001-pagos/spec.md --title "integrar provider de pagos"
```

#### `straymark charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any] [path]`

Enumera Charters como tabla.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` | Directorio del proyecto |
| `--status` | `all` | Filtra por status del ciclo de vida |
| `--origin` | `any` (sin filtro) | Filtra por tipo de origen: `ailog`, `spec`, o `any` |

Los archivos que no parsean se reportan como warnings en stderr sin abortar el comando — la tabla muestra lo que puede.

#### `straymark charter status [CHARTER-ID] [--path <dir>]`

Con un ID: imprime el detalle completo del Charter (frontmatter, ubicación del archivo, lista de secciones del cuerpo). Sin ID: imprime los 5 Charters más recientes por NN descendente.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `CHARTER-ID` | — | Identificador del Charter. Acepta el `charter_id` completo (`CHARTER-01-test`), el prefijo `CHARTER-NN` (`CHARTER-01`), o solo el NN numérico (`01` o `1`). El match numérico es permisivo respecto al zero-padding. |
| `--path` | `.` | Directorio del proyecto. Es flag (no positional) para evitar colisión con el positional opcional `CHARTER-ID`. |

#### `straymark charter close <CHARTER-ID> [--from-template] [--non-interactive] [--path <dir>]`

Registra la telemetría post-ejecución y mueve el status del Charter a `closed`. La telemetría se escribe en `.straymark/charters/CHARTER-NN.telemetry.yaml` (archivo lateral, **no** embebido en el frontmatter del Charter — el frontmatter es declarativo ex-ante; la telemetría es voluminosa ex-post). El shape se valida contra `.straymark/schemas/charter-telemetry.schema.v0.json`.

Dos modos:

| Modo | Combinación de flags | Cuándo usar |
|---|---|---|
| **Interactivo** (default) | (ninguno) | Recorre el schema campo por campo con prompts. Tiempo objetivo: 5–10 min. |
| **From template** | `--from-template` | Copia el esqueleto YAML junto al Charter para edición manual. Pre-llena `charter_id`, título, `closed_at`. |
| **From template, scripted** | `--from-template --non-interactive` | Uso CI / batch. Omite todos los prompts; idempotente al re-ejecutar. |

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `CHARTER-ID` | — | Mismas reglas de resolución que `charter status`. |
| `--from-template` | false | Copia el esqueleto del template en lugar de correr el flujo interactivo. |
| `--non-interactive` | false | Omite todos los prompts. Requiere `--from-template`. |
| `--path` | `.` | Directorio del proyecto. |

**Ejemplo:**

```bash
$ straymark charter close CHARTER-01

  Closing CHARTER-01-test-charter
    Title: Test charter
  Press Enter to accept defaults; type to override.

  ── Trigger ──
  Declared trigger kind › event_trigger
  Declared trigger description › first false-positive ticket
  Fired at (YYYY-MM-DD) [2026-05-02]:
  ...

  ✔ Charter CHARTER-01 closed.
    Telemetry: .straymark/charters/CHARTER-01.telemetry.yaml
    Status updated: in-progress/declared → closed

  ── Follow-ups ──
  ✔ Extracted 2 follow-ups from 1 AILOG(s) into the registry (`## Bucket: ready`).
    Review against the TDE-promotion criteria (AGENT-RULES.md §3): ...
  Promote FU-078 — wire the retry budget into the sync loop to a TDE? [y/N]
```

**Reconciliación de follow-ups** *(cli-3.22.0+, RFC #135 Tier 3)*. Tras un cierre **interactivo**, el comando corre el scan por defecto de `followups drift` (rango git commiteado ∪ working tree) sobre los AILOGs recién escritos, extrae el contenido `§Follow-ups` / `R<N> (new)` **que aún no está en el registry** a `## Bucket: ready`, y luego ofrece **promoción a TDE por entrada** contra los cuatro criterios de §3 (herencia de Charter previo, alcance multi-módulo/Charter, Charter dedicado, priorización humana). Declinar un prompt deja el follow-up extraído (capturado en el registry, solo no promovido); aceptar corre el flujo `followups promote` (crea el TDE con trazabilidad `promoted_from_followup`). Es **no-op** cuando el proyecto no tiene registry de follow-ups o cuando no hay nada sin extraer, y se **salta en las rutas `--from-template`** (sin contexto de prompt interactivo) — corre `straymark followups drift --apply` ahí.

#### `straymark charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--no-batch-ledger-check] [--path <dir>]`

Detecta drift archivo-vs-commit al cierre del Charter. **Nativo en Rust desde cli-3.23.0** (#237) — ya no delega en el (ahora deprecado) `.straymark/scripts/check-charter-drift.sh`, así que corre en Windows-nativo (sin WSL, sin Git Bash). La propiedad de cero falsos positivos (PLAN-05 retrospectivo + PLAN-06 prospectivo en Sentinel) se preserva mediante la suite de tests de equivalencia con el script. El valor agregado del CLI sobre el script crudo es la **AILOG-awareness**: los paths reportados como "declarados pero no modificados" se silencian cuando aparecen en la sección `## Risk` / `## Riesgos` / `## 风险` de algún AILOG referenciado por `originating_ailogs` del Charter. Usa `--no-ailog-suppress` para deshabilitarlo.

**Gate de Batch Ledger** *(cli-3.13.0+)*. Cuando el Charter está en estado `in-progress` o `closed`, el comando también revisa cada AILOG originante por entradas `## Batch Ledger` que sigan en `(pending)` y falla con un diagnóstico claro listando los batches faltantes. Los AILOGs sin ledger no contribuyen — la sección es opt-in. Usa `--no-batch-ledger-check` para bypass (pensado para adopters que consolidan la ledger post-close).

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `CHARTER-ID` | — | Mismas reglas de resolución que `charter status`. |
| `--range` | `HEAD~1..HEAD` | Rango de revisiones git a chequear. |
| `--no-ailog-suppress` *(cli-3.10.0+ siempre emite una línea INFO de confirmación)* | false | Deshabilita la supresión AILOG-aware (muestra todo path declarado-omitido). Cuando se pasa el flag, el CLI siempre imprime una línea `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` — incluso cuando N=0 — para que el modo diagnóstico sea visible en la salida aun en una corrida limpia. |
| `--no-batch-ledger-check` *(cli-3.13.0+)* | false | Deshabilita el gate de Batch Ledger. Usar cuando el AILOG del Charter optó por no usar el patrón al momento del close. |
| `--path` | `.` | Directorio del proyecto. |

**Códigos de salida:** `0` si no hay drift (o solo AILOG-suprimido) y ningún batch pendiente; `1` si hay drift no contabilizado o algún `### Batch N` sigue `(pending)`; `2` para errores de uso (Charter no encontrado, bash ausente, etc.).

**Ejemplo:**

```bash
$ straymark charter drift CHARTER-01 --range origin/main..HEAD
=== Charter drift check ===
  Charter: .straymark/charters/01-test.md
  Range:   origin/main..HEAD
  Declared: 5 files
  Modified: 3 files

WARNING: Declared in Charter but NOT modified (1 files):
  - src/services/policy/repository.go

AILOG-suppressed: 1 path(s)
  - src/services/policy/repository.go [documented in AILOG-2026-05-02-001]

OK all declared-omitted paths are documented in AILOGs — drift accepted.
```

> **Nota de plataforma.** El chequeo de drift delega en `bash`. En Linux/macOS/WSL/Git Bash funciona out-of-the-box. En Windows nativo sin WSL, instalar Git Bash; un fallback puro Rust está en el roadmap pero no en fw-4.6.x.

#### Soporte de wildcards en paths declarados *(fw-4.9.0+)*

El chequeo de drift resuelve dos formas de wildcard en `## Files to modify`:

| Forma | Ejemplo | Caso de uso |
|---|---|---|
| Elipsis | `` `.straymark/07-ai-audit/agent-logs/AILOG-...md` `` | Cualquier path modificado con ese prefijo satisface el wildcard. Usado históricamente cuando un número desconocido de AILOGs serían creados durante la ejecución. |
| Glob | `` `AILOG-*.md` `` o `` `src/services/foo-*.rs` `` | Cualquier path modificado que matchee el glob (`*` → `.*`) satisface el wildcard. Usado para declaraciones bulk de Charter donde un set parametrizado es tocado. Añadido en fw-4.9.0 tras la fricción surgida en Sentinel CHARTER-04 ([issue #81](https://github.com/StrangeDaysTech/straymark/issues/81)). |

Ambas formas se manejan en ambas direcciones: un wildcard declarado suprime tanto warnings de "declarado pero no modificado" (cuando al menos un archivo matching fue modificado) como warnings de "modificado pero no declarado" (cuando un path modificado matchea un wildcard declarado).

#### Por diseño: rutas de gobernanza siempre están en scope

Los paths bajo `.straymark/charters/*` y `.straymark/07-ai-audit/*` **nunca** se reportan como "modificado pero no declarado". Es opinionated por diseño — esos paths son siempre legítimos cuando el Charter mismo o el AILOG de ejecución son tocados. Validado empíricamente en Sentinel CHARTER-04: un `git add -A` accidental stageó archivos no-tracked del usuario (`.claude/skills/`, `cmd/sentinel/sentinel`); la regla suprimió correctamente el ruido de gobernanza sin esconder la expansión genuina de archivos del proyecto ([issue #81 W2](https://github.com/StrangeDaysTech/straymark/issues/81#issuecomment-update)).

Si corres un Charter cuyo scope explícito es churn de gobernanza (p.ej. un Charter de aprobación bulk que toca solo `.straymark/07-ai-audit/`), el chequeo reportará 0 archivos modificados y necesitarás verificar el scope leyendo el AILOG. Un flag `--strict-scope` que deshabilite la regla "siempre en scope" está sobre la mesa para una minor futura si un adopter real reporta la asimetría como fricción.

#### `straymark charter batch-complete <CHARTER-ID> <N> [--note <body>] [--non-interactive] [--path <dir>]`

*Disponible desde **cli-3.13.0** + **fw-4.14.0**.*

Marca un batch del Charter como completado en la `## Batch Ledger` del AILOG originante. El comando sustituye el placeholder `(pending)` bajo `### Batch <N>` con notas del batch capturadas interactivamente (por defecto) o vía `--note` (one-shot / scripted). El gate de drift al cierre (`straymark charter drift`) rechaza cualquier `### Batch N` que quede como `(pending)`, haciendo la actualización por-batch load-bearing en vez de un recordatorio de disciplina.

**Cuándo usarlo.** Solo para Charters multi-batch que abarquen 3+ lotes o >1 día de ejecución. AILOGs de un solo batch documentan todo en `## Acciones Realizadas` y saltan la ledger por completo. El template Charter en §Tasks recuerda al autor mantener la ledger cuando aplica.

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `CHARTER-ID` | — | Mismas reglas de resolución que `charter status`. |
| `N` | — | Número de batch (1-based) — coincide con el heading `### Batch <N>` en `## Batch Ledger` del AILOG. |
| `--note <body>` | — | Cuerpo de la nota pre-llenado. Con este flag, el comando escribe la nota no-interactivamente y salta los prompts. Diseñado para agentes y scripts. |
| `--non-interactive` | false | Deshabilita prompts. Requiere `--note` (la ausencia de `--note` aborta en lugar de colgarse). |
| `--path` | `.` | Directorio del proyecto. |

El comando lee `originating_ailogs[0]` del frontmatter del Charter para localizar el AILOG destino. Rechaza con error claro cuando:

- el Charter no tiene entrada `originating_ailogs` (no puede resolver archivo destino);
- el archivo AILOG no existe en `.straymark/07-ai-audit/agent-logs/`;
- el AILOG no tiene sección `## Batch Ledger` (añadir la sección primero, o saltar el comando si el Charter es de un solo batch);
- no existe heading `### Batch <N>` bajo `## Batch Ledger`;
- el batch destino ya está completado (rehúsa sobrescribir — editar el AILOG manualmente para correcciones).

**Flujo interactivo.** Tres prompts: archivos tocados (one-liner), tests añadidos o estado (one-liner), y nota de diseño multilínea (terminar con `.` solo en una línea o Ctrl-D). Los campos vacíos se omiten en la salida. El cuerpo resultante se escribe bajo el heading `### Batch <N>`; el placeholder `(pending)` se reemplaza.

**Ejemplos:**

```bash
# Interactivo — humanos pegando notas
$ straymark charter batch-complete CHARTER-17 5
✔ AILOG: .straymark/07-ai-audit/agent-logs/AILOG-2026-05-13-048-charter-17.md
✔ ### Batch 5 — Migración 022 + handlers

OK `### Batch 5` written.
Reminder: `git add .straymark/07-ai-audit/agent-logs/AILOG-2026-05-13-048-charter-17.md` before pushing.

# One-shot — agentes / scripts
$ straymark charter batch-complete CHARTER-17 5 \
    --note "Migración 022 + handlers. Archivos: migrations/022.sql, services/handler_x.go. Tests pasando. R8 surgió (CHECK constraint sin ARCHIVED), fix atómico."
OK `### Batch 5` written.
```

**Integración con workflow.** Según la guía del template Charter en §Tasks, correr `batch-complete` *inmediatamente después* de que aterriza el commit del batch pero *antes* de pushear. La actualización del AILOG y el trabajo que documenta viajan en el mismo push. El gate de drift al cierre (`straymark charter drift CHARTER-NN`) rechaza cualquier batch pendiente e imprime la lista — convirtiendo "olvidé actualizar la ledger" en una falla dura en lugar de erosión silenciosa del audit trail.

#### `straymark charter audit <CHARTER-ID> [--range <REV..REV>] [--prepare | --merge-reports] [--merge-into <PATH>] [--path <dir>]` {#straymark-charter-audit}

*Disponible desde **cli-3.8.0** + **fw-4.7.0**. Flujo unificado v1 shippeado en **cli-3.10.0** + **fw-4.9.0** — reemplaza los 3 pasos v0 (PREPARE/CALIBRATE/FINALIZE) por 2 (PREPARE/MERGE-REPORTS), unifica la plantilla del auditor, y mueve los paths canónicos a `.straymark/audits/`.*

Orquesta una revisión externa multi-modelo de la ejecución de un Charter. **Orchestration-only** — el CLI prepara la plantilla unificada del audit, valida los reports de auditores contra el schema, y emite/mergea el bloque YAML `external_audit`. **NO invoca APIs de LLM.** El operador corre N CLIs auditoras (agy, claude-cli, copilot-cli, codex-cli — la que tenga) configuradas con acceso read-only al filesystem; cada una invoca el skill `/straymark-audit-execute` para leer el prompt, auditar con tool use citando `path:line`, y escribir el report.

Dos pasos, cada uno invocable independientemente:

| Paso | Flag | Qué pasa |
|---|---|---|
| 1. PREPARE | `--prepare` (default) | Resuelve la plantilla unificada del audit contra el Charter + git diff + AILOGs origen. La escribe en `.straymark/audits/<CHARTER-ID>/audit-prompt.md`. |
| 2. MERGE-REPORTS | `--merge-reports` | Lee todos los archivos `report-*.md` en `.straymark/audits/<CHARTER-ID>/` (uno por auditor que terminó). Valida cada uno contra `audit-output.schema.v0.json`. Emite el array YAML `external_audit` — combina con `--merge-into <PATH>` para anexarlo directamente a la telemetría del Charter. |

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `<CHARTER-ID>` | — | Mismas reglas de resolución que `charter status`. |
| `--range` | `origin/main..HEAD` (con fallback a `origin/master..HEAD`, luego `HEAD~1..HEAD` con warning) | Rango git que los auditores revisan. El default captura el set completo de commits de la feature branch; el override explícito vía `--range <REV..REV>` no prueba upstream. **Pitfall multi-batch:** para una auditoría phase-scoped de un Charter cuyas fases anteriores ya se mergearon a la rama base, el default `origin/main..HEAD` *excluye* los commits mergeados y sub-cubre la fase silenciosamente — pasa un `--range <primer-commit-del-charter>..HEAD` explícito para abarcarla. `--prepare` imprime un aviso cuando detecta batches completados sin rango explícito. |
| `--prepare` | off (acción default cuando ningún otro flag se pasa) | Corre el paso 1. Mutuamente excluyente con `--merge-reports`. |
| `--merge-reports` | off | Corre el paso 2. Mutuamente excluyente con `--prepare`. |
| `--merge-into <PATH>` | — | Con `--merge-reports`: anexa el array `external_audit:` directamente a la telemetría YAML en `<PATH>` en lugar de imprimir a stdout. El CLI rechaza re-audit (la telemetría ya tiene la clave) con error claro. |
| `--include-audit-artifacts` | off | Embebe `.straymark/audits/**` en el diff que se manda a los auditores. Apagado por default desde **cli-3.38.0** — ver §Aislamiento de auditores abajo. Pásalo solo cuando auditar el propio rastro de auditoría sea el punto. |
| `--path` | `.` | Directorio del proyecto. |

**Flags v0 deprecated (ocultos en `--help`):**

- `--calibrate` — emite warning y sale con error. El paso v0 calibrate se reemplaza por la skill `/straymark-audit-review` que reconcilia N reports inline con acceso al filesystem (sin prompt paste-based separado).
- `--finalize` — alias deprecated de `--merge-reports` con comportamiento backwards-compat. Emite warning y rutea por la nueva ruta.

##### Aislamiento de auditores — los artefactos de auditoría se excluyen del diff *(cli-3.38.0+)*

`--prepare` embebe el git diff del rango auditado en el prompt. Como el flujo indica que los reportes y reviews vivan bajo `.straymark/audits/`, un commit que los aterriza mete **los reportes y el review consolidado de la ronda anterior dentro del mismísimo diff que la siguiente ronda debe auditar**. El auditor lee entonces la opinión de sus pares antes de formar la propia — y N reportes que heredaron un mismo encuadre son un solo dato con N sombreros, no N datos. La convergencia entre modelos solo es señal si cada auditor llegó a ella de forma independiente.

Desde **cli-3.38.0** el diff embebido excluye `.straymark/audits/**` por default (un pathspec `:(exclude)` sobre el `git diff`). Los reportes de auditoría nunca son el objeto de una auditoría; son subproducto de gobernanza que casualmente vive en el árbol versionado. Cuando el rango sí los toca, `--prepare` lo dice y lista lo que descartó:

```text
  ℹ Excluded 2 audit artifact(s) from the embedded diff (prior-round reports/reviews) — auditor isolation preserved.
      .straymark/audits/CHARTER-55/ronda-1/report-gemini-3-pro.md
      .straymark/audits/CHARTER-55/ronda-1/review.md
```

`--include-audit-artifacts` reactiva la inclusión y advierte que la convergencia de esa ronda no es evidencia independiente.

> **Por qué prevención y no detección.** La skill `/straymark-audit-review` tiene un guard de contaminación que marca reportes con señales de haber leído a sus pares. Ese guard se queda — pero solo puede *marcar* un reporte contaminado, nunca descontaminarlo, y un reporte marcado es una auditoría desperdiciada. Una regla en el prompt o un README que diga "no leas esto" tampoco es aislamiento: un modelo puede racionalizar los reportes previos como contexto útil. El lugar más barato para hacer valer la independencia es el momento en que se construye el prompt. (Reportado por el adoptante Sentinel en un ciclo de 4 rondas donde 1,092 de 1,581 líneas del diff embebido eran prosa de auditoría de la ronda anterior — issue #372.)

##### Recomendación de heterogeneidad (no enforced en v0)

El par de auditores debería ser de **familias de modelo distintas**: uno Anthropic + uno Google + uno OpenAI, en cualquier combinación, nunca dos de la misma familia. La heterogeneidad inter-familia es lo que hace que la convergencia en findings sea de alta señal — auditores de la misma familia comparten blind spots.

v1 soporta **N≥2 auditores** (ya no fijo a 2). El operador puede optar por 3 o 4 auditores para Charters de alto riesgo, incluyendo modelos especializados. La skill `/straymark-audit-review` itera sobre todos los archivos `report-*.md` en el audit dir.

El rol calibrador se mueve de una plantilla paste-based (v0) al agente principal in-conversation vía la skill `/straymark-audit-review` — su tarea es definicional (reconciliar veredictos ya producidos), por lo que la heterogeneidad respecto al implementador NO es requerida.

##### Layout canónico producido (v1)

```
.straymark/audits/CHARTER-NN/
├── audit-prompt.md                          # resuelto por --prepare (single unified prompt)
├── report-claude-sonnet-4-6.md              # escrito por /straymark-audit-execute en claude-cli
├── report-gemini-2-5-pro.md                 # escrito por /straymark-audit-execute en agy
├── report-gpt-5-3-codex.md                  # 3er auditor opcional
├── review.md                                # escrito por /straymark-audit-review (análisis consolidado de 6 secciones)
└── external-audit-pending.yaml              # escrito por /straymark-audit-review cuando la telemetría aún no existe (Branch B)
```

El directorio está namespaceado bajo `.straymark/` para evitar colisiones con carpetas `audit/` que el adoptante haya definido. El shape `<UNIT-TYPE>-<UNIT-ID>` deja espacio para futuras categorías de unidad de auditoría más allá de Charter (ej. `MODULE-payments/`, `RELEASE-v2.0/`) sin reestructurar.

Los adopters pueden `git add` el directorio entero `.straymark/audits/` para un audit trail completamente versionado, o `.gitignore` si prefieren un ciclo efímero.

**Ejemplo (v1, con los wrappers de skills — recomendado para flujos IDE-driven):**

```bash
# En el IDE principal (Claude Code, Antigravity CLI, Cursor, ...):
> /straymark-audit-prompt CHARTER-05
  → corre `straymark charter audit CHARTER-05 --prepare`
  → escribe .straymark/audits/CHARTER-05/audit-prompt.md
  → instruye al operador abrir CLIs auditoras

# En claude-cli (con acceso read al repo):
> /straymark-audit-execute CHARTER-05
  → escribe .straymark/audits/CHARTER-05/report-claude-sonnet-4-6.md
  → recuerda al operador esperar a TODAS las auditorías antes de review

# En agy:
> /straymark-audit-execute CHARTER-05
  → escribe .straymark/audits/CHARTER-05/report-gemini-2-5-pro.md

# De vuelta en el IDE principal, después de que TODAS las auditorías terminen:
> /straymark-audit-review CHARTER-05
  → lee N reports, verifica cada finding contra el código
  → escribe .straymark/audits/CHARTER-05/review.md (consolidado de 6 secciones)
  → corre `straymark charter audit CHARTER-05 --merge-reports --merge-into <telemetría>`
  → external_audit YAML mergeado en la telemetría del Charter
```

> **¿Por qué orchestration-only?** Implementar 3 HTTP clients (OpenAI / Google / Anthropic) son 1-2 semanas + mantenimiento perpetuo. v1 audit-skills extiende el orchestration-only a un segundo modo (CLI auditor-side con tool use enforcement) donde el operador corre sus propias CLIs auditoras y los prompts de StrayMark enforzan la disciplina (`citar path:línea de archivos efectivamente abiertos`). StrayMark no maneja API keys, no invoca APIs, no mantiene HTTP clients.

> **Alternativa con skill *(fw-4.9.0+, expandida en fw-4.9.0)*.** Tres skills envuelven el CLI para flujos IDE-driven: `/straymark-audit-prompt CHARTER-ID` (llama a `--prepare`), `/straymark-audit-execute CHARTER-ID` (corre en CLIs auditoras para leer el prompt y escribir un report), y `/straymark-audit-review CHARTER-ID` (consolida N reports en `review.md` y mergea YAML). Con estas skills el operador nunca copia/pega prompts ni reports — el intercambio sucede vía paths canónicos del filesystem bajo `.straymark/audits/`. Ver la sección [Skills](#skills) más abajo. El CLI sigue siendo la fuente única de verdad — las skills solo añaden UX-inline.

---

### `straymark followups <subcommand>` *(cli-3.19.0+)*

Gestiona el **registro del backlog de follow-ups** (`.straymark/follow-ups-backlog.md`) — el artefacto de primera clase que agrega las entradas `§Follow-ups` y `R<N> (new, not in Charter)` a través de los AILOGs. Schema: `.straymark/schemas/follow-ups-backlog.schema.v1.json` (v1 experimental). Convención: `FOLLOW-UPS-BACKLOG-PATTERN.md` y `STRAYMARK.md §16`; directivas de agente distribuidas en `AGENT-RULES.md §13`.

El parsing es **tolerante**: los registros v0 (pre-fw-4.21.0) se leen sin errores; el primer comando de escritura (`drift --apply` — incluso sin nada que extraer, cli-3.20.0+ —, `recount` o `promote`) los actualiza a v1 in place, de forma no destructiva. Los contadores `total_*` del frontmatter son **propiedad del CLI** — se recalculan en cada escritura; nunca los edites a mano.

- `straymark followups list` — enumera las entradas *(cli-3.19.0+)*
- `straymark followups status` — pulso del registro / detalle de una entrada *(cli-3.19.0+)*
- `straymark followups drift` — sincroniza el registro con los AILOGs (reemplazo nativo del `check-followups-drift.sh` adopter-side, ya deprecado) *(cli-3.19.0+)*
- `straymark followups recount` — recalcula los contadores propiedad del CLI tras una sesión de triage manual *(cli-3.20.0+)*
- `straymark followups promote` — eleva una entrada a un documento TDE *(cli-3.19.0+)*
- `straymark followups verify` — re-verifica la premisa de una hipótesis fechada en tiempo de ejecución *(cli-3.37.0+)*
- `straymark followups merge-driver` — merge driver de git que resuelve conflictos del registro estructuralmente (#391) *(cli-3.41.0+)*

#### `straymark followups list [--bucket <name>] [--status <s>] [--severity <s>] [--label <tag>] [path]`

Tabla de entradas: id FU, status, severidad, bucket, destino, descripción. Las advertencias de parsing (encabezados `### FU-` malformados) van a stderr sin hacer fallar el comando.

| Flag | Default | Descripción |
|------|---------|-------------|
| `--bucket <name>` | — | Filtra por bucket: `ready`, `time-triggered`, `charter-triggered`, `phase-blocked`, `operational` |
| `--status <s>` | — | Filtra por status: `open`, `in-progress`, `suspected-closed`, `closed`, `superseded`, `promoted` |
| `--severity <s>` | — | Filtra por severidad: `normal`, `blocking` |
| `--label <tag>` | — | Filtra por etiqueta (match exacto case-insensitive sobre una sola etiqueta) |

```bash
$ straymark followups list --severity blocking
  FU      STATUS  SEV       BUCKET  DEST          DESCRIPTION
  FU-010  open    blocking  ready   mini-charter  Harden staging probe
```

#### `straymark followups status [FU-NNN] [--path <dir>]`

Sin un id: el pulso del registro — contadores **recalculados al vuelo** a partir de los status reales de las entradas (fiables incluso cuando el frontmatter del archivo está obsoleto; la divergencia se señala), desglose de entradas `open` por bucket, alertas de `blocking` y `suspected-closed`, validación de schema advisory. Con un id: el detalle completo de los campos de la entrada.

#### `straymark followups drift [--apply] [--scan-all] [--range <REV..REV>] [--path <dir>]`

Detecta los AILOGs cuyo contenido de follow-ups aún no se ha extraído al registro. La granularidad es **por-AILOG** (la lista `fully_extracted_ailogs` del frontmatter) — la decisión de diseño validada empíricamente (0 falsos positivos a través de 76 AILOGs en el adoptante de referencia).

| Flag | Default | Descripción |
|------|---------|-------------|
| *(default)* | — | Escanea los AILOGs cambiados en `origin/main..HEAD` (fallback `origin/master..HEAD`, luego `HEAD~1..HEAD` con una advertencia). Avisa + **exit 1** ante drift. |
| `--apply` | off | Extrae las entradas faltantes a `## Bucket: ready` con ids `FU-NNN` auto-numerados, añade los ids de los AILOGs a `fully_extracted_ailogs`, **recalcula los contadores**, y actualiza los registros v0 a v1 in place. Siembra el registro desde el template del framework cuando no existe. Desde cli-3.20.0 los contadores se recalculan **incluso cuando no hay nada que extraer** (#222 Finding 1). Las entradas cuyo **título** ya existe en el registro se omiten (#391) — los ids son posicionales y se renumeran al regenerar, así que el título es la identidad estable; esto evita que una declaración que cambió de sección genere una entrada duplicada `open` que opaque el status del operador. |
| `--scan-all` | off | Barre cada AILOG del proyecto en lugar del rango de git. |
| `--range <REV..REV>` | — | Rango de git explícito para el escaneo por defecto. |

**Refinamiento anti-ruido** (issue #214 Signal 1): los bullets cuyo texto del AILOG lleva un marcador de cierre explícito — `closed in-Charter`, `fixed in batch N`, un commit hash entre backticks, o *(cli-3.20.0+, #222 Finding 2)* un modismo born-resolved (un verbo de cierre `updated`/`corrected`/`remediated`/`resolved`/`fixed`/`closed` seguido de `in this PR` / `in this commit`, p.ej. `updated atomically in this PR`) — se extraen como **`suspected-closed`** en lugar de `open`, así el trabajo ya resuelto deja de contaminar el bucket `ready` como ruido TBD. El operador confirma (→ `closed`) o reabre en el siguiente triage.

```bash
$ straymark followups drift --scan-all --apply
✓ Extracted 4 entries from 1 AILOG(s) into `## Bucket: ready`.
  ! 1 extracted as suspected-closed (closure marker in source AILOG) — confirm at the next triage.
  Counters recomputed: 3 open / 1 suspected-closed / 0 promoted (total 4).
```

#### `straymark followups recount [--path <dir>]` *(cli-3.20.0+)*

Recalcula los contadores `total_*` propiedad del CLI desde los estados reales de las entradas y reescribe el frontmatter — sin escanear AILOGs, sin extraer y sin tocar entradas. La vía conforme al §13 para reconciliar contadores tras una **sesión de triage manual** (estados cambiados a mano según el ciclo de vida sancionado de Triage/Consumo, sin nada que extraer ni promover — #222 Finding 1, primer adopter externo). Idempotente: una segunda corrida reporta los contadores ya en sync. Actualiza registros v0 a v1 in situ, como todo comando de escritura.

```bash
$ straymark followups recount
✓ Counters recomputed: 0 open / 0 suspected-closed / 2 promoted (total 2).
```

#### `straymark followups promote <FU-NNN> [--title <title>] [--premise-verified] [--path <dir>]`

Automatiza la elevación FU → TDE (`FOLLOW-UPS-BACKLOG-PATTERN.md` §Promotion to TDE): crea el documento TDE desde el template del framework con trazabilidad `promoted_from_followup: FU-NNN`, cambia la entrada a `Status: promoted` con `Destination`/`Promoted to` apuntando al id del TDE, y recalcula los contadores. No interactivo (agent-friendly); la descripción del FU se vuelve el título del TDE salvo que `--title` lo sobreescriba. La priorización y la asignación siguen siendo humanas (`AGENT-RULES.md §3`).

Desde **cli-3.37.0** el comando superficie el `Premise` de la entrada (o los `Notes` si falta) con un recordatorio de re-verificación — un follow-up es una *hipótesis fechada* (§Estatus epistémico), así que re-chequea su premisa contra el código antes de construir sobre ella. `--premise-verified` registra que lo hiciste, sellando `Verified-at: <hoy>`. El recordatorio es informativo — la promoción procede de cualquier forma.

```bash
$ straymark followups promote FU-010 --premise-verified
✓ FU-010 promoted → TDE-2026-06-04-001
  TDE created: .straymark/06-evolution/technical-debt/TDE-2026-06-04-001-harden-staging-probe.md
  Premise re-verification recorded: Verified-at → 2026-06-04.
```

#### `straymark followups verify <FU-NNN> [--premise "..."] [--verified] [--at <YYYY-MM-DD>] [--path <dir>]` *(cli-3.37.0+)*

Re-verifica la premisa de un follow-up **en tiempo de ejecución** y la registra. Una entrada del registry es una **hipótesis fechada y decadente** (`AIDEC-2026-07-18-001`, de #365): su premisa pudo ser falsa en la captura o haberse vuelto obsoleta, y el único bug real es actuar sobre una sin re-testear su premisa. El registry es un buffer especulativo — la captura barata es su valor — así que la verificación pertenece al momento barato (actuar sobre la entrada), no a la captura. Este verbo cubre el caso común de una entrada actuada como chore que nunca promueve.

| Flag | Default | Descripción |
|------|---------|-------------|
| `--premise <texto>` | — | Registra o actualiza el `Premise` de la entrada (la suposición a re-chequear). Omitido → se superficie la premisa existente. |
| `--verified` | off | Sella `Verified-at`, confirmando que la premisa se re-chequeó contra el código. |
| `--at <YYYY-MM-DD>` | hoy | Fecha de verificación. |

Sin `--premise`/`--verified` es **read-only** — superficie la premisa y hace un nudge. El juicio humano queda fuera del CLI: superficie y sella, nunca decide la verdad.

```bash
$ straymark followups verify FU-016 --premise "yrs tiene una referencia independiente (Yjs)" --verified
✓ FU-016 premise recorded.
✓ FU-016 verified — Verified-at → 2026-06-04.
```

#### `straymark followups note <FU-NNN> "<texto>" [--source <ID>] [--path <dir>]` *(cli-3.39.0+)*

Anexa una anotación fechada a los `Notes` de una entrada en una sola edición validada. Antes de este verbo (#355), registrar que una entrada recibió una mitigación *parcial* — sin cambiar su status — implicaba editar a mano un archivo parseado por el CLI: una edición que puede malformar la entrada y romper `list`/`status`/`drift`, sin nada que registre cuándo se hizo la nota ni qué la motivó.

| Flag | Default | Descripción |
|------|---------|-------------|
| `--source <ID>` | — | El Charter o AILOG que motivó la nota (ej. `CHARTER-04`), registrado junto a la fecha para que la anotación siga siendo atribuible. |

`Notes` es un campo de una sola línea por contrato del parser, así que las anotaciones se **componen** sobre el valor existente en vez de apilarse como bullets nuevos.

#### `straymark followups set-status <FU-NNN> <status> [--path <dir>]` *(cli-3.39.0+)*

Cambia el status de una entrada **y** recomputa los contadores CLI-owned en el mismo paso. Cierra la ventana de desincronización que `recount` existe para limpiar (#355): el dos-pasos que reemplaza — editar el bullet `Status` y luego acordarse de `recount` — se desincroniza en cuanto olvidas la segunda mitad, dejando a los contadores mintiendo en silencio sobre el backlog.

Statuses válidos: `open` · `in-progress` · `suspected-closed` · `closed` · `superseded`. Un valor fuera de ese vocabulario se **rechaza**, no se escribe — el parser es indulgente, así que un typo no fallaría: sacaría la entrada de todos los contadores en silencio. `promoted` redirige a `followups promote`, que además escribe el TDE que le da a ese status algo a lo que apuntar.

#### `straymark followups merge-driver <base> <ours> <theirs>` *(cli-3.41.0+)*

Merge driver de git para `.straymark/follow-ups-backlog.md` (#391). El registro es CLI-owned, así que todo PR paralelo que toca follow-ups genera conflicto en él — y resolverlo tomando un lado y re-ejecutando `drift --apply` **revertía en silencio los cierres del otro lado** (los statuses viven solo en el archivo, y una re-extracción renumera los ids, por lo que ni comparando ids se detecta la pérdida). Conectado como merge driver, el conflicto desaparece: git entrega las tres versiones del archivo a la CLI y el resultado se escribe de vuelta en `ours`.

El merge es **estructural, no textual**: las entradas se emparejan entre lados **por título** (los ids son posicionales y no sobreviven a una regeneración; los títulos sí), y se reconcilian así:

| Situación | Resolución |
|---|---|
| Misma entrada, distinto status | Gana el status de mayor rango (`open` < `in-progress` < `suspected-closed` < `closed`/`superseded`/`promoted`) — un cierre hecho en cualquier lado sobrevive. Desacuerdos del mismo rango conservan `ours` y se reportan por stderr. |
| Entrada solo en `theirs` | Se añade (renumerada si su id colisiona con una entrada de `ours`). |
| Entrada eliminada por `theirs` | Se quita de `ours` salvo que `ours` haya cambiado su status (modificar/eliminar → se conserva + se reporta). |
| `Notes` | Gana `theirs` cuando es una extensión append-only de `ours` (la forma de `followups note`). |
| Frontmatter | `fully_extracted_ailogs` unido, `last_scan` más reciente, contadores recomputados del cuerpo ya mergeado. |

El exit code sigue el contrato de merge-driver de git: `0` = mergeado (conflictos suaves reportados por stderr), distinto de cero = sin resolver (git marca el archivo en conflicto).

**Configuración (una vez por clon) — `straymark followups install-merge-driver [--path .]` *(cli-3.44.0+)*:**

```bash
straymark followups install-merge-driver
```

Escribe las dos mitades: la línea de `.gitattributes` (commiteable, para que el equipo la herede) y `merge.straymark-followups.driver` en la config git del clon. Idempotente; si ya hay un binding o un driver configurado distinto, lo respeta y lo reporta. `straymark init` lo ofrece como prompt, o acepta `--merge-driver` / `--no-merge-driver` para instalaciones desatendidas.

> **Hace falta en cada clon, y saltárselo no sale gratis.** `.git/config` nunca se commitea, así que quien reciba la línea de `.gitattributes` sin ejecutar el comando **no** obtiene un conflicto normal: git aborta el merge con `fatal: custom merge driver straymark-followups lacks command line`.

El equivalente a mano:

```bash
echo '.straymark/follow-ups-backlog.md merge=straymark-followups' >> .gitattributes
git config merge.straymark-followups.driver 'straymark followups merge-driver %O %A %B'
```

#### `straymark followups new --title <título> --origin <origen> [--bucket <name>] [--status <s>] [--trigger <t>] [--destination <d>] [--cost <c>] [--premise <p>] [--path <dir>]` *(cli-3.39.0+)*

Crea una entrada cuyo origen es una **declaración de Charter** (ex-ante), antes de que exista ejecución alguna (#360). Las dos rutas de poblado anteriores asumen origen ex-post: `drift --apply` extrae de AILOGs, y un diferimiento decidido *en tiempo de declaración* — "el job de CI de Redis queda fuera de alcance; registra el hueco de cobertura para que quede diferido, no silenciado" — precede a cualquier AILOG por diseño.

El riesgo que cierra es de corrección, no de ergonomía. A falta de un verbo de creación, el adoptante que lo reportó forward-referenció `FU-011` en el cuerpo del Charter sin nada que lo reservara; y como los ids se acuñan `max(existente) + 1` en tiempo de extracción, el siguiente `drift --apply` no relacionado le daría `FU-011` a otra entrada, apuntando en silencio las citas del Charter al follow-up equivocado. `new` asigna el id de forma atómica y lo imprime, así que cuando el Charter lo cita la entrada ya existe.

La entrada se escribe con `Origin-class: ex-ante-planning` y **sin `Source-hash`**: no hay AILOG que hashear, e inventar uno haría que un `drift --apply` posterior creyera haber extraído algo que nunca vio.

> **Los tres se niegan a escribir un registro con avisos de parseo.** Una edición quirúrgica contra una estructura mal leída puede corromper entradas vecinas, así que primero hay que arreglar la entrada malformada. `recount` sigue siendo la vía de escape para una sesión de triage manual masivo — y el chequeo idempotente de que estos verbos hicieron bien la aritmética.

---

### `straymark compliance [path] [--standard <nombre>] [--region <nombre>] [--all] [--output <formato>]`

Verifica cumplimiento regulatorio. Por defecto evalúa los estándares cuya región esté incluida en `regional_scope` de `.straymark/config.yml` (default `[global, eu]`). Seis frameworks chinos disponibles opt-in cuando `china` se añade a `regional_scope`.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--standard` | — | Verificar estándar específico: `eu-ai-act`, `iso-42001`, `nist-ai-rmf`, `china-tc260`, `china-pipl`, `china-gb45438`, `china-cac`, `china-gb45652`, `china-csl` |
| `--region` | — | Ejecutar todos los estándares de una región: `global`, `eu`, `china`, o `all` |
| `--all` | — | Verificar todos los estándares (ignora `regional_scope`) |
| `--output` | `text` | Formato de salida: `text`, `markdown`, o `json` |

Precedencia: `--standard` > `--all` > `--region` > el `regional_scope` del proyecto.

**Estándares chinos (opt-in vía `regional_scope: china`):**

- **TC260 v2.0**: existe TC260RA; niveles altos requieren review; los tres criterios (escenario × inteligencia × escala) están completos
- **PIPL**: PIPIA cuando `pipl_applicable: true`; transferencia transfronteriza documentada; retención ≥ 3 años (Art. 56)
- **GB 45438**: AILABEL para contenido generativo; estrategia explícita + implícita declaradas; campos de metadata mandatorios
- **CAC**: CACFILE cuando es requerido; `cac_filing_status` explícito; `cac_filing_number` cuando el estado es `*_approved`
- **GB/T 45652**: SBOM y MCARD declaran cumplimiento de seguridad de datos de entrenamiento
- **CSL 2026**: cada INC con `csl_severity_level`; horas coherentes con severidad (1h ↔ particularly_serious, 4h ↔ relatively_major); post-mortem 30 días para incidentes major+

**Ejemplos:**

```bash
# Default: solo estándares cuya región esté en regional_scope
$ straymark compliance

# Los seis frameworks chinos (requiere regional_scope: china)
$ straymark compliance --region china

# Un solo framework chino
$ straymark compliance --standard china-pipl --output json

# Todos los estándares ignorando regional_scope
$ straymark compliance --all
```

> **Activación**: para evaluar los frameworks chinos automáticamente, añadir a `.straymark/config.yml`:
>
> ```yaml
> regional_scope:
>   - global
>   - eu
>   - china
> ```

---

### `straymark metrics [path] [--period <periodo>] [--output <formato>]`

Muestra métricas de gobernanza y estadísticas de documentación.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--period` | `last-30-days` | Período: `last-7-days`, `last-30-days`, `last-90-days`, o `all` |
| `--output` | `text` | Formato de salida: `text`, `markdown`, o `json` |

**Métricas incluidas:**

- Conteo de documentos por tipo dentro del período
- Tasa de cumplimiento de revisiones
- Distribución de niveles de riesgo
- Actividad por agente
- Tendencias vs período anterior (↑/↓/→)

---

### `straymark analyze [path] [--threshold <N>] [--output <formato>] [--top <N>]`

Analiza la complejidad del código fuente usando métricas cognitivas y ciclomáticas, impulsado por [arborist-metrics](https://crates.io/crates/arborist-metrics).

**Argumentos y flags:**

| Argumento/Flag | Predeterminado | Descripción |
|----------------|----------------|-------------|
| `path` | `.` (directorio actual) | Directorio a analizar |
| `--threshold` | `8` (o desde config) | Umbral de complejidad cognitiva |
| `--output` | `text` | Formato de salida: `text`, `json` o `markdown` |
| `--top` | — | Mostrar solo las N funciones más complejas |

**Lenguajes soportados:** Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift

**Resolución de umbral:** flag CLI → `.straymark/config.yml` → predeterminado (8)

**Configuración** (opcional, en `.straymark/config.yml`):

```yaml
complexity:
  threshold: 8
```

**Ejemplos:**

```bash
# Analizar directorio actual
$ straymark analyze

# Umbral personalizado y top 10
$ straymark analyze --threshold 5 --top 10

# Salida JSON para integración CI
$ straymark analyze --output json

# Analizar un proyecto específico
$ straymark analyze /ruta/al/proyecto
```

**Ejemplo de salida:**

```
  StrayMark Analyze
  /home/user/project
  Threshold: cognitive complexity > 8

  Functions exceeding threshold (3 of 42 total)

    FILE                                     FUNCTION                  LINE  COGN  CYCL  SLOC
    src/parser.rs                            parse_expression            42    18    12    45
    src/compiler.rs                          Compiler::emit             128    15     9    38
    src/eval.rs                              evaluate                    67    12     8    29

  Summary
    → Files analyzed: 12
    → Total functions: 42
    → Above threshold: 3 (7.1%)
    → Max cognitive complexity: 18 (src/parser.rs:parse_expression)
    → Average cognitive complexity: 3.8
```

> **Nota:** Este comando funciona sin `straymark init`. Opera sobre archivos fuente, no documentos StrayMark. La feature `analyze` se puede desactivar en compilación con `--no-default-features`.

> **Trigger de documentación:** Los agentes de IA usan `straymark analyze --output json` como método primario para determinar cuándo crear documentos AILOG. Si `summary.above_threshold > 0` en la salida JSON, el agente debe crear un AILOG. Cuando el CLI no está disponible, los agentes usan la heurística de >20 líneas de lógica de negocio como alternativa.

> **Impulsado por arborist-metrics:** el cálculo del factor de complejidad cognitiva y ciclomática se realiza mediante [`arborist-metrics`](https://github.com/StrangeDaysTech/arborist-metrics/) — nuestra librería Rust open source para métricas de código multi-lenguaje, desarrollada también por StrangeDaysTech S.A.S. de C.V. Disponible también de forma standalone en [crates.io](https://crates.io/crates/arborist-metrics).

---

### `straymark analyze declared-vs-wired [path] [--profile <nombre> | --declared-glob … --wired-glob … --declared-pattern … --wired-pattern …] [--show-orphans] [--output <formato>]` *(cli-3.18.0+)*

Marca símbolos declarados que **no tienen contraparte de cableado** del lado de la implementación — el anti-patrón *"declaración de superficie sin cableado"*, subclase 5 (método proxy IPC/RPC client-side vs interfaz del servidor). Cristalizado a partir de la validación N=2 de LNXDrive (hallazgos [#209](https://github.com/StrangeDaysTech/straymark/issues/209)/[#210](https://github.com/StrangeDaysTech/straymark/issues/210)); ver `.straymark/00-governance/POLISH-CHARTER-PATTERN.md`.

Es un **set-difference dirigido por config**, agnóstico de lenguaje/IPC por construcción: provees un lado *declarado* y un lado *cableado* como pares `(glob, regex)`; el **grupo de captura 1** de cada regex es el nombre del símbolo. El comando reporta **D \ W** (declarado pero no cableado) y, con `--show-orphans`, **W \ D** (cableado pero nunca declarado).

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|---------------|---------|-------------|
| `path` | `.` | Directorio objetivo |
| `--profile` | — | Perfil con nombre desde `.straymark/config.yml` (`declared_vs_wired.profiles`). Alternativa a los cuatro flags inline. |
| `--declared-glob` | — | Glob (relativo a `path`) de archivos con declaraciones (proxy/stub/cliente). |
| `--wired-glob` | — | Glob (relativo a `path`) de archivos con implementaciones (interfaz del daemon/servidor). |
| `--declared-pattern` | — | Regex sobre archivos declarados; **grupo de captura 1** = nombre del símbolo. |
| `--wired-pattern` | — | Regex sobre archivos cableados; **grupo de captura 1** = nombre del símbolo. |
| `--show-orphans` | off | Reporta también símbolos cableados pero nunca declarados (`W \ D`). |
| `--output` | `text` | `text`, `json`, o `markdown`. |

Debes pasar **o** `--profile` **o** los cuatro globs/patterns inline.

**Códigos de salida:** `0` limpio (cada símbolo declarado está cableado); `1` al menos un símbolo declarado sin contraparte de cableado (un hallazgo) — apto como compuerta de CI.

**Configuración** (opcional, en `.straymark/config.yml`):

```yaml
declared_vs_wired:
  profiles:
    - name: dbus
      declared_glob: "client/**/*.rs"     # el proxy D-Bus del cliente GTK
      declared_pattern: "fn (\\w+)"
      wired_glob: "daemon/src/interface.rs" # la interfaz implementada del daemon
      wired_pattern: "fn (\\w+)"
```

**Ejemplos:**

```bash
# Inline (puntual)
$ straymark analyze declared-vs-wired \
    --declared-glob "client/**/*.rs" --declared-pattern 'fn (\w+)' \
    --wired-glob "daemon/**/*.rs"    --wired-pattern 'fn (\w+)'

# Perfil con nombre (commiteado una vez), JSON para CI
$ straymark analyze declared-vs-wired --profile dbus --output json
```

> **Alcance v0:** esta es la verificación cross-stack mecánicamente tratable (subclase 5). Las variantes basadas en AST de las subclases 1–4 (docs de env-var, instrumentos métricos, embeds HTML, marcadores de ruta pública) y las verificaciones runtime dinámicas siguen siendo project-local — ver las Preguntas abiertas del doc del patrón.

---

### `straymark audit [path] [--from <fecha>] [--to <fecha>] [--system <nombre>] [--output <formato>]`

Genera reportes de trazas de auditoría con línea temporal, mapa de trazabilidad y resumen de cumplimiento.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--from` | — | Fecha de inicio del período (YYYY-MM-DD) |
| `--to` | — | Fecha de fin del período (YYYY-MM-DD) |
| `--system` | — | Filtrar por nombre de sistema/componente (busca en tags y título) |
| `--output` | `text` | Formato de salida: `text`, `markdown`, `json`, o `html` |

**El reporte incluye:**

- Línea temporal cronológica de todos los documentos
- Mapa de trazabilidad mostrando cadenas de relaciones (ej. REQ → ADR → AILOG → TES)
- Distribución de riesgo
- Resumen de cumplimiento (EU AI Act, ISO 42001, NIST AI RMF)

**Formatos de salida:**

| Formato | Caso de uso |
|---------|------------|
| `text` | Revisión en terminal (coloreado, formateado) |
| `markdown` | Incluir en PRs, wikis o reportes |
| `json` | Integración con herramientas externas |
| `html` | Reportes independientes con tablas estilizadas y gráfico SVG de riesgo |

---

### `straymark explore [path]`

Explora y lee la documentación de StrayMark interactivamente en una interfaz de terminal (TUI).

**Argumentos:**

| Argumento | Default | Descripción |
|-----------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |

**Flags:**

| Flag | Default | Descripción |
|------|---------|-------------|
| `--lang <código>` | resuelto desde el proyecto (ver abajo) | Idioma del shell del TUI y los docs de gobernanza del framework (`en`, `es`, `zh-CN`). Cae silenciosamente al inglés si falta la traducción. |

**Orden de resolución del idioma** (desde cli-3.5.2):

1. Flag `--lang <código>`, cuando se especifica
2. Campo `language` en `.straymark/config.yml`, cuando el archivo existe (un valor explícito — incluso `language: en` — se respeta como una decisión deliberada del usuario)
3. Variables de entorno `$LC_ALL` / `$LANG`, mapeadas a un idioma soportado (p.ej., `zh_CN.UTF-8` → `zh-CN`, `es_MX.UTF-8` → `es`). Chino tradicional (`zh_TW` / `zh_HK`) y otros locales no soportados pasan al siguiente fallback.
4. `en`

**Características:**

- Layout de dos paneles: árbol de navegación + visor de documentos
- Panel de metadatos con estado, confianza, riesgo, tags y enlaces relacionados
- Renderizado de Markdown con colores, tablas, bloques de código e indentación por niveles
- Navegación entre documentos relacionados mediante hipervínculos
- Búsqueda por nombre de archivo, título, tags o fecha
- Modo pantalla completa, con `j` / `k` como teclas alternas para `↓` / `↑`
- Consciente de localización: los docs del framework (`QUICK-REFERENCE`, `AGENT-RULES`, guías regulatorias de China, etc.) se sirven en el idioma definido por `language` en `.straymark/config.yml` o por `--lang`

**Atajos de teclado:**

| Tecla | Acción |
|-------|--------|
| `↑↓` / `j/k` | Navegar / Scroll |
| `Enter` | Expandir grupo / Abrir documento |
| `Tab` | Ciclar paneles: Navegación → Metadatos → Documento |
| `f` | Pantalla completa del documento |
| `/` | Buscar |
| `L` | Cambiar idioma de la interfaz (`en → es → zh-CN`) |
| `Esc` | Volver / Colapsar / Limpiar búsqueda |
| `?` | Popup de ayuda con todos los atajos |
| `q` | Salir |

**Ejemplos:**

```bash
$ straymark explore                       # usa config.language (default en)
$ straymark explore --lang zh-CN          # navegar docs del framework en chino simplificado
$ straymark explore --lang es             # override de sesión a español
```

> **Nota:** El comando `explore` requiere la feature `tui` (habilitada por defecto). Para compilar sin ella: `cargo build --no-default-features`.

---

### `straymark about`

Muestra información de versión, autoría y licencia.

**Ejemplo:**

```bash
$ straymark about
StrayMark CLI
  CLI version:       cli-3.5.2
  Framework version: fw-4.15.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/straymark
  Website:           https://strangedays.tech
```

---

### `straymark architecture <generate|sync|validate>` *(cli-3.25.0+, EXPERIMENTAL)*

Crea y mantén el **modelo de arquitectura** que alimenta la vista del Plan de Arquitectura de Loom (Spec 002): un `model.yml` semántico (componentes → globs de archivos + capas + enlaces) emparejado con un plano `plan.drawio`, vinculados por un `component_id` estable (la separación BIM "modelo vs dibujo"). La proyección textual sobre este modelo es `straymark status --where`; la superposición visual es Loom (A2).

> ⚠️ **EXPERIMENTAL (Loom v0).** El esquema del modelo, las convenciones de DrawIO y la superficie de comandos pueden cambiar sin ciclo de deprecación. Los tres subcomandos operan sobre `.straymark/architecture/` por defecto; `--out <dir>` lo sobrescribe.

**`straymark architecture generate [path] [--force] [--out <dir>]`** — escribe un primer borrador de `model.yml` + `plan.drawio` minando la estructura de tu base de código (un componente por directorio de fuentes, glob `dir/**`) enriquecido con señal de ADRs (los diagramas C4 Mermaid + las tablas "Affected Components" mejoran las etiquetas y añaden enlaces). El minador desciende por directorios *contenedores* (`internal/`, `src/`, `pkg/`, `lib/`, `app/`, `modules/`, …) para que un árbol como `internal/modules/<x>` se desglose en componentes reales en vez de un solo bloque, y omite el *andamiaje* de build (`src/main/java`, `src/main/kotlin`, …) para que un módulo Maven/Gradle se atribuya a su directorio de módulo en lugar de colapsar en un único cajón `main`. Los componentes aterrizan en una capa `unassigned` de relleno y las etapas 00–09 de `.straymark` siembran la lista de capas — luego refinas a mano (reasignar/renombrar capas, ajustar globs, añadir enlaces). `--force` sobrescribe los artefactos existentes.

> **Ajustar el escaneo a tu stack (`config.yml`).** El seed es consciente de lenguaje y estructura de fábrica, pero puedes extenderlo para ecosistemas no predeterminados mediante una sección `architecture:` en `.straymark/config.yml`. Las listas son **aditivas** — extienden los valores por defecto integrados, nunca los reemplazan:
>
> ```yaml
> architecture:
>   source_extensions:   [rb, ex, exs, gleam]   # añade lenguajes que el seed debe contar
>   container_dirs:      [services, domains]     # dirs extra por los que descender
>   scaffolding_prefixes: [src/main/java]        # andamiaje de build extra a omitir (defaults Maven/Gradle incluidos)
>   excluded_dirs:       [generated]             # dirs extra a omitir
> ```
>
> El overlay de estado en sí (matching glob → archivo) ya es agnóstico al lenguaje; esto solo da forma al seed generado.

**`straymark architecture sync [path] [--out <dir>] [--apply]`** — reconciliación **append-only** (solo-añadir): detecta nuevos directorios de fuentes de nivel superior / componentes de ADRs todavía no cubiertos por el modelo y los añade a `model.yml` + `plan.drawio`, sin **nunca** pisar las ediciones humanas ni la geometría de DrawIO. Dry-run por defecto; `--apply` escribe.

**`straymark architecture validate [path] [--out <dir>] [--output <text|json|markdown>]`** — reporta señales de integridad modelo↔plan: **undrawn** (componente sin celda de DrawIO), **unmodeled** (una celda de DrawIO ausente del modelo), **empty** (globs que no coinciden con ningún archivo en disco). **Sale con 1** cuando se encuentra cualquier señal (gestionable en CI). Degrada a solo-cobertura-de-globs cuando `plan.drawio` está ausente.

| Subcomando | Flags clave | ¿Escribe? |
|---|---|---|
| `generate` | `--force`, `--out` | Sí (rehúsa sobrescribir sin `--force`) |
| `sync` | `--apply`, `--out` | Solo con `--apply` (append-only) |
| `validate` | `--output`, `--out` | No (solo-lectura; sale con 1 ante cualquier señal) |

**Ejemplo:**

```bash
$ straymark architecture generate
✓ Wrote .straymark/architecture/model.yml (4 components, 9 layers)
✓ Wrote .straymark/architecture/plan.drawio
→ Mined 3 ADRs: 2 labels improved, 1 link added.
→ Refine by hand: reassign components from `unassigned` to real layers, then open the plan in DrawIO.

$ straymark architecture validate
✓ Architecture model is consistent (4 components).
```

---

### `straymark loom serve [path] [--port <puerto>] [--no-open]` *(cli-3.24.0+, EXPERIMENTAL)*

Lanza **Loom**, el servidor EXPERIMENTAL de visualización del grafo de conocimiento: un dashboard web solo-loopback y solo-lectura que renderiza los documentos StrayMark del proyecto como un grafo de fuerzas en vivo (nodos coloreados por tipo de documento, dimensionados por conectividad; seleccionar un nodo ilumina todo su hilo de relaciones; las ediciones a archivos `.md` vigilados actualizan el navegador abierto en menos de un segundo).

> ⚠️ **Loom es EXPERIMENTAL (v0).** Su API, superficie de CLI y su propia existencia pueden cambiar o eliminarse sin ciclo de deprecación. El binario `straymark-loom` **no** viene incluido en el CLI — se descarga bajo demanda de los releases `loom-*` de GitHub en el primer uso y se cachea en `~/.straymark/bin/`. La puerta de descarga *es* la frontera de opt-in.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `path` | `.` (directorio actual) | Directorio del proyecto. Loom vigila su subdirectorio `.straymark/` si existe; si no, el directorio mismo |
| `--port` | `7700` | Puerto en `127.0.0.1` donde servir |
| `--no-open` | off | No abrir el navegador automáticamente |

**Postura de seguridad:** liga exclusivamente a `127.0.0.1` (rehúsa arrancar de otro modo), rechaza headers `Host` no-loopback (anti DNS-rebinding) y nunca escribe en el directorio vigilado.

**Ejemplo:**

```bash
$ straymark loom serve

  ⚠  LOOM IS EXPERIMENTAL (v0)
     Unstable: API, CLI surface, and on-disk layout may change or be
     removed without a deprecation cycle. Loopback-only. Read-only.

ℹ Downloading Loom 0.4.2 (x86_64-unknown-linux-gnu) — first use is opt-in by download
✔ Loom 0.4.2 cached at ~/.straymark/bin/straymark-loom
loom: watching /project/.straymark (142 docs, 318 links)
loom: serving http://127.0.0.1:7700
```

---

## Skills

StrayMark incluye un conjunto de skills (slash commands) para usar dentro de un asistente IA (Claude Code, Antigravity CLI, Codex CLI, Qoder, Qwen Code, Cursor, runtimes de agente genérico). Cada skill se instala en 5 formas paralelas durante `straymark init`:

- `.claude/skills/<skill>/SKILL.md` (Claude — frontmatter con `allowed-tools`)
- `.codex/skills/<skill>/SKILL.md` *(fw-4.19.0+)* (Codex — frontmatter mínimo, solo `name`+`description`; generado desde la variante Claude)
- `.qoder/skills/<skill>/SKILL.md` (Qoder — mismo frontmatter completo que la variante Claude)
- `.qwen/skills/<skill>/SKILL.md` *(fw-4.41.0+)* (Qwen Code — mismo frontmatter completo que la variante Claude)
- `.agent/skills/<skill>/SKILL.md` *(fw-4.42.0+)* (Antigravity CLI `agy` — frontmatter mínimo, generado desde la variante Claude; `.agent/` es una de las raíces de customización de Antigravity)

Claude, Antigravity, Qoder y Qwen Code descubren los skills directamente del árbol del proyecto. **Codex es la excepción: lee los skills solo desde `~/.codex/skills/` (a nivel de usuario)** — ejecuta `straymark install-skills --agent codex` una vez después de `straymark init` (y después de cada actualización del framework) para poblar ese directorio desde `.codex/skills/`. Para Qoder y Qwen Code el comando equivalente (`--agent qoder`, `--agent qwen`) es opcional: copia los skills a `~/.qoder/skills/` o `~/.qwen/skills/` para tenerlos también fuera de este proyecto.

| Skill | Propósito | Archivos producidos |
|---|---|---|
| `/straymark-status` | Verificar cumplimiento de documentación para cambios recientes. | ninguno (read-only) |
| `/straymark-new` | Crear cualquier tipo de documento interactivamente. Sugiere el más adecuado al contexto. | `.straymark/<dir-tipo>/<TIPO>-YYYY-MM-DD-NNN-*.md` |
| `/straymark-ailog` | Atajo de creación rápida de AILOG. | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` |
| `/straymark-aidec` | Atajo de creación rápida de AIDEC. | `.straymark/07-ai-audit/decisions/AIDEC-*.md` |
| `/straymark-adr` | Atajo de creación rápida de ADR. | `.straymark/02-design/decisions/ADR-*.md` |
| `/straymark-mcard` | Flujo interactivo de creación de Model Card. | `.straymark/09-ai-models/MCARD-*.md` |
| `/straymark-sec` | Flujo interactivo SEC (security assessment). | `.straymark/08-security/SEC-*.md` |
| `/straymark-charter-new` *(fw-4.12.0+)* | Andamiar un Charter — unidad de trabajo declarativa ex-ante. Envuelve `straymark charter new` (derivación de slug, numeración secuencial, sustitución de plantilla); el skill conduce la selección de origen/esfuerzo y la disciplina de reconocimiento-antes-de-declarar. | `.straymark/charters/NN-slug.md` |
| `/straymark-followups` *(fw-4.22.0+)* | Mantener el registry de follow-ups (`AGENT-RULES.md §13`): al inicio de sesión responder "¿qué está pendiente?" desde el registry canónico, `followups drift --apply` pre-commit viajando en el mismo commit que el AILOG, triage post-cierre de Charter y `promote` aprobado por el operador. Wrapper delgado sobre `straymark followups` — nunca edita los counters CLI-owned. | ninguno directamente (las escrituras pasan por `straymark followups drift --apply` / `promote` → `.straymark/follow-ups-backlog.md`, TDE al promover) |
| `/straymark-audit-prompt CHARTER-ID` *(fw-4.9.0+, refactorizada en fw-4.9.0)* | Genera la plantilla unificada del audit prompt para un Charter en el path canónico. Envuelve `straymark charter audit --prepare`. El operador entonces abre N CLIs auditoras en el mismo repo e invoca `/straymark-audit-execute` en cada una — sin copy/paste. | `.straymark/audits/<CHARTER-ID>/audit-prompt.md` |
| `/straymark-audit-execute [CHARTER-ID]` *(fw-4.9.0+)* | **Corre dentro de una CLI auditora** (agy, claude-cli, copilot-cli, codex-cli, ...). Lee el prompt preparado del disco, audita con tool use citando `path:línea`, escribe un report con el id del modelo en el nombre. El argumento CHARTER-ID es opcional — auto-descubre prompts que aún no tienen report de este modelo. | `.straymark/audits/<CHARTER-ID>/report-<sluggified-model-id>.md` |
| `/straymark-audit-review CHARTER-ID` *(fw-4.9.0+, expandida en fw-4.9.0)* | Contraparte de `/straymark-audit-prompt`. Lee N reports en `.straymark/audits/<CHARTER-ID>/`, verifica cada finding contra el código real (Explore agents en paralelo), produce un `review.md` consolidado de seis secciones (Resumen ejecutivo, Alcance, Evaluación por auditor, Plan de remediación P0-P4, Hallazgos descartados, Calificación de auditores), y corre `straymark charter audit --merge-reports --merge-into` para anexar `external_audit:` en la telemetría del Charter. Si la telemetría aún no existe (Charter no cerrado), escribe `external-audit-pending.yaml` para merge posterior al close. | `.straymark/audits/<CHARTER-ID>/review.md`, array `external_audit:` mergeado en telemetría (o pending YAML) |

### Skill vs CLI

Las tres skills de auditoría son **wrappers** sobre los comandos del CLI y la disciplina del flujo. Los paths canónicos bajo `.straymark/audits/`, la plantilla unificada del prompt, la validación de schema, y el shape de `external_audit` viven en el CLI + framework — las skills manejan la parte UX-inline: dispatchan al operador a través del audit cycle sin gestión manual de archivos. **El operador nunca copia/pega prompts ni reports** — las skills intercambian artefactos vía los paths canónicos del filesystem.

Adoptantes que usen StrayMark sin asistente IA en el loop pueden manejar el mismo workflow directamente vía `straymark charter audit` (`--prepare` / `--merge-reports [--merge-into <path>]`). El audit prompt en `.straymark/audits/<id>/audit-prompt.md` funciona igualmente bien pegado en un LLM de chat si no hay CLI auditora disponible — la skill solo automatiza el intercambio de archivos.

### Audit checkpoint *(fw-4.9.0+)*

`.straymark/00-governance/AGENT-RULES.md` §12 codifica un checkpoint del workflow donde el agente proactivamente ofrece la auditoría en un momento específico — cuando la implementación del Charter está lista, drift está limpio, y `charter close` no se ha invocado aún. La recomendación es SÍ/NO basada en heurísticas (superficie de seguridad, componentes nuevos, riesgos AILOG, complejidad). La auditoría externa es **completamente opcional**; el checkpoint es **soft** — nunca bloquea `charter close`, nunca enforced (decisión de diseño v0+v1 permanente).

---

## Variables de Entorno

| Variable | Descripción |
|----------|-------------|
| `GITHUB_TOKEN` | Token de acceso personal de GitHub para solicitudes autenticadas a la API. Útil para evitar límites de tasa al descargar releases. |

---

## Códigos de Salida

| Código | Significado |
|--------|-------------|
| `0` | Éxito |
| `1` | Error (detalles impresos en stderr) |
