# StrayMark - Referencia CLI

**Referencia completa de la herramienta de línea de comandos `straymark`.**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)


---

## Tabla de Contenidos

1. [Instalación](#instalación)
2. [Versionado](#versionado)
3. [Comandos](#comandos) — init, update, remove, status, repair, validate, new, charter, compliance, metrics, analyze, audit, explore, about
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
| Framework | `fw-` | `fw-4.17.0` | Plantillas (12 tipos), docs de gobernanza, directivas |
| CLI | `cli-` | `cli-3.15.0` | El binario `straymark` |

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
4. Configura archivos de directivas de agentes IA (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
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

### `straymark validate [path] [--fix] [--staged] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

Valida documentos StrayMark verificando cumplimiento y corrección.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--fix` | — | Corregir automáticamente problemas simples |
| `--staged` | — | Validar solo archivos staged en Git (ideal para hooks pre-commit) |
| `--include-charters` | — | Validar también los Charters en `.straymark/charters/` contra el JSON Schema y la integridad referencial (los IDs en `originating_ailogs` resuelven; el path en `originating_spec` existe). Opt-in, default `false` para no afectar a proyectos que no usan el patrón. Por ahora solo se honra fuera de `--staged`; la validación de Charters en modo staged llega en cli-3.10.0. |
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
```

#### `straymark charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--no-batch-ledger-check] [--path <dir>]`

Detecta drift archivo-vs-commit al cierre del Charter. Envuelve el script del framework `.straymark/scripts/check-charter-drift.sh` (cero falsos positivos validados empíricamente en PLAN-05 retrospectivo + PLAN-06 prospectivo en Sentinel). El valor agregado del CLI sobre el script crudo es la **AILOG-awareness**: los paths reportados como "declarados pero no modificados" se silencian cuando aparecen en la sección `## Risk` / `## Riesgos` / `## 风险` de algún AILOG referenciado por `originating_ailogs` del Charter. Usa `--no-ailog-suppress` para deshabilitarlo.

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

Orquesta una revisión externa multi-modelo de la ejecución de un Charter. **Orchestration-only** — el CLI prepara la plantilla unificada del audit, valida los reports de auditores contra el schema, y emite/mergea el bloque YAML `external_audit`. **NO invoca APIs de LLM.** El operador corre N CLIs auditoras (gemini-cli, claude-cli, copilot-cli, codex-cli — la que tenga) configuradas con acceso read-only al filesystem; cada una invoca el skill `/straymark-audit-execute` para leer el prompt, auditar con tool use citando `path:line`, y escribir el report.

Dos pasos, cada uno invocable independientemente:

| Paso | Flag | Qué pasa |
|---|---|---|
| 1. PREPARE | `--prepare` (default) | Resuelve la plantilla unificada del audit contra el Charter + git diff + AILOGs origen. La escribe en `.straymark/audits/<CHARTER-ID>/audit-prompt.md`. |
| 2. MERGE-REPORTS | `--merge-reports` | Lee todos los archivos `report-*.md` en `.straymark/audits/<CHARTER-ID>/` (uno por auditor que terminó). Valida cada uno contra `audit-output.schema.v0.json`. Emite el array YAML `external_audit` — combina con `--merge-into <PATH>` para anexarlo directamente a la telemetría del Charter. |

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `<CHARTER-ID>` | — | Mismas reglas de resolución que `charter status`. |
| `--range` | `origin/main..HEAD` (con fallback a `origin/master..HEAD`, luego `HEAD~1..HEAD` con warning) | Rango git que los auditores revisan. El default captura el set completo de commits de la feature branch; el override explícito vía `--range <REV..REV>` no prueba upstream. |
| `--prepare` | off (acción default cuando ningún otro flag se pasa) | Corre el paso 1. Mutuamente excluyente con `--merge-reports`. |
| `--merge-reports` | off | Corre el paso 2. Mutuamente excluyente con `--prepare`. |
| `--merge-into <PATH>` | — | Con `--merge-reports`: anexa el array `external_audit:` directamente a la telemetría YAML en `<PATH>` en lugar de imprimir a stdout. El CLI rechaza re-audit (la telemetría ya tiene la clave) con error claro. |
| `--path` | `.` | Directorio del proyecto. |

**Flags v0 deprecated (ocultos en `--help`):**

- `--calibrate` — emite warning y sale con error. El paso v0 calibrate se reemplaza por la skill `/straymark-audit-review` que reconcilia N reports inline con acceso al filesystem (sin prompt paste-based separado).
- `--finalize` — alias deprecated de `--merge-reports` con comportamiento backwards-compat. Emite warning y rutea por la nueva ruta.

##### Recomendación de heterogeneidad (no enforced en v0)

El par de auditores debería ser de **familias de modelo distintas**: uno Anthropic + uno Google + uno OpenAI, en cualquier combinación, nunca dos de la misma familia. La heterogeneidad inter-familia es lo que hace que la convergencia en findings sea de alta señal — auditores de la misma familia comparten blind spots.

v1 soporta **N≥2 auditores** (ya no fijo a 2). El operador puede optar por 3 o 4 auditores para Charters de alto riesgo, incluyendo modelos especializados. La skill `/straymark-audit-review` itera sobre todos los archivos `report-*.md` en el audit dir.

El rol calibrador se mueve de una plantilla paste-based (v0) al agente principal in-conversation vía la skill `/straymark-audit-review` — su tarea es definicional (reconciliar veredictos ya producidos), por lo que la heterogeneidad respecto al implementador NO es requerida.

##### Layout canónico producido (v1)

```
.straymark/audits/CHARTER-NN/
├── audit-prompt.md                          # resuelto por --prepare (single unified prompt)
├── report-claude-sonnet-4-6.md              # escrito por /straymark-audit-execute en claude-cli
├── report-gemini-2-5-pro.md                 # escrito por /straymark-audit-execute en gemini-cli
├── report-gpt-5-3-codex.md                  # 3er auditor opcional
├── review.md                                # escrito por /straymark-audit-review (análisis consolidado de 6 secciones)
└── external-audit-pending.yaml              # escrito por /straymark-audit-review cuando la telemetría aún no existe (Branch B)
```

El directorio está namespaceado bajo `.straymark/` para evitar colisiones con carpetas `audit/` que el adoptante haya definido. El shape `<UNIT-TYPE>-<UNIT-ID>` deja espacio para futuras categorías de unidad de auditoría más allá de Charter (ej. `MODULE-payments/`, `RELEASE-v2.0/`) sin reestructurar.

Los adopters pueden `git add` el directorio entero `.straymark/audits/` para un audit trail completamente versionado, o `.gitignore` si prefieren un ciclo efímero.

**Ejemplo (v1, con los wrappers de skills — recomendado para flujos IDE-driven):**

```bash
# En el IDE principal (Claude Code, Gemini Code, Cursor, ...):
> /straymark-audit-prompt CHARTER-05
  → corre `straymark charter audit CHARTER-05 --prepare`
  → escribe .straymark/audits/CHARTER-05/audit-prompt.md
  → instruye al operador abrir CLIs auditoras

# En claude-cli (con acceso read al repo):
> /straymark-audit-execute CHARTER-05
  → escribe .straymark/audits/CHARTER-05/report-claude-sonnet-4-6.md
  → recuerda al operador esperar a TODAS las auditorías antes de review

# En gemini-cli:
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

## Skills

StrayMark incluye un conjunto de skills (slash commands) para usar dentro de un asistente IA (Claude Code, Gemini Code, Cursor, runtimes de agente genérico). Cada skill se instala en 3 formas paralelas durante `straymark init`:

- `dist/.claude/skills/<skill>/SKILL.md` (Claude — frontmatter con `allowed-tools`)
- `dist/.gemini/skills/<skill>/SKILL.md` (Gemini — frontmatter sin `allowed-tools`)
- `dist/.agent/workflows/<skill>.md` (agente genérico — frontmatter solo `description`)

| Skill | Propósito | Archivos producidos |
|---|---|---|
| `/straymark-status` | Verificar cumplimiento de documentación para cambios recientes. | ninguno (read-only) |
| `/straymark-new` | Crear cualquier tipo de documento interactivamente. Sugiere el más adecuado al contexto. | `.straymark/<dir-tipo>/<TIPO>-YYYY-MM-DD-NNN-*.md` |
| `/straymark-ailog` | Atajo de creación rápida de AILOG. | `.straymark/07-ai-audit/agent-logs/AILOG-*.md` |
| `/straymark-aidec` | Atajo de creación rápida de AIDEC. | `.straymark/07-ai-audit/decisions/AIDEC-*.md` |
| `/straymark-adr` | Atajo de creación rápida de ADR. | `.straymark/02-design/decisions/ADR-*.md` |
| `/straymark-mcard` | Flujo interactivo de creación de Model Card. | `.straymark/09-ai-models/MCARD-*.md` |
| `/straymark-sec` | Flujo interactivo SEC (security assessment). | `.straymark/08-security/SEC-*.md` |
| `/straymark-audit-prompt CHARTER-ID` *(fw-4.9.0+, refactorizada en fw-4.9.0)* | Genera la plantilla unificada del audit prompt para un Charter en el path canónico. Envuelve `straymark charter audit --prepare`. El operador entonces abre N CLIs auditoras en el mismo repo e invoca `/straymark-audit-execute` en cada una — sin copy/paste. | `.straymark/audits/<CHARTER-ID>/audit-prompt.md` |
| `/straymark-audit-execute [CHARTER-ID]` *(fw-4.9.0+)* | **Corre dentro de una CLI auditora** (gemini-cli, claude-cli, copilot-cli, codex-cli, ...). Lee el prompt preparado del disco, audita con tool use citando `path:línea`, escribe un report con el id del modelo en el nombre. El argumento CHARTER-ID es opcional — auto-descubre prompts que aún no tienen report de este modelo. | `.straymark/audits/<CHARTER-ID>/report-<sluggified-model-id>.md` |
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

---

<div align="center">

**StrayMark** — Porque cada cambio cuenta una historia.

[Volver a docs](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/es/README.md) • [README](https://github.com/StrangeDaysTech/straymark/blob/main/README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
