---
slug: cuatro-nombres-en-cuatro-meses
title: Cuatro nombres en cuatro meses
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - prehistoria
  - identidad
  - governance
date: 2026-04-05T00:00:00.000Z
description: Buscar el concepto buscando el nombre
---
> **Nota editorial**: este es el primer post publicado de forma retroactiva en el blog. La `date` del frontmatter corresponde a un momento dentro del arco que el post narra, no al día en que se escribió. El blog completo se está construyendo así, hacia atrás, desde el episodio que motivó empezar a escribirlo (`emergent-observation-design`, 2026-05-16). No tiene sentido fingir lo contrario.

---

## 1. El día siguiente

El 28 de enero de 2026, a las 17:29 hora del centro, este commit entró al repo:

> `chore: rebrand Chronicle to Monimen`

Un día. Veintinueve horas, para ser exacto, después del *initial commit* del proyecto. El framework apenas existía y ya estaba cambiando de nombre.

Lo cuento en pasado, pero esto pasó hace tres meses. Y hace tres meses, dos cambios de nombre después, todavía no se llamaba StrayMark. El proyecto que hoy se vende como *Documentation Governance for AI-Assisted Software Development* arrastra una prehistoria corta y desordenada que vale la pena nombrar antes de seguir adelante.

---

## 2. Cinco eventos, cuatro meses

