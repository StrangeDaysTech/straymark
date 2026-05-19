---
slug: abrir-el-framework
title: Abrir el framework
authors:
  - jose
tags:
  - straymark
  - i18n
  - cla
  - governance
  - apertura
date: 2026-05-15T00:00:00.000Z
description: Setenta y dos horas de gestos que sólo cobran sentido juntos
---
*Dos gestos en setenta y dos horas: cerrar el último 5% de la paridad i18n (20/20/20 docs de gobernanza en EN + ES + zh-CN) y publicar el badge de CLA en el README. Pequeños por separado; juntos, el momento en que el framework deja de hablarse a sí mismo y empieza a hablarle a cualquiera que quiera leerlo.*

<!-- truncate -->

## 1. Dos gestos en setenta y dos horas

El 13 de mayo de 2026, a las 06:20 UTC, se fusionó `fw-4.13.4`. Cerró dos huecos pequeños de cobertura i18n — el documento de referencia ISO-25010 que solo estaba en inglés, y el template del Charter que faltaba en chino simplificado. Veintidós archivos canónicos del framework alcanzaron paridad estructural EN/ES/zh-CN.

El 15 de mayo, a las 00:14 UTC, sesenta y cinco horas después, se fusionó el PR [#154](https://github.com/StrangeDaysTech/straymark/pull/154). Agregó un badge al README — el de CLA Assistant, ese cuadrito visible que avisa al visitante que el repo acepta contribuciones bajo un Contributor License Agreement. La política del CLA ya existía desde meses antes en `CONTRIBUTING.md`; lo único que cambió fue que ahora aparece en el README al primer vistazo.

Este post cubre los dos gestos. Vistos por separado, son pequeños — uno completa el último 5% de un trabajo técnico, el otro publica algo que ya estaba escrito. Vistos juntos, son el momento en que el framework decide presentarse al exterior. La pieza que un proyecto open-source hace cuando deja de hablar solo consigo mismo y empieza a hablar con quien quiera leerlo.

---

## 2. El último 5% de la cobertura i18n

`fw-4.13.4` no es un release grande. El PR [#144](https://github.com/StrangeDaysTech/straymark/pull/144) tiene 511 líneas de adición y casi todas son traducciones. Pero el resumen del PR dice algo que vale la pena registrar:

> *"Brings governance docs to full 20/20/20 parity across EN + ES + zh-CN."*

Veinte sobre veinte sobre veinte. Veintidós archivos canónicos en `dist/.straymark/00-governance/` (el conteo creció después; en mayo eran veinte), todos con su contraparte en `i18n/es/` y `i18n/zh-CN/`. Paridad completa, no aproximada.

Los dos archivos que cerraron la paridad son específicos:

- `ISO-25010-2023-REFERENCE.md` — la referencia normativa al estándar internacional de calidad de software, citada desde principios del framework. Hasta `fw-4.13.4` existía solo en inglés. El release añadió `i18n/es/ISO-25010-2023-REFERENCE.md` y `i18n/zh-CN/ISO-25010-2023-REFERENCE.md`, 150 líneas cada uno.
- `charter-template.md` en zh-CN. El template del Charter ya estaba traducido al español; faltaba el chino simplificado, 211 líneas.

No es un trabajo glamoroso. Es la pieza que cierra un cuadro. Pero importa por dos cosas. La primera: hasta ese release, un adoptante chino que clonara el framework iba a encontrar templates esenciales en inglés solamente, lo cual contradice la promesa de que el framework está disponible en tres idiomas. La segunda: el blog ya documentó en Post 8 que la cobertura i18n del framework era casi completa, con dos gaps menores quedando como deuda anotada. `fw-4.13.4` cerró esa deuda. Lo que parecía una nota lateral en un post sobre el outlier del audit-prompt resultó ser un release entero, con su propia entrada en CHANGELOG.

---

## 3. Lo que se traduce y lo que no

Hay una regla operativa que el PR #144 codifica explícitamente y que vale la pena dejar registrada, porque es la regla más clara del framework respecto a i18n:

> *"LLM-processed assets (skills, workflows, schemas) stay EN-only; human-primary artifacts get translated."*

Es decir: lo que leen los agentes IA — skills, workflows en `.claude/`, `.gemini/`, `.agent/`, schemas en `dist/.straymark/schemas/`, prompts en `dist/.straymark/audit-prompts/` — vive solo en inglés. Lo que leen los humanos — documentación, principios, guías de contribución, templates con copy human-facing — se traduce a los tres idiomas.

Es el mismo principio que el Post 13 documentó como ancestro intelectual del AGENTS.md universal: los agentes son un solo público técnico que no se beneficia de traducción. Lo registrable aquí es que la regla, en mayo, está escrita en un PR como criterio operativo, no como intuición. Las propuestas técnicas la citan; los reviewers la usan para decidir si un archivo nuevo va a `i18n/` o no.

Un detalle adicional del PR #144 vale la pena registrar literal:

> *"docs/contributors/TRANSLATION-GUIDE.md gap was left intentional — its target audience already reads English to read the guide."*

La guía de traducción quedó intencionalmente en inglés porque su audiencia — los contribuidores que traducen el framework a otros idiomas — necesariamente lee inglés ya. Es honestidad operativa: la paridad 20/20/20 no es 22/22/22 para todos los archivos del repo. Es 20/20/20 para los archivos cuya traducción aporta valor al usuario. La diferencia entre cobertura total y cobertura aplicable.

---

## 4. El badge que no introdujo nada nuevo

PR #154 es el opuesto pedagógico de `fw-4.13.4`. Donde `fw-4.13.4` cierra un trabajo técnico, PR #154 publica algo que ya estaba ahí. Sus 12 líneas de cambio agregan exactamente esto al README (en inglés, español y chino):

```markdown
[![CLA assistant](https://cla-assistant.io/readme/badge/StrangeDaysTech/straymark)](https://cla-assistant.io/StrangeDaysTech/straymark)
```

Un badge. Una imagen pequeña que aparece arriba del README, junto a los demás badges del proyecto (licencia, versión, downloads). Visualmente nada del otro mundo.

Pero el badge no es decoración. Es señal. Y la señal dice algo concreto: *"este repo acepta contribuciones externas, y aquí están las reglas"*. Cualquier visitante que abra el README puede hacer clic en el badge y llegar al CLA Assistant — el servicio que automáticamente comenta en el primer pull request de cualquier contribuidor, pidiendo que firme el Contributor License Agreement. Una sola firma cubre todas las futuras contribuciones al proyecto.

Lo importante es que el CLA, el `CONTRIBUTING.md`, la política completa de revisión, todo eso ya existía antes de PR #154. Estaba codificado, vivo, operativo. El framework ya aceptaba contribuciones externas — solo que el visitante casual del repo tenía que abrir `CONTRIBUTING.md` para enterarse de cómo funcionaba.

PR #154 hace una distinción que vale la pena nombrar: **tener la política** y **publicar la política** son dos cosas distintas. La primera es disciplina interna; la segunda es invitación pública. Hasta el 15 de mayo, el framework tenía la política. Desde el 15 de mayo, la publica.

Es un cambio editorial, no arquitectónico. Pero el día que un proyecto open-source decide poner el badge de CLA Assistant arriba del README es el día que decide presentarse formalmente como adoptable por terceros. Es el momento en que el framework deja de ser un proyecto-de-uno-y-su-adopter (Sentinel) y se vuelve un proyecto que *acepta* tener más adopters, más contribuidores, más manos.

---

## 5. Cuándo un framework se siente listo para ser visto

Estos dos hitos, leídos juntos, registran un momento específico en la vida de un framework joven: el momento en que se siente listo para ser visto por gente que no lo escribió.

`fw-4.13.4` cierra la promesa técnica. Si el framework dice ser bilingüe español-inglés con soporte experimental para chino simplificado, los veintidós archivos canónicos deben estar en los tres idiomas. No diecinueve sobre veinte. No veinte sobre veintiuno con dos pendientes. Veinte sobre veinte sobre veinte. La paridad completa es lo que vuelve la promesa creíble cuando el visitante revisa.

PR #154 cierra la promesa social. Si el framework dice aceptar contribuciones externas, el badge del CLA en el README es la forma estándar de la industria de decirlo. Sin el badge, la apertura existe pero hay que descubrirla; con el badge, está señalada. El primer click del visitante encuentra el camino.

Vale la pena nombrar la asimetría de los dos gestos. Cerrar el último 5% de un trabajo técnico cuesta exactamente eso — un release dedicado, dos archivos traducidos, una entrada de CHANGELOG. Publicar una política preexistente cuesta doce líneas de Markdown. El cuesta-poco es a veces el gesto operativamente más significativo, porque cambia cómo el mundo exterior percibe el proyecto sin cambiar nada del proyecto en sí.

Es la versión "abrir el framework" del patrón que el Post 5 documentó como *visibilidad estructural*: lo que existe pero no está nombrado, no existe para el agente. Aquí: lo que existe pero no está publicado, no existe para el adoptante externo. Setenta y dos horas en mayo, dos hitos pequeños — uno técnico, uno editorial — y el framework cruza la línea entre *"funciona pero quien lo escribió lo sabe"* y *"funciona y cualquiera puede verlo"*.

---

## 6. Cierre simbólico del arco

Con este post, el blog cubre los últimos hitos pendientes de la segunda tanda que `RETOMAR-AQUI.md §5` listó después del cierre del arco principal H-01 → H-13. Los cuatro candidatos identificados — `H-?-C` (validate + schemas), `H-?-A` (AGENTS.md universal), `H-?-B` (cobertura i18n completa), `H-?-E` (CLA badge) — están todos cubiertos: tres en posts propios (Posts 12, 13, este) y uno combinado.

Lo que aprendí escribiendo la segunda tanda, mirando los cuatro hitos juntos:

- **Phase 2 / validate** (Post 12) es estructural: el framework empieza a verificar lo que prometía.
- **TUI explore** (Post 11) es de visibilidad: el framework se vuelve navegable.
- **AGENTS.md universal** (Post 13) es de alcance: el framework se vuelve descubrible por cualquier CLI.
- **Cobertura i18n completa + CLA badge** (este post) es de apertura: el framework se vuelve visible al visitante externo.

Cada uno cubre una dimensión distinta de la misma idea, que el blog ya dejó codificada en posts anteriores: lo que un framework dice que es no basta para que sea; tiene que estar presente en cada superficie donde alguien — agente, adopter externo, visitante — pueda construir un modelo del proyecto. Los hitos de la segunda tanda son la versión madura de la *visibilidad estructural* que el Post 5 nombró con un Issue de gobernanza.

Ahora sí, después de Post 14, no hay candidatos más en la cola. Si emergen nuevos hitos del framework entre hoy y cuando la próxima sesión del blog se active, RETOMAR-AQUI volverá a usarse como punto de retomo. Si no emergen, este post es el cierre real — el cierre con el que el blog completa el inventario que se propuso cubrir.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Cobertura total cuesta poco al final del trabajo y mucho al principio.** El último 5% — esos dos archivos en chino — son release dedicado. Pero solo son trivialmente cerrables porque el otro 95% ya estaba hecho. La paridad 20/20/20 no nace al principio; se persigue durante meses y se cierra al final con un PR pequeño.

2. **Tener una política y publicarla son dos cosas distintas.** El CLA del framework existía meses antes que el badge. Doce líneas de Markdown convirtieron una política interna en una invitación pública. El día que pones el badge no inventas nada; declaras que estás listo para que te encuentren.

3. **La regla operativa de qué se traduce es archivística.** Lo que leen humanos se traduce; lo que leen agentes vive en inglés. Esa distinción, intuida desde el primer AIDEC de enero, se vuelve criterio explícito de revisión en mayo. Cada archivo nuevo del framework pasa por esa pregunta antes de elegir destino.

4. **Cuándo un framework se vuelve adoptable por terceros es decisión, no estado.** Hay un día en el que el operador decide que el framework ya puede ser visto sin pena por gente que no lo escribió. Ese día se materializa en gestos pequeños — completar traducciones, publicar el badge — que no cambian la funcionalidad pero cambian la postura. El framework deja de hablar consigo mismo.

Con esto el blog cierra la segunda tanda. El arco que arrancó retroactivamente con la prehistoria del proyecto (Post 2, enero) y se completó con los meta-patrones (Post 10, mayo) tiene ahora también su corolario: los hitos que volvieron al framework adoptable. Catorce posts, veintiocho archivos (catorce ES + catorce EN), entre veinticinco y veintiséis mil palabras por idioma. Como decía el cierre del Post 10: *el blog termina de contar la historia. El framework sigue*.

---

*Anclas: PR [#144](https://github.com/StrangeDaysTech/straymark/pull/144) — `fw-4.13.4` (fusionado 2026-05-13 06:20 UTC, cobertura i18n completa). PR [#154](https://github.com/StrangeDaysTech/straymark/pull/154) — badge CLA Assistant en READMEs (fusionado 2026-05-15 00:14 UTC). Política completa: `CONTRIBUTING.md` (raíz del repo + traducciones en `docs/i18n/{es,zh-CN}/CONTRIBUTING.md`).*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*

*— Fin de la segunda tanda del blog StrayMark.*
