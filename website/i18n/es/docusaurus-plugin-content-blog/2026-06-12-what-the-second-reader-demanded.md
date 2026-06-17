---
slug: what-the-second-reader-demanded
title: Lo que el segundo lector exigió
authors:
  - jose
tags:
  - straymark
  - loom
  - knowledge-graph
  - architecture
  - rust
  - cli
  - sentinel
date: 2026-06-12T00:00:00.000Z
description: Antes de que Loom pudiera dibujar nada, el modelo de documentos de StrayMark tenía que dejar de ser propiedad privada del CLI. El refactor que entregó cero cambios visibles para el usuario fue la precondición de todo lo visual que vino después — y una pequeña lección que el framework venía predicándoles a los adopters desde siempre.
---
*Los documentos de StrayMark siempre han formado un grafo — cada enlace `related`, `supersedes`, `originating_ailogs` es una arista. El CLI construía ese grafo internamente para `straymark audit`, y un humano podía leerlo de a un documento por vez en la TUI de `explore`. Entonces apareció un segundo consumidor — Loom, una vista experimental en el navegador de todo el corpus — y pidió parsear los mismos documentos. La respuesta honesta a "¿puede reutilizar el parser del CLI?" era no, porque el parser no era una librería; estaba enterrado en `cli/src/document.rs`. El arreglo salió como `cli-3.23.1` con **cero cambio de comportamiento visible para el usuario**. Fue además el release más importante del mes.*

<!-- truncate -->

> *"No puedes tener dos parsers del mismo corpus."*

Esa frase es el post entero, pero hizo falta un componente nuevo para volverla concreta. Durante casi toda la vida de StrayMark hubo exactamente un programa que leía documentos de StrayMark: el CLI. Un parser, un consumidor, ninguna posibilidad de desacuerdo — porque no había nada con qué estar en desacuerdo. El grafo que enlaza Charters con AILOGs con TDEs con ADRs existía, pero vivía en la memoria privada del CLI, ensamblado bajo demanda para `straymark audit` y nunca mostrado entero a nadie.

