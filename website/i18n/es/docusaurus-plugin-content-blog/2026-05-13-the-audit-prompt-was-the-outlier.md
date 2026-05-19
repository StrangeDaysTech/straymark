---
slug: el-audit-prompt-era-el-outlier
title: El audit-prompt era el outlier
authors:
  - jose
tags:
  - straymark
  - i18n
  - audit
  - governance
date: 2026-05-13T00:00:00.000Z
description: El único archivo del framework que vivía en otro idioma
---
*Un archivo entre decenas vivía al revés — canónico en español en un framework canónico en inglés, durante seis semanas, sin que nadie lo notara. Un fix de mantenimiento que registra algo con lo que el blog se encuentra una y otra vez: los outliers enseñan la convención por contraste.*

<!-- truncate -->

## 1. Una sola línea del commit

Esta es la entrada operativa del post 8. El commit `a8a1ac5` del 12 de mayo de 2026 carga un cuerpo que se abre con esta frase:

> *"The unified audit-prompt template at `dist/.straymark/audit-prompts/` was the only framework artifact whose canonical content lived in Spanish — every other template, skill, workflow, and governance doc follows the EN-canonical + `i18n/<lang>/` overlay pattern resolved by `cli/src/utils.rs:146`."*

Un solo archivo. Entre docenas. Vivía al revés. Y vivía así desde mayo del año anterior, sin que nadie lo notara.

