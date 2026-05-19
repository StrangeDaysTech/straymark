---
slug: validate-y-los-schemas-como-capa-formal
title: Validate y los schemas como capa formal
authors:
  - jose
tags:
  - straymark
  - validate
  - schemas
  - governance
  - phase-2
date: 2026-05-11T00:00:00.000Z
description: 'Phase 2: cuando el framework empieza a verificar lo que prometía'
---
*464 líneas de bash reemplazadas por una flag. De "verifico que esto parezca un documento" a "verifico que esto satisface el schema canónico, en el modo que pidas, con consecuencias operacionales concretas." Fase 2 del roadmap del CLI y la formalización de validate.*

<!-- truncate -->

## 1. Tener schemas no es lo mismo que validar contra schemas

`straymark validate` existía desde el 25 de marzo de 2026 — PR #27, parte del primer arco de Phase 1 del CLI. Lo que hacía entonces era pequeño: verificar que los documentos en `.straymark/` tuvieran sintaxis Markdown válida, frontmatter parseable, y los campos mínimos del tipo correcto. Era útil. No era *enforcement*.

Lo que cambió el 11 de mayo de 2026, con el release `fw-4.6.0 / cli-3.7.0`, fue otra cosa: Phase 2 del roadmap del CLI cerró un arco completo de siete PRs (#73 a #79) y los consolidó en uno solo (#80). El CHANGELOG lo dice sin metáfora:

> *"The first feature-bearing release since the repositioning... Phase 2 of `Propuesta/devtrail-cli-roadmap.md` lands as 7 bisect-safe PRs (#73–#79), grouped here for reviewers. The release closes the empirical loop the Sentinel experiment opened: telemetry at Charter close, drift detection at Charter close, and a canonical approval signal for `review_required: true` documents."*

Lo que importa para este post no son los siete PRs como hitos individuales; son los tres que tocaron el comando `validate` y los schemas. Lo que en marzo era *"verifico que esto parece un documento"* en mayo se volvió *"verifico que esto cumple el schema canónico del tipo, en el modo que pidas, con consecuencias operativas concretas"*.

---

## 2. Del bash artesanal al flag de comando

Una de las piezas más visibles del cambio es pequeña: el flag `--staged`.

Hasta Phase 2, el patrón recomendado para que los adoptantes validaran sus documentos antes de un commit era un script bash llamado `pre-commit-docs.sh`. Vivía en `dist/.straymark/scripts/` y tenía 464 líneas. Hacía lo razonable: leer `git diff --cached --name-only`, filtrar archivos en `.straymark/`, parsear su frontmatter con `awk`, verificar campos. Funcionaba. Y era 464 líneas de bash que cada adoptante tenía que confiar en que ejecutaba lo correcto, en su versión correcta, con su parser de YAML correcto.

PR del 28 de marzo (commit `d9ea874`) reemplazó las 464 líneas por un flag:

```bash
straymark validate --staged
```

Lee los staged files igual, filtra por `.straymark/` igual, valida igual. La diferencia: hoy todo eso vive en Rust dentro del binario, no en bash en disco. El adoptante no tiene que mantener el script; el script no puede divergir entre versiones; el comportamiento es el mismo en Linux, macOS y Windows. El framework dejó de pedirle al adoptante que confiara en un script suelto y empezó a darle un comando con la promesa implícita del binario versionado.

Es el mismo patrón que el Post 6 documentó para la disciplina de archivo (preservar git history en lugar de reescribir) o que el Post 4 documentó con la decisión A1 (orquestar sin invocar APIs): pasar de *"el operador hace X manualmente con riesgo de divergir"* a *"el framework canoniza X en un comando declarativo"*. El `--staged` no es feature exótico; es deuda manual barrida.

---

## 3. Los tres schemas v0

Phase 2 también consolidó los tres schemas que viven en `dist/.straymark/schemas/`:

| Schema | Qué describe | Origen |
| --- | --- | --- |
| `charter.schema.v0.json` | Estructura del Charter como entidad de primera clase (Post 4) | Sentinel `/plan-audit` (6 ciclos, format v1 → v2 → v3) |
| `charter-telemetry.schema.v0.json` | Telemetría de Charters (Post 3, ampliado en Post 10) | 5 archivos `PLAN-NN.telemetry.yaml` de Sentinel |
| `audit-output.schema.v0.json` | Output canónico de auditoría externa (Post 4 + Post 10) | Sentinel dual-audit (Copilot + Gemini + Claude crítico) |

Cada schema lleva un campo `$comment` en su raíz. El de `charter.schema.v0.json` dice literal:

> *"EXPERIMENTAL v0. Crystallized from Sentinel /plan-audit (6 cycles, format v1 → v2 → v3). Will not stabilize to v1.0 until validated in a second domain (frontend, ML pipeline, or infra-as-code)."*

El de `charter-telemetry.schema.v0.json`, parecido:

> *"EXPERIMENTAL v0. Derived from 5 PLAN-NN.telemetry.yaml files in Sentinel (one project, Go backend) — same N=1-domain caveat as charter.schema.v0.json. Will not stabilize to v1.0 until validated in a second domain."*

El de `audit-output.schema.v0.json`, idem:

> *"EXPERIMENTAL v0.x. Phase 3 of the CLI roadmap. Crystallized from the dual-audit pattern validated empirically in Sentinel."*

Los tres comparten una declaración explícita: *no crystallizes without a second domain*. Es el principio #12 del framework, no como párrafo aparte en una documentación de gobernanza, sino **dentro del propio JSON Schema, en un campo que cualquier validador estándar lee y cualquier herramienta de schema-explorer renderiza al usuario**. El framework no esconde la provisionalidad de sus schemas; la pone donde nadie pueda perderla.

Esto importa más de lo que parece. La práctica habitual en ecosistemas de schemas (JSON Schema, OpenAPI, Avro) es publicar `v1` y luego bumpear conforme cambias. Marcar `v0` con un `$comment` que dice *"esto es experimental hasta que un segundo dominio lo valide"* es admitir, de origen, que la primera cristalización no es la definitiva. La schema misma es honesta.

---

## 4. Tres modos del comando

`straymark validate` ya no es un comando con un solo flujo. Después de Phase 2 tiene tres modos operativos, cada uno acoplado a un momento del ciclo de vida:

- **Modo `all` (default)** — `straymark validate` sin flags. Camina recursivamente `.straymark/`, parsea cada archivo `.md`, intenta resolver su tipo (AILOG, AIDEC, ADR, ETH, REQ, TES, INC, TDE, MCARD, SBOM, SEC, DPIA, Charter), aplica el schema correspondiente, y reporta violaciones por archivo. Es el modo *auditoría completa*. Útil para CI, útil para revisar un repo nuevo, útil cuando el adoptante quiere medir cuánta deuda formal acumuló.

- **Modo `--staged`** — `straymark validate --staged`. El que reemplazó al bash. Solo valida archivos que `git diff --cached --name-only` reporta como staged dentro de `.straymark/`. Su lugar natural es `.git/hooks/pre-commit`, y desde fw-4.6.0 hay un `straymark init --hooks` que lo instala automáticamente. Es el modo *gate operativo* — el que evita que documentos rotos lleguen a `main`.

- **Modo `--check-pending-reviews`** — `straymark validate --check-pending-reviews`. El más sutil de los tres, y el que justifica más párrafos.

Hasta Phase 2, un documento que el agente marcaba `review_required: true` en el frontmatter (un AIDEC con confianza baja, un AILOG con riesgo alto, etc.) quedaba esperando *aprobación humana*. Pero no había una forma canónica de *señalar* esa aprobación. El operador leía el documento, mentalmente lo aprobaba, seguía. No quedaba rastro estructural de la aprobación. Esto era exactamente el Issue [#67](https://github.com/StrangeDaysTech/straymark/issues/67).

`--check-pending-reviews` cierra ese hueco. El comando recorre todos los documentos con `review_required: true` y los listsa con su `confidence`, su `risk_level`, su autor (humano o agente), y su fecha. La aprobación es un commit que cambia `review_required: true` → `review_required: false` con co-autoría humana en el commit message — y `validate --check-pending-reviews` deja de listar el documento. La aprobación es un *signal canónico*, no un acuerdo verbal con uno mismo.

Es la pieza que cierra el ciclo del agente. El framework permite al agente crear documentos sin pre-aprobación humana (Post 1, propiedad #2: *"permiso cultural sin control bloqueante"*); pero los marca para revisión cuando corresponde; y ahora tiene un comando para listar lo pendiente, sin escapatoria. Ningún documento de auditoría se pierde en silencio.

---

## 5. Phase 2 cierra el loop empírico

Vale la pena rescatar la frase del CHANGELOG entera porque condensa lo que el release significa:

> *"The release closes the empirical loop the Sentinel experiment opened: telemetry at Charter close, drift detection at Charter close, and a canonical approval signal for `review_required: true` documents (resolving issue #67)."*

Tres piezas que cierran un arco que arrancó en el experimento de los seis Planes (Post 3):

1. **Telemetría al cierre del Charter** — el bloque YAML que Sentinel llenaba a mano durante el experimento se convirtió en el schema `charter-telemetry.schema.v0.json` validable.
2. **Drift detection al cierre del Charter** — el script bash de 145 líneas (`check-plan-drift.sh`) se convirtió en el subcomando `straymark charter drift`, integrado al validate flow.
3. **Approval signal canónico** — lo que el operador hacía mentalmente se convirtió en un comando con consecuencias estructurales: `--check-pending-reviews`.

Es el patrón que el blog ha visto repetirse: cada vez que el framework cristaliza una práctica que Sentinel hacía manualmente, lo hace después de validar empíricamente que la práctica funciona. La diferencia esta vez es que **el comando que verifica la cristalización es parte del paquete**. Antes del Phase 2, Sentinel tenía templates, schemas declarados, y la promesa implícita de que los documentos seguían los schemas. Después de Phase 2, hay un comando que **lo comprueba**. No es un cambio de capacidad; es un cambio de garantías.

---

## 6. El principio #12, hecho schema

La pieza filosófica del post — y la que más me interesa dejar registrada — es esta:

Los `$comment` de los tres schemas v0 codifican un principio que el framework venía declarando en prosa desde abril. El Post 4 lo nombró así: *"schema marked `v0` on purpose, not `v1`, because the framework's principle #12 says no schema crystallizes without a second domain validating it"*. El Post 7 lo aplicó al diseño del TDE: *"the framework validated principle #12 again: no schema crystallizes without a second domain"*. El Post 10 lo aplicó a Pattern 1 y Pattern 2: *"the document says so without shortcuts: the pattern is empirically validated in Sentinel. Wait for the second domain before crystallizing it higher"*.

Phase 2 hace algo distinto. **No declara el principio; lo escribe dentro del schema mismo, en un lugar que cualquier consumidor del schema va a leer.** El `$comment` no es marketing; es protocolo. Cualquier validador de JSON Schema 2020-12 lo expone; cualquier IDE que parsea el schema lo muestra. Es la primera vez que el framework formaliza la provisionalidad de sus formalizaciones.

Si quisiera resumirlo en una sola frase: el framework valida lo que tiene, sabiendo que lo que tiene es provisional, y deja eso por escrito *en el lugar donde la validación ocurre*. Validar lo provisional es disciplina. Cristalizar prematuramente y validar lo cristalizado es teología.

---

## 7. Cierre

Lo que aprendí del proceso, en cuatro tesis:

1. **Tener schemas no es lo mismo que validar contra schemas.** El primer paso es declarativo; el segundo es operativo. Phase 2 hizo el segundo, no el primero. La diferencia entre los dos modos del framework — *"tengo la promesa"* vs. *"tengo el comando que la comprueba"* — es lo que determina si el adoptante puede confiar en lo declarado.

2. **El patrón "bash artesanal → flag de comando" es estructural.** El `--staged` no es feature aislado. Es la versión Phase 2 del mismo patrón que Post 4 documentó para el audit cycle externo, que Post 10 documentó para los CLI helpers de chain evolution: imperativo verboso → declarativo legible → versionado. Cada vez que el framework barre un script bash por un flag, sube una capa de garantías.

3. **El `--check-pending-reviews` cierra el ciclo del agente.** Lo que Post 1 nombró como permiso cultural sin pre-aprobación se completa aquí: el agente puede crear, pero los documentos marcados para revisión no se pierden — el comando los lista hasta que un humano los apruebe con un commit explícito. Sin ese signal, el permiso cultural del Post 1 sería negligencia.

4. **Provisionalidad declarada > formalización prematura.** Los `$comment` de los schemas v0 hacen el principio #12 protocolo, no doctrina. Cualquier herramienta que consuma el schema sabe que la formalización es provisional. Es la honestidad estructural más fuerte que puede tener un framework joven: validar lo que tiene, sin pretender que lo tenido es definitivo.

---

*Anclas: PR [#27](https://github.com/StrangeDaysTech/straymark/pull/27) — versión original de `validate` (marzo 2026). Phase 2: PRs #73 a #79 + PR [#80](https://github.com/StrangeDaysTech/straymark/pull/80) — `fw-4.6.0 / cli-3.7.0` (fusionado 2026-05-11). Schemas: `dist/.straymark/schemas/{charter,charter-telemetry,audit-output}.schema.v0.json`. [Issue #67](https://github.com/StrangeDaysTech/straymark/issues/67) — origen del `--check-pending-reviews`.*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*