Tres reviewers independientes del framework señalaron la misma brecha: a medida que un corpus crece — el adopter de referencia, [Sentinel](https://github.com/StrangeDaysTech/sentinel), es el caso que sigue aflorando estas cosas — leer la red de documentos de a una tarjeta por vez en una TUI deja de transmitir *forma*. No puedes ver el cluster, el documento puente, el huérfano, el ciclo. La respuesta iba a ser una vista gráfica. Y en el momento en que esa vista se volviera real, StrayMark tendría su primer segundo lector.

## Despejar la cubierta primero

Un componente nuevo es un frente nuevo. Abrir uno encima de asuntos sin terminar es como un proyecto acumula el tipo de deuda por la que después escribe posts de blog disculpándose. Así que la semana antes de la primera línea de código de Loom, se cerraron tres ítems diferidos — cada uno pequeño, cada uno en la lista de "ya debería haber hecho esto".

- **Skills instalados portables** ([`fw-4.24.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.24.0), [#232](https://github.com/StrangeDaysTech/straymark/issues/232)). Un review de Codex (`gpt-5.5`) sobre los propios skills de StrayMark corriendo bajo Codex CLI en Sentinel encontró cinco bugs de portabilidad — el más filoso siendo que los skills generados para Codex todavía escribían `agent: claude-code-v1.0` en el frontmatter que producían, *distorsionando la telemetría de procedencia en una herramienta cuyo trabajo entero es la procedencia*. Los skills ahora resuelven su propia identidad de runtime desde `AGENT-RULES.md §1` en vez de hardcodear una.
- **`charter close` cierra el loop de los follow-ups** ([`cli-3.22.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.22.0), [#135](https://github.com/StrangeDaysTech/straymark/issues/135) Tier 3). El último tier abierto del roadmap de follow-ups — cerrar un Charter ahora barre sus AILOGs en busca de trabajo diferido, extrae lo que no está en el registry, y ofrece promoción a TDE por entrada. Había estado gateado hasta que la detección de drift fuera confiable; lo era, así que salió.
- **`charter drift` ahora es Rust nativo** ([`fw-4.26.0` / `cli-3.23.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.23.0), [#237](https://github.com/StrangeDaysTech/straymark/issues/237)). El comando solía llamar a un script bash, lo que significaba que simplemente no corría en Windows sin WSL o Git Bash. Portar el set-difference declarado-vs-modificado in-process cerró la última brecha funcional de Windows-nativo — y borrar el intermediario que parseaba el stdout del script eliminó una clase entera de bugs. La suite de integración, que ya fijaba el comportamiento, ahora corre en cada plataforma y hace doble función como garantía de equivalencia-con-el-script.

Ninguno de estos es un titular. Juntos son la diferencia entre arrancar un componente nuevo desde una base limpia y arrancarlo desde una base de la que en silencio te avergüenzas.

## Un parser, o dos verdades

Entonces el pivote de verdad: [`cli-3.23.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.23.1), registrado en [`ADR-2026-06-02-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md), hito M0 de Loom.

La raíz del repositorio se volvió un **workspace** de Cargo (`core` + `cli`). El modelo de documentos — `DocType`, `Frontmatter`, `StrayMarkDocument`, `parse_document`, `discover_documents` — se mudó verbatim fuera de `cli/src/document.rs` a un crate nuevo, **`straymark-core`**, publicado en crates.io. Y `audit_engine::build_traceability`, el ensamblador de grafo privado del CLI, se generalizó en `core::graph`: un knowledge graph tipado, bidireccional, preservador-de-huérfanos sobre los cross-links del frontmatter, con una decisión de diseño deliberada que importa más adelante — **las referencias colgantes se conservan como aristas `resolved: false` de primera clase**, no se descartan. Un enlace a un documento que no existe es dato, no un error para tragarse.

¿Por qué tomarse el trabajo de un workspace y un crate publicado en vez de dejar que Loom simplemente copie la lógica de parsing? Porque copiar es como obtienes dos verdades. Si Loom parseara documentos de StrayMark con su propio código, los dos parsers derivarían — un campo nuevo de frontmatter reconocido por uno y no por el otro, una variante de heading matcheada aquí y omitida allá — y la visualización estaría en silencio en desacuerdo con el audit. Ese es *precisamente* el modo de falla que StrayMark existe para prevenir en los proyectos de los adopters: dos fuentes de verdad para el mismo hecho, divergiendo en silencio. Entregarlo dentro de nuestro propio tooling habría sido mala praxis con cara de palo.

Así que la vara para M0 era la vara que el principio exige: **el comportamiento del CLI tenía que ser byte-for-byte idéntico antes y después.** La suite de tests completa pasó sin cambios; `straymark audit` produjo salida idéntica. Las notas del release dicen "sin cambios de comportamiento visibles para el usuario", lo cual se lee como un no-evento y era de hecho el punto entero. Un refactor que no cambia nada que el usuario pueda ver, al servicio de garantizar que dos programas nunca puedan estar en desacuerdo, es el release con la forma más StrayMark imaginable.

## Lo que deliberadamente no hicimos

La sección de contención, porque todos los posts de esta serie tienen una.

`straymark-core` está publicado en crates.io — ahora es una librería compartida de verdad. **Loom no.** Sale solo como GitHub-release bajo tags `loom-*`, marcado experimental (v0 / N=1), detrás de un download gate opt-in en `straymark loom serve`. El CLI **no** ganó ninguna dependencia de `axum` ni `tokio` de nada de esto — el server vive enteramente en `experimento/`, y el único conocimiento que el CLI tiene de él es un launcher que baja un binario en el primer uso. La *gramática* compartida graduó a crate publicado en el momento en que un segundo lector la necesitó; el *componente* experimental queda con cortafuegos hasta que se gane la salida. Una de esas decisiones es permanente y la otra es reversible, y se mantienen a lados opuestos de la línea a propósito.

## Si llegaste hasta aquí

El ejercicio portátil: encuentra el formato que tu proyecto parsea en más de un lugar. Archivos de config leídos tanto por la app como por un script de CI. Una forma de log consumida por un shipper y un dashboard. Un contrato de API serializado de un lado y deserializado del otro por código escrito a mano. Ahora hazte la pregunta que M0 fue construido para responder: ¿esos dos lectores comparten un parser, o meramente *se parecen* hoy? El parecido no es una garantía — es una coincidencia con fecha de vencimiento, y la fecha es cuando alguien edita un lado y se olvida del otro. El único arreglo duradero es el poco glamoroso: extrae la gramática, publícala una vez, y vuelve la divergencia imposible en vez de meramente improbable.

---

*StrayMark `fw-4.24.0` → `fw-4.26.0`, `cli-3.22.0` → `cli-3.23.1` — ADR [2026-06-02-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md) · Charter [`CHARTER-01-loom-server`](https://github.com/StrangeDaysTech/straymark/blob/main/experimento/CHARTER-01-loom-server.md) · Issues [#135](https://github.com/StrangeDaysTech/straymark/issues/135) · [#232](https://github.com/StrangeDaysTech/straymark/issues/232) · [#237](https://github.com/StrangeDaysTech/straymark/issues/237) · PRs [#238](https://github.com/StrangeDaysTech/straymark/pull/238) · [#239](https://github.com/StrangeDaysTech/straymark/pull/239). Predecesor: [Lo que el script bash decía que estaba en sync](what-the-bash-script-said-was-in-sync). Siguiente: [Lo que el grafo todavía no podía dibujar](what-the-graph-couldnt-draw-yet).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad por el contenido recae en el autor humano.*
