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
| Framework | `fw-` | `fw-4.4.1` | Plantillas (12 tipos), docs de gobernanza, directivas |
| CLI | `cli-` | `cli-3.6.0` | El binario `devtrail` |

Framework y CLI se publican de forma independiente. Una actualización del framework no requiere actualización del CLI, y viceversa.

**Verificar versiones instaladas:**

```bash
devtrail about    # Muestra versión CLI + versión framework (si está instalado)
devtrail status   # Muestra estado completo de la instalación incluyendo versiones
```

---

## Comandos

### `devtrail init [path]`

Inicializa DevTrail en un directorio de proyecto.

**Argumentos:**

| Argumento | Por defecto | Descripción |
|-----------|-------------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto destino |

**Qué hace:**

1. Descarga el último release del framework (`fw-*`) desde GitHub
2. Crea la estructura de directorios `.devtrail/`
3. Crea `DEVTRAIL.md` con las reglas de gobernanza
4. Configura archivos de directivas de agentes IA (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
5. Copia workflows de CI/CD

**Ejemplo:**

```bash
$ devtrail init .
✔ Downloaded DevTrail fw-4.4.1
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
✔ Framework updated to fw-4.4.1
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
✔ Framework updated to fw-4.4.1
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
Framework version: fw-4.4.1
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

### `devtrail validate [path] [--fix] [--staged] [--include-charters]`

Valida documentos DevTrail verificando cumplimiento y corrección.

**Argumentos y flags:**

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `path` | `.` (directorio actual) | Directorio del proyecto |
| `--fix` | — | Corregir automáticamente problemas simples |
| `--staged` | — | Validar solo archivos staged en Git (ideal para hooks pre-commit) |
| `--include-charters` | — | Validar también los Charters en `docs/charters/` contra el JSON Schema y la integridad referencial (los IDs en `originating_ailogs` resuelven; el path en `originating_spec` existe). Opt-in, default `false` para no afectar a proyectos que no usan el patrón. Por ahora solo se honra fuera de `--staged`; la validación de Charters en modo staged llega en cli-3.7.0. |

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

### `devtrail charter <subcomando>`

Gestiona **Charters**: unidades acotadas y auditables de trabajo, declaradas ex-ante y validadas ex-post. Un Charter empareja scope declarativo (archivos a tocar, riesgos, comandos de verificación ejecutables) con el ancla de auditoría ex-post (drift detection, auditoría multi-modelo). Los Charters viven en `docs/charters/NN-slug.md` (a nivel del project root, **no** bajo `.devtrail/`).

> **Nota histórica.** En el experimento Sentinel `/plan-audit` que cristalizó este patrón (abril 2026, 6 ciclos), los Charters se llamaban *Plans*. El CLI DevTrail usa **Charter** going-forward para evitar la colisión nominal con el `plan.md` de GitHub SpecKit. Los archivos históricos de Sentinel preservan "Plan" deliberadamente. El alcance conceptual completo y la justificación del rename viven en `Propuesta/que-es-un-charter.md`.

**Subcomandos:**

- `devtrail charter new` — crea un nuevo Charter desde el template del framework
- `devtrail charter list` — enumera Charters con filtros opcionales
- `devtrail charter status` — muestra detalle de un Charter, o los 5 más recientes

La Fase 2 del CLI roadmap añadirá `charter close` (telemetría interactiva) y `charter drift` (chequeo de drift archivo-vs-commit). La Fase 3 añadirá `charter audit` (auditoría externa multi-modelo).

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

Con un ID: imprime el detalle completo del Charter (frontmatter, ubicación del archivo, lista de secciones del cuerpo, placeholders de Fase 2). Sin ID: imprime los 5 Charters más recientes por NN descendente.

| Argumento/Flag | Default | Descripción |
|----------------|---------|-------------|
| `CHARTER-ID` | — | Identificador del Charter. Acepta el `charter_id` completo (`CHARTER-01-test`), el prefijo `CHARTER-NN` (`CHARTER-01`), o solo el NN numérico (`01` o `1`). El match numérico es permisivo respecto al zero-padding. |
| `--path` | `.` | Directorio del proyecto. Es flag (no positional) para evitar colisión con el positional opcional `CHARTER-ID`. |

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
  Framework version: fw-4.4.1
  Author:            Strange Days Tech, S.A.S.
  License:           MIT
  Repository:        https://github.com/StrangeDaysTech/devtrail
  Website:           https://strangedays.tech
```

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
