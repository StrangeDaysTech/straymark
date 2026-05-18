---
slug: explorando-el-framework
title: Explorando el framework
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - tui
  - cli
  - i18n
  - visibilidad-estructural
date: 2026-04-27T00:00:00.000Z
description: La TUI que produjo visibilidad sin proponérselo
---
*Para abril de 2026 el framework tenía ~50 archivos Markdown gobernados y se había vuelto opaco. `straymark explore` — un TUI que renderiza todo el repo como superficie navegable — nació tres semanas antes de revelar accidentalmente el outlier del Post 8. Las herramientas nuevas producen visibilidad sin proponérselo.*

<!-- truncate -->

## 1. La herramienta que dos semanas después reveló el outlier

El Post 8 de este blog cubrió un episodio muy específico: el descubrimiento del único archivo del framework cuyo idioma canónico era el español. El operador no lo encontró por inspección sistemática; lo encontró ejecutando `straymark explore --lang es` durante una preparación de auditoría y notando, visualmente, que un template estaba escrito al revés respecto a los demás. La frase de aquel post fue:

> *"Una nueva superficie del framework (la TUI) puso todos los archivos en visión simultánea, y la inconsistencia se hizo evidente como cuando uno ordena los libros del librero y nota que uno tiene el lomo al revés."*

Este post cubre la herramienta. La TUI `straymark explore` nació casi tres semanas antes del descubrimiento que reveló — el 25 de abril de 2026 en `cli-3.4.0` — y maduró en cuatro releases más, todos cerrados en menos de 48 horas. No es un post sobre código. Es un post sobre una pieza específica de tooling que ratifica una tesis que el blog ya nombró antes: las herramientas nuevas producen visibilidad sin proponérselo.

---

## 2. Por qué un TUI en un framework de Markdown

A finales de abril de 2026, el framework tenía algo así como cincuenta archivos Markdown gobernados — `PRINCIPLES.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `QUICK-REFERENCE.md`, `SPECKIT-CHARTER-BRIDGE.md`, plus templates en `dist/.straymark/templates/`, plus audit prompts, plus las versiones en tres idiomas de cada uno (`i18n/es/`, `i18n/zh-CN/`). Todo en disco, todo navegable con `find` y `cat`, todo perfectamente accesible.

Y, al mismo tiempo, todo invisible. Para un adoptante que clonaba el framework por primera vez, la primera pregunta no era *"¿dónde está la regla N?"*. Era *"¿qué hay aquí?"*. Y un `ls` recursivo no responde esa pregunta — la abruma. Un `grep` requiere saber qué buscar. Un editor abre un archivo a la vez. El framework había llegado al tamaño donde su propia superficie se había vuelto opaca.

`straymark explore` nació exactamente para resolver eso: una superficie navegable que respondiera *"¿qué hay aquí?"* en tres segundos. No un IDE; no un sistema de docs estilo Read-the-Docs; no un site estático. Un TUI — terminal user interface — que se ejecuta sobre el repo del adoptante, indexa los archivos canónicos del framework, los muestra agrupados por categoría, y permite leerlos con un visor de Markdown coloreado dentro de la terminal.

La decisión de hacer TUI y no web está alineada con dos cosas que el blog ya argumentó. Primero, el principio #10 del Post 4 — *"StrayMark no es un LLM gateway"* — generaliza a *"StrayMark no es lo que ya hace otra cosa"*. No competimos con MkDocs ni con Material for MkDocs. Segundo, la decisión A1 del mismo Post 4: orchestration-only. El TUI no requiere servidor, no requiere build paso, no requiere conexión: corre sobre el repo del adoptante, donde ya está el framework. Es la pieza más humilde que podía resolver el problema.

---

## 3. Lo que entró en `cli-3.4.0`

El [PR #57](https://github.com/StrangeDaysTech/straymark/pull/57) del 25 de abril, fusionado como `cli-3.4.0`, hizo tres cosas concretas:

- Introdujo el comando `straymark explore`. Layout de dos paneles cuando la terminal tiene 100 columnas o más: navegación a la izquierda (30%), documento a la derecha (70%). Paneles más estrechos colapsan a un solo panel para terminales angostas. Status bar al pie con los atajos relevantes.
- Indexó los archivos canónicos del framework en una jerarquía navegable: governance docs (`AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, ...), templates (`dist/.straymark/templates/`), audit prompts, y los directorios del adoptante (`docs/charters/`, `.straymark/07-ai-audit/`, ...). Cada nodo expansible con `Enter`.
- Cableó el resolver i18n. La opción `--lang <code>` permite ejecutar `straymark explore --lang es` y obtener todos los governance docs en español *si* hay traducción disponible. Si no la hay, fallback silencioso al canónico EN. La descripción literal del commit:

> *"Wire `language` config and a new `--lang` flag through the explore TUI so framework governance docs are served from `i18n/<lang>/` when a translation exists, falling back silently to English otherwise."*

