---
slug: el-rebrand-a-straymark
title: El rebrand a StrayMark
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - rebranding
  - governance
  - identidad
date: 2026-05-09T00:00:00.000Z
description: 'El cuarto rename, esta vez con ADR público y arco disciplinado'
---
## 1. El rename que no buscaba el concepto

El ADR `2026-05-08-001` se abre con una frase que, para cualquiera que haya leído el Post 2 de este blog, suena ajena:

> *"The decision is motivated by legal certainty over the trademark rather than by product strategy or user feedback."*

Los tres renames de enero (Chronicle → Monimen → DevTrail) fueron búsqueda apurada del concepto. Cada uno ratificaba una intuición distinta sobre qué era el producto. Ninguno tenía ADR; los commit messages eran toda la justificación documental que existió.

El cuarto rename, el de mayo, es de otra naturaleza. No nace de buscar al concepto. Nace de una investigación de conflicto de *trademark* — comisionada vía sesión de Claude.ai a inicios de mayo — que arrojó *"legal uncertainty about trademark ownership as the project gained adopters"*. En enero el proyecto tenía cero adopters y los renames podían improvisarse. En mayo el proyecto tenía un adoptante (Sentinel), 286 descargas del *crate*, dos issues abiertos, y un manifesto público que prometía gobernanza estructurada. El umbral para improvisar otro rebrand ya había sido cruzado.

Este post cubre tres cosas: la decisión disciplinada que se tomó el 8 de mayo, el arco operativo de cinco PRs en cuarenta y tres minutos del día siguiente, y los residuos que aparecieron dos días después. Y, por dentro, una distinción más sutil: cuándo un rebrand es solo de superficie y cuándo, sin proponérselo del todo, cambia también la identidad.

---

## 2. El primer rename disciplinado

Lo nuevo del 8 de mayo es la forma. Esta vez había:

- **Una investigación previa.** No se decidió rebrand y luego se justificó; primero hubo un conflicto detectado y luego, en consecuencia, la decisión.
- **Un ADR público.** Tres alternativas consideradas explícitamente (mantener "DevTrail" y aceptar el riesgo; branding híbrido legacy + nuevo en paralelo; reset a v0.1.0 bajo StrayMark como "proyecto nuevo"). Las tres descartadas con argumento literal.
- **Un cálculo temporal.** El propio ADR enmarca el costo así: *"The rename window narrows over time — every new adopter increases the cost of changing later... The legal risk dominates. Acting now, with one self-owned adopter, is materially cheaper than acting later under pressure."*

La razón por la que el cálculo importa es estructural. Si el rebrand hubiera esperado dos meses más — hasta que apareciera el segundo adopter, o el tercero — el costo de migración se habría multiplicado. Cada repo adopter habría tenido que ejecutar el `mv .devtrail .straymark`, actualizar sus `CLAUDE.md`/`AGENT.md`, regenerar sus pipelines. Con un solo adopter (el propio operador), el costo era contenido y revertible. Lo que el ADR llama *"acting now is materially cheaper"* es exactamente eso: el rebrand fue una operación de minimización de deuda futura, no una decisión de marketing.

Hay un eco no buscado con el Post 2. Aquel cerró con la frase *"probablemente no vuelva a haber rebranding"*. Este post, escrito desde mayo, sabe que sí lo hubo. La diferencia entre prometer y ratificar es que el cuarto rename, a diferencia de los tres primeros, fue forzado por circunstancias externas que ningún operador puede prever — un conflicto de trademark — y se enfrentó con la disciplina que enero no tenía.

---

## 3. *In scope*, *out of scope* — la pieza maestra del ADR

La sección del ADR que da forma al resto del post es la que distingue qué se renombra y qué se preserva. Citada literal:

> *"In scope (the 'live state' of the project): all identifiers in source code, Cargo metadata, source-of-truth path, 30 skill/workflow files, public documentation, CI workflows, GitHub repository name."*

> *"Out of scope (immutable history, preserved verbatim): all commits, commit messages, and git history; all tags published before this ADR; all release titles and bodies of releases published before; all prior CHANGELOG.md sections."*

