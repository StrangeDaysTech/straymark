---
slug: agents-md-como-estandar-universal
title: AGENTS.md como estándar universal
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - agents-md
  - governance
  - estándares-abiertos
  - i18n
date: 2026-05-15T00:00:00.000Z
description: Cuatro meses entre la intuición y el estándar abierto
---
*Cuatro meses entre el primer AIDEC del proyecto — "los agentes de IA son una única audiencia técnica" — y el estándar abierto que lo operacionalizó. `AGENTS.md` en la raíz del repo, leído por quince CLIs de distintos vendors, reemplazando la proliferación de CLAUDE.md / GEMINI.md / .cursorrules y una docena más.*

<!-- truncate -->

## 1. Cuatro meses entre la intuición y el estándar

El 15 de mayo de 2026, a las 19:05 UTC, el PR [#155](https://github.com/StrangeDaysTech/straymark/pull/155) cerró el framework como `fw-4.15.0 / cli-3.13.2`. Lo que trajo es un solo archivo nuevo en la lista de directivas que el CLI inyecta: `AGENTS.md`, en la raíz del repo del adoptante, paralelo a los que ya estaban (`CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`, `.cursorrules`, `.cursor/rules/straymark.md`).

Esto sería trivial si no fuera porque hace exactamente cuatro meses, el 27 de enero de 2026, el primer AIDEC del proyecto — el que el Post 2 cubrió con la anécdota del año tipográfico — había declarado:

> *"AI agents (Claude, Gemini, Copilot, Cursor) process instructions equally well in any language, so translating their config files provides no functional benefit."*

Esa frase, escrita seis horas después del *initial commit* del proyecto, contenía una intuición que el framework venía operativizando desde entonces: los agentes son un solo público técnico. Mantenían los archivos de configuración en inglés (mientras la documentación humana se traducía a tres idiomas) porque la audiencia "agentes IA" no se beneficiaba de la traducción. Lo que faltaba para cerrar esa intuición era darle un solo lugar al que apuntar, sin tener que clonar el archivo cuatro o cinco veces para cada vendor distinto.

Este post cubre exactamente eso: cómo `AGENTS.md` — un estándar abierto donado a la Agentic AI Foundation en diciembre de 2025 — completó cuatro meses después la decisión que el primer AIDEC había abierto en enero.

---

## 2. La fragmentación que `AGENTS.md` resuelve

Entre 2024 y 2026, mientras los CLIs de agentes IA proliferaban, cada uno inventó su propio formato de archivo de instrucciones:

- Anthropic publicó `CLAUDE.md` con la primera versión de Claude Code.
- Google adoptó `GEMINI.md` para Gemini CLI.
- GitHub estableció `.github/copilot-instructions.md` para Copilot CLI.
- Cursor empezó con `.cursorrules` y migró luego a `.cursor/rules/*.md`.
- OpenAI usó otro path para Codex CLI; Aider, Devin, Continue, Roo Code, Factory Droids, Sourcegraph Amp, Zed AI, Windsurf, Amazon Q tenían cada uno el suyo.

Un adoptante de StrayMark que usara dos o tres de esos CLIs tenía que mantener dos o tres copias del mismo contenido, sincronizadas a mano, esperando que ninguna divergiera. El framework hasta `fw-4.14.x` reconocía esa fragmentación en su comando `straymark init`: inyectaba en los cinco archivos vendor-específicos más comunes. Pero los otros diez o quince CLIs quedaban fuera. El adoptante los manejaba copiando contenido a mano.

A finales de 2025, la comunidad de proyectos open-source que producía CLIs de agentes hizo lo que se hace cuando una fragmentación se vuelve costosa: acordaron un estándar. `AGENTS.md` en la raíz del repo, leído por todos. La donación a la Agentic AI Foundation (Linux Foundation, diciembre 2025) le dio gobernanza institucional. La especificación es deliberadamente mínima: un archivo Markdown, sin schema impuesto, en una ubicación predecible. Lo que cada CLI hace con ese archivo es asunto suyo; la pregunta canónica — *"¿dónde está la configuración de agentes para este repo?"* — tiene ahora una sola respuesta.

Para mayo de 2026, la lista de CLIs que leen `AGENTS.md` incluye (cita literal del cuerpo del PR #155):

> *"Claude Code, OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue, Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q and others."*

Quince CLIs. Algunos siguen leyendo *también* sus archivos vendor-específicos por compatibilidad histórica. Pero los quince saben buscar `AGENTS.md` primero.

---

## 3. Adoptar sin matar lo propio

La decisión más interesante de `fw-4.15.0` no es adoptar `AGENTS.md`; eso era inevitable una vez que el estándar tenía masa crítica. Lo interesante es *cómo*. El cuerpo del PR lo articula sin atenuar:

> *"Parallel to CLAUDE.md / GEMINI.md: marker block points to STRAYMARK.md, with a minimum-viable body below for readers that cannot follow relative links."*

Tres palabras clave: **parallel**, **marker block**, **minimum-viable body**.

*Parallel* significa que `AGENTS.md` no reemplaza los archivos vendor-específicos. Coexiste. El framework sigue inyectando `CLAUDE.md`, `GEMINI.md`, `.cursorrules` y los demás. La razón es pragmática: algunos CLIs (notablemente Cursor en su versión legacy) requieren que el contenido completo esté embebido, no detrás de un link relativo. Mantener los archivos hermanos preserva esa compatibilidad sin que cada adoptante tenga que pelear con su CLI específico.

*Marker block* es la convención que el framework ya usaba en los demás archivos vendor-específicos: un bloque HTML-commented entre `<!-- straymark:begin -->` y `<!-- straymark:end -->` que apunta a `STRAYMARK.md` como source-of-truth. Cualquier `straymark update-framework` reemplaza solo lo que está entre esos marcadores. Lo que el adoptante haya escrito *afuera* de ellos — por ejemplo, instrucciones propias del proyecto — se preserva intacto. El framework respeta el archivo del adoptante; solo gobierna su propia sección.

*Minimum-viable body* es la concesión a los CLIs que no siguen relative links. Debajo del bloque de marcadores, `AGENTS.md` tiene un cuerpo corto con lo esencial: identidad del agente, requisitos de revisión, checklist pre-commit, snippet de frontmatter regulatorio, categorías de riesgo NIST AI 600-1, reglas de observabilidad, convención de nombrado. Es suficiente para que un agente que no pueda leer `STRAYMARK.md` opere correctamente; es insuficiente para reemplazar al documento canónico. La asimetría es deliberada: queremos que el agente *quiera* abrir `STRAYMARK.md`.

El detalle defensivo del CLI (`cli/src/commands/remove.rs:13`) cierra el flanco operativo: `AGENTS.md` se agregó a `LEGACY_DIRECTIVE_TARGETS`. Si un adoptante pierde su `.straymark/dist-manifest.yml` y ejecuta `straymark remove`, el fallback legacy limpia también `AGENTS.md` en lugar de dejarlo huérfano. Es housekeeping aburrido, pero es la diferencia entre un framework que se desinstala completamente y uno que deja basura.

---

## 4. Sin ADR

Vale la pena nombrar una ausencia. La decisión de adoptar `AGENTS.md` no tuvo ADR propio.

Es un contraste deliberado con el rebrand a StrayMark del Post 6, que sí tuvo `ADR-2026-05-08-001` público con tres alternativas evaluadas y consecuencias enumeradas. Es también contraste con Phase 2 del Post 12, que tuvo una entrada de CHANGELOG densa con justificación arquitectónica. `fw-4.15.0` tuvo solo el cuerpo del PR como justificación documental — un cuerpo bueno, con argumento claro, pero no un ADR.

¿Por qué? La respuesta operativa: la decisión se percibió como **agregativa**, no estructural. Adoptar un estándar abierto que coexiste con los formatos existentes no cambia el modelo del framework; solo lo extiende. No hay alternativa cuyo descarte requiera justificación formal: no adoptar `AGENTS.md` significaría dejar fuera a 15 CLIs por preferencia estética, y nadie tenía ese argumento que hacer.

Pero hay una lección registrable aquí. El blog ha visto el patrón antes — Phase 2 cierra un loop empírico, el rebrand cambia identidad, los meta-patrones del Post 10 nombran lo que ya estaba — y en cada caso la pregunta *"¿esto amerita ADR?"* tiene una respuesta distinta. La regla operativa que el framework parece haber adoptado, sin nombrarla, es esta: **un ADR se justifica cuando la decisión cierra alternativas costosas y deja registro de las que no se eligieron**. `fw-4.15.0` no cerró alternativas costosas — solo agregó una capa. Por eso bastó con el PR.

Lo dejo registrado aquí porque vale la pena reconocer que no todo cambio merece ADR, ni todo PR es un documento histórico. La disciplina archivística tiene sus propios criterios.

---

## 5. Visibilidad para agentes, dos capas

Diez días antes de `fw-4.15.0`, el 9 de mayo, se cerró el Issue [#113](https://github.com/StrangeDaysTech/straymark/issues/113) — el episodio que el Post 5 documentó como *"Charters invisibles para los agentes"*. El operador había trabajado seis horas con un agente capaz, atento, con el aparato canónico de onboarding cargado, y el agente nunca había propuesto usar un Charter. La conclusión que aquel post canonizó: la *visibilidad estructural* es responsabilidad del framework, no propiedad del agente.

`fw-4.15.0` cubre la otra cara del mismo problema.

El Post 5 trataba de **visibilidad estructural interna**: cómo aseguramos que un agente que ya está leyendo nuestro repo descubra los conceptos centrales del framework. La respuesta del PR #122 (Post 5) fue multiplicar las superficies internas donde Charter aparece como concepto: `STRAYMARK.md §15`, directivas `CLAUDE.md`/`GEMINI.md`, skills `/straymark-*`, output de `status`, templates, puente con SpecKit. Nueve superficies internas.

Este post trata de **visibilidad estructural externa**: cómo aseguramos que cualquier agente que entra al repo, sin importar su vendor, encuentre el aparato del framework. La respuesta de `fw-4.15.0` es adoptar el punto de encuentro canónico que la comunidad eligió. Una superficie externa, leída por quince vendors.

Las dos capas tienen el mismo eje conceptual — *un artefacto técnico solo existe para el agente si la superficie lo nombra* — pero distinto alcance. Visibilidad interna garantiza que el agente vea los Charters cuando está leyendo `STRAYMARK.md`. Visibilidad externa garantiza que el agente vea `STRAYMARK.md` cuando entra al repo. Una capa sin la otra es incompleta.

---

## 6. Lo que el adoptante hace al ejecutar `update-framework`

La parte operativa del release, citada literal del CHANGELOG:

> *"straymark update-framework brings the new template and injects AGENTS.md at the project root. The injection follows the same rules as every other directive target: it creates the file if absent, replaces the marker block on subsequent runs, and appends safely when the file pre-exists without StrayMark markers (very common in 2026 — many adopters already hand-maintain an AGENTS.md)."*

Tres comportamientos importantes:

1. **Si no existe**, el CLI lo crea con el template canónico.
2. **Si ya existe y tiene marcadores StrayMark**, el CLI reemplaza solo el contenido entre marcadores.
3. **Si ya existe sin marcadores** — caso muy común en 2026, porque muchos adoptantes ya escribían su propio `AGENTS.md` a mano — el CLI **agrega sus marcadores al final del archivo**, sin tocar lo que el adoptante había escrito.

Es el mismo respeto archivístico que el blog ya documentó en Posts 3 y 6: el framework gobierna *su sección*, no el archivo entero. Si el adoptante quiere escribir instrucciones propias del proyecto en su `AGENTS.md` — políticas internas, convenciones de naming, instrucciones específicas para una codebase particular — esas instrucciones quedan intactas en cada `update`. El bloque de StrayMark vive en su pequeña frontera marcada, y se actualiza dentro de esa frontera.

La nota del CHANGELOG cierra con una pequeña advertencia operativa que vale la pena registrar: *"If your .gitignore excludes AGENTS.md, adjust it before update-framework so the injection lands in version control."* Es honesta sobre los edge cases que el framework no puede resolver por su cuenta — la elección del adoptante de ignorar el archivo es soberanía, no error.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Cuatro meses entre intuición y materialización es lo que toma cerrar bien un loop.** El AIDEC de enero contenía la intuición — *"los agentes son un solo público"*. `fw-4.15.0` la cerró con un estándar abierto. Entre las dos, el framework iteró sobre arquetipos vendor-específicos hasta que apareció el sustituto universal. La intuición temprana fue correcta; el plazo fue el que la maduración del ecosistema requería.

2. **Adoptar un estándar abierto no exige renunciar al formato propio.** `AGENTS.md` coexiste con `CLAUDE.md`, `GEMINI.md`, `.cursorrules`. La decisión no es "uno reemplaza a los demás" sino "uno se suma a los demás cuando aporta". El framework gana visibilidad externa sin perder compatibilidad con CLIs que requieren formatos específicos.

3. **No todo cambio amerita ADR.** El rebrand a StrayMark cerró alternativas costosas y dejó registro formal de las descartadas. `fw-4.15.0` solo agregó una capa universal: ningún ADR habría enriquecido la decisión. La disciplina archivística tiene criterios operativos, no doctrinas.

4. **Visibilidad para agentes opera en dos capas.** La capa interna (Post 5, `fw-4.12.0`) asegura que el agente vea los conceptos del framework cuando está leyendo el repo. La capa externa (`fw-4.15.0`) asegura que el agente entre al repo por la puerta correcta sin importar qué CLI lo orqueste. Una sin la otra deja agentes capaces leyendo en silencio o agentes desorientados sin punto de entrada.

---

*Anclas: PR [#155](https://github.com/StrangeDaysTech/straymark/pull/155) — `fw-4.15.0 / cli-3.13.2` (fusionado 2026-05-15 19:05 UTC). AIDEC original (Post 2): `.chronicle/07-ai-audit/decisions/AIDEC-2025-01-27-001-i18n-strategy.md` (commit `7b7193e`, ID con año tipográfico). Template inyectado: `dist/dist-templates/directives/AGENTS.md`. Especificación del estándar: [agents.md](https://agents.md) — donado a la Agentic AI Foundation (Linux Foundation), diciembre 2025.*

*Co-escrito por José Villaseñor Montfort (Strange Days Tech) y Claude Opus 4.7 (1M context).*
