# StrayMark — Paridad Windows/PowerShell y el rol de Microsoft Coreutils

**Versión:** 0.2 — **Opción A ejecutada** en [#237](https://github.com/StrangeDaysTech/straymark/issues/237) (fw-4.26.0 / cli-3.23.0): `charter drift` portado a Rust nativo, `check-charter-drift.sh` deprecado. Microsoft Coreutils descartado como vehículo de paridad (sigue siendo ergonomía opcional del adoptante, §5). Pendiente sin ejecutar: verificación empírica de §6.3 antes de cualquier mención en `ADOPTION-GUIDE.md`.
**Fecha:** 3 de junio de 2026 (v0.1) · 12 de junio de 2026 (v0.2)
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Evaluar la viabilidad de usar el recién anunciado `microsoft/coreutils` (Build 2026) para emparejar la experiencia StrayMark en equipos Windows/PowerShell, e identificar cuál es realmente la ruta de menor costo hacia la paridad.
**Documentos relacionados:** `ADR-2026-06-03-001-followups-first-class.md` (precedente de cristalización bash → CLI nativo), `2026-05-03-cli-roadmap.md` (items diferidos con criterio de salida explícito), `CHANGELOG.md` (entrada "pure-Rust fallback for Windows-without-bash deferred until requested").

---

## 1. Contexto y disparador

StrayMark conserva deliberadamente varios scripts bash porque replicar sus funciones implicaría que el CLI Rust reimplementara utilidades POSIX complejas (`grep`, `sed`, `awk`). Esa decisión produjo un desfase para adoptantes cuyo equipo trabaja en Windows nativo (sin WSL): el único script PowerShell del proyecto es el instalador `install.ps1`.

El 2 de junio de 2026, en Build 2026, Microsoft anunció **Coreutils for Windows** ([github.com/microsoft/coreutils](https://github.com/microsoft/coreutils)): un build mantenido por Microsoft del proyecto [uutils](https://github.com/uutils/coreutils) (rewrite en Rust de GNU coreutils) junto con findutils y una implementación de grep GNU-compatible — más de 70 utilidades, licencia MIT, instalable con `winget install Microsoft.Coreutils`, en estado **preview**. Cobertura: [BleepingComputer](https://www.bleepingcomputer.com/news/microsoft/microsofts-coreutils-project-brings-linux-commands-to-windows/), [Phoronix](https://www.phoronix.com/news/MS-Coreutils-For-Windows).

La pregunta de este documento: ¿es esto el vehículo para cerrar la brecha Windows de StrayMark?

**Respuesta corta (spoiler):** no para los scripts — pero sí cambia el cálculo de ergonomía para adoptantes Windows, y la investigación reveló que la brecha real es más pequeña y más barata de cerrar de lo que se asumía.

---

## 2. Inventario de scripts y por qué bash sobrevivió

| Script | Tipo | Invocado desde | Utilidades POSIX | Estado |
|---|---|---|---|---|
| `install.sh` | POSIX sh | onboarding de usuarios Unix | curl/wget, tar, sed, uname, mktemp | Vivo — paridad lograda vía `install.ps1` |
| `install.ps1` | PowerShell 5.1+ | onboarding de usuarios Windows | N/A (PS nativo) | Vivo |
| `dist/.straymark/hooks/pre-pr.sh` | bash/sh | hook pre-push opt-in (`straymark init --hooks`) | grep, awk, tr, command | Vivo — bash-only |
| `dist/.straymark/scripts/check-charter-drift.sh` | POSIX bash (213 líneas) | envuelto por `straymark charter drift` | grep `-oP`, sed, awk, sort, git | Vivo — bash-only, **la brecha real** |
| `check-followups-drift.sh` (vivía en el repo del adoptante Sentinel, ~296 líneas; nunca shippeado en `dist/`) | POSIX sh | (histórico) pre-commit hooks de adopters | grep, sed, awk, git | **Deprecado** desde cli-3.19.0 → `straymark followups drift` nativo (`FOLLOW-UPS-BACKLOG-PATTERN.md` §"Legacy bash script") |

Por qué se conservaron (fuentes en el repo):

- `cli/src/commands/charter/drift.rs` (doc-comment, líneas 11–16): *"Bash delegation only. Windows native (no WSL, no Git Bash) currently has no path; [...] A Rust-native fallback is feasible but deferred until a real adopter reports the need."*
- `CHANGELOG.md`: *"Bash delegation only; pure-Rust fallback for Windows-without-bash deferred until requested."*
- `ADR-2026-06-03-001-followups-first-class.md`: los scripts bash son **prototipos de validación empírica** (Sentinel, N=91 FUs); una vez validado el patrón, la inversión va al CLI Rust ("citizenship"), no al script. `check-followups-drift.sh` ya recorrió ese camino completo.

---

## 3. Qué es Microsoft Coreutils — y qué no es

**Qué es:**

- Build de Microsoft de **uutils/coreutils + findutils + grep GNU-compatible**, escrito en Rust, licencia MIT.
- Binario único `coreutils.exe` (multi-call); el instalador crea **hardlinks NTFS** por comando (`ls.exe`, `grep.exe`, `find.exe`, …) en `C:\Program Files\coreutils\`.
- `winget install Microsoft.Coreutils`. Integración con PowerShell 7.4+ vía PSReadLine (mejora el comportamiento de quoting).
- Objetivo declarado: *"frictionless moving between Linux, macOS, WSL, containers, and Windows"*.

**Qué NO es (los tres hechos que deciden este análisis):**

1. **No incluye ningún shell.** No hay bash ni sh. Un script `.sh` sigue sin poder ejecutarse en PowerShell por mucho que `grep` y `find` existan en PATH. Los tres scripts bash-only de StrayMark son *scripts*, no comandos sueltos.
2. **No incluye `sed` ni `awk`.** `check-charter-drift.sh` usa ambos extensivamente (parsing awk de tablas markdown con estado, transformaciones sed glob→regex). Incluso un hipotético port línea-por-línea a PowerShell no tendría esas piezas.
3. **Excluye comandos en conflicto con Windows:** `dir`, `more`, `whoami`, `kill`, `timeout` (sin señales POSIX), `chmod`/`chown` (sin permisos POSIX).

**Riesgos adicionales para cualquier uso documentado:**

- Estado **preview** — no apto como dependencia dura de un framework de gobernanza.
- **Precedencia de alias en PowerShell:** `ls`, `cat`, `sort`, `tee` son alias de cmdlets nativos que ganan a los ejecutables del PATH; el adoptante obtiene `Sort-Object` y no `sort.exe` salvo que invoque `sort.exe` explícitamente. La integración PSReadLine mitiga quoting, no precedencia.
- **Soporte PCRE incierto:** `check-charter-drift.sh` usa `grep -oP` con `\K` (Perl regex). La implementación grep del bundle se describe como "GNU-compatible" sin afirmar soporte `-P`. *Requires empirical verification on Windows* antes de cualquier afirmación en docs de adoptantes.

---

## 4. Análisis de brecha: dónde duele realmente Windows

Punto por punto, la brecha es más estrecha de lo que la memoria institucional sugiere:

| Superficie | ¿Brecha real en Windows nativo? |
|---|---|
| Instalación del CLI | **No** — `install.ps1` existe y tiene paridad funcional con `install.sh`. |
| `straymark followups drift` | **No** — nativo en Rust desde cli-3.19.0. El precedente de cristalización ya existe. |
| Agent directives (`AGENT-RULES.md`, `FOLLOW-UPS-BACKLOG-PATTERN.md`) | **No** — instruyen comandos del CLI (`straymark followups drift --apply`, `straymark charter drift`), no one-liners POSIX. Esto fue una decisión correcta que hoy paga dividendos. |
| Hook `pre-pr.sh` | **Matizada** — git en Windows es, en la práctica, Git for Windows, que ejecuta los hooks con su `sh.exe` incluido (MSYS2). El hook *ya funciona* para la mayoría de adoptantes Windows; el matiz solo debe documentarse. |
| `straymark charter drift` | **Sí — la única brecha funcional real.** `drift.rs:74-81` falla con error explícito si no hay `bash` en PATH, porque delega a `check-charter-drift.sh`. En Windows nativo puro (sin Git Bash en PATH, sin WSL) el comando no tiene ruta. |

**Hallazgo clave que cambia el costo:** la parte más compleja del script — el parser multilingüe (EN/ES/zh-CN) de `## Files to modify`, el filtro de extensiones reconocidas y la detección de wildcards — **ya fue portada a Rust puro** en `cli/src/charter_files.rs`, como fuente única de verdad para la regla de validación `CHARTER-FILES-EXIST` y el nudge de reconocimiento de `charter new` (fw-4.20.0/cli-3.17.0, finding #210). Su doc-comment lo dice explícitamente: *"Ported from the awk extraction in `check-charter-drift.sh` [...] so the CLI's pure-Rust consumers agree byte-for-byte with the drift script."*

Lo que falta para un `charter drift` 100% Rust:

1. `git diff --name-only <range>` vía `std::process::Command` — git ya es invocado cross-platform por el CLI en otros comandos.
2. Transformación glob→match para paths declarados con `...`/`*` (hoy ~6 líneas de sed; trivial en Rust, e `is_wildcard()` ya existe en `charter_files.rs`).
3. Set-difference declarados vs. modificados (omisión + expansión de scope).
4. Formato del reporte (hoy el CLI ya parsea la salida del script para la supresión AILOG-aware — esa lógica se simplifica al desaparecer el parsing intermedio).

Estimación honesta: el "pure-Rust fallback deferred until requested" se diseñó como diferimiento de un port completo; con `charter_files.rs` shippeado, lo que queda es ~30% del trabajo original, y la pieza con mayor riesgo de divergencia (el parser) ya está validada byte-for-byte contra el script.

---

## 5. Opciones y recomendación

### Opción A — Completar el port Rust-nativo de `charter drift` (recomendada)

Reusar `charter_files.rs` y eliminar la delegación bash de `drift.rs`. El script `check-charter-drift.sh` sigue el mismo ciclo de vida que `check-followups-drift.sh`: queda como referencia/prototipo en `dist/`, deprecado y sin mantenimiento, hasta retirarse.

- **Pros:** paridad Windows total sin dependencias externas (ni WSL, ni Git Bash, ni MS coreutils); precedente directo (cli-3.19.0 + ADR-2026-06-03); elimina una clase entera de bugs (parsing de la salida del script en `drift.rs`); el criterio de salida del diferimiento ("until a real adopter reports the need") puede leerse como satisfecho preventivamente por la búsqueda activa de adoptantes Windows-based.
- **Contras:** trabajo de CLI (~release minor, candidato cli-3.20.0); requiere suite de tests de equivalencia script-vs-Rust antes de deprecar (el script tiene cero falsos positivos validados en Sentinel PLAN-05/PLAN-06 — esa propiedad debe preservarse).

### Opción B — Port PowerShell de los scripts apoyado en MS coreutils (descartada)

- Doble mantenimiento perpetuo (bash + ps1) del artefacto más delicado del framework.
- `sed`/`awk` no existen en el bundle: el port no sería mecánico sino una reescritura en PowerShell idiomático — el mismo costo que el port Rust pero en un lenguaje que el proyecto no usa.
- Dependencia de un producto en preview con soporte PCRE sin confirmar.
- Contradice la dirección ya sentada por ADR-2026-06-03: la inversión va al CLI, no a más scripts.

### Opción C — Status quo + documentar WSL/Git Bash (lo actual)

Aceptable mientras no haya adoptante Windows real, pero no resuelve Windows nativo y deja el mensaje de error de `drift.rs` como única UX. Nota colateral: ese mensaje referencia "fw-4.6.0" — desactualizado; corregirlo es un follow-up independiente de la decisión grande.

### Rol acotado de Microsoft Coreutils (complementario a la Opción A, no alternativo)

MS coreutils **no** es el vehículo de paridad, pero sí mejora la ergonomía de humanos y agentes en sesiones PowerShell:

- One-liners de verificación en docs (`grep`, `find`, `sort -u`) se vuelven ejecutables tal cual en Windows.
- Agentes AI operando en PowerShell dejan de fallar en comandos POSIX sueltos que hoy emiten por hábito.
- Reduce la presión de "instala WSL" para tareas menores.

Recomendación: mención **opcional** en `ADOPTION-GUIDE.md` (sección Windows) — `winget install Microsoft.Coreutils` como mejora de calidad de vida, con advertencia explícita de estado preview y del matiz de alias de PowerShell. **Nunca como requisito** ni como dependencia de ningún flujo del framework. Las agent directives no deben cambiarse: ya hablan CLI, no POSIX.

---

## 6. Próximos pasos sugeridos (para otra sesión)

1. **Issue: port Rust-nativo de `charter drift`** (candidato cli-3.20.0). Alcance: pasos 1–4 de §4; criterio de aceptación: equivalencia de salida contra `check-charter-drift.sh` sobre los fixtures existentes + casos PLAN-05/PLAN-06; deprecación del script en `dist/` al estilo `check-followups-drift.sh` (mismo lenguaje de deprecación en `FOLLOW-UPS-BACKLOG-PATTERN.md`).
2. **Follow-up menor:** actualizar el mensaje de error de `drift.rs:79` que referencia "fw-4.6.0".
3. **Verificación empírica en una máquina Windows** antes de tocar `ADOPTION-GUIDE.md`: (a) ¿`grep.exe` del bundle soporta `-P`/`\K`?; (b) comportamiento real de precedencia de alias en PowerShell 7.4+ con la integración PSReadLine; (c) ¿`straymark charter drift` funciona hoy en Git Bash de Git for Windows sin fricción? (validaría que el matiz de §4 es correcto).
4. **Decisión documentada (ADR ligero o nota en este proposal v0.2)** una vez ejecutado el port: la postura oficial del proyecto es "el CLI Rust es la capa de portabilidad; los scripts bash son prototipos con ciclo de vida hacia deprecación; MS coreutils es ergonomía opcional del adoptante".

---

*Documento especulativo: ninguna afirmación de §3 sobre Microsoft Coreutils ha sido verificada empíricamente en Windows; las marcadas requieren esa verificación antes de propagarse a documentación de adoptantes.*