La parte del resolver i18n importa más que el TUI en sí. Hasta `cli-3.4.0`, el helper `resolve_localized_path` lo usaba solo `straymark new` (para resolver qué template inyectar al adoptante en el idioma correcto). El PR #57 lo extrajo a `cli/src/utils.rs:146` como helper compartido, de modo que **una sola definición de cómo se resuelven los overlays `i18n/<lang>/`** sostiene tanto el comando `new` como el comando `explore`. El comportamiento queda predecible para el adoptante: si traduce un template a `i18n/es/`, el TUI lo va a mostrar igual que la generación lo va a usar. Sin sorpresas.

El renderizado del Markdown usa `pulldown-cmark` para el parser y los widgets de `ratatui` para pintar. Sintaxis coloreada, code blocks con borde, tablas con bordes redondeados, headings indentados según nivel. No es ostentoso; es legible.

---

## 4. Cuatro refinamientos en treinta y seis horas

Lo que pasó entre el 25 y el 27 de abril es lo que el Post 9 nombró meses después como propiedad del proceso: cuando la propuesta es bien escrita, la implementación es ejecución. Cuatro PRs más, cerrados en menos de 36 horas, llevaron el TUI de "funciona" a "está terminado":

| PR | Tag | Hora | Qué agregó |
|---|---|---|---|
| [#60](https://github.com/StrangeDaysTech/straymark/pull/60) | cli-3.5.0 | 25 abr 22:36 | **Live language switcher.** Tecla `L` cicla el idioma del display sin salir del TUI: `en → es → zh-CN → en`. El índice se reconstruye in-place. |
| [#61](https://github.com/StrangeDaysTech/straymark/pull/61) | cli-3.5.0 (mismo release) | 25 abr 23:41 | **OS locale auto-detect.** Si el adoptante no tiene `config.yml`, el TUI lee `$LC_ALL` / `$LANG` y mapea el POSIX locale (e.g. `es_MX.UTF-8` → `es`) al idioma soportado más cercano. |
| [#62](https://github.com/StrangeDaysTech/straymark/pull/62) | cli-3.5.1 | 26 abr 00:07 | **Metadata panel traducido.** 25 nuevas entradas i18n para labels y títulos, con padding visual para mantener alineación entre idiomas. |
| [#63](https://github.com/StrangeDaysTech/straymark/pull/63) | cli-3.5.2 | 26 abr 00:13 | **Cleanup de keybindings.** Elimina los aliases vim `l` y `h` no documentados (la `l` chocaba con `L` del language switcher). Mantiene `j/k/g/G/n/N` documentados. |

La velocidad del arco repite el patrón. Los cinco PRs de fw-4.11.0 que cubrió el Post 6 (rebrand a StrayMark) se cerraron en cuarenta y tres minutos. Los tres releases del audit cycle externo que cubrió el Post 4 se cerraron en un día. La curva del TUI es parecida: el diseño quedó claro en el PR #57, y los refinamientos vinieron cuando el operador empezó a usar el comando con frecuencia y notó las fricciones.

La anécdota que vale la pena dejar registrada es el `cli-3.5.2`: las teclas `l` y `h` que eliminó eran aliases vim — el operador las usa de hábito muscular en su editor. Pero cuando se introdujo el language switcher en mayúsculas (`L`) horas antes, la `l` minúscula y la `L` mayúscula compartían la misma tecla física, y la transición visual era confusa. La solución fue eliminar los aliases no documentados: el ortografía completa (`j/k/g/G`) se mantiene; las abreviaturas que duplicaban funciones desaparecen. Es un detalle UX pequeño que ilustra cómo las decisiones del TUI van afinando contra el uso real, no contra teoría de UX.

---

## 5. Lo que la TUI reveló dos semanas después

Aquí el post recoge el hilo del Post 8 desde el otro extremo. Lo que `cli-3.4.0` añadió al framework no fue solo un comando de navegación; fue una **superficie de visualización simultánea**. Es decir: antes del TUI, los archivos del framework existían pero se leían uno a uno. Después del TUI, podían verse todos a la vez, agrupados, etiquetados, en el mismo modo de presentación.

Eso cambia lo que el operador puede notar.

El 12 de mayo, dos semanas y media después del merge de `cli-3.4.0`, el operador ejecutó `straymark explore --lang es` antes de un ciclo de auditoría externa. Llegó al grupo de "audit prompts". Lo abrió. Y notó algo que llevaba existiendo un año: el `audit-prompt.md` en root del directorio estaba escrito *en español*, mientras todos los demás archivos canónicos del framework tenían inglés en root y overlays `i18n/es/`. Un solo archivo, al revés respecto a la convención.

El detalle no se descubrió por revisión sistemática. Se descubrió por contraste visual. Es exactamente lo que el Post 8 §3 codificó como tesis:

> *"Las herramientas nuevas producen visibilidad."*

El TUI no causó el outlier — el outlier existía desde el primer commit que portó el skill `plan-audit` de Sentinel al framework canónico (Post 3 lo registra). El TUI tampoco buscó el outlier; no es una herramienta de detección de inconsistencias. Lo único que hizo fue poner todos los archivos en la misma pantalla, en el mismo modo de presentación, y permitir que el ojo humano notara lo que había estado ahí desde el principio. La conclusión que me interesa registrar es estructural: cualquier framework que pase de cincuenta archivos canónicos necesita una superficie de visualización simultánea, no porque la superficie sea pedagógicamente importante en sí, sino porque sin ella ciertas regularidades del repo se vuelven inobservables.

---

## 6. Pequeñas decisiones técnicas que importan

Tres detalles del diseño del TUI que vale la pena dejar registrados, todos coherentes con principios del framework que el blog ya nombró:

**Feature flag `tui`.** El `Cargo.toml` del CLI declara el TUI como dependencia opcional (`ratatui` + `crossterm` + `pulldown-cmark` viven detrás del flag `tui`, habilitado por defecto). Un adoptante que solo quiere `init`, `update`, `validate` puede compilar el binario sin TUI con `cargo build --no-default-features`. Cierra una sub-decisión arquitectónica que se repite en el framework: dar valor al adoptante por defecto, pero permitirle elegir un binario más liviano cuando el TUI no aporta nada al flow.

**Lazy-load de documentos.** El `DocIndex` mantiene un `HashMap<PathBuf, Document>` que se llena al abrir cada nodo, no al arrancar. Para un repo con cien archivos governance, esto significa que `straymark explore` arranca en menos de 200ms y solo lee disco cuando el operador pulsa `Enter`. La decisión es trivial técnicamente; lo no trivial es que se tomó conscientemente. El TUI no compite con un IDE; compite con un `cat`. Tiene que sentirse igual de inmediato.

**History stack para hipervínculos.** Cuando un documento del framework menciona a otro (e.g., `AGENT-RULES.md §3` refiere a `DOCUMENTATION-POLICY.md §6`), el TUI permite saltar al referenciado con un clic — y volver al anterior con `Esc`. Internamente hay una pila de navegación que restaura el scroll position del documento anterior. Es la pieza que convierte la lectura del framework de *"abro un archivo, lo cierro, abro otro"* a *"sigo una conversación entre documentos sin perder el hilo"*. La cantidad de cross-references que el framework tiene hoy — entre principios, agent rules, schemas, templates — hace que esta navegación sea estructuralmente importante, no cosmética.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Un framework que pasa cierto tamaño necesita una superficie de visualización simultánea.** No es ergonomía; es condición para que ciertas regularidades del repo sean observables. El Post 8 documentó la consecuencia; este post documenta la herramienta.

2. **El TUI no causa los descubrimientos; los permite.** El outlier del audit-prompt existía desde mayo del año anterior. Lo único que cambió en abril fue que apareció una pantalla donde se podía ver junto a sus pares. La herramienta no opina sobre el contenido; cambia las condiciones bajo las cuales el ojo humano puede notar regularidades.

3. **Cuando el diseño está claro, la implementación es cuestión de horas, no de sprints.** Cinco PRs en treinta y seis horas — language-aware, live switcher, OS detection, i18n del panel, cleanup de keybindings — solo son posibles porque el diseño quedó cerrado en el PR #57. Es el mismo patrón de los Posts 4, 6 y 9.

4. **La decisión técnica más interesante puede ser una flag de compilación.** El feature flag `tui` declara explícitamente que el TUI es opcional, que el framework sigue funcionando sin él, que el adoptante decide. Esa humildad arquitectónica es la diferencia entre un CLI que asume cómo el adoptante trabaja y un CLI que se ofrece donde puede ser útil.

Con este post el blog cubre el hito `H-14` que la primera tanda dejó como deuda explícita. Los hitos candidatos restantes que `PLAN-INVESTIGACION.md §1.43-55` registró — `AGENTS.md` universal inject, cobertura i18n completa, `straymark validate` como capa formal, CLA assistant — siguen disponibles para futuras tandas. Sin promesa de cadencia, pero registrados.

---

*Anclas: PR [#57](https://github.com/StrangeDaysTech/straymark/pull/57) — `cli-3.4.0` (language-aware explore, 25 abr 2026). PRs [#60](https://github.com/StrangeDaysTech/straymark/pull/60) · [#61](https://github.com/StrangeDaysTech/straymark/pull/61) · [#62](https://github.com/StrangeDaysTech/straymark/pull/62) · [#63](https://github.com/StrangeDaysTech/straymark/pull/63) — refinamientos `cli-3.5.0` a `cli-3.5.2` (25-26 abr 2026). Código: `cli/src/tui/` (15 archivos). Doc adopter: `docs/adopters/CLI-REFERENCE.md` §`straymark explore`.*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*
