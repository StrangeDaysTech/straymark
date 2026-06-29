# StrayMark - Guía de Adopción

**Una guía completa para adoptar StrayMark en proyectos nuevos o existentes.**


---

## Tabla de Contenidos

1. [¿Qué es StrayMark?](#qué-es-straymark-framework)
2. [¿Para quién es?](#para-quién-es)
3. [Beneficios](#beneficios)
4. [Cumplimiento de Estándares](#cumplimiento-de-estándares)
5. [Ruta de Adopción A: Proyectos Nuevos](#ruta-de-adopción-a-proyectos-nuevos)
6. [Ruta de Adopción B: Proyectos Existentes](#ruta-de-adopción-b-proyectos-existentes)
7. [Configuración](#configuración)
8. [Verificación](#verificación)
9. [Preguntas Frecuentes](#preguntas-frecuentes)

---

## ¿Qué es StrayMark? {#qué-es-straymark-framework}

StrayMark es un **framework + CLI** que externaliza la disciplina cognitiva del trabajo de ingeniería de software senior — alcance explícito, decisiones declaradas, riesgos nombrados, alternativas registradas, rastros auditables — en archivos versionados que viven junto al código. La intención es acotar el espacio de decisión del agente para que el trabajo asistido por IA se mantenga coherente a través de muchos turnos en lugar de derivar hacia deuda técnica oculta.

Proporciona:

- **12 tipos de documentos estructurados** cubriendo el ciclo de vida completo del desarrollo y la IA
- **Charter** como unidad de ejecución del agente — trabajo acotado declarado ex-ante, auditado ex-post
- **Responsabilidad de agentes IA** mediante identificación obligatoria, seguimiento de confianza y límites de autonomía
- **Supervisión humana** a través de flujos de revisión requeridos para cambios críticos y de alto riesgo
- **Trazabilidad** conectando requisitos → diseño → implementación → pruebas → incidentes
- **Evidencia regulatoria** — como side effect, los artefactos producidos mapean limpiamente a EU AI Act, ISO 42001, NIST AI RMF y (opt-in) PIPL/TC260/GB-45438/CAC

### Principio Fundamental

> **"Ningún cambio significativo sin un rastro documentado — y un espacio de decisión acotado para el agente."**

StrayMark asegura que cada cambio significativo — ya sea hecho por humano o IA — esté documentado, atribuido y sea auditable. La disciplina produce evidencia compatible con **ISO/IEC 42001**, **EU AI Act** y **NIST AI RMF** — pero la meta es la calidad de ingeniería primero; el cumplimiento es lo que cae como subproducto cuando la disciplina es real (ver Principio #4 en [`Propuesta/straymark-design-principles.md`](https://github.com/StrangeDaysTech/straymark/blob/main/Propuesta/straymark-design-principles.md)).

### ¿Por Qué Ahora?

Los agentes de IA generan código rápido. No generan código coherente. Después de suficientes turnos, un agente pierde el hilo: re-introduce patrones rechazados por el equipo, acumula deuda técnica oculta y produce trabajo que compila pero no encaja con el grano del sistema. Cuanto más rápido va el agente, más difícil es ver esa deuda — hasta que una regresión, un incidente o una refactorización la sacan a la superficie.

En paralelo, el piso regulatorio sube — el **EU AI Act es obligatorio desde agosto 2026** e **ISO/IEC 42001** es ahora el estándar internacional para Sistemas de Gestión de IA. Adoptar StrayMark aborda primero el problema de ingeniería; la evidencia regulatoria se produce como subproducto de la disciplina, lista cuando un auditor la pida.

### Lo que StrayMark NO Es

- No es un generador de documentación — proporciona estructura, plantillas y reglas de gobernanza
- No es un reemplazo para comentarios de código o documentación de API
- No es una herramienta de gestión de proyectos o sistema de control de versiones
- No es un gateway de LLM, un evaluador de modelos ni un filtro de alucinaciones
- No es una certificación ISO 42001 completa — produce evidencia compatible dentro de su alcance, no la certificación misma
- No es un sustituto del juicio de un ingeniero senior

---

## ¿Para Quién Es?

### Usuario primario

El usuario primario de StrayMark es el **ingeniero senior orquestando agentes de IA sobre un sistema no trivial** — alguien con criterio técnico fuerte que usa agentes para abordar trabajo que no podría hacer solo de forma realista, y que necesita disciplina cognitiva externalizada para que el agente no introduzca caos sistémico. Los flujos, defaults y lenguaje del framework están afinados para esa persona; las audiencias secundarias se sirven sobre esa base, nunca a su costa.

### Usuarios Objetivo

| Tipo de Usuario | Razones de Adopción |
|-----------------|---------------------|
| **Ingenieros senior orquestando agentes de IA** | Externalizar alcance/decisiones/riesgos para que el agente se mantenga dentro de los rieles a través de muchos turnos |
| **Tech leads y arquitectos** | Estandarizar cómo trabaja el equipo con asistentes de IA; preservar memoria institucional de decisiones |
| **Desarrolladores individuales en proyectos serios** | Rastrear decisiones y cambios asistidos por IA con estructura, sin inventarla cada vez |
| **Mantenedores Open Source** | Documentar decisiones de contribución de forma transparente y revisable |
| **Equipos en entornos regulados** (finanzas, salud, UE, China) | Trazabilidad integrada en el flujo en lugar de reconstruida para auditorías |
| **Organizaciones buscando ISO 42001 / EU AI Act readiness** | Producir evidencia lista para certificación como side effect del trabajo de ingeniería normal |

### Entornos de Desarrollo Compatibles

StrayMark proporciona archivos de configuración para:

| Plataforma | Archivo de Configuración | Estado |
|------------|--------------------------|--------|
| **Universal (estándar AGENTS.md)** | `AGENTS.md` | Soportado |
| **Claude Code** (Anthropic) | `CLAUDE.md` | Soportado |
| **Cursor** | `.cursorrules` | Soportado |
| **GitHub Copilot CLI** | `.github/copilot-instructions.md` | Soportado |
| **Gemini CLI** (Google) | `GEMINI.md` | Soportado |
| **Otras Herramientas IA** | Copiar reglas de cualquier archivo de config | Adaptable |

### Metodologías Compatibles

StrayMark funciona con cualquier metodología de desarrollo:

| Metodología | Cómo se Integra StrayMark |
|-------------|---------------------------|
| **Agile/Scrum** | Documentos REQ se mapean a historias de usuario; ADRs capturan decisiones de sprint |
| **Cascada** | Trazabilidad completa desde requisitos hasta implementación |
| **DevOps/SRE** | Documentos INC para post-mortems; TDE para seguimiento de deuda técnica |
| **Domain-Driven Design** | ADRs documentan decisiones de contextos delimitados |
| **Test-Driven Development** | Documentos TES capturan estrategias de prueba |

---

## Beneficios

### Para Disciplina de Ingeniería (la intención primaria)

| Beneficio | Descripción |
|-----------|-------------|
| **Espacio de decisión acotado para el agente** | Los Charters declaran alcance/decisiones/riesgos ex-ante para que el agente ejecute contra restricciones en lugar de inventarlas |
| **Coherencia a través de muchos turnos** | La memoria del proyecto vive en archivos versionados; el agente (y el siguiente mantenedor) puede reconstruir contexto sin depender de una sesión de chat |
| **Memoria institucional** | Las decisiones sobreviven a cambios de personal, rotación de agentes y churn de tooling |
| **Aceleración de onboarding** | Los nuevos miembros (humanos o agentes) entienden el *por qué* a través de ADRs y AIDECs, no solo el *qué* a través del código |
| **Reducción de retrabajo** | El contexto preservado previene errores repetidos cuando el alcance se reabre meses después |
| **Responsabilidad clara** | Saber quién (o qué agente) hizo cada cambio, con qué confianza |

### Para Desarrollo Asistido por IA

| Beneficio | Descripción |
|-----------|-------------|
| **Transparencia de IA** | Cada acción de IA se registra con niveles de confianza |
| **Supervisión humana** | Decisiones críticas requieren aprobación humana |
| **Salvaguardas éticas** | Documentos ETH y DPIA marcan preocupaciones para decisión humana |
| **Métricas de gobernanza** | `straymark metrics` rastrea tasas de revisión, distribución de riesgo y tendencias |

### Para Cumplimiento Regulatorio (como side effect)

| Beneficio | Descripción |
|-----------|-------------|
| **Listo para EU AI Act** | Plantillas de clasificación de riesgo, reporte de incidentes y transparencia integradas |
| **Compatible con ISO 42001** | La estructura de documentación se alinea con requisitos de auditoría de certificación |
| **Mapeado a NIST AI RMF** | 12 categorías de riesgo GenAI y funciones de gobernanza cubiertas explícitamente |
| **Trazas de auditoría completas** | `straymark audit` genera reportes exportables de línea temporal y trazabilidad |
| **Scoring de cumplimiento** | `straymark compliance` proporciona análisis de brechas regulatorias basado en porcentajes |

---

## Cumplimiento de Estándares

La disciplina que StrayMark externaliza — alcance explícito, decisiones declaradas, riesgos nombrados, alternativas registradas — produce, como efecto secundario, evidencia que mapea limpiamente a los principales marcos de ingeniería y de gobernanza de IA. Las tablas a continuación describen *cómo* los artefactos de StrayMark se alinean con cada estándar; la meta es el trabajo de ingeniería, la alineación es lo que cae como subproducto:

### Estándares de Ingeniería de Software

| Estándar | Cómo Ayuda StrayMark |
|----------|---------------------|
| **IEEE 830** (SRS) | Documentos REQ siguen formato estructurado de requisitos |
| **ISO/IEC 25010** | Atributos de calidad documentados en ADRs |
| **ISO/IEC 12207** | Cobertura de documentación del ciclo de vida |

### Documentación de Arquitectura

| Estándar | Cómo Ayuda StrayMark |
|----------|---------------------|
| **ADR (Architecture Decision Records)** | Soporte nativo de ADR con metadatos extendidos |
| **arc42** | ADRs complementan documentación de decisiones de arc42 |
| **Modelo C4** | ADRs documentan decisiones en cada nivel de C4 |

### Cumplimiento y Gobernanza

| Regulación | Cómo Ayuda StrayMark |
|------------|---------------------|
| **GDPR** | Documentos ETH para evaluaciones de impacto de privacidad |
| **SOC 2** | Documentación de cambios y registro de accesos |
| **ISO 27001** | Documentación de decisiones de seguridad |
| **HIPAA** | Pistas de auditoría para aplicaciones de salud |

### Gobernanza de IA

| Marco | Cómo Ayuda StrayMark |
|-------|---------------------|
| **EU AI Act** | Transparencia a través de AILOG/AIDEC; supervisión humana via ETH |
| **NIST AI RMF** | Documentación de riesgos en registros de decisión |
| **IEEE 7000** | Consideraciones éticas en documentos ETH |

### Cobertura Regulatoria China *(opt-in vía `regional_scope: china`)*

| Estándar | Cómo Ayuda StrayMark |
|----------|---------------------|
| **TC260 AI Safety Governance Framework v2.0** | Clasificación de riesgo en 5 niveles (TC260RA), campos `tc260_*` en ETH/MCARD/AILOG/SEC |
| **PIPL — Personal Information Protection Law** | Plantilla PIPIA con secciones del Art. 55-56, campos `pipl_*`, retención ≥ 3 años (`TYPE-003`) |
| **GB 45438-2025** *(obligatorio)* | Plantilla AILABEL cubriendo etiquetado explícito + implícito para contenido generativo |
| **CAC Algorithm Filing** | Plantilla CACFILE con seguimiento de registro simple + dual; cross-checks desde MCARD vía `cac_filing_required` |
| **GB/T 45652-2025** | Sección de seguridad de datos de entrenamiento en SBOM y MCARD; campo `gb45652_training_data_compliance` |
| **CSL 2026** | Extensión de INC con `csl_severity_level`, reglas de coherencia de plazos (ventanas 1h / 4h+72h+30d) |

> Activar añadiendo `china` a `regional_scope` en `.straymark/config.yml`. Ver la sección *Configuración* abajo y la guía instalada `CHINA-REGULATORY-FRAMEWORK.md` para detalles.

---

## Ruta de Adopción A: Proyectos Nuevos

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

Luego inicializa y haz commit:

```bash
cd tu-proyecto
straymark init .

git add .straymark/ STRAYMARK.md
git commit -m "chore: adoptar StrayMark"
```

El CLI automáticamente:
- Descarga la última versión de StrayMark desde GitHub
- Configura la estructura de directorios `.straymark/`
- Crea `STRAYMARK.md` con las reglas de gobernanza
- Configura las directivas de agentes IA (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
- Copia workflows de CI/CD

### Opción 2: Configuración Manual

1. **Descargar el último release**

   Ve a [GitHub Releases](https://github.com/StrangeDaysTech/straymark/releases) y descarga el último release `fw-*` (ej. `fw-4.13.4`).

2. **Extraer en tu proyecto**
   ```bash
   unzip straymark-fw-*.zip -d tu-proyecto/
   ```

3. **Commit de la estructura**
   ```bash
   git add .straymark/ STRAYMARK.md
   git commit -m "chore: adoptar StrayMark para gobernanza de documentación"
   ```

---

## Ruta de Adopción B: Proyectos Existentes

### Fase 1: Evaluación (Día 1)

1. **Evaluar documentación actual**

   Responde estas preguntas:
   - ¿Tienes ADRs existentes? ¿Dónde están ubicados?
   - ¿Tienes una carpeta `docs/`? ¿Qué contiene?
   - ¿Hay convenciones de nomenclatura ya establecidas?
   - ¿Usas algún asistente de codificación con IA?

2. **Planificar la migración**

   | Estado Actual | Acción Recomendada |
   |---------------|-------------------|
   | Sin documentación | Comenzar desde cero con StrayMark |
   | Docs en carpeta `docs/` | Mantener `docs/` para docs orientados al usuario, agregar `.straymark/` para docs de desarrollo |
   | ADRs existentes | Migrar a `.straymark/02-design/decisions/` con nueva nomenclatura |
   | Documentación mixta | Categorizar y migrar gradualmente |

### Fase 2: Instalación (Día 1-2)

1. **Agregar estructura StrayMark**
   ```bash
   # Usando CLI (recomendado)
   straymark init .

   # O manualmente: descargar el último release fw-* desde GitHub Releases
   # https://github.com/StrangeDaysTech/straymark/releases
   ```

2. **Resolver conflictos con `docs/` existente**

   StrayMark usa `.straymark/` específicamente para evitar conflictos:

   ```
   tu-proyecto/
   ├── docs/                    ← Mantener para docs de API, guías de usuario, etc.
   │   ├── api/
   │   └── user-guide/
   ├── .straymark/              ← Agregar para documentación de desarrollo
   │   ├── 00-governance/
   │   ├── 01-requirements/
   │   └── ...
   └── src/
   ```

### Fase 3: Migración (Semana 1-2)

1. **Migrar ADRs existentes**

   Para cada ADR existente:
   ```bash
   # Antiguo: docs/adr/001-usar-postgresql.md
   # Nuevo: .straymark/02-design/decisions/ADR-2024-01-15-001-usar-postgresql.md
   ```

   Agregar metadatos StrayMark al front-matter:
   ```yaml
   ---
   id: ADR-2024-01-15-001
   title: Usar PostgreSQL para base de datos principal
   status: accepted
   created: 2024-01-15
   agent: human
   confidence: high
   review_required: false
   risk_level: high
   # Preservar metadatos originales
   original_id: "001"
   migrated_from: "docs/adr/001-usar-postgresql.md"
   ---
   ```

2. **Documentar la migración**

   Crear un AILOG documentando la migración:
   ```
   .straymark/07-ai-audit/agent-logs/AILOG-2025-01-27-001-adopcion-straymark.md
   ```

### Fase 4: Adopción del Equipo (Semana 2-4)

1. **Actualizar guías de contribución**

   Agregar a tu `CONTRIBUTING.md`:
   ```markdown
   ## Documentación

   Este proyecto usa [StrayMark](https://github.com/StrangeDaysTech/straymark) para gobernanza de documentación.

   - Todos los cambios significativos deben documentarse en `.straymark/`
   - Cambios asistidos por IA requieren entradas AILOG
   - Decisiones arquitectónicas requieren documentos ADR

   Ver `.straymark/QUICK-REFERENCE.md` para tipos de documentos y nomenclatura.
   ```

2. **Habilitar hooks pre-commit (opcional)**
   ```bash
   # Crear el hook pre-commit
   echo '#!/bin/sh
   straymark validate --staged' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit

   # O con Husky
   npx husky add .husky/pre-commit "straymark validate --staged"
   ```

3. **Habilitar GitHub Actions (opcional)**

   El flujo de trabajo en `.github/workflows/docs-validation.yml` validará automáticamente la documentación en PRs.

### Fase 5: Implementación Gradual

| Semana | Enfoque |
|--------|---------|
| Semana 1 | Equipo principal adopta StrayMark para nuevas decisiones |
| Semana 2 | Migrar ADRs existentes críticos |
| Semana 3 | Habilitar validación en CI/CD |
| Semana 4 | Adopción completa del equipo; documentar deuda técnica existente |

---

## Configuración

### Alcance Regulatorio Regional

`.straymark/config.yml` controla qué frameworks de cumplimiento evalúa `straymark compliance` y qué tipos de documento expone `straymark new`:

```yaml
regional_scope:
  - global   # NIST AI RMF + ISO/IEC 42001 (siempre recomendado)
  - eu       # EU AI Act + GDPR
  - china    # TC260 v2.0, PIPL/PIPIA, GB 45438, CAC, GB/T 45652, CSL 2026
```

**Default si se omite**: `[global, eu]` — preserva el comportamiento de toda versión de StrayMark anterior a `fw-4.3.0`.

Cuando añades `china` a la lista:

- Cuatro tipos de documento específicos de China están disponibles vía `straymark new`: `PIPIA`, `CACFILE`, `TC260RA`, `AILABEL`.
- Seis nuevos checkers de cumplimiento se ejecutan con `straymark compliance` (o vía `--region china` / `--standard china-*`).
- Doce reglas de validación scope-aware se activan (`CROSS-004` a `CROSS-011`, `TYPE-003` a `TYPE-006`).
- Cinco nuevas guías de gobernanza referenciadas desde `.straymark/00-governance/` (`CHINA-REGULATORY-FRAMEWORK.md`, más guías por framework para TC260, PIPL/PIPIA, registro CAC, y etiquetado GB 45438) — todas disponibles en EN, ES y zh-CN.

Un proyecto sin `china` en `regional_scope` no se ve afectado: ningún archivo nuevo, ningún prompt nuevo, ninguna regla nueva. Añadir `china` después es totalmente reversible.

### Personalizar Identificadores de Agente

Cada plataforma de IA tiene su propio archivo de configuración que:

1. Identifica al agente (ej. `claude-code-v1.0`)
2. Define cuándo documentar (>10 líneas, cambios de seguridad, etc.)
3. Establece límites de autonomía
4. Especifica ubicación de plantillas
5. Requiere reporte de documentación
6. **Impone flujo de trabajo Git** (nomenclatura de ramas, conventional commits, sin commits directos a `main`)

Actualiza el identificador de agente para que coincida con tu versionado:

```yaml
# En cualquier archivo de config de agente
agent: claude-code-v1.0      # Por defecto
agent: claude-code-v2.1      # Tu versión personalizada
agent: acme-corp-claude-v1   # Específico de organización
```

### Personalizar Tipos de Documento

Para agregar un nuevo tipo de documento:

1. **Crear la plantilla**
   ```
   .straymark/templates/TEMPLATE-NUEVOTIPO.md
   ```

2. **Actualizar docs de gobernanza**

   Agregar el nuevo tipo a:
   - `.straymark/00-governance/DOCUMENTATION-POLICY.md`
   - `.straymark/00-governance/AGENT-RULES.md`
   - `.straymark/QUICK-REFERENCE.md`

3. **Actualizar configs de agente**

   Agregar el nuevo tipo a todos los archivos de configuración de agente.

4. **Actualizar validación**

   Agregar el nuevo prefijo de tipo a:
   - Reglas de validación en el CLI (`straymark validate`)
   - `.github/workflows/docs-validation.yml`

### Personalizar Estructura de Carpetas

La estructura de carpetas numerada (`00-governance`, `01-requirements`, etc.) está diseñada para:
- Ordenamiento lógico en exploradores de archivos
- Clara separación de responsabilidades
- Navegación fácil

Puedes renombrar carpetas, pero actualiza todas las referencias en:
- Archivos de configuración de agente
- Documentos de gobernanza
- Reglas de validación del CLI

---

## Verificación

### Verificación con Skills (Claude Code)

Si usas Claude Code, verifica el cumplimiento de documentación con el skill integrado:

```bash
/straymark-status
```

Este skill muestra:
- Qué documentos StrayMark fueron creados recientemente
- Qué archivos modificados pueden necesitar documentación
- Estado general de cumplimiento de documentación

### Verificación Manual

Después de la adopción, verifica tu configuración:

```bash
# Ejecutar la validación (multiplataforma)
straymark validate
```

### Lista de Verificación

- [ ] Estructura de carpetas `.straymark/` existe
- [ ] Al menos un archivo de config de agente existe (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, etc.)
- [ ] Documentos de gobernanza presentes en `.straymark/00-governance/`
- [ ] Plantillas presentes en `.straymark/templates/`
- [ ] Estrategia de branching Git documentada en `.straymark/03-implementation/`
- [ ] `QUICK-REFERENCE.md` es accesible
- [ ] `straymark validate` se ejecuta sin errores
- [ ] (Opcional) Hook pre-commit está instalado
- [ ] (Opcional) Flujo de trabajo de GitHub Actions está habilitado

---

## Auditoría Externa (Opcional)

A partir de `fw-4.9.0`, cuando co-implementas Charters con un asistente IA en el loop (Claude Code, Gemini Code, Cursor), puedes opcionalmente correr una auditoría externa multi-modelo al cierre del Charter. Dos skills envuelven la orquestación subyacente del CLI:

- **`/straymark-audit-prompt CHARTER-XX`** — escribe el audit prompt unificado en el path canónico `.straymark/audits/<id>/audit-prompt.md`. El operador abre N CLIs auditoras y corre `/straymark-audit-execute` en cada una. Sin copy/paste.
- **`/straymark-audit-execute [CHARTER-XX]`** *(fw-4.9.0+)* — corre dentro de una CLI auditora (gemini-cli, claude-cli, copilot-cli, codex-cli). Lee el prompt, audita con tool use citando `path:línea`, escribe un report con el id del modelo en el nombre.
- **`/straymark-audit-review CHARTER-XX`** — consolida N reports en un `review.md` de seis secciones (Resumen ejecutivo / Alcance / Evaluación por auditor / Plan de remediación P0-P4 / Descartados / Calificación de auditores) y mergea el YAML `external_audit:` en la telemetría del Charter.

El agente **proactivamente ofrecerá** la auditoría en un momento específico del workflow — cuando la implementación está lista, el drift check está limpio, y `charter close` no se ha invocado. La recomendación es SÍ/NO basada en la superficie de riesgo y complejidad del Charter (heurísticas en `.straymark/00-governance/AGENT-RULES.md` §12).

**La auditoría externa es completamente opcional** y **nunca enforced**. El scope declarativo del Charter + drift check + disciplina AILOG ya proporcionan cierre riguroso. La auditoría agrega señal cross-modelo cuando el Charter tocó superficie de seguridad, introdujo componentes nuevos, o tiene funciones de alta complejidad en el diff. Declina libremente si el costo (2-3 auditores LLM) no se ajusta al valor de tu caso.

Para los comandos CLI subyacentes (PREPARE / CALIBRATE / FINALIZE / `--merge-into`), ver [`CLI-REFERENCE.md` § straymark charter audit](./CLI-REFERENCE.md#straymark-charter-audit).

---

## Preguntas Frecuentes

### Preguntas Generales

**P: ¿StrayMark reemplaza mi documentación existente?**

R: No. StrayMark es para *documentación del proceso de desarrollo* (decisiones, cambios, revisiones). Mantén tu carpeta `docs/` existente para documentación orientada al usuario, referencias de API y guías.

**P: ¿Necesito usar asistentes de codificación con IA para beneficiarme de StrayMark?**

R: No. StrayMark funciona también para equipos solo de humanos. Las características de auditoría de IA (AILOG, AIDEC, ETH) se vuelven especialmente valiosas al usar asistentes de IA, pero ADR, REQ, TDE y otros tipos de documentos son útiles para cualquier equipo.

**P: ¿Cuánto overhead agrega StrayMark?**

R: StrayMark sigue un principio de "documentación mínima viable". Solo los cambios significativos requieren documentación. Los cambios triviales (erratas, formato) están explícitamente excluidos.

### Preguntas Técnicas

**P: ¿Por qué usar `.straymark/` en lugar de `docs/`?**

R: La carpeta `docs/` se usa comúnmente para documentación orientada al usuario, GitHub Pages o docs de API generados. Usar `.straymark/` evita conflictos y separa claramente la documentación de desarrollo de la documentación de usuario.

**P: ¿Puedo usar StrayMark con monorepos?**

R: Sí. Puedes:
- Tener un `.straymark/` en la raíz para todo el monorepo
- Tener carpetas `.straymark/` separadas en cada paquete/servicio
- Usar un enfoque híbrido con gobernanza compartida en la raíz

**P: ¿Cómo manejo información sensible?**

R: StrayMark prohíbe explícitamente documentar credenciales, tokens o secretos. Los scripts de validación verifican patrones sensibles comunes y te advierten. Para decisiones genuinamente sensibles, documenta la *existencia* de la decisión sin los detalles sensibles.

### Preguntas de Adopción

**P: Mi equipo es resistente a más documentación. ¿Cómo los convenzo?**

R: Comienza pequeño:
1. Comienza solo con ADRs para decisiones arquitectónicas
2. Muestra valor a través de onboarding más rápido de nuevos miembros
3. Demuestra tiempo ahorrado al revisar decisiones antiguas
4. Expande gradualmente a otros tipos de documentos

**P: ¿Cómo manejo documentos creados antes de adoptar StrayMark?**

R: Tienes tres opciones:
1. **Migrar**: Convertir documentos antiguos al formato StrayMark (recomendado para docs importantes)
2. **Referenciar**: Mantener docs antiguos en su lugar, referenciarlos desde docs StrayMark
3. **Archivar**: Mover docs antiguos a una carpeta de archivo, comenzar de nuevo con StrayMark

**P: ¿Qué pasa si mi asistente de IA no sigue las reglas?**

R: Las reglas de StrayMark son instrucciones, no cumplimiento forzado. Si un asistente de IA crea documentación no conforme:
1. El hook pre-commit detectará errores de validación
2. CI/CD marcará problemas en PRs
3. Puedes corregir manualmente y educar a la IA en el siguiente prompt

---

## Obtener Ayuda

- **Referencia CLI**: [CLI-REFERENCE.md](CLI-REFERENCE.md) — referencia detallada de comandos
- **Flujos de Trabajo**: [WORKFLOWS.md](WORKFLOWS.md) — patrones de uso diario recomendados
- **Issues**: [GitHub Issues](https://github.com/StrangeDaysTech/straymark/issues)
- **Discusiones**: [GitHub Discussions](https://github.com/StrangeDaysTech/straymark/discussions)
- **Contribuir**: Ver [CONTRIBUTING.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/i18n/es/CONTRIBUTING.md)