Este post es corto a propósito. La corrección que cubre — PR [#142](https://github.com/StrangeDaysTech/straymark/pull/142), `fw-4.13.3` y `cli-3.12.3`, fusionado el 13 de mayo — no es un hito narrativo grande. Es una pieza de mantenimiento. Pero registra una de las regularidades que más me sigo encontrando en este oficio: los outliers son útiles. Te enseñan la convención por contraste.

---

## 2. De dónde venía el outlier

El audit-prompt no nació en el framework. Nació como una *skill* local de Sentinel — `sentinel/.claude/skills/plan-audit/` — durante el experimento de los seis Planes que cubrió el Post 3, en abril. Era una skill *context-specific*, escrita por el operador (yo) en español por razones que ahora se pueden enumerar honestamente: era un experimento privado, el operador habla español, y nadie iba a leer la skill nunca más. La improvisé en mi idioma porque era para mí.

Cuando el experimento se cristalizó en el framework — el arco de fw-4.4.0 a fw-4.9.0 que cubrió el Post 4 — la skill se portó al canonical path `dist/.straymark/audit-prompts/audit-prompt.md`. El porte fue mecánico: copiar el archivo, generalizarlo lo suficiente para que no mencionara componentes específicos de Sentinel, ajustar la nomenclatura *Plan* → *Charter*. Lo que no se hizo: traducirlo al inglés. El archivo entró al framework *en el idioma en que lo había escrito*. Sin que nadie lo decidiera, el español pasó a ser la versión canónica de ese único artefacto.

El resto del framework — todos los demás templates, todas las skills, los governance docs canónicos — habían sido escritos en inglés desde enero, con español y zh-CN como overlays bajo `i18n/<lang>/`. El audit-prompt fue la excepción silenciosa, durante seis semanas, hasta el 12 de mayo.

---

## 3. Cómo se descubrió: por inspección visual

No fue observación emergente del agente. Lo notó el operador, visualmente. La entrada del CHANGELOG lo registra sin metáfora:

> *"Surfaced empirically when Sentinel audited the `straymark explore` rendering and the user noted that audit-cycle templates were ES while all other templates and skills were EN."*

`straymark explore` es la TUI del framework — el navegador interactivo de documentación que el Post 14 cubrirá. Su valor es renderizar todos los templates lado a lado, en la misma vista. Y cuando todos los templates están lado a lado, uno en otro idioma se nota. Esa es la observación.

Vale la pena dejar el dato porque importa para la honestidad del blog. El Post 1 articuló el patrón de *observación emergente* — un agente señalando algo que nadie le pidió señalar — y el Post 5 documentó qué pasa cuando ese patrón falta por *visibilidad estructural*. Este episodio no es ninguno de los dos. Es una observación más pedestre: una superficie nueva del framework (la TUI) puso todos los archivos en visión simultánea, y la inconsistencia se hizo evidente como cuando uno ordena los libros del librero y nota que uno tiene el lomo al revés. Las herramientas también producen visibilidad.

---

## 4. El fix: tres movimientos en un commit

PR #142 hizo tres cosas en el mismo commit:

- **`dist/.straymark/audit-prompts/audit-prompt.md`** reescrito en inglés como versión canónica. 312 líneas. La traducción cuidó preservar la estructura de severidad y los ejemplos de calibración del original.
- **`dist/.straymark/audit-prompts/i18n/es/audit-prompt.md`** creado como overlay. 318 líneas. Es el contenido español que antes vivía en root, ahora en su lugar correcto.
- **`cli/src/commands/charter/audit.rs`** cableado al resolver i18n. Hasta entonces, el subcomando hardcodeaba el path `dist/.straymark/audit-prompts/audit-prompt.md` sin consultar el `language` field del `.straymark/config.yml` del adopter. Después del PR, lee la configuración del proyecto y resuelve la ruta localizada — si el adopter tiene `language: es`, recibe el overlay español; si tiene `language: zh-CN`, hace fallback al EN canonical (no hay traducción zh-CN del audit-prompt todavía); si tiene `language: en` o no especifica, recibe el canonical.

Tres tests nuevos cubren los tres caminos. Cero cambio funcional para usuarios en inglés (que es la mayoría). Para usuarios en español, mejora: antes recibían el prompt en español por accidente (porque el path canónico apuntaba al ES); ahora lo reciben en español por convención (porque el resolver lo decide).

---

## 5. El movimiento lateral

Al hacer el switch de idioma, el operador aprovechó para extraer una especificidad que había quedado del origen Sentinel. La sección §Step 5 del prompt — sobre calibración de severidad de hallazgos — citaba un caso literal del experimento de abril: *"Etapa 12 Pub/Sub stub vs gochannel"*. Un ejemplo perfectamente didáctico, pero específico de un componente de Sentinel que ningún otro adopter conocería.

La traducción al inglés reemplazó ese ejemplo por una formulación vendor-neutra: *"declared deferral, not a defect — a charter that introduces a thin adapter slated for replacement in a future Charter"*. La idea — un *stub* que es deuda declarada, no defecto — se preserva. Lo que se va es el nombre del componente.

El movimiento es pequeño pero coherente con el principio del blog. Cuando hay oportunidad de desligar el framework de un adoptante específico sin perder pedagogía, vale la pena tomarla. Es la misma operación de generalización que Post 3 documentó para los seis Planes y Post 4 para el primer Charter: el contenido empírico se preserva *en* Sentinel; lo que entra al framework es la abstracción vendor-neutra. El audit-prompt, casi un año después, finalmente terminó de cumplir esa regla.

---

## 6. Cierre

Tres tesis cortas:

1. **En un framework con convención, un solo archivo que la viola es señal de legado, no de diseño.** El audit-prompt era el outlier porque nació antes, y nadie lo migró cuando los demás artefactos se alinearon. Los outliers acumulan procedencia.

2. **Las herramientas nuevas producen visibilidad.** El switch ES → EN no se decidió por revisión sistemática del repo. Se decidió porque una TUI nueva puso todos los templates en la misma pantalla y la inconsistencia se hizo evidente. Lo mismo aplica a `grep`, a vistas agregadas, a outputs de `status`: cada superficie nueva expone qué estaba desalineado.

3. **Los rebrands chicos son oportunidad de desidentificación.** Cuando tocas un archivo para cambiar idioma, costo marginal cero para también extraer la especificidad de adopter que quedó del origen. El blog ha registrado este patrón antes (Plan → Charter, DevTrail → StrayMark); este es la versión miniatura. El framework se vuelve más vendor-neutro un archivo a la vez.

Sigue, en el siguiente post, un episodio que ya rozó al Post 1 pero merece su propio tratamiento: *"La disciplina manual antes del patrón"* (`H-11`). Cómo Sentinel manejó `CHARTER-18` antes de que existiera el `Pattern 1` de chain evolution — el aprendizaje empírico que el `Post 1` después nombró como meta-patrón.

---

*Anclas: PR [#142](https://github.com/StrangeDaysTech/straymark/pull/142). Releases: `fw-4.13.3` / `cli-3.12.3`. Commit clave: `a8a1ac5`. Archivos: `dist/.straymark/audit-prompts/audit-prompt.md` (EN canonical) + `dist/.straymark/audit-prompts/i18n/es/audit-prompt.md` (overlay). CLI: `cli/src/commands/charter/audit.rs` cableado al resolver i18n.*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*
