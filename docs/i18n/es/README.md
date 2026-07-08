<div align="center">

<img src="../../../assets/straymark-symbol.svg" alt="StrayMark — por Strange Days Tech" width="240" />

# StrayMark

**La disciplina cognitiva que tus proyectos asistidos por IA necesitan**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/straymark-cli.svg)](https://crates.io/crates/straymark-cli)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![CLA assistant](https://cla-assistant.io/readme/badge/StrangeDaysTech/straymark)](https://cla-assistant.io/StrangeDaysTech/straymark)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](../../../dist/.straymark/QUICK-REFERENCE.md)
[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

[Inicio Rápido](#inicio-rápido) •
[¿Por qué StrayMark?](#por-qué-straymark) •
[¿Para quién es?](#para-quién-es-straymark) •
[Principios de Diseño](#principios-de-diseño) •
[Características](#características) •
[Compliance](#compliance) •
[Documentación](#documentación)

**Idiomas**: [English](../../../README.md) | Español | [简体中文](../zh-CN/README.md)

</div>

---

## El Problema

Los agentes de IA generan código rápido. No generan código coherente. Después de suficientes turnos, un agente pierde el hilo: re-introduce patrones que el equipo había rechazado, acumula deuda técnica oculta y produce trabajo que compila pero no encaja con el grano del sistema. Cuanto más rápido va el agente, más difícil es ver esa deuda — hasta que una regresión, un incidente o una refactorización la sacan a la superficie.

Los ingenieros senior que orquestan estos agentes no necesitan *más* autonomía del agente. Necesitan lo opuesto: una forma de externalizar el alcance, las decisiones y los riesgos a un ritmo al que se pueda someter al agente — para que ejecute contra restricciones en lugar de inventarse las propias.

## La Solución

StrayMark es un **framework + CLI** que externaliza la disciplina cognitiva del trabajo de ingeniería de software senior — alcance explícito, decisiones declaradas, riesgos nombrados, alternativas registradas, rastros auditables — en archivos versionados que viven junto al código.

> **"Ningún cambio significativo sin un rastro documentado — y un espacio de decisión acotado para el agente."**

Como efecto secundario, la disciplina produce evidencia compatible con **ISO/IEC 42001**, **EU AI Act**, **NIST AI RMF** y (opt-in) la pila regulatoria china de IA/datos. Pero la meta es la calidad de ingeniería primero; el cumplimiento es lo que cae como subproducto cuando la disciplina es real.

---

## ¿Por qué StrayMark?

El código es solo el rastro fósil de una batalla mental. La verdadera ingeniería sucede en el caos de las decisiones, los riesgos calculados y las rutas que decidiste no tomar. Tradicionalmente, todo ese rastro humano se descarta como stray marks (manchas accidentales) en el historial de un proyecto. En Strange Days Tech, creemos que esas marcas son la señal, no el ruido. Mientras el software de compliance corporativo te obliga a mentir en retrospectiva para llenar formularios vacíos, StrayMark te permite marcar tu territorio en tiempo real.

Para nosotros, el cumplimiento normativo (ISO 42001, EU AI Act, NIST) no es el objetivo final, sino el hilado. Utilizamos estos estándares como un tejido estructural para construir una memoria técnica multidimensional. No se trata de marcar casillas para un auditor; se trata de usar esos hilos para tejer un andamiaje que soporte procesos cognitivos complejos durante el diseño y la implementación. StrayMark convierte la burocracia en una malla táctica que te ayuda a pensar, navegar y construir con una resolución que el código por sí solo no puede alcanzar.

En esta era de agentes autónomos, StrayMark no es un mecanismo de refuerzo para la sustitución del operador humano. Todo lo contrario: es una herramienta para humanizar los procesos de alta velocidad y vasta complejidad en los que operan los agentes de IA. Mientras la máquina se mueve a ritmos que desbordan la intuición, StrayMark actúa como el ancla de intención humana, forzando a los agentes a operar dentro de un espacio de decisiones restringido, legible y auditable. No buscamos automatizar la ingeniería hasta el olvido, sino dotar a la velocidad de la IA de una consciencia y una estructura profundamente humana.

**Captura el ruido. Teje la señal. Humaniza la máquina.**

---

## ¿Para quién es StrayMark?

El usuario primario de StrayMark es el **ingeniero senior orquestando agentes de IA sobre un sistema no trivial** — alguien con criterio técnico fuerte que usa agentes para abordar trabajo que no podría hacer solo de forma realista, y que necesita disciplina cognitiva externalizada para que el agente no introduzca caos sistémico.

Si esa persona eres tú, los flujos, defaults y lenguaje de StrayMark están afinados para ti.

StrayMark también sirve a tres audiencias secundarias, sobre esa base — nunca a su costa:

- **Tech leads y arquitectos** estandarizando cómo trabaja su equipo con asistentes de IA.
- **Compliance officers y auditores** que necesitan evidencia de desarrollo de IA gobernado (ISO 42001, EU AI Act, NIST AI RMF, PIPL, TC260, …).
- **Adoptantes en entornos regulados** (finanzas, salud, sector público, China) que necesitan trazabilidad integrada en el flujo en lugar de reconstruida después del hecho.

StrayMark **no** intenta ser: un gateway de LLM, un evaluador de modelos, una capa de productividad estilo *"código 10× más rápido"* ni un sustituto del juicio del ingeniero. Ver [Límites Honestos](#límites-honestos) abajo.

---

## Principios de Diseño

Las decisiones de producto de StrayMark se anclan en doce principios explícitos. Están ordenados por jerarquía: cuando dos entran en conflicto, gana el que viene antes.

1. **La herramienta sirve al oficio, no al producto.** La métrica es si el ingeniero produce trabajo del que se siente orgulloso — no adopción, retención ni revenue.
2. **El usuario primario es el ingeniero senior orquestando agentes.** No el VP, no el CISO, no el compliance officer.
3. **Open source estricto, sin asteriscos en el núcleo.** Framework, CLI y TUI son MIT, sin features capadas que empujen al pago.
4. **El cumplimiento regulatorio es un side effect, no el producto.** ISO 42001, EU AI Act, NIST AI RMF son frames útiles; no son la meta.
5. **Schema-driven antes que feature-driven.** Las entidades centrales (Stage Closure Bundle, Charter, Document) se definen primero como schemas versionados, las features después.
6. **Disciplina cognitiva sobre productividad bruta.** StrayMark compite contra el caos que el código rápido con IA genera en proyectos serios — no contra la velocidad misma.
7. **Local-first, Cloud como amplificador.** El CLI funciona completo, sin red. Cloud puede agregar valor (agregación cross-repo, evidencia firmada) pero nunca capa el núcleo.
8. **La memoria del proyecto vive en el repo, no en una base de datos externa.** AILOGs, ADRs, AIDECs, Charters y bundles son archivos versionados junto al código, en markdown + JSON Schema.
9. **Simplicidad sobre capacidad.** Cuando dos diseños cumplen el objetivo, gana el más simple. Los patrones cristalizan después de validarse en proyectos reales, no antes.
10. **Honestidad sobre lo que la herramienta no hace.** Sin evaluación de modelos, sin gateway de LLM, sin certificación automática de compliance, sin reemplazar al juicio del ingeniero.
11. **La comunidad cuida la herramienta, no al revés.** Las contribuciones y el feedback se toman en serio sin volverse democracia.
12. **La velocidad del producto es la velocidad del aprendizaje.** Sin cristalización prematura; los schemas marcados `v0` hasta validarse contra un segundo dominio.

El documento completo, con anotaciones empíricas de los ciclos de validación, vive en [`docs/i18n/es/contributors/DESIGN-PRINCIPLES.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/es/contributors/DESIGN-PRINCIPLES.md).

---

## Características

### 📋 Documentación Estructurada

Doce tipos de documentos que cubren el ciclo de vida completo del desarrollo:

| Tipo | Propósito | Ejemplo |
|------|-----------|---------|
| **REQ** | Requisitos | Requisitos del sistema, historias de usuario |
| **ADR** | Decisiones de Arquitectura | Elecciones tecnológicas, patrones de diseño |
| **TES** | Planes de Prueba | Estrategias de prueba, objetivos de cobertura |
| **INC** | Post-mortems de Incidentes | Análisis de causa raíz, lecciones aprendidas |
| **TDE** | Deuda Técnica | Deuda identificada, planes de remediación |
| **AILOG** | Logs de Acciones de IA | Qué hicieron los asistentes de IA y por qué |
| **AIDEC** | Decisiones de IA | Elecciones hechas por IA con alternativas |
| **ETH** | Revisiones Éticas | Privacidad, sesgo, IA responsable |
| **SEC** | Evaluaciones de Seguridad | Modelado de amenazas, análisis de vulnerabilidades |
| **MCARD** | Tarjetas de Modelo/Sistema | Documentación de modelos de IA |
| **SBOM** | Lista de Materiales de Software | Inventario de componentes de IA |
| **DPIA** | Evaluación de Impacto en Protección de Datos | Análisis de impacto en privacidad |

### 🤖 Soporte para Agentes IA

Pre-configurado para asistentes de codificación con IA populares:

- **Universal (estándar AGENTS.md)** → `AGENTS.md` — leído por Claude Code, OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue, Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q y otros
- **Claude Code** (Anthropic) → `CLAUDE.md`
- **Cursor** → `.cursorrules`
- **GitHub Copilot CLI** → `.github/copilot-instructions.md`
- **Gemini CLI** (Google) → `GEMINI.md`

Cada configuración instruye a la IA a:
- Identificarse en cada documento
- Declarar niveles de confianza
- Solicitar revisión humana cuando sea apropiado
- Seguir convenciones de nomenclatura
- **Seguir estrategia de branching Git** (nunca hacer commit directamente a `main`)

### 👁️ Supervisión Humana

Salvaguardas incorporadas que aseguran que los humanos mantengan el control:

- **Niveles de autonomía**: Algunos tipos de documentos requieren aprobación humana
- **Disparadores de revisión**: Baja confianza o alto riesgo → revisión obligatoria
- **Revisiones éticas**: Preocupaciones de privacidad y sesgo marcadas para decisión humana

### ✅ Herramientas CLI

Comandos integrados que convierten la disciplina en feedback accionable:

- **`straymark charter <new|list|status|close|drift|batch-complete|audit|refresh-suggest|amend>`** — Unidades acotadas declaradas ex-ante, auditadas ex-post. `close` registra telemetría; `drift` detecta drift archivos-vs-commits con supresión AILOG-aware y (cli-3.13.0+) hace gate sobre entradas `### Batch N (pending)` en `## Batch Ledger` del AILOG; `batch-complete` (cli-3.13.0+) marca un batch como completado en la ledger para Charters multi-batch (3+ lotes o >1 día); `audit` orquesta revisión externa multi-modelo (flujo de 3 pasos prepare/calibrate/finalize, orchestration-only — sin invocación de APIs de LLM); `refresh-suggest` (cli-3.14.0+) imprime una recomendación heurística para un refresh SpecKit pre-declare cuando la media móvil de `r_n_plus_one_emergent_count` de un módulo multi-Charter supera un umbral; `amend` (cli-3.14.0+) hace scaffolding de una enmienda post-close Batch N.4 (remediación dirigida por auditoría) sobre la misma rama de execute sin abrir un Charter nuevo. Para flujos IDE-driven, las skills inline `/straymark-audit-prompt` y `/straymark-audit-review` envuelven al CLI para mostrar prompts en la conversación y mergear findings en la telemetría.
- **`straymark followups <list|status|drift|recount|promote>`** *(cli-3.19.0+; `recount` cli-3.20.0+)* — Registro de backlog de follow-ups de primera clase (`.straymark/follow-ups-backlog.md`, schema v1 experimental): `drift --apply` extrae las entradas `§Follow-ups` / `R<N> (new)` de los AILOGs por-AILOG (los bullets con marcador de cierre aterrizan como `suspected-closed`), los contadores son propiedad del CLI y se recalculan en cada escritura, `recount` los reconcilia tras una sesión de triage manual, y `promote` eleva las entradas a documentos TDE con trazabilidad completa. Ver `STRAYMARK.md §16` y `FOLLOW-UPS-BACKLOG-PATTERN.md`.
- **`straymark approve <doc-id>`** — Registra una aprobación humana formal (escribe `reviewed_by` / `reviewed_at` / `review_outcome` y la sección body `## Approval` en una sola edición; cierra el gap canonizado en DOCUMENTATION-POLICY §3.5)
- **`straymark validate`** — 25+ reglas de validación para corrección documental (12 específicas de China son scope-aware); `--include-charters` extiende a `.straymark/charters/`; `--check-pending-reviews` lista el backlog de aprobaciones (warn-only)
- **`straymark metrics`** — KPIs de gobernanza, tasas de revisión, distribución de riesgo, tendencias
- **`straymark analyze`** — Análisis de complejidad de código (cognitiva + ciclomática) impulsado por [arborist-metrics](https://github.com/StrangeDaysTech/arborist-metrics/), nuestra librería open-source en Rust para métricas de código multi-lenguaje, desarrollada también por StrangeDaysTech S.A.S. de C.V.
- **`straymark audit`** — Reportes de auditoría con línea temporal, mapas de trazabilidad y exportación HTML
- **`straymark compliance`** — Puntuación de cumplimiento regulatorio como side effect del trabajo documentado (EU AI Act, ISO 42001, NIST AI RMF; seis frameworks chinos opt-in vía `--region china`)
- **`straymark explore`** — TUI interactivo para navegar el grafo de documentación del proyecto, incluyendo una vista de Charters (estado del ciclo de vida, AILOG/spec de origen, ubicación del archivo)
- **Hooks pre-commit** + **GitHub Actions** para validación CI/CD

---

## Límites Honestos

StrayMark **no** hace lo siguiente:

- evaluar, comparar o rankear LLMs;
- actuar como gateway o capa de routing de LLMs;
- prevenir alucinaciones ni garantizar la corrección del agente;
- certificar automáticamente cumplimiento regulatorio — produce evidencia, no certificaciones;
- reemplazar el juicio de un ingeniero senior.

Si tu problema es uno de esos, StrayMark no es la herramienta.

---

## Compliance

La disciplina que StrayMark externaliza — alcance explícito, decisiones declaradas, riesgos nombrados, alternativas registradas — produce, como efecto secundario, evidencia que mapea limpiamente a los principales marcos de gobernanza de IA. Por eso el cumplimiento se posiciona como *consecuencia de hacer bien el trabajo de ingeniería*, no como el producto en sí (Principio #4).

### Alineación con estándares

| Estándar | Integración con StrayMark |
|----------|--------------------------|
| **ISO/IEC 42001:2023** | Estándar vertebral — gobernanza de Sistemas de Gestión de IA |
| **EU AI Act** | Clasificación de riesgo, reporte de incidentes, transparencia |
| **NIST AI RMF / 600-1** | 12 categorías de riesgo GenAI en ETH/AILOG |
| **ISO/IEC 25010:2023** | Modelo de calidad de software en REQ/ADR |
| **ISO/IEC/IEEE 29148:2018** | Ingeniería de requisitos en REQ |
| **ISO/IEC/IEEE 29119-3:2021** | Documentación de pruebas en TES |
| **GDPR** | Protección de datos en ETH/DPIA |
| **OpenTelemetry** | Observabilidad (opcional) |

### Cobertura regulatoria de China — opt-in vía `regional_scope: china`

| Estándar | Integración con StrayMark |
|----------|--------------------------|
| **TC260 AI Safety Governance Framework v2.0** | Cinco niveles de riesgo (TC260RA) |
| **PIPL — Personal Information Protection Law** | Personal Information Protection Impact Assessment (PIPIA), retención ≥ 3 años |
| **GB 45438-2025** *(obligatorio)* | Etiquetado de contenido generado por IA — explícito + implícito (AILABEL) |
| **CAC Algorithm Filing** | Registro de algoritmo, proceso de doble registro (CACFILE) |
| **GB/T 45652-2025** | Seguridad de datos de pre-entrenamiento y fine-tuning (SBOM/MCARD) |
| **CSL 2026** | Reporte de incidentes de ciberseguridad (ventanas 1h / 4h+72h+30d) en INC |

StrayMark cubre seis regulaciones chinas de IA / datos como **scope regional opt-in**. Activa añadiendo `regional_scope: china` en `.straymark/config.yml`; los proyectos sin esa configuración no se ven afectados. Cuando se activa, cuatro tipos de documento específicos de China (PIPIA, CACFILE, TC260RA, AILABEL) quedan disponibles, doce reglas de validación comienzan a aplicar las nuevas referencias cruzadas, y `straymark compliance --region china` produce un score por marco. Las guías detalladas viven bajo `.straymark/00-governance/` (`CHINA-REGULATORY-FRAMEWORK.md`, `TC260-IMPLEMENTATION-GUIDE.md`, `PIPL-PIPIA-GUIDE.md`, `CAC-FILING-GUIDE.md`, `GB-45438-LABELING-GUIDE.md`). El [README en chino](../zh-CN/README.md#中国法规支持) tiene la versión completa para adoptantes que operan en China continental.

---

## Inicio Rápido

### Opción 1: CLI (Recomendado)

**Instalación rápida (binario precompilado):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.ps1 | iex
```

O instalar desde el código fuente con Cargo:

```bash
cargo install straymark-cli
```

> **Nota:** `straymark update-cli` detecta automáticamente cómo instalaste el CLI. Las instalaciones con binario precompilado se actualizan desde GitHub Releases; las instalaciones con cargo se actualizan via `cargo install`. Puedes forzar el método con `--method=github` o `--method=cargo`.

Luego inicializa en tu proyecto:

```bash
cd tu-proyecto
straymark init .
```

El CLI descarga la última versión de StrayMark, configura el framework y los archivos de directivas de agentes IA automáticamente.

### Versionado

StrayMark usa tags de versión independientes para cada componente:

| Componente | Prefijo de tag | Ejemplo | Incluye |
|------------|---------------|---------|---------|
| Framework | `fw-` | `fw-4.34.0` | Plantillas (12 tipos), gobernanza, directivas, plantilla + schema de Charter |
| CLI | `cli-` | `cli-3.30.0` | El binario `straymark` |
| Loom (EXPERIMENTAL) | `loom-` | `loom-0.4.2` | El servidor de visualización `straymark-loom`, descargado bajo demanda por `straymark loom serve` |

Verifica las versiones instaladas con `straymark status` o `straymark about`.

### Comandos CLI

| Comando | Descripción |
|---------|-------------|
| `straymark init [path]` | Inicializar StrayMark en un proyecto |
| `straymark update` | Actualizar framework y CLI |
| `straymark update-framework` | Actualizar solo el framework |
| `straymark update-cli` | Actualizar el binario del CLI |
| `straymark remove [--full]` | Eliminar StrayMark del proyecto |
| `straymark status [path]` | Mostrar estado de la instalación y estadísticas |
| `straymark repair [path]` | Restaurar directorios y archivos del framework faltantes |
| `straymark validate [path]` | Validar documentos por cumplimiento y corrección (use `--include-charters` para Charters, `--check-pending-reviews` para el backlog de aprobaciones) |
| `straymark charter <subcomando>` | Gestionar Charters: `new`, `list`, `status`, `close` (registra telemetría), `drift` (drift archivos-vs-commit con AILOG-awareness) |
| `straymark followups <subcomando>` *(cli-3.19.0+)* | Gestiona el registro del backlog de follow-ups: `list` (entradas filtrables), `status` (pulso con contadores propiedad del CLI recalculados al vuelo), `drift` (detecta/extrae AILOGs no procesados — reemplazo nativo del script bash v0, con extracción anti-ruido `suspected-closed`), `promote` (FU → TDE con trazabilidad `promoted_from_followup`) |
| `straymark approve <doc-id>` | Registra una aprobación humana formal en un documento `review_required: true` (frontmatter + sección body canónica) |
| `straymark compliance [path]` | Verificar cumplimiento regulatorio (EU AI Act, ISO 42001, NIST) |
| `straymark metrics [path]` | Mostrar métricas de gobernanza y estadísticas |
| `straymark analyze [path]` | Analizar complejidad de código (métricas cognitiva + ciclomática) |
| `straymark audit [path]` | Generar reportes de auditoría con línea temporal y trazabilidad |
| `straymark explore [path]` | Explorar documentación interactivamente en terminal (TUI) |
| `straymark about` | Mostrar información de versión y licencia |

Ver [Referencia CLI](adopters/CLI-REFERENCE.md) para uso detallado.

### Opción 2: Configuración Manual

```bash
# Descargar el último release ZIP del framework desde GitHub
# Ve a https://github.com/StrangeDaysTech/straymark/releases
# y descarga el último release fw-* (ej. fw-4.28.0)

# Extraer y copiar a tu proyecto
unzip straymark-fw-*.zip -d tu-proyecto/
cd tu-proyecto

# Commit
git add .straymark/ STRAYMARK.md
git commit -m "chore: adoptar StrayMark"
```

**Ver [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) para instrucciones detalladas, estrategias de migración y planes de implementación en equipos.**

---

## Documentación

La documentación de StrayMark está organizada por audiencia:

| Track | Para | Empieza aquí |
|-------|------|--------------|
| [**Adoptantes**](adopters/) | Equipos que adoptan StrayMark en sus proyectos | [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) |
| [**Contribuidores**](../../../docs/contributors/) | Desarrolladores que contribuyen a StrayMark | [TRANSLATION-GUIDE.md](../../../docs/contributors/TRANSLATION-GUIDE.md) |

**Adoptantes**: Sigue la [Guía de Adopción](adopters/ADOPTION-GUIDE.md) para instrucciones paso a paso, la [Referencia CLI](adopters/CLI-REFERENCE.md) para detalles de comandos, y la [Guía de Flujos de Trabajo](adopters/WORKFLOWS.md) para patrones de uso diario.

**Contribuidores**: Consulta [CONTRIBUTING.md](CONTRIBUTING.md) para guías de desarrollo, y la [Guía de Traducción](../../../docs/contributors/TRANSLATION-GUIDE.md) para agregar nuevos idiomas.

### Referencias Clave

| Documento | Descripción |
|-----------|-------------|
| [**Referencia Rápida**](../../../dist/.straymark/QUICK-REFERENCE.md) | Resumen de tipos de documentos y nomenclatura |
| [STRAYMARK.md](../../../dist/STRAYMARK.md) | Reglas de gobernanza unificadas (fuente de verdad) |
| [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) | Guía de adopción para proyectos nuevos/existentes |
| [CLI-REFERENCE.md](adopters/CLI-REFERENCE.md) | Referencia completa de comandos CLI |
| [WORKFLOWS.md](adopters/WORKFLOWS.md) | Flujos de trabajo diarios y patrones de equipo |

### Estructura Interna

Una vez adoptado, StrayMark crea un directorio `.straymark/` en tu proyecto para gobernanza de desarrollo:

```
.straymark/
├── 00-governance/           # Políticas y reglas
├── 01-requirements/         # Documentos REQ
├── 02-design/decisions/     # Documentos ADR
├── 03-implementation/       # Guías de implementación (incl. estrategia Git)
├── 04-testing/              # Documentos TES
├── 05-operations/incidents/ # Documentos INC
├── 06-evolution/technical-debt/ # Documentos TDE
├── 07-ai-audit/
│   ├── agent-logs/          # Documentos AILOG
│   ├── decisions/           # Documentos AIDEC
│   └── ethical-reviews/     # Documentos ETH
└── templates/               # Plantillas de documentos
```

### Convención de Nomenclatura

```
[TIPO]-[YYYY-MM-DD]-[NNN]-[descripcion].md
```

Ejemplo: `ADR-2025-01-27-001-usar-postgresql-para-persistencia.md`

---

## Cómo Funciona

### 1. La IA Hace un Cambio

Un asistente de IA trabajando en tu código automáticamente:

```yaml
# Crea: .straymark/07-ai-audit/agent-logs/AILOG-2025-01-27-001-implementar-auth.md
---
id: AILOG-2025-01-27-001
title: Implementar autenticación JWT
agent: claude-code-v1.0
confidence: high
risk_level: high
review_required: true
---
```

### 2. Humano Revisa (Cuando es Necesario)

Cambios de alto riesgo o baja confianza son marcados:

```
AILOG-2025-01-27-001-implementar-auth.md
   Agent: claude-code-v1.0
   Confidence: high
   Risk Level: high
   Review Required: YES
```

### 3. Las Decisiones se Preservan

Al elegir entre alternativas, las decisiones se documentan:

```yaml
# Crea: .straymark/07-ai-audit/decisions/AIDEC-2025-01-27-001-estrategia-auth.md
---
id: AIDEC-2025-01-27-001
title: Elegir JWT sobre autenticación basada en sesiones
alternatives_considered:
  - JWT tokens (elegido)
  - Session cookies
  - Solo OAuth
justification: "Requisito de arquitectura sin estado..."
---
```

### 4. Preocupaciones Éticas son Marcadas

Cuando la IA encuentra consideraciones éticas:

```yaml
# Crea: .straymark/07-ai-audit/ethical-reviews/ETH-2025-01-27-001-datos-usuario.md
---
id: ETH-2025-01-27-001
title: Alcance de recolección de datos de usuario
status: draft  # Requiere aprobación humana
review_required: true
concerns:
  - Cumplimiento GDPR
  - Minimización de datos
---
```

---

## Validación

### Hook Pre-commit

Configura un hook de Git que ejecute la validación automáticamente antes de cada commit:

```bash
# Crear el hook pre-commit
echo '#!/bin/sh
straymark validate --staged' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Validación Manual

```bash
# Multiplataforma (Linux, macOS, Windows)
straymark validate
```

### GitHub Actions

El flujo de trabajo incluido (`.github/workflows/docs-validation.yml`) valida automáticamente:
- Convenciones de nomenclatura de archivos
- Campos de metadatos requeridos
- Detección de información sensible
- Formato Markdown
- Integridad de enlaces internos

---

## Skills

StrayMark incluye skills para agentes IA que habilitan la **creación activa de documentación**.

> **Sistema Binario**: StrayMark usa un sistema pasivo (agentes auto-documentan via instrucciones de contexto) y un sistema activo (usuarios invocan skills para crear documentación manualmente o cuando el agente omitió algo).

### Skills Disponibles

| Skill | Propósito | Claude | Gemini | Codex |
|-------|-----------|--------|--------|-------|
| `/straymark-status` | Verificar cumplimiento de documentación | ✅ | ✅ | ✅ |
| `/straymark-new` | Crear cualquier tipo de documento (unificado) | ✅ | ✅ | ✅ |
| `/straymark-ailog` | Creación rápida de AILOG | ✅ | ✅ | ✅ |
| `/straymark-aidec` | Creación rápida de AIDEC | ✅ | ✅ | ✅ |
| `/straymark-adr` | Creación rápida de ADR | ✅ | ✅ | ✅ |
| `/straymark-architecture` *(fw-4.29.0+, EXPERIMENTAL)* | Generar + refinar con agente el modelo de arquitectura (reasignar capas, cablear links, sincronizar DrawIO, validar) | ✅ | ✅ | ✅ |
| `/straymark-architecture-sync` *(fw-4.29.0+, EXPERIMENTAL)* | Mantener el modelo de arquitectura al día conforme crece el código (append-only) | ✅ | ✅ | ✅ |
| `/straymark-loom` *(fw-4.29.0+, EXPERIMENTAL)* | Manejar el ciclo de vida del servidor Loom (up / down / status), sin terminal | ✅ | ✅ | ✅ |

> **Usuarios de Codex CLI** *(fw-4.19.0+)*: Codex lee los skills desde `~/.codex/skills/` (a nivel de usuario), no desde el árbol del proyecto. Después de `straymark init` (o de cualquier `straymark update` posterior), ejecuta `straymark install-skills --agent codex` una vez para poblarlo desde `.codex/skills/` del proyecto.

### Ejemplos de Uso

```bash
# Verificar estado de documentación
/straymark-status

# Crear documentación (agente sugiere tipo)
/straymark-new

# Forzar tipo específico
/straymark-new ailog

# Accesos directos
/straymark-ailog
/straymark-aidec
/straymark-adr
```

### Comandos CLI (Uso Manual)

Para usuarios que prefieren línea de comandos o usan agentes sin soporte de skills:

```bash
# Creación interactiva de documentos
straymark new

# Crear tipo específico directamente
straymark new --doc-type ailog

# Verificar estado de documentación
straymark status
```

### Reporte de Agentes

Los agentes IA reportan su estado de documentación al final de cada tarea:

| Estado | Significado |
|--------|-------------|
| `StrayMark: Created AILOG-...` | Documentación fue creada |
| `StrayMark: No documentation required` | Cambio menor (<10 líneas) |
| `StrayMark: Documentation pending` | Puede necesitar revisión manual |

### Arquitectura Multi-Agente

StrayMark proporciona soporte nativo de skills para múltiples agentes IA a través de una arquitectura en capas:

```
tu-proyecto/
├── .agent/workflows/       # 🌐 Agnóstico (Antigravity, futuros agentes)
│   ├── straymark-new.md
│   ├── straymark-status.md
│   └── ...
├── .gemini/skills/         # 🔵 Gemini CLI (Google)
│   ├── straymark-new/SKILL.md
│   └── ...
├── .claude/skills/         # 🟣 Claude Code (Anthropic)
│   ├── straymark-new/SKILL.md
│   └── ...
└── .codex/skills/          # 🟢 Codex CLI (OpenAI) — se instala en ~/.codex/skills/
    ├── straymark-new/SKILL.md
    └── ...
```

| Directorio | Agente | Producto | Formato |
|------------|--------|----------|---------|
| `.agent/workflows/` | Antigravity, genérico | Extensiones VS Code/Cursor | `skill-name.md` con frontmatter YAML |
| `.gemini/skills/` | Gemini CLI | CLI terminal de Google | `skill-name/SKILL.md` |
| `.claude/skills/` | Claude Code | Agente de codificación de Anthropic | `skill-name/SKILL.md` |
| `.codex/skills/` *(fw-4.19.0+)* | Codex CLI | Agente de codificación de OpenAI | `skill-name/SKILL.md` (frontmatter mínimo) — se instala en `~/.codex/skills/` vía `straymark install-skills --agent codex` |

> **Nota**: `.agent/` es el estándar **agnóstico de proveedor**. Los directorios específicos de agentes (`.gemini/`, `.claude/`) proporcionan compatibilidad para esas plataformas siguiendo sus convenciones nativas.

Todas las implementaciones de skills son **funcionalmente idénticas**—solo difiere el formato para coincidir con los requisitos de cada agente.

---

## Plataformas Soportadas

### Asistentes de Codificación IA

| Plataforma | Archivo de Config | Estado |
|------------|-------------------|--------|
| Universal (estándar AGENTS.md) | `AGENTS.md` | Soporte completo |
| Claude Code | `CLAUDE.md` | Soporte completo |
| Cursor | `.cursorrules` | Soporte completo |
| GitHub Copilot CLI | `.github/copilot-instructions.md` | Soporte completo |
| Gemini CLI | `GEMINI.md` | Soporte completo |
| Codex CLI (OpenAI) *(fw-4.19.0+)* | `AGENTS.md` + `~/.codex/skills/` | Soporte completo (ejecuta `straymark install-skills --agent codex`) |

### Sistemas Operativos

| SO | Validación |
|----|------------|
| Linux | `straymark validate` |
| macOS | `straymark validate` |
| Windows | `straymark validate` |

### Plataformas CI/CD

| Plataforma | Soporte |
|------------|---------|
| GitHub Actions | Flujo de trabajo incluido |
| GitLab CI | Adaptable desde GitHub Actions |
| Azure DevOps | Adaptable desde GitHub Actions |

---

## Contribuir

¡Damos la bienvenida a contribuciones! Ver [CONTRIBUTING.md](CONTRIBUTING.md) para guías.

### Formas de Contribuir

- Reportar bugs
- Sugerir características
- Mejorar documentación
- Enviar pull requests
- Agregar traducciones

---

## Licencia

Este proyecto está licenciado bajo la Licencia MIT - ver el archivo [LICENSE](../../../LICENSE) para detalles.

---

## Acerca de Strange Days Tech, S.A.S.

<div align="center">

**[Strange Days Tech](https://strangedays.tech)** construye herramientas para desarrollo de software responsable asistido por IA.

Nuestro ecosistema open-source:

| Proyecto | Descripción |
|----------|-------------|
| **[StrayMark](https://github.com/StrangeDaysTech/straymark)** | La disciplina cognitiva que tus proyectos asistidos por IA necesitan |
| **[arborist-metrics](https://github.com/StrangeDaysTech/arborist-metrics/)** | Librería de análisis de complejidad de código multi-lenguaje para Rust — [crates.io](https://crates.io/crates/arborist-metrics) |

[Sitio Web](https://strangedays.tech) • [GitHub](https://github.com/StrangeDaysTech)

</div>

---

<div align="center">

**StrayMark** — Disciplina de ingeniería, externalizada. Compliance, como side effect.

[Volver arriba](#straymark)

</div>