Esa distinción es la armadura del rebrand. Lo *live* — paths, comandos, URLs, assets de release, README — se renombra de un golpe. Lo *historic* — git log, tags `devtrail-cli@3.10.0`, sections previas del CHANGELOG, issues cerrados — se preserva literal. El proyecto no se reescribe hacia atrás.

Es la misma disciplina archivística que el Post 3 documentó para el rename Plan → Charter, ahora aplicada al rebrand operativo del framework completo. Lo expreso en una tabla porque ayuda a verlo:

| Capa | Se renombra | Se preserva |
|---|---|---|
| Paths del filesystem | `.devtrail/` → `.straymark/` | — |
| Source Rust | `devtrail-cli` → `straymark-cli` | — |
| Skills/workflows | 30 archivos `devtrail-*` → `straymark-*` | — |
| Docs públicas | README, CLAUDE.md, ADOPTION-GUIDE | — |
| CI/CD | nombres de assets, paths de binarios | Prefijos de tag (`fw-`, `cli-`) sin tocar |
| Repositorio GitHub | `StrangeDaysTech/devtrail` → `StrangeDaysTech/straymark` | — |
| CHANGELOG | Nueva entrada `fw-4.11.0` arriba | Secciones previas literal |
| Tags publicados | — | `fw-4.10.0`, `cli-3.10.0`, etc. preservados |
| Git log entero | — | Mensajes con "DevTrail" intactos |
| *Crate* legacy | — | `devtrail-cli@3.10.0` en crates.io, no *yanked* |

La continuidad de versiones es la prueba más visible. El siguiente release no fue `0.1.0` ni `1.0.0`; fue `fw-4.11.0` y `cli-3.11.0`. La numeración declara, sin metáfora, que el proyecto es el mismo proyecto.

---

## 4. Cinco PRs, cuarenta y tres minutos

El 9 de mayo, entre las 06:05 y las 06:48 UTC, se fusionaron cinco PRs en cadena:

| PR | Hora UTC | Qué hizo |
|---|---|---|
| [#114](https://github.com/StrangeDaysTech/straymark/pull/114) | 06:05 | Merge del ADR mismo. Cero cambios de código — primero se fija la decisión. |
| [#115](https://github.com/StrangeDaysTech/straymark/pull/115) | 06:42 | 174 *renames* + 74 modificaciones. Source Rust (38 archivos), tests (18), `dist/.devtrail/` → `dist/.straymark/` (122 archivos vía `git mv`), 30 skills × 3 plataformas. |
| [#116](https://github.com/StrangeDaysTech/straymark/pull/116) | 06:44 | Docs públicas en tres idiomas (EN/ES/zh-CN). Preámbulo del CHANGELOG actualizado: *"StrayMark (formerly DevTrail; rebranded 2026-05-08)"*. |
| [#117](https://github.com/StrangeDaysTech/straymark/pull/117) | 06:45 | Workflows de release. Nombres de assets (`straymark-cli-*`), títulos de release. Prefijos de tag intactos. |
| [#118](https://github.com/StrangeDaysTech/straymark/pull/118) | 06:48 | Bump `fw-4.11.0` / `cli-3.11.0`. Nueva sección del CHANGELOG. |

El orden es deliberado y va de afuera hacia adentro: primero la ley (ADR), después el código interno, después lo que ven los adopters (docs), después la distribución (CI), finalmente el hecho consumado (release). Esta secuencia tiene una lectura archivística: cuando alguien dentro de seis meses revise `git log` en orden cronológico, primero verá la decisión, después la implementación. La causalidad queda en el repo.

Vale la pena nombrar una pieza del arco que rompió el plan original. El ADR había contemplado nueve PRs distintos (uno por capa lógica). En la práctica, los tests del CLI dependían de paths *hardcoded* — `include_str!("../../dist/.devtrail/...")` — y romper el path sin actualizar los tests al mismo tiempo dejaba CI en rojo. El plan se compactó a cinco porque las capas estaban acopladas técnicamente, no porque la disciplina conceptual fallara. La nota del PR #115 lo registra sin atenuar: *"The consolidation was forced by tight coupling: tests use `include_str!` on `dist/.devtrail/` paths."*

Cuarenta y tres minutos para un rebrand que tocó alrededor de doscientos sesenta archivos. La cadencia solo es posible porque el ADR había cerrado todas las decisiones antes de tocar código. El principio #6 del framework — *"cuando la propuesta es bien escrita, la implementación es ejecución, no diseño"* — encontró otro caso de validación, parecido al del audit cycle externo del Post 4.

---

## 5. Por qué StrayMark sí cambia algo más que el nombre

Hasta aquí el ADR es claro: el rebrand es de superficie, la identidad conceptual se preserva. Pero hay un matiz que vale la pena nombrar.

El PR [#120](https://github.com/StrangeDaysTech/straymark/pull/120) — fusionado horas después del arco principal, el mismo 9 de mayo — añadió al README la sección *"Why StrayMark?"*. No es código; es un manifesto. Y dice cosas que ningún documento previo del proyecto había articulado:

> *"Code is just the fossil trace of a mental battle. Real engineering happens in the chaos of decisions, calculated risks, and the paths you chose not to take. Traditionally, all that human trail is discarded as stray marks (accidental smudges) in a project's history. At Strange Days Tech, we believe those marks are the signal, not the noise."*

La elección del nombre, vista desde el manifesto, no es arbitraria. Las *stray marks* — las marcas erráticas, las trazas accidentales que la mayoría de proyectos descartan — son exactamente lo que el framework arrastra a primer plano: AIDECs, AILOGs, TDEs, Charters. Lo que en cualquier proyecto convencional sería ruido descartable, en este proyecto es la materia prima de la auditoría. El nombre encarna la tesis del producto en una forma que *DevTrail* nunca lo hizo. *DevTrail* era un recorrido neutro; *StrayMark* es una afirmación.

El manifesto cierra con tres líneas que después se volvieron lo que un editor llamaría *tagline* operativo:

> *"Capture the noise. Weave the signal. Humanize the machine."*

Aquí está la pequeña tensión interesante del rebrand. El ADR dice que solo cambió la superficie; el manifesto del mismo día sugiere que también se articuló — quizás por primera vez con claridad — la identidad. No es contradicción: el momento del rebrand operativo fue aprovechado para escribir lo que ya estaba en el proyecto pero no se había nombrado todavía. La identidad no cambió. Se hizo legible.

---

## 6. Lo que quedó pendiente

Aun con ADR público y arco disciplinado, *"completo"* sigue siendo aproximación.

Dos días después del rebrand, mientras Sentinel preparaba su siguiente ciclo de auditoría externa, el agente señaló tres clases de residuos. La cronología — captura honesta en `CHANGELOG.md` como erratas inline — vale la pena dejarla a la vista:

- **PR [#137](https://github.com/StrangeDaysTech/straymark/pull/137)** (11 may, 22:55) — *Skill directories órfanos*. El rebrand reescribió el campo `name:` dentro de cada `SKILL.md`, pero no renombró los directorios de las tres plataformas (`.gemini/skills/devtrail-*`, `.claude/skills/devtrail-*`, `.agent/workflows/devtrail-*.md`). Treinta artefactos órfanos: nombre nuevo, ruta vieja. Gemini CLI lo reportó como diez warnings de conflicto al arrancar.
- **PR [#138](https://github.com/StrangeDaysTech/straymark/pull/138)** (11 may, 23:32) — *Charter paths legacy*. Tres artefactos del framework seguían referenciando `docs/charters/` después de que `fw-4.12.0` migró el path canónico a `.straymark/charters/`. Lo más grave: el hook `pre-pr.sh` estaba gated en `[ ! -d docs/charters ]`, lo cual hacía *exit 0* silencioso para cualquier adopter post-4.12.0. El drift check llevaba siete meses instalado pero no-op.
- **PR [#139](https://github.com/StrangeDaysTech/straymark/pull/139)** (12 may, 12:02) — *Instaladores rotos*. `install.sh` e `install.ps1` aún apuntaban a `REPO=StrangeDaysTech/devtrail`, `BINARY=devtrail`, asset name `devtrail-cli-v*`. El README — ya rebranded — apuntaba a la nueva URL. Resultado: cualquier usuario que copiara el comando del README desde el 9 de mayo recibía un 404 de GitHub al ejecutar el instalador. *El producto estuvo roto en producción durante setenta y dos horas.*
- **PR [#140](https://github.com/StrangeDaysTech/straymark/pull/140)** (12 may, 16:49) — *Micro-residuo*. La línea uno de `.gitignore` decía aún `# DevTrail - .gitignore`. Más una entrada nueva para `Aparador/` en patterns internos.

La lección no es que la disciplina fue insuficiente. Es que la disciplina *no previene* residuos — los hace encontrables, documentables, y reparables en errata inline pública. Los tres primeros renames (enero) probablemente tuvieron residuos equivalentes; nunca los catalogamos porque no había hábito documental. El cuarto rebrand los catalogó. Es el primer rebrand del proyecto que admite, en el repo público y en el CHANGELOG mismo, que el operario no atrapó todo a la primera.

El detalle más incómodo — los instaladores rotos por setenta y dos horas — es el que más me importa registrar. Si en lugar de un adopter solo (el operador) hubiera habido un equipo, esa ventana habría afectado a usuarios reales. Acting now, with one self-owned adopter, was materially cheaper, says the ADR. Eso es exactamente lo que esa frase compraba: el derecho a equivocarse durante setenta y dos horas sin lastimar a nadie.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **No todos los renames son del mismo tipo.** Los tres de enero buscaban al concepto; el de mayo defendía al proyecto contra deuda futura legal. Solo el de mayo podía hacerse con ADR público, porque solo el de mayo nacía de evidencia externa y no de intuición interna.

2. **La distinción superficie vs. identidad pertenece al documento, no a la retórica.** El ADR mismo declara qué se renombra y qué se preserva. Esa lista — paths sí, git log no; release titles sí, tags previos no — es la armadura del rebrand. Sin esa lista explícita, *"rebrand"* significa cualquier cosa.

3. **La identidad puede hacerse legible aunque no cambie.** El manifesto *"Why StrayMark?"* no inventó lo que el proyecto era; lo articuló por primera vez con claridad. El rebrand operativo fue aprovechado para escribir lo que ya estaba en el código pero no había salido al README. La identidad no se reescribió, se publicó.

4. **La disciplina no previene residuos; los hace encontrables.** Cuatro PRs *errata* dos días después no son falla del rebrand; son evidencia de que el rebrand fue lo suficientemente honesto como para admitir qué se le escapó. Los tres renames de enero probablemente tuvieron equivalentes; nunca aparecieron documentados.

Sigue, en el siguiente post, un episodio que conecta directamente con el último punto: *TDE como mecanismo para nombrar deuda transversal* (`H-09`). El primer caso real que el framework usó para articular cómo evidenciar la deuda — y que dejó una lección dura sobre stacked-PRs que merece su propio párrafo cuando llegue ahí.

---

*Anclas: ADR [`2026-05-08-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-05-08-rebranding-straymark.md). Arco nuclear: PRs [#114](https://github.com/StrangeDaysTech/straymark/pull/114) · [#115](https://github.com/StrangeDaysTech/straymark/pull/115) · [#116](https://github.com/StrangeDaysTech/straymark/pull/116) · [#117](https://github.com/StrangeDaysTech/straymark/pull/117) · [#118](https://github.com/StrangeDaysTech/straymark/pull/118). Manifesto: PR [#120](https://github.com/StrangeDaysTech/straymark/pull/120). Residuos: PRs [#137](https://github.com/StrangeDaysTech/straymark/pull/137) · [#138](https://github.com/StrangeDaysTech/straymark/pull/138) · [#139](https://github.com/StrangeDaysTech/straymark/pull/139) · [#140](https://github.com/StrangeDaysTech/straymark/pull/140). Release: `fw-4.11.0` / `cli-3.11.0`.*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*
