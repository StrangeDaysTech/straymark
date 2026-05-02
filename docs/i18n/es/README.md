<div align="center">

# DevTrail

**Plataforma de Gobernanza de IA para Desarrollo de Software Responsable**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../../LICENSE)
[![Crates.io](https://img.shields.io/crates/v/devtrail-cli.svg)](https://crates.io/crates/devtrail-cli)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Handbook](https://img.shields.io/badge/docs-Handbook-orange.svg)](../../../dist/.devtrail/QUICK-REFERENCE.md)
[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

[Inicio Rápido](#inicio-rápido) •
[Características](#características) •
[Documentación](#documentación) •
[Contribuir](#contribuir)

**Idiomas**: [English](../../../README.md) | Español | [简体中文](../zh-CN/README.md)

</div>

---

## El Problema

A medida que la IA se vuelve parte integral del desarrollo de software, las organizaciones enfrentan tres presiones convergentes:

- **Cumplimiento regulatorio**: El EU AI Act es obligatorio desde agosto 2026. ISO/IEC 42001 es el estándar internacional para gobernanza de IA. Los equipos necesitan evidencia documentada.
- **Brecha de gobernanza**: No hay forma estructurada de demostrar que las decisiones de IA están gobernadas, auditables y en cumplimiento — cada cambio de IA sin documentar es una responsabilidad.
- **Riesgo operacional**: ¿Quién hizo este cambio? ¿Qué alternativas se consideraron? ¿Fue apropiada la supervisión humana? Sin respuestas, el desarrollo asistido por IA es una caja negra.

## La Solución

DevTrail es una **plataforma de gobernanza de IA alineada con ISO 42001** que asegura que cada cambio significativo — ya sea hecho por humano o IA — esté documentado, atribuido y sea auditable.

> **"Ningún cambio significativo sin un rastro documentado — y prueba de gobernanza."**

Los equipos que adoptan DevTrail producen evidencia compatible con **certificación ISO/IEC 42001**, **cumplimiento del EU AI Act** y gestión de riesgos **NIST AI RMF** — mientras mejoran la calidad y trazabilidad del desarrollo.

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

### 📐 Alineación con Estándares

| Estándar | Integración con DevTrail |
|----------|--------------------------|
| **ISO/IEC 42001:2023** | Estándar vertebral — gobernanza de Sistemas de Gestión de IA |
| **EU AI Act** | Clasificación de riesgo, reporte de incidentes, transparencia |
| **NIST AI RMF / 600-1** | 12 categorías de riesgo GenAI en ETH/AILOG |
| **ISO/IEC 25010:2023** | Modelo de calidad de software en REQ/ADR |
| **ISO/IEC/IEEE 29148:2018** | Ingeniería de requisitos en REQ |
| **ISO/IEC/IEEE 29119-3:2021** | Documentación de pruebas en TES |
| **GDPR** | Protección de datos en ETH/DPIA |
| **OpenTelemetry** | Observabilidad (opcional) |

### 🤖 Soporte para Agentes IA

Pre-configurado para asistentes de codificación con IA populares:

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

### ✅ Automatización de Compliance

Herramientas CLI integradas para gobernanza:

- **`devtrail validate`** — 13 reglas de validación para corrección documental
- **`devtrail compliance`** — Puntuación de cumplimiento regulatorio (EU AI Act, ISO 42001, NIST AI RMF)
- **`devtrail metrics`** — KPIs de gobernanza, tasas de revisión, distribución de riesgo, tendencias
- **`devtrail analyze`** — Análisis de complejidad de código (cognitiva + ciclomática) impulsado por [arborist-metrics](https://github.com/StrangeDaysTech/arborist), nuestra librería open-source en Rust para métricas de código multi-lenguaje
- **`devtrail audit`** — Reportes de auditoría con línea temporal, mapas de trazabilidad y exportación HTML
- **Hooks pre-commit** + **GitHub Actions** para validación CI/CD

---

## Inicio Rápido

### Opción 1: CLI (Recomendado)

**Instalación rápida (binario precompilado):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

O instalar desde el código fuente con Cargo:

```bash
cargo install devtrail-cli
```

> **Nota:** `devtrail update-cli` detecta automáticamente cómo instalaste el CLI. Las instalaciones con binario precompilado se actualizan desde GitHub Releases; las instalaciones con cargo se actualizan via `cargo install`. Puedes forzar el método con `--method=github` o `--method=cargo`.

Luego inicializa en tu proyecto:

```bash
cd tu-proyecto
devtrail init .
```

El CLI descarga la última versión de DevTrail, configura el framework y los archivos de directivas de agentes IA automáticamente.

### Versionado

DevTrail usa tags de versión independientes para cada componente:

| Componente | Prefijo de tag | Ejemplo | Incluye |
|------------|---------------|---------|---------|
| Framework | `fw-` | `fw-4.4.0` | Plantillas (12 tipos), gobernanza, directivas |
| CLI | `cli-` | `cli-3.6.0` | El binario `devtrail` |

Verifica las versiones instaladas con `devtrail status` o `devtrail about`.

### Comandos CLI

| Comando | Descripción |
|---------|-------------|
| `devtrail init [path]` | Inicializar DevTrail en un proyecto |
| `devtrail update` | Actualizar framework y CLI |
| `devtrail update-framework` | Actualizar solo el framework |
| `devtrail update-cli` | Actualizar el binario del CLI |
| `devtrail remove [--full]` | Eliminar DevTrail del proyecto |
| `devtrail status [path]` | Mostrar estado de la instalación y estadísticas |
| `devtrail repair [path]` | Restaurar directorios y archivos del framework faltantes |
| `devtrail validate [path]` | Validar documentos por cumplimiento y corrección (use `--include-charters` para validar también `docs/charters/`) |
| `devtrail charter <subcomando>` | Gestionar Charters: `new`, `list`, `status` (unidades acotadas de trabajo declaradas ex-ante, auditadas ex-post) |
| `devtrail compliance [path]` | Verificar cumplimiento regulatorio (EU AI Act, ISO 42001, NIST) |
| `devtrail metrics [path]` | Mostrar métricas de gobernanza y estadísticas |
| `devtrail analyze [path]` | Analizar complejidad de código (métricas cognitiva + ciclomática) |
| `devtrail audit [path]` | Generar reportes de auditoría con línea temporal y trazabilidad |
| `devtrail explore [path]` | Explorar documentación interactivamente en terminal (TUI) |
| `devtrail about` | Mostrar información de versión y licencia |

Ver [Referencia CLI](adopters/CLI-REFERENCE.md) para uso detallado.

### Opción 2: Configuración Manual

```bash
# Descargar el último release ZIP del framework desde GitHub
# Ve a https://github.com/StrangeDaysTech/devtrail/releases
# y descarga el último release fw-* (ej. fw-4.4.0)

# Extraer y copiar a tu proyecto
unzip devtrail-fw-*.zip -d tu-proyecto/
cd tu-proyecto

# Commit
git add .devtrail/ DEVTRAIL.md
git commit -m "chore: adoptar DevTrail"
```

**Ver [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) para instrucciones detalladas, estrategias de migración y planes de implementación en equipos.**

---

## Documentación

La documentación de DevTrail está organizada por audiencia:

| Track | Para | Empieza aquí |
|-------|------|--------------|
| [**Adoptantes**](adopters/) | Equipos que adoptan DevTrail en sus proyectos | [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) |
| [**Contribuidores**](../../../docs/contributors/) | Desarrolladores que contribuyen a DevTrail | [TRANSLATION-GUIDE.md](../../../docs/contributors/TRANSLATION-GUIDE.md) |

**Adoptantes**: Sigue la [Guía de Adopción](adopters/ADOPTION-GUIDE.md) para instrucciones paso a paso, la [Referencia CLI](adopters/CLI-REFERENCE.md) para detalles de comandos, y la [Guía de Flujos de Trabajo](adopters/WORKFLOWS.md) para patrones de uso diario.

**Contribuidores**: Consulta [CONTRIBUTING.md](CONTRIBUTING.md) para guías de desarrollo, y la [Guía de Traducción](../../../docs/contributors/TRANSLATION-GUIDE.md) para agregar nuevos idiomas.

### Referencias Clave

| Documento | Descripción |
|-----------|-------------|
| [**Referencia Rápida**](../../../dist/.devtrail/QUICK-REFERENCE.md) | Resumen de tipos de documentos y nomenclatura |
| [DEVTRAIL.md](../../../dist/DEVTRAIL.md) | Reglas de gobernanza unificadas (fuente de verdad) |
| [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) | Guía de adopción para proyectos nuevos/existentes |
| [CLI-REFERENCE.md](adopters/CLI-REFERENCE.md) | Referencia completa de comandos CLI |
| [WORKFLOWS.md](adopters/WORKFLOWS.md) | Flujos de trabajo diarios y patrones de equipo |

### Estructura Interna

Una vez adoptado, DevTrail crea un directorio `.devtrail/` en tu proyecto para gobernanza de desarrollo:

```
.devtrail/
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
# Crea: .devtrail/07-ai-audit/agent-logs/AILOG-2025-01-27-001-implementar-auth.md
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
# Crea: .devtrail/07-ai-audit/decisions/AIDEC-2025-01-27-001-estrategia-auth.md
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
# Crea: .devtrail/07-ai-audit/ethical-reviews/ETH-2025-01-27-001-datos-usuario.md
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
devtrail validate --staged' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Validación Manual

```bash
# Multiplataforma (Linux, macOS, Windows)
devtrail validate
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

DevTrail incluye skills para agentes IA que habilitan la **creación activa de documentación**.

> **Sistema Binario**: DevTrail usa un sistema pasivo (agentes auto-documentan via instrucciones de contexto) y un sistema activo (usuarios invocan skills para crear documentación manualmente o cuando el agente omitió algo).

### Skills Disponibles

| Skill | Propósito | Claude | Gemini |
|-------|-----------|--------|--------|
| `/devtrail-status` | Verificar cumplimiento de documentación | ✅ | ✅ |
| `/devtrail-new` | Crear cualquier tipo de documento (unificado) | ✅ | ✅ |
| `/devtrail-ailog` | Creación rápida de AILOG | ✅ | ✅ |
| `/devtrail-aidec` | Creación rápida de AIDEC | ✅ | ✅ |
| `/devtrail-adr` | Creación rápida de ADR | ✅ | ✅ |

### Ejemplos de Uso

```bash
# Verificar estado de documentación
/devtrail-status

# Crear documentación (agente sugiere tipo)
/devtrail-new

# Forzar tipo específico
/devtrail-new ailog

# Accesos directos
/devtrail-ailog
/devtrail-aidec
/devtrail-adr
```

### Comandos CLI (Uso Manual)

Para usuarios que prefieren línea de comandos o usan agentes sin soporte de skills:

```bash
# Creación interactiva de documentos
devtrail new

# Crear tipo específico directamente
devtrail new --doc-type ailog

# Verificar estado de documentación
devtrail status
```

### Reporte de Agentes

Los agentes IA reportan su estado de documentación al final de cada tarea:

| Estado | Significado |
|--------|-------------|
| `DevTrail: Created AILOG-...` | Documentación fue creada |
| `DevTrail: No documentation required` | Cambio menor (<10 líneas) |
| `DevTrail: Documentation pending` | Puede necesitar revisión manual |

### Arquitectura Multi-Agente

DevTrail proporciona soporte nativo de skills para múltiples agentes IA a través de una arquitectura en capas:

```
tu-proyecto/
├── .agent/workflows/       # 🌐 Agnóstico (Antigravity, futuros agentes)
│   ├── devtrail-new.md
│   ├── devtrail-status.md
│   └── ...
├── .gemini/skills/         # 🔵 Gemini CLI (Google)
│   ├── devtrail-new/SKILL.md
│   └── ...
└── .claude/skills/         # 🟣 Claude Code (Anthropic)
    ├── devtrail-new/SKILL.md
    └── ...
```

| Directorio | Agente | Producto | Formato |
|------------|--------|----------|---------|
| `.agent/workflows/` | Antigravity, genérico | Extensiones VS Code/Cursor | `skill-name.md` con frontmatter YAML |
| `.gemini/skills/` | Gemini CLI | CLI terminal de Google | `skill-name/SKILL.md` |
| `.claude/skills/` | Claude Code | Agente de codificación de Anthropic | `skill-name/SKILL.md` |

> **Nota**: `.agent/` es el estándar **agnóstico de proveedor**. Los directorios específicos de agentes (`.gemini/`, `.claude/`) proporcionan compatibilidad para esas plataformas siguiendo sus convenciones nativas.

Todas las implementaciones de skills son **funcionalmente idénticas**—solo difiere el formato para coincidir con los requisitos de cada agente.

---

## Plataformas Soportadas

### Asistentes de Codificación IA

| Plataforma | Archivo de Config | Estado |
|------------|-------------------|--------|
| Claude Code | `CLAUDE.md` | Soporte completo |
| Cursor | `.cursorrules` | Soporte completo |
| GitHub Copilot CLI | `.github/copilot-instructions.md` | Soporte completo |
| Gemini CLI | `GEMINI.md` | Soporte completo |

### Sistemas Operativos

| SO | Validación |
|----|------------|
| Linux | `devtrail validate` |
| macOS | `devtrail validate` |
| Windows | `devtrail validate` |

### Plataformas CI/CD

| Plataforma | Soporte |
|------------|---------|
| GitHub Actions | Flujo de trabajo incluido |
| GitLab CI | Adaptable desde GitHub Actions |
| Azure DevOps | Adaptable desde GitHub Actions |

---

## Alineación con Estándares

DevTrail se alinea con:

- **ADR** (Architecture Decision Records) - Soporte nativo
- **IEEE 830** - Estructura de documentación de requisitos
- **ISO/IEC 25010** - Atributos de calidad en ADRs
- **GDPR** - Documentación de impacto de privacidad (ETH)
- **EU AI Act** - Transparencia de IA y supervisión humana
- **NIST AI RMF** - Documentación de riesgos

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
| **[DevTrail](https://github.com/StrangeDaysTech/devtrail)** | Plataforma de gobernanza de IA para desarrollo de software responsable |
| **[arborist-metrics](https://github.com/StrangeDaysTech/arborist)** | Librería de análisis de complejidad de código multi-lenguaje para Rust — [crates.io](https://crates.io/crates/arborist-metrics) |

[Sitio Web](https://strangedays.tech) • [GitHub](https://github.com/StrangeDaysTech)

</div>

---

<div align="center">

**DevTrail** — Porque cada cambio cuenta una historia.

[Volver arriba](#devtrail)

</div>