| Fecha (UTC-6) | Hora | Evento | Ancla |
|---|---|---|---|
| 2026-01-27 | 12:14 | *Initial commit*: **Enigmora Chronicle Framework v1.0.0** | [`7c58b6d`](https://github.com/StrangeDaysTech/straymark/commit/7c58b6d) |
| 2026-01-28 | 17:29 | Rebrand: Chronicle → **Monimen** | [`0b772bc`](https://github.com/StrangeDaysTech/straymark/commit/0b772bc) |
| 2026-01-29 | 19:30 | Rebrand: Monimen → **DevTrail** (`BREAKING CHANGE`) | [`25ab7a4`](https://github.com/StrangeDaysTech/straymark/commit/25ab7a4) |
| 2026-03-01 | 19:05 | Rebrand de la organización: Enigmora → **Strange Days Tech** + nace el CLI Rust | [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) |
| 2026-05-08/09 | — | DevTrail → **StrayMark** (ADR + arco de 5 PRs) | ADR `2026-05-08-001`, PRs #114-#118 |

Tres renames en tres días. Después una pausa de un mes entero. Después el rebrand de la organización con CLI Rust incluido. Después otra pausa de dos meses. Después StrayMark, esta vez con ADR público anclando la decisión.

Detalle que me parece honesto subrayar: el commit del 28 de enero (Chronicle → Monimen) no se marcó como `BREAKING CHANGE`. El del 29 de enero (Monimen → DevTrail) sí. *"BREAKING CHANGE: Complete rebrand from Monimen to DevTrail"*, dice el body del commit. La diferencia es pequeña pero reveladora: el segundo rename se sintió más definitivo. Lo fue por dos meses.

---

## 3. Lo que no se mueve

Esta es la sección que me interesa más. El proyecto cambia cuatro veces de nombre en cuatro meses. La idea, no.

El README del *initial commit* — el que vivía en el repo cuando se llamaba *Enigmora Chronicle Framework* — abre con esta línea:

> **Documentation Governance for AI-Assisted Software Development**

Es la misma línea que abre el README de StrayMark hoy. Cuatro nombres, un slogan.

Más adentro, el mismo README v1.0.0 enuncia la tesis del proyecto en un bloque de cita explícito:

> *"No significant change without a documented trace."*

Esa frase, hoy, es el Principio #1 del framework. Está en `PRINCIPLES.md §1 — Total Traceability`. La redacción cambió ligeramente al traducirse al español, pero el peso de la afirmación quedó intacto: ningún cambio significativo sin rastro documentado.

Los ocho tipos de documento que listaba el README v1.0.0 — REQ, ADR, TES, INC, TDE, AILOG, AIDEC, ETH — siguen vivos. Después se agregaron otros (MCARD, SBOM, SEC, DPIA), pero ninguno desplazó a los originales. La taxonomía de enero se sostuvo.

**El nombre se mueve cuatro veces, la idea no.** Lo que cambió en esos cuatro meses no fue el qué, sino la etiqueta que servía para nombrarlo.

---

## 4. El AIDEC con año tipográfico

Hay una anécdota pequeña que me sigue gustando.

El primer documento estructurado del proyecto — el primer AIDEC, el documento que en la propia taxonomía del framework registra una decisión tomada con asistencia de un agente — es `AIDEC-2025-01-27-001-i18n-strategy.md`. Está en el commit [`7b7193e`](https://github.com/StrangeDaysTech/straymark/commit/7b7193e), agregado a las 18:01 del 27 de enero de 2026, seis horas después del *initial commit*.

El ID dice **2025**. El commit es de **2026**.

El propio documento que sentó la convención de identificadores tipo `[TYPE]-[YYYY-MM-DD]-[NNN]` tiene un error tipográfico en el año. El framework arrancó imperfecto y nadie lo corrigió a tiempo. Si alguien clona el repo y mira al ID de ese AIDEC con cuidado, lo notará — y verá también que el commit que lo introdujo está fechado correctamente. Es la mejor evidencia que tengo de que la disciplina de auditoría es algo que llega después; no nace con el repo.

Pero más interesante que el error es lo que decide ese AIDEC. La pregunta que se planteó José en enero, con asistencia de Claude Opus 4.5, era: ¿cómo internacionalizamos un framework que existe para los humanos pero cuya configuración la leen agentes? La respuesta, en la sección de justificación del propio AIDEC:

> *"AI agents (Claude, Gemini, Copilot, Cursor) process instructions equally well in any language, so translating their config files provides no functional benefit."*

La decisión que tomó ese documento — traducir lo que leen humanos (documentación, plantillas), mantener en inglés lo que leen agentes (CLAUDE.md, GEMINI.md, .cursorrules) — fue una intuición temprana sobre que el framework tiene dos audiencias distintas. Hoy el blog mismo ratifica esa intuición: bilingüe español/inglés para los humanos que lo lean, mientras las skills y prompts que orquestan agentes siguen viviendo solo en inglés. Tres meses después, sigue siendo la decisión correcta.

---

## 5. El nacimiento del CLI — y el número que faltó

### 5a. El día que el proyecto se volvió software

El 1 de marzo de 2026, a las 19:05, entró el commit [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026):

> *"feat: rebrand to Strange Days Tech, add CLI scaffolder, restructure repo"*

Hasta ese día, el proyecto era prosa. Eran templates, skills, reglas de gobernanza, y un montón de Markdown que dependía de que el operador (yo) los copiara a mano dentro de los repos donde quería usarlos. Había un script bash llamado `copy-devtrail.sh`. Era de juguete.

Ese commit del 1 de marzo introdujo `cli/Cargo.toml` y `cli/src/main.rs`. El primer CLI se llamó `devtrail-cli`, versión `2.0.0`, y tenía tres comandos:

```rust
enum Commands {
    Init { path: String },
    Update,
    Remove { full: bool },
}
```

`init`, `update`, `remove`. Tres operaciones. El resto vino después: `status`, `repair`, `validate`, `new`, `compliance`, `metrics`, `analyze`, `audit`, `explore`. Pero los tres originales siguen ahí, casi sin tocar.

Lo importante del commit no son los tres comandos: es que ese día el proyecto dejó de ser un cuerpo de documentación que vivía en su propio repo y empezó a ser una herramienta ejecutable que se instalaba en otros repos. La frontera entre *"proyecto de gobernanza documental"* y *"framework con tooling"* se cruzó ese domingo de marzo. Es más importante que cualquiera de los renames que vinieron antes.

Es también, por cierto, el día en que el proyecto se mudó de cuenta en GitHub: de `enigmora/devtrail` a `StrangeDaysTech/devtrail`. La organización pasó de llamarse Enigmora a Strange Days Tech, S.A.S., en el mismo commit que estrenó el CLI. Renames simultáneos: el del producto (de Monimen a DevTrail) había ocurrido en enero; el de la empresa, en marzo. Dos capas separadas de identidad moviéndose en distintos relojes.

(Una nota lateral, importante para el cuidado del archivo: la co-autoría con un agente — ese `Co-Authored-By: Claude Opus X.Y` que aparece en commits — no nace en marzo. El *initial commit* del 27 de enero ya viene firmado por Claude Opus 4.5. La transparencia de co-autoría con IA fue regla desde la primera línea de código. Lo que cambia el 1 de marzo no es la práctica, es la escala: el CLI es el primer artefacto donde la co-autoría produce *código ejecutable*, no documentación.)

### 5b. El número que faltó

Hay un detalle del versionado del framework que se entiende mejor mirando `git tag`:

```
fw-2.0.0
fw-2.1.0
fw-4.0.0
fw-4.1.0
...
```

No hay `fw-3.x.x`. El proyecto saltó deliberadamente del 2 al 4.

El salto se sincroniza con el commit [`21e03b2`](https://github.com/StrangeDaysTech/straymark/commit/21e03b2), del 27 de marzo de 2026, cuyo cuerpo describe el cambio así:

> *"Reposition DevTrail from 'documentation helper' to 'ISO 42001-aligned AI governance platform' across all user-facing docs. Lead with regulatory urgency (EU AI Act Aug 2026) and compliance value proposition."*

Hasta marzo, DevTrail se presentaba como *ayudante de documentación*. A partir de ese commit, se presentó como *plataforma de gobernanza de IA alineada con ISO 42001*. El cambio de número (2 → 4, saltando el 3) marcó la magnitud del cambio de tesis. Fue una decisión consciente: una versión mayor no alcanzaba para señalar la diferencia.

Y aquí es donde la prehistoria empieza a tener sentido retrospectivo. La intuición que estuvo en el repo desde enero — la idea de un sistema de registro robusto, normativo, alineable con estándares de gobernanza — solo se nombró explícitamente en marzo. Lo veremos con más claridad en la próxima sección, al hablar del segundo nombre.

---

## 6. Sobre los nombres

Tres de los cuatro renames no tienen ADR. La justificación vive solo en los commit messages, y a veces ni siquiera ahí: el commit `chore: rebrand Chronicle to Monimen` no dice por qué Monimen. El siguiente, `chore: rebrand Monimen to DevTrail`, tampoco dice por qué DevTrail. La práctica del ADR como artefacto disciplinado llegó después; en enero el nombre cambiaba con un commit y la justificación se evaporaba en la conversación que la había producido.

Pero algo de etimología sí podemos rescatar.

- **Chronicle** *(27 ene)*. Registro histórico. Bitácora. El nombre habla por sí: lo que se hace con un *chronicle* es anotar lo que sucedió, en orden, para que se pueda revisar después. No hay ADR pero la palabra carga su propio sentido.

- **Monimen** *(28 ene)*. Este es el único nombre cuya intuición sí recuerdo y vale contar. *Monimen* surgió como juego de palabras a partir de *Monumento* (Monument en inglés). La analogía que motivó la sugerencia: las normas que ya entonces se intuían — AIDEC, AILOG, alineables con la incipiente normativa ISO 42001 — representaban una estructura sólida, duradera. Un *monumento* es algo que se erige para durar; un pilar de referencia inamovible para una comunidad. Eso quería ser el framework. El sufijo cortado — *Monimen* en vez de *Monument* — buscaba darle al término un toque más ágil, más de software, menos de mármol. Duró 25 horas. Pero la intuición no se perdió: lo que en marzo terminó nombrándose explícitamente como *ISO 42001-aligned AI governance platform* es la versión madura de ese mismo impulso. Monimen era un nombre con la tesis correcta y la lengua equivocada.

- **DevTrail** *(29 ene)*. El recorrido del developer. Menos solemne que Monimen, más concreto. El commit lo marcó `BREAKING CHANGE` — lo cual, mirado desde mayo, tiene una ironía: el rebrand más definitivo de los primeros tres fue precisamente el que duró tres meses y medio antes de ser sustituido.

- **StrayMark** *(8-9 may)*. La marca, la seña que queda. Pero ese rebrand pide su propio post: ya hay ADR público (`2026-05-08-001`), un arco de cinco PRs nucleares (#114-#118), y un manifesto *"Why StrayMark?"* en el README que merece tratamiento aparte. Aquí solo lo cierro como el rebrand que vino con disciplina documental detrás.

Lo importante no es la etimología en sí. Es que cada nombre ratifica una intuición distinta sobre qué es el producto: **bitácora** (Chronicle), **pilar normativo** (Monimen), **recorrido del developer** (DevTrail), **marca residual** (StrayMark). Cuatro nombres son cuatro hipótesis sobre el concepto. La búsqueda termina cuando el concepto se estabiliza — y solo entonces el nombre puede quedarse quieto.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **No buscábamos un nombre. Buscábamos al concepto.** Los cuatro renames son evidencia de búsqueda, no de indecisión. El concepto se fue afilando a través de los nombres que probaba.

2. **Tres de cuatro renames sin ADR.** La práctica del ADR como artefacto disciplinado llegó después. En enero todavía no había hábito. Eso no es deuda: es honestidad sobre el ritmo. Los hábitos de gobernanza son el resultado de querer registrar lo que ya estaba pasando, no la precondición para empezar.

3. **El proyecto se volvió software el 1 de marzo de 2026. Antes, era prosa.** El CLI Rust es la frontera. Cualquier discusión sobre StrayMark *como framework* tiene que reconocer que hay un antes y un después de ese commit.

4. **Probablemente no vuelva a haber rebranding.** Pero el blog no lo promete. El proyecto es honesto sobre su propia mutabilidad, y prometerlo sería traicionar la primera tesis del primer post.

Cuando empecé a escribir este blog, hace dos semanas, no tenía intención de cubrir la prehistoria. La idea era hablar de Charters, de observación emergente, de las cosas que el framework hoy hace. Pero el blog mismo es un acto retroactivo — nace porque algo me sorprendió lo suficiente como para empezar a escribir. Y una vez que decidí escribir hacia atrás, fue inevitable llegar a enero. A los tres renames en tres días. Al AIDEC con año tipográfico. A *Monimen*.

Sigue, en el siguiente post, el primer experimento metodológico real del framework: los seis Planes de Sentinel y el día que *Plan* se renombró a *Charter*. Ahí empieza la historia que el blog vino a contar.

---

*Anclas: commits [`7c58b6d`](https://github.com/StrangeDaysTech/straymark/commit/7c58b6d) · [`0b772bc`](https://github.com/StrangeDaysTech/straymark/commit/0b772bc) · [`25ab7a4`](https://github.com/StrangeDaysTech/straymark/commit/25ab7a4) · [`7b7193e`](https://github.com/StrangeDaysTech/straymark/commit/7b7193e) · [`c7e9026`](https://github.com/StrangeDaysTech/straymark/commit/c7e9026) · [`21e03b2`](https://github.com/StrangeDaysTech/straymark/commit/21e03b2). README original: `git show v1.0.0:README.md`.*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*
