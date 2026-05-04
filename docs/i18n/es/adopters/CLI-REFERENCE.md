# DevTrail - Referencia CLI

**Referencia completa de la herramienta de línea de comandos `devtrail`.**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

**Idiomas**: [English](../../../adopters/CLI-REFERENCE.md) | Español | [简体中文](../../zh-CN/adopters/CLI-REFERENCE.md)

---

## Tabla de Contenidos

1. [Instalación](#instalación)
2. [Versionado](#versionado)
3. [Comandos](#comandos) — init, update, remove, status, repair, validate, new, charter, compliance, metrics, analyze, audit, explore, about
4. [Variables de Entorno](#variables-de-entorno)
5. [Códigos de Salida](#códigos-de-salida)

---

## Instalación

Instala el CLI de DevTrail usando uno de los métodos a continuación. Para instrucciones completas de configuración, consulta el [README](../README.md#inicio-rápido).

**Instalación rápida (binario precompilado):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

**Desde el código fuente:**

```bash
cargo install devtrail-cli
```

---

## Versionado

DevTrail usa **tags de versión independientes** para cada componente:

| Componente | Prefijo de tag | Ejemplo | Qué incluye |
|------------|---------------|---------|-------------|
| Framework | `fw-` | `fw-4.8.0` | Plantillas (12 tipos), docs de gobernanza, directivas |
| CLI | `cli-` | `cli-3.9.0` | El binario `devtrail` |

Framework y CLI se publican de forma independiente. Una actualización del framework no requiere actualización del CLI, y viceversa.

**Verificar versiones instaladas:**

```bash
devtrail about    # Muestra versión CLI + versión framework (si está instalado)
devtrail status   # Muestra estado completo de la instalación incluyendo versiones
```

---

## Comandos

### `devtrail init [path] [--hooks]`

Inicializa DevTrail en un directorio de proyecto.

**Argumentos y flags:**

| Argumento/Flag | Por defecto | Descripción |
|----------------|-------------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto destino |
| `--hooks` *(cli-3.7.0+)* | off | Tras `init`, instala el hook pre-PR del framework (`.devtrail/hooks/pre-pr.sh`) como `.git/hooks/pre-push`. Ejecuta `devtrail charter drift` automáticamente antes de cada push. Opt-in por principio #6 (disciplina cognitiva > productividad cruda). Se rehúsa a sobreescribir un `pre-push` ya existente; se omite silenciosamente si no es un repositorio git. |

**Qué hace:**

1. Descarga el último release del framework (`fw-*`) desde GitHub
2. Crea la estructura de directorios `.devtrail/`
3. Crea `DEVTRAIL.md` con las reglas de gobernanza
4. Configura archivos de directivas de agentes IA (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
5. Copia workflows de CI/CD
6. *(`--hooks`)* instala el hook pre-PR

**Ejemplo:**

```bash
$ devtrail init .
✔ Downloaded DevTrail fw-4.8.0
✔ Created .devtrail/ directory structure
✔ Created DEVTRAIL.md
✔ Configured AI agent directives
DevTrail initialized successfully!
Next: git add .devtrail/ DEVTRAIL.md && git commit -m "chore: adopt DevTrail"
```

---

### `devtrail update`

Actualiza **ambos** framework y CLI a sus últimas versiones. Equivale a ejecutar `update-framework` seguido de `update-cli`.

Si `.devtrail/` no existe en el directorio actual, la actualización del framework se omite con una advertencia.

**Ejemplo:**

```bash
$ devtrail update
Updating framework...
✔ Framework updated to fw-4.8.0
Updating CLI...
✔ CLI updated to cli-3.5.2
```

---

### `devtrail update-framework`

Actualiza solo los archivos del framework. Busca el último release `fw-*` en GitHub.

**Manejo de conflictos:** Si has modificado archivos del framework (ej. docs de gobernanza o plantillas), la actualización preserva tus cambios y reporta conflictos para resolución manual.

**Ejemplo:**

```bash
$ devtrail update-framework
✔ Framework updated to fw-4.8.0
```

---

### `devtrail update-cli`

Auto-actualiza el binario `devtrail`. Detecta automáticamente el método de instalación y usa el mecanismo de actualización apropiado:

- **Binario precompilado** (instalado via `install.sh` / `install.ps1`): Descarga el último binario de GitHub Releases
- **Cargo** (instalado via `cargo install`): Ejecuta `cargo install --force devtrail-cli`

Usa `--method` para forzar el método: `--method=github` o `--method=cargo`.

**Ejemplo:**

```bash
$ devtrail update-cli
✔ CLI updated to cli-3.5.2

$ devtrail update-cli --method=cargo
Compiling from source, this may take a few minutes...
✔ CLI updated to cli-3.5.2
```

---

### `devtrail remove [--full]`

Elimina DevTrail del proyecto actual.

**Flags:**

| Flag | Descripción |
|------|-------------|
| `--full` | Elimina todo, incluyendo documentos creados por el usuario en `.devtrail/`. Pide confirmación. |

**Comportamiento por defecto** (sin `--full`): elimina la estructura del framework pero preserva los documentos que creaste dentro de `.devtrail/`.

**Ejemplo:**

```bash
$ devtrail remove
✔ DevTrail framework removed. User documents preserved in .devtrail/.

$ devtrail remove --full
⚠ This will delete all DevTrail files including your documents.
Continue? [y/N]: y
✔ DevTrail completely removed.
```

---

### `devtrail status [path]`

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
$ devtrail status
DevTrail Status
───────────────
Path:              /home/user/my-project
Framework version: fw-4.8.0
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

### `devtrail repair [path]`

Repara una instalación de DevTrail rota restaurando directorios y archivos del framework faltantes.

**Argumentos:**

| Argumento | Default | Descripción |
|-----------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |

**Qué hace:**

1. Verifica directorios faltantes y los restaura con `.gitkeep`
2. Descarga el release del framework **una sola vez** si se necesitan archivos (templates, governance, config)
3. Re-inyecta directivas si falta `DEVTRAIL.md`
4. Recalcula checksums después de la reparación
5. Nunca modifica ni elimina documentos generados por el usuario

**Ejemplo:**

```bash
$ devtrail repair
Repairing DevTrail in /home/user/mi-proyecto
  → Found 1 issue(s) to repair
→ Restoring 1 missing directory...
✓ Restored .devtrail/templates/
→ Downloading framework to restore missing files...
✓ Restored 16 file(s) from framework

✓ DevTrail repaired successfully!
```

---

### `devtrail validate [path] [--fix] [--staged] [--include-charters] [--check-pending-reviews [--max-pending-days N]]`

Valida documentos DevTrail verificando cumplimiento y corrección.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--fix` | — | Corregir automáticamente problemas simples |
| `--staged` | — | Validar solo archivos staged en Git (ideal para hooks pre-commit) |
| `--include-charters` | — | Validar también los Charters en `docs/charters/` contra el JSON Schema y la integridad referencial (los IDs en `originating_ailogs` resuelven; el path en `originating_spec` existe). Opt-in, default `false` para no afectar a proyectos que no usan el patrón. Por ahora solo se honra fuera de `--staged`; la validación de Charters en modo staged llega en cli-3.9.0. |
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

### `devtrail new [path] [-t <tipo>] [--title <titulo>]`

Crea un nuevo documento DevTrail a partir de una plantilla.

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
$ devtrail new

# Crear un AILOG con título (no-interactivo)
$ devtrail new -t ailog --title "Implementar autenticación JWT"

# Crear un ADR
$ devtrail new --doc-type adr --title "Elegir PostgreSQL como base de datos"
```

**Ejemplo de salida:**

```
$ devtrail new -t ailog --title "Refactorizar módulo de pagos"

  ✔ Created: .devtrail/07-ai-audit/agent-logs/AILOG-2026-04-01-001-refactorizar-modulo-de-pagos.md

  Next steps:
    1. Edit the document to fill in details
    2. Commit: git add .devtrail/07-ai-audit/agent-logs/AILOG-2026-04-01-001-refactorizar-modulo-de-pagos.md
```

---

### `devtrail approve <doc-id> --outcome <outcome> --reviewer <id> [--at YYYY-MM-DD] [--notes "..."] [--path <dir>]`

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
$ devtrail approve AIDEC-2026-05-02-001 \
    --outcome approved \
    --reviewer pepe@example.com \
    --notes "Revisado contra ADR-007. LGTM."

  ✔ AIDEC-2026-05-02-001 marked as approved.
    Reviewer: pepe@example.com
    Date:     2026-05-02
    File:     .devtrail/07-ai-audit/decisions/AIDEC-2026-05-02-001-foo.md

# Ciclo iterativo: revisions_requested → re-aprobación
$ devtrail approve AIDEC-... --outcome revisions_requested --reviewer reviewer@x.io
# (autor itera)
$ devtrail approve AIDEC-... --outcome approved --reviewer reviewer@x.io
# Frontmatter muestra la última (approved); el cuerpo conserva AMBOS bloques cronológicamente.

# Visibilidad del backlog
$ devtrail validate --check-pending-reviews --max-pending-days 14
```

> Ver `dist/.devtrail/00-governance/DOCUMENTATION-POLICY.md` §3.5 "Recording Approval" para la definición canónica del flujo (semántica de cierre, formato del cuerpo, convención multi-revisor).

---

### `devtrail charter <subcomando>`

Gestiona **Charters**: unidades acotadas y auditables de trabajo, declaradas ex-ante y validadas ex-post. Un Charter empareja scope declarativo (archivos a tocar, riesgos, comandos de verificación ejecutables) con el ancla de auditoría ex-post (drift detection, auditoría multi-modelo). Los Charters viven en `docs/charters/NN-slug.md` (a nivel del project root, **no** bajo `.devtrail/`).

> **Nota histórica.** En el experimento Sentinel `/plan-audit` que cristalizó este patrón (abril 2026, 6 ciclos), los Charters se llamaban *Plans*. El CLI DevTrail usa **Charter** going-forward para evitar la colisión nominal con el `plan.md` de GitHub SpecKit. Los archivos históricos de Sentinel preservan "Plan" deliberadamente. El alcance conceptual completo y la justificación del rename viven en `Propuesta/que-es-un-charter.md`.

**Subcomandos:**

- `devtrail charter new` — crea un nuevo Charter desde el template del framework
- `devtrail charter list` — enumera Charters con filtros opcionales
- `devtrail charter status` — muestra detalle de un Charter, o los 5 más recientes
- `devtrail charter close` *(cli-3.7.0+)* — registra la telemetría post-ejecución y mueve el Charter a `closed`
- `devtrail charter drift` *(cli-3.7.0+)* — chequea drift archivo-vs-commit con supresión AILOG-aware
- `devtrail charter audit` *(cli-3.8.0+)* — orquesta una revisión externa multi-modelo (orchestration-only, ver más abajo)

#### `devtrail charter new [-t XS|S|M|L] [--from-ailog <id> | --from-spec <path>] [--title <titulo>] [path]`

Crea un Charter desde el template del framework en `docs/charters/NN-slug.md`. Si no se pasa `--title`, se solicita interactivamente. Los dos flags de origen son mutuamente excluyentes a nivel de clap.

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
$ devtrail charter new --type M

# Modo mantenimiento / post-MVP — Charter rooteado en un AILOG existente
$ devtrail charter new -t S --from-ailog AILOG-2026-04-28-021 --title "thresholds por servicio"

# Modo greenfield — Charter implementando un spec de SpecKit
$ devtrail charter new -t L --from-spec specs/001-pagos/spec.md --title "integrar provider de pagos"
```

#### `devtrail charter list [--status declared|in-progress|closed|all] [--origin ailog|spec|any] [path]`

Enumera Charters como tabla.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` | Directorio del proyecto |
| `--status` | `all` | Filtra por status del ciclo de vida |
| `--origin` | `any` (sin filtro) | Filtra por tipo de origen: `ailog`, `spec`, o `any` |

Los archivos que no parsean se reportan como warnings en stderr sin abortar el comando — la tabla muestra lo que puede.

#### `devtrail charter status [CHARTER-ID] [--path <dir>]`

Con un ID: imprime el detalle completo del Charter (frontmatter, ubicación del archivo, lista de secciones del cuerpo). Sin ID: imprime los 5 Charters más recientes por NN descendente.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `CHARTER-ID` | — | Identificador del Charter. Acepta el `charter_id` completo (`CHARTER-01-test`), el prefijo `CHARTER-NN` (`CHARTER-01`), o solo el NN numérico (`01` o `1`). El match numérico es permisivo respecto al zero-padding. |
| `--path` | `.` | Directorio del proyecto. Es flag (no positional) para evitar colisión con el positional opcional `CHARTER-ID`. |

#### `devtrail charter close <CHARTER-ID> [--from-template] [--non-interactive] [--path <dir>]`

Registra la telemetría post-ejecución y mueve el status del Charter a `closed`. La telemetría se escribe en `.devtrail/charters/CHARTER-NN.telemetry.yaml` (archivo lateral, **no** embebido en el frontmatter del Charter — el frontmatter es declarativo ex-ante; la telemetría es voluminosa ex-post). El shape se valida contra `.devtrail/schemas/charter-telemetry.schema.v0.json`.

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
$ devtrail charter close CHARTER-01

  Closing CHARTER-01-test-charter
    Title: Test charter
  Press Enter to accept defaults; type to override.

  ── Trigger ──
  Declared trigger kind › event_trigger
  Declared trigger description › first false-positive ticket
  Fired at (YYYY-MM-DD) [2026-05-02]:
  ...

  ✔ Charter CHARTER-01 closed.
    Telemetry: .devtrail/charters/CHARTER-01.telemetry.yaml
    Status updated: in-progress/declared → closed
```

#### `devtrail charter drift <CHARTER-ID> [--range <REV..REV>] [--no-ailog-suppress] [--path <dir>]`

Detecta drift archivo-vs-commit al cierre del Charter. Envuelve el script del framework `.devtrail/scripts/check-charter-drift.sh` (cero falsos positivos validados empíricamente en PLAN-05 retrospectivo + PLAN-06 prospectivo en Sentinel). El valor agregado del CLI sobre el script crudo es la **AILOG-awareness**: los paths reportados como "declarados pero no modificados" se silencian cuando aparecen en la sección `## Risk` / `## Riesgos` / `## 风险` de algún AILOG referenciado por `originating_ailogs` del Charter. Usa `--no-ailog-suppress` para deshabilitarlo.

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `CHARTER-ID` | — | Mismas reglas de resolución que `charter status`. |
| `--range` | `HEAD~1..HEAD` | Rango de revisiones git a chequear. |
| `--no-ailog-suppress` *(cli-3.9.0+ siempre emite una línea INFO de confirmación)* | false | Deshabilita la supresión AILOG-aware (muestra todo path declarado-omitido). Cuando se pasa el flag, el CLI siempre imprime una línea `INFO: AILOG-aware suppression bypassed (would have suppressed: N path(s)…)` — incluso cuando N=0 — para que el modo diagnóstico sea visible en la salida aun en una corrida limpia. |
| `--path` | `.` | Directorio del proyecto. |

**Códigos de salida:** `0` si no hay drift (o solo AILOG-suprimido); `1` si hay drift no contabilizado; `2` para errores de uso (Charter no encontrado, bash ausente, etc.).

**Ejemplo:**

```bash
$ devtrail charter drift CHARTER-01 --range origin/main..HEAD
=== Charter drift check ===
  Charter: docs/charters/01-test.md
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

#### Soporte de wildcards en paths declarados *(fw-4.8.0+)*

El chequeo de drift resuelve dos formas de wildcard en `## Files to modify`:

| Forma | Ejemplo | Caso de uso |
|---|---|---|
| Elipsis | `` `.devtrail/07-ai-audit/agent-logs/AILOG-...md` `` | Cualquier path modificado con ese prefijo satisface el wildcard. Usado históricamente cuando un número desconocido de AILOGs serían creados durante la ejecución. |
| Glob | `` `AILOG-*.md` `` o `` `src/services/foo-*.rs` `` | Cualquier path modificado que matchee el glob (`*` → `.*`) satisface el wildcard. Usado para declaraciones bulk de Charter donde un set parametrizado es tocado. Añadido en fw-4.8.0 tras la fricción surgida en Sentinel CHARTER-04 ([issue #81](https://github.com/StrangeDaysTech/devtrail/issues/81)). |

Ambas formas se manejan en ambas direcciones: un wildcard declarado suprime tanto warnings de "declarado pero no modificado" (cuando al menos un archivo matching fue modificado) como warnings de "modificado pero no declarado" (cuando un path modificado matchea un wildcard declarado).

#### Por diseño: rutas de gobernanza siempre están en scope

Los paths bajo `docs/charters/*` y `.devtrail/07-ai-audit/*` **nunca** se reportan como "modificado pero no declarado". Es opinionated por diseño — esos paths son siempre legítimos cuando el Charter mismo o el AILOG de ejecución son tocados. Validado empíricamente en Sentinel CHARTER-04: un `git add -A` accidental stageó archivos no-tracked del usuario (`.claude/skills/`, `cmd/sentinel/sentinel`); la regla suprimió correctamente el ruido de gobernanza sin esconder la expansión genuina de archivos del proyecto ([issue #81 W2](https://github.com/StrangeDaysTech/devtrail/issues/81#issuecomment-update)).

Si corres un Charter cuyo scope explícito es churn de gobernanza (p.ej. un Charter de aprobación bulk que toca solo `.devtrail/07-ai-audit/`), el chequeo reportará 0 archivos modificados y necesitarás verificar el scope leyendo el AILOG. Un flag `--strict-scope` que deshabilite la regla "siempre en scope" está sobre la mesa para una minor futura si un adopter real reporta la asimetría como fricción.

#### `devtrail charter audit <CHARTER-ID> [--range <REV..REV>] [--calibrate | --finalize] [--path <dir>]`

*Disponible desde **cli-3.8.0** + **fw-4.7.0** (Fase 3 v0).*

Orquesta una revisión externa multi-modelo de la ejecución de un Charter. **Orchestration-only** — el CLI prepara prompts, valida outputs contra el schema, e imprime findings listos para pegar en la telemetría del Charter. **NO invoca APIs de LLM.** El operador corre los prompts en su auditor de elección (Copilot, Gemini, Claude, etc.) y guarda las respuestas en paths canónicos.

Tres pasos, cada uno invocable independientemente:

| Paso | Flag | Qué pasa |
|---|---|---|
| 1. PREPARE | (default) | Resuelve los prompts `auditor-primary` y `auditor-secondary` contra el Charter + git diff + AILOGs originadores. Los escribe bajo `audit/charters/<CHARTER-ID>/prompts/`. |
| 2. CALIBRATE | `--calibrate` | Lee `auditor-primary.md` y `auditor-secondary.md` (el operador debe guardarlos entre pasos 1 y 2). Los valida contra `audit-output.schema.v0.json`. Resuelve el prompt del calibrador con ambas respuestas embebidas. |
| 3. FINALIZE | `--finalize` | Lee la respuesta del calibrador. Valida los 3 outputs. Imprime un bloque YAML `external_audit` listo para pegar en la telemetría del Charter. |

| Argumento/Flag | Default | Descripción |
|---|---|---|
| `<CHARTER-ID>` | — | Mismas reglas de resolución que `charter status`. |
| `--range` | `HEAD~1..HEAD` | Rango git que los auditores revisarán. |
| `--calibrate` | off | Corre el paso 2. Mutuamente excluyente con `--finalize`. |
| `--finalize` | off | Corre el paso 3. Mutuamente excluyente con `--calibrate`. |
| `--path` | `.` | Directorio del proyecto. |

##### Recomendación de heterogeneidad (no enforced en v0)

Por la justificación de diseño (`devtrail-cli-roadmap.md` §5.2), el par de auditores debería ser de **familias de modelo distintas**: uno Anthropic + uno Google + uno OpenAI, en cualquier combinación, nunca dos de la misma familia. La heterogeneidad inter-familia es lo que hace que la convergencia en findings sea de alta señal — auditores de la misma familia comparten blind spots.

El calibrador-reconciliador PUEDE ser de cualquier familia (incluida la del implementador) porque su tarea es definicional (aplicar el schema sobre veredictos ya producidos), no de descubrimiento. La heterogeneidad importa para el par auditor, no para el calibrador.

v0 documenta esta recomendación pero no la auto-detecta ni enforza. Un flag `--implementer-family X` con rechazo de configuraciones monocromáticas es candidato v1 cuando un adopter reporte un caso real.

##### Layout producido

```
audit/charters/CHARTER-NN/
├── prompts/
│   ├── auditor-primary.prompt.md      # resuelto por el paso 1, lo que se envió
│   ├── auditor-secondary.prompt.md    # resuelto por el paso 1
│   └── calibrator-reconciler.prompt.md  # resuelto por el paso 2
├── auditor-primary.md                 # el operador pega la respuesta del auditor 1
├── auditor-secondary.md               # el operador pega la respuesta del auditor 2
└── calibrator-reconciler.md           # el operador pega la respuesta del calibrador
```

El subdirectorio `prompts/` persiste lo que se envió a cada auditor *antes* de la API call (cierra [RFC #82](https://github.com/StrangeDaysTech/devtrail/issues/82) sobre visibilidad de auditoría). Los adopters pueden `git add` el directorio entero `audit/` para un audit trail completamente versionado, o `.gitignore` si prefieren un ciclo efímero.

**Ejemplo:**

```bash
$ devtrail charter audit CHARTER-05
  Step 1/3: PREPARE (CHARTER-05)
  ✔ Wrote audit/charters/CHARTER-05/prompts/auditor-primary.prompt.md
  ✔ Wrote audit/charters/CHARTER-05/prompts/auditor-secondary.prompt.md

  Next:
    1. Paste each prompt into your auditor of choice (use a model
       of a different family per auditor — see CLI-REFERENCE).
    2. Save the auditor responses to:
         audit/charters/CHARTER-05/auditor-primary.md
         audit/charters/CHARTER-05/auditor-secondary.md
    3. Run: devtrail charter audit CHARTER-05 --calibrate

# (el operador corre auditor 1 en Copilot, guarda respuesta. Corre auditor 2
# en Gemini, guarda respuesta.)

$ devtrail charter audit CHARTER-05 --calibrate
  Step 2/3: CALIBRATE (CHARTER-05)
  ✔ Validated audit/charters/CHARTER-05/auditor-primary.md
  ✔ Validated audit/charters/CHARTER-05/auditor-secondary.md
  ✔ Wrote audit/charters/CHARTER-05/prompts/calibrator-reconciler.prompt.md

  Next:
    1. Run the calibrator prompt in a model of your choice (calibrator
       may be of any family).
    2. Save the response to: audit/charters/CHARTER-05/calibrator-reconciler.md
    3. Run: devtrail charter audit CHARTER-05 --finalize

# (el operador corre el calibrador en Claude, guarda respuesta.)

$ devtrail charter audit CHARTER-05 --finalize
  Step 3/3: FINALIZE (CHARTER-05)
  ✔ Validated audit/charters/CHARTER-05/auditor-primary.md (5 findings)
  ✔ Validated audit/charters/CHARTER-05/auditor-secondary.md (4 findings)
  ✔ Validated audit/charters/CHARTER-05/calibrator-reconciler.md

  Charter audit complete.

  external_audit YAML — paste into telemetry:
    - auditor: "copilot-v1.0.37"
      findings_total: 5
      findings_by_category:
        hallucination: 0
        implementation_gap: 2
        real_debt: 2
        false_positive: 1
      audit_quality: "high"
      audit_notes: "see audit/charters/<charter-id>/auditor-primary.md"
    - auditor: "gemini-cli-v1.5"
      findings_total: 4
      findings_by_category: ...

  Calibrator summary (copy to outcome.scope_change_notes if relevant):
    audit/charters/CHARTER-05/calibrator-reconciler.md
```

> **¿Por qué orchestration-only?** Implementar 3 HTTP clients (OpenAI / Google / Anthropic) son 1-2 semanas + mantenimiento perpetuo cuando las APIs cambian. La Fase 3 v0 es experimental — el valor del CLI es el canon (forma del prompt + schema de output + integración con telemetría), no la API call. v1 puede agregar HTTP clients cuando un adopter reporte una necesidad real; hasta entonces el patrón humano-en-el-loop coincide con el `/plan-audit` empírico de Sentinel que motivó la Fase 3.

> **Alternativa con skill *(fw-4.8.0+)*.** Cuando trabajas con un asistente IA en el loop (Claude Code, Gemini Code, Cursor, etc.), las skills `/devtrail-audit-prompt CHARTER-ID` y `/devtrail-audit-review CHARTER-ID` envuelven este comando y muestran los prompts inline en la conversación. Las skills también manejan el paso del calibrador (el agente que conduce la conversación corre el calibrador) y disparan `--finalize --merge-into` para que el array `external_audit:` se anexe a la telemetría sin copy-paste manual. Ver la sección [Skills](#skills) más abajo. El CLI sigue siendo la fuente única de verdad — las skills solo agregan UX-inline.

---

### `devtrail compliance [path] [--standard <nombre>] [--region <nombre>] [--all] [--output <formato>]`

Verifica cumplimiento regulatorio. Por defecto evalúa los estándares cuya región esté incluida en `regional_scope` de `.devtrail/config.yml` (default `[global, eu]`). Seis frameworks chinos disponibles opt-in cuando `china` se añade a `regional_scope`.

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
$ devtrail compliance

# Los seis frameworks chinos (requiere regional_scope: china)
$ devtrail compliance --region china

# Un solo framework chino
$ devtrail compliance --standard china-pipl --output json

# Todos los estándares ignorando regional_scope
$ devtrail compliance --all
```

> **Activación**: para evaluar los frameworks chinos automáticamente, añadir a `.devtrail/config.yml`:
>
> ```yaml
> regional_scope:
>   - global
>   - eu
>   - china
> ```

---

### `devtrail metrics [path] [--period <periodo>] [--output <formato>]`

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

### `devtrail analyze [path] [--threshold <N>] [--output <formato>] [--top <N>]`

Analiza la complejidad del código fuente usando métricas cognitivas y ciclomáticas, impulsado por [arborist-metrics](https://crates.io/crates/arborist-metrics).

**Argumentos y flags:**

| Argumento/Flag | Predeterminado | Descripción |
|----------------|----------------|-------------|
| `path` | `.` (directorio actual) | Directorio a analizar |
| `--threshold` | `8` (o desde config) | Umbral de complejidad cognitiva |
| `--output` | `text` | Formato de salida: `text`, `json` o `markdown` |
| `--top` | — | Mostrar solo las N funciones más complejas |

**Lenguajes soportados:** Rust, Python, JavaScript, TypeScript, Java, Go, C, C++, C#, PHP, Kotlin, Swift

**Resolución de umbral:** flag CLI → `.devtrail/config.yml` → predeterminado (8)

**Configuración** (opcional, en `.devtrail/config.yml`):

```yaml
complexity:
  threshold: 8
```

**Ejemplos:**

```bash
# Analizar directorio actual
$ devtrail analyze

# Umbral personalizado y top 10
$ devtrail analyze --threshold 5 --top 10

# Salida JSON para integración CI
$ devtrail analyze --output json

# Analizar un proyecto específico
$ devtrail analyze /ruta/al/proyecto
```

**Ejemplo de salida:**

```
  DevTrail Analyze
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

> **Nota:** Este comando funciona sin `devtrail init`. Opera sobre archivos fuente, no documentos DevTrail. La feature `analyze` se puede desactivar en compilación con `--no-default-features`.

> **Trigger de documentación:** Los agentes de IA usan `devtrail analyze --output json` como método primario para determinar cuándo crear documentos AILOG. Si `summary.above_threshold > 0` en la salida JSON, el agente debe crear un AILOG. Cuando el CLI no está disponible, los agentes usan la heurística de >20 líneas de lógica de negocio como alternativa.

---

### `devtrail audit [path] [--from <fecha>] [--to <fecha>] [--system <nombre>] [--output <formato>]`

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

### `devtrail explore [path]`

Explora y lee la documentación de DevTrail interactivamente en una interfaz de terminal (TUI).

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
2. Campo `language` en `.devtrail/config.yml`, cuando el archivo existe (un valor explícito — incluso `language: en` — se respeta como una decisión deliberada del usuario)
3. Variables de entorno `$LC_ALL` / `$LANG`, mapeadas a un idioma soportado (p.ej., `zh_CN.UTF-8` → `zh-CN`, `es_MX.UTF-8` → `es`). Chino tradicional (`zh_TW` / `zh_HK`) y otros locales no soportados pasan al siguiente fallback.
4. `en`

**Características:**

- Layout de dos paneles: árbol de navegación + visor de documentos
- Panel de metadatos con estado, confianza, riesgo, tags y enlaces relacionados
- Renderizado de Markdown con colores, tablas, bloques de código e indentación por niveles
- Navegación entre documentos relacionados mediante hipervínculos
- Búsqueda por nombre de archivo, título, tags o fecha
- Modo pantalla completa, con `j` / `k` como teclas alternas para `↓` / `↑`
- Consciente de localización: los docs del framework (`QUICK-REFERENCE`, `AGENT-RULES`, guías regulatorias de China, etc.) se sirven en el idioma definido por `language` en `.devtrail/config.yml` o por `--lang`

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
$ devtrail explore                       # usa config.language (default en)
$ devtrail explore --lang zh-CN          # navegar docs del framework en chino simplificado
$ devtrail explore --lang es             # override de sesión a español
```

> **Nota:** El comando `explore` requiere la feature `tui` (habilitada por defecto). Para compilar sin ella: `cargo build --no-default-features`.

---

### `devtrail about`

Muestra información de versión, autoría y licencia.

**Ejemplo:**

```bash
$ devtrail about
DevTrail CLI
  CLI version:       cli-3.5.2
  Framework version: fw-4.8.0
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/devtrail
  Website:           https://strangedays.tech
```

---

## Skills

DevTrail incluye un conjunto de skills (slash commands) para usar dentro de un asistente IA (Claude Code, Gemini Code, Cursor, runtimes de agente genérico). Cada skill se instala en 3 formas paralelas durante `devtrail init`:

- `dist/.claude/skills/<skill>/SKILL.md` (Claude — frontmatter con `allowed-tools`)
- `dist/.gemini/skills/<skill>/SKILL.md` (Gemini — frontmatter sin `allowed-tools`)
- `dist/.agent/workflows/<skill>.md` (agente genérico — frontmatter solo `description`)

| Skill | Propósito | Archivos producidos |
|---|---|---|
| `/devtrail-status` | Verificar cumplimiento de documentación para cambios recientes. | ninguno (read-only) |
| `/devtrail-new` | Crear cualquier tipo de documento interactivamente. Sugiere el más adecuado al contexto. | `.devtrail/<dir-tipo>/<TIPO>-YYYY-MM-DD-NNN-*.md` |
| `/devtrail-ailog` | Atajo de creación rápida de AILOG. | `.devtrail/07-ai-audit/agent-logs/AILOG-*.md` |
| `/devtrail-aidec` | Atajo de creación rápida de AIDEC. | `.devtrail/07-ai-audit/decisions/AIDEC-*.md` |
| `/devtrail-adr` | Atajo de creación rápida de ADR. | `.devtrail/04-architecture/decisions/ADR-*.md` |
| `/devtrail-mcard` | Flujo interactivo de creación de Model Card. | `.devtrail/09-ai-models/MCARD-*.md` |
| `/devtrail-sec` | Flujo interactivo SEC (security assessment). | `.devtrail/08-security/SEC-*.md` |
| `/devtrail-audit-prompt CHARTER-ID` *(fw-4.8.0+)* | Genera prompts de auditoría externa multi-modelo inline. Envuelve `devtrail charter audit` PREPARE — corre el CLI para resolver `auditor-primary.prompt.md` y `auditor-secondary.prompt.md`, y muestra ambos prompts en la conversación para que el operador los pegue en 2 LLMs de familias distintas sin salir del chat. | `audit/charters/<CHARTER-ID>/prompts/auditor-{primary,secondary}.prompt.md` (vía el CLI que envuelve) |
| `/devtrail-audit-review CHARTER-ID` *(fw-4.8.0+)* | Contraparte de `/devtrail-audit-prompt`. Valida las respuestas de auditores guardadas por el operador, corre el calibrador inline (el agente que conduce la conversación ES un calibrador válido porque la heterogeneidad solo es requisito para el par auditor), y ejecuta `devtrail charter audit --finalize --merge-into` para anexar `external_audit:` directamente en `.devtrail/charters/<CHARTER-ID>.telemetry.yaml`. Si la telemetría no existe (Charter no cerrado aún), escribe `audit/charters/<CHARTER-ID>/external-audit-pending.yaml` para merge manual posterior. | `audit/charters/<CHARTER-ID>/calibrator-reconciler.md`, array `external_audit:` mergeado en telemetría |

### Skill vs CLI

Las dos skills de auditoría son **wrappers** sobre los comandos del CLI. El layout del directorio `audit/`, los prompts, la validación de schema, y el shape de `external_audit` viven en el CLI — las skills solo manejan la parte UX-inline (mostrar prompts en la conversación, correr el calibrador inline, disparar el merge). Adoptantes que usen DevTrail sin asistente IA en el loop pueden manejar el mismo workflow directamente vía `devtrail charter audit` (PREPARE / `--calibrate` / `--finalize [--merge-into <path>]`).

### Audit checkpoint *(fw-4.8.0+)*

`.devtrail/00-governance/AGENT-RULES.md` §12 codifica un checkpoint del workflow donde el agente proactivamente ofrece la auditoría en un momento específico — cuando la implementación del Charter está lista, drift está limpio, y `charter close` no se ha invocado aún. La recomendación es SÍ/NO basada en heurísticas (superficie de seguridad, componentes nuevos, riesgos AILOG, complejidad). La auditoría externa es **completamente opcional**; el checkpoint es **soft** — nunca bloquea `charter close`, nunca enforced (decisión de diseño v0+v1 permanente).

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

**DevTrail** — Porque cada cambio cuenta una historia.

[Volver a docs](../../README.md) • [README](../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
