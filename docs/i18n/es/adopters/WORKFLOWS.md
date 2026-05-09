# StrayMark - Flujos de Trabajo Recomendados

**Patrones y cadencias para usar StrayMark en el día a día.**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

**Idiomas**: [English](../../../adopters/WORKFLOWS.md) | Español | [简体中文](../../zh-CN/adopters/WORKFLOWS.md)

---

## Tabla de Contenidos

1. [Después de la Configuración Inicial](#después-de-la-configuración-inicial)
2. [Desarrollo Diario](#desarrollo-diario)
3. [Mantener StrayMark Actualizado](#mantener-straymark-actualizado)
4. [Verificar el Estado del Proyecto](#verificar-el-estado-del-proyecto)
5. [Usar Skills (Documentación Activa)](#usar-skills-documentación-activa)
6. [Patrones de Equipo](#patrones-de-equipo)
7. [Entender las Versiones](#entender-las-versiones)

---

## Después de la Configuración Inicial

Ejecutaste `straymark init .` e hiciste commit del resultado. ¿Ahora qué?

1. **Abre tu proyecto** con tu asistente de codificación IA (Claude Code, Cursor, Gemini CLI, etc.)
2. El asistente **leerá automáticamente** las directivas de StrayMark (`CLAUDE.md`, `GEMINI.md`, etc.)
3. A partir de este punto, el asistente **crea documentación** en `.straymark/` como parte de su flujo de trabajo normal
4. **No se necesita configuración adicional** — StrayMark funciona de forma pasiva a través de los archivos de directivas

---

## Desarrollo Diario

### El Ciclo Pasivo

1. Trabaja normalmente con tu asistente IA — escribe features, corrige bugs, refactoriza
2. La IA crea documentos en `.straymark/` según las reglas de gobernanza:
   - **AILOG** para implementaciones significativas (>10 líneas cambiadas)
   - **AIDEC** al elegir entre alternativas
   - **ADR** para decisiones arquitectónicas
   - **ETH** cuando surgen preocupaciones éticas
3. Revisa los documentos marcados con `review_required: true`
4. Haz commit de la documentación junto con los cambios de código correspondientes

### Cuándo Crear Documentos Manualmente

Usa el sistema activo (skills) cuando:

- La IA omitió documentar un cambio significativo
- Tú (un humano) tomaste una decisión que debería registrarse
- Quieres crear un documento REQ, TES, TDE o INC
- Quieres verificar el cumplimiento de documentación

---

## Mantener StrayMark Actualizado

### Cadencia Recomendada

- **Mensualmente** o cuando veas un nuevo release en GitHub
- Consulta la [página de releases](https://github.com/StrangeDaysTech/straymark/releases) para changelogs

### Comandos de Actualización

| Objetivo | Comando |
|----------|---------|
| Actualizar framework y CLI | `straymark update` |
| Actualizar solo plantillas y docs de gobernanza | `straymark update-framework` |
| Actualizar solo el binario CLI | `straymark update-cli` |

Framework y CLI tienen **versiones independientes** — puedes actualizar uno sin el otro. Ver [Entender las Versiones](#entender-las-versiones).

### Después de Actualizar

1. Revisa los cambios en archivos de directivas y docs de gobernanza
2. Haz commit de los archivos actualizados: `git add .straymark/ && git commit -m "chore: update StrayMark framework"`
3. Si personalizaste archivos del framework, verifica si hay conflictos

---

## Verificar el Estado del Proyecto

### Estado via CLI

```bash
straymark status
```

Muestra: versión del framework, versión del CLI, integridad de la estructura de directorios y estadísticas de documentos por tipo. Úsalo para verificar que la instalación está saludable.

### Cumplimiento de Documentación (Skill)

```bash
/straymark-status
```

El skill `/straymark-status` (disponible en Claude Code y Gemini CLI) analiza:

- Qué cambios de código recientes carecen de documentación correspondiente
- Cumplimiento de documentos contra las reglas de gobernanza
- Estado general de documentación

---

## Usar Skills (Documentación Activa)

StrayMark tiene dos sistemas de documentación:

| Sistema | Cómo funciona | Cuándo usar |
|---------|---------------|-------------|
| **Pasivo** | La IA auto-documenta via archivos de directivas | Por defecto — sucede automáticamente |
| **Activo** | El usuario invoca skills para crear docs | Cuando el pasivo omitió algo, o para decisiones humanas |

### Skills Disponibles

| Skill | Propósito |
|-------|-----------|
| `/straymark-status` | Verificar cumplimiento de documentación |
| `/straymark-new` | Crear cualquier tipo de documento (sugiere el más adecuado) |
| `/straymark-ailog` | Creación rápida de AILOG |
| `/straymark-aidec` | Creación rápida de AIDEC |
| `/straymark-adr` | Creación rápida de ADR |
| `/straymark-audit-prompt CHARTER-XX` *(fw-4.8.0+, refactorizada en fw-4.9.0)* | Genera el audit prompt unificado en el path canónico `.straymark/audits/<id>/audit-prompt.md`. Envuelve `straymark charter audit --prepare`. El operador entonces abre N CLIs auditoras y corre `/straymark-audit-execute` en cada una — sin copy/paste. |
| `/straymark-audit-execute [CHARTER-XX]` *(fw-4.9.0+)* | **Corre dentro de una CLI auditora** (gemini-cli, claude-cli, copilot-cli, codex-cli). Lee el prompt del disco, audita con tool use citando `path:línea`, escribe un report con el id del modelo. Argumento opcional — auto-descubre prompts pendientes de este modelo. |
| `/straymark-audit-review CHARTER-XX` *(fw-4.8.0+, expandida en fw-4.9.0)* | Contraparte de `audit-prompt`. Lee N reports, verifica findings contra el código real, produce `review.md` consolidado de 6 secciones (Resumen ejecutivo / Alcance / Evaluación por auditor / Plan de remediación P0-P4 / Descartados / Calificación de auditores), y mergea YAML `external_audit:` en la telemetría. |

Para detalles completos de skills, consulta el [README](../README.md#skills).

### Charter audit checkpoint *(fw-4.8.0+)*

Cuando estés co-implementando un Charter, el agente proactivamente ofrecerá una auditoría externa en un momento específico: cuando la implementación esté lista, el drift esté limpio, y `charter close` aún no se haya invocado. La recomendación es SÍ/NO basada en la superficie de riesgo y complejidad del Charter (heurísticas completas en `.straymark/00-governance/AGENT-RULES.md` §12).

La auditoría externa es **completamente opcional** y **nunca enforced**. El scope declarativo del Charter + drift check + disciplina AILOG ya proporcionan cierre riguroso sin ella. La auditoría agrega señal cross-modelo cuando el Charter tocó superficie de seguridad, introdujo componentes nuevos, o tiene funciones de alta complejidad en el diff. Declina libremente si el costo (2-3 auditores LLM) no se ajusta al valor de tu caso.

---

## Patrones de Equipo

### Revisión de PRs

- Verifica que los cambios de código significativos incluyan documentos correspondientes en `.straymark/`
- Revisa cualquier documento con `review_required: true`
- Verifica que los AILOGs describan con precisión lo que hizo la IA

### Onboarding de Nuevos Miembros

1. Apúntalos a `.straymark/QUICK-REFERENCE.md` para una vista rápida
2. Pídeles que lean los ADRs recientes para entender el contexto arquitectónico
3. Muéstrales AILOGs de features recientes para ver cómo funciona la documentación en la práctica

### Retrospectivas de Sprint

- Revisa AILOGs y AIDECs del sprint para entender patrones de contribución de la IA
- Identifica decisiones no documentadas que deberían haberse registrado
- Revisa documentos TDE para deuda técnica acumulada

### Uso Compartido de Asistentes IA

Cuando múltiples miembros del equipo usan asistentes IA en el mismo proyecto:

- Cada sesión de asistente produce sus propios documentos
- El campo `agent` en los metadatos identifica qué asistente creó cada documento
- Revisa AIDECs superpuestos o contradictorios durante la revisión de PRs

---

## Flujo de Cumplimiento China *(opt-in)*

Si tu proyecto opera en China continental o procesa información personal de usuarios de China continental, habilita el alcance china y sigue este flujo.

### Configuración Inicial

1. Edita `.straymark/config.yml` y añade `china` a `regional_scope`:
   ```yaml
   regional_scope:
     - global
     - eu      # si también está sujeto a EU
     - china
   ```
2. Ejecuta `straymark compliance --region china` para ver el baseline (todos los checks fallarán hasta crear los documentos correspondientes).
3. Lee las guías instaladas en `.straymark/00-governance/`:
   - `CHINA-REGULATORY-FRAMEWORK.md` — visión general y matriz de cobertura
   - `TC260-IMPLEMENTATION-GUIDE.md` — clasificación de riesgo en 5 niveles
   - `PIPL-PIPIA-GUIDE.md` — cuándo se requiere PIPIA y qué debe contener
   - `CAC-FILING-GUIDE.md` — registro simple vs dual, ciclo de vida de estado
   - `GB-45438-LABELING-GUIDE.md` — diseño de etiquetado explícito + implícito

### Cuando Añades un Modelo de IA Generativa

Conjunto de documentos a crear juntos (cross-linked vía `related:`):

| Documento | Propósito | Requerido cuando |
|-----------|-----------|------------------|
| `MCARD` | Model card con `cac_filing_required`, `gb45438_applicable`, `tc260_risk_level` | Siempre para modelos en alcance |
| `TC260RA` | Clasificación de riesgo (escenario × inteligencia × escala → 5 niveles) | Siempre |
| `AILABEL` | Etiquetado explícito + implícito per GB 45438 | Cuando el modelo genera contenido |
| `CACFILE` | Registro de algoritmo CAC | Cuando `cac_filing_required: true` |
| `PIPIA` | Evaluación de impacto de info personal (Art. 55-56) | Cuando se procesa info personal |
| `SBOM` | Inventario de datos de entrenamiento + cumplimiento GB/T 45652 | Siempre |

`straymark compliance --region china` confirma que el conjunto está completo.

### Cuando Ocurre un Incidente

La plantilla `INC` incluye una sección *CSL 2026 Incident Reporting*. Setea:

```yaml
csl_severity_level: relatively_major   # o particularly_serious | major | general
csl_report_deadline_hours: 4           # 1 para particularly_serious, 4 para relatively_major
```

`straymark validate` aplica la coherencia severidad-deadline (`CROSS-008`, `CROSS-009`). Los incidentes major+ deben cerrarse (status `accepted`) en 30 días para que el check `CSL-003` apruebe.

### Transferencia Transfronteriza de Datos

Cuando un proceso involucra transferencia de información personal fuera de China continental, setea `pipl_cross_border_transfer: true` en el PIPIA y documenta el mecanismo elegido (evaluación de seguridad CAC / certificación / contrato estándar) en la sección *Cross-Border Transfer Analysis*. `CROSS-011` advertirá si no hay ninguno documentado.

### Verificación Diaria de Cumplimiento

```bash
# Antes de mergear una rama de feature que toca servicios IA
straymark validate                    # cross-rules incluyendo CROSS-004..011
straymark compliance --region china   # score por framework
```

---

## Entender las Versiones

StrayMark usa **versionado independiente** para sus dos componentes:

| Componente | Prefijo de tag | Contiene | Se actualiza con |
|------------|---------------|----------|-----------------|
| **Framework** | `fw-` | Plantillas, docs de gobernanza, directivas, scripts | `straymark update-framework` |
| **CLI** | `cli-` | El binario `straymark` | `straymark update-cli` |

### ¿Por Qué Versiones Independientes?

- Los cambios de framework (nuevas plantillas, reglas actualizadas) son más frecuentes
- Los cambios de CLI (nuevos comandos, corrección de bugs) siguen una cadencia diferente
- Puedes actualizar docs de gobernanza sin necesitar un nuevo binario del CLI

### Verificar Tus Versiones

```bash
straymark about     # Verificación rápida de versiones
straymark status    # Reporte completo de salud incluyendo versiones
```

Para información detallada del CLI, consulta la [Referencia CLI](CLI-REFERENCE.md#versionado).

---

<div align="center">

**StrayMark** — Porque cada cambio cuenta una historia.

[Volver a docs](../../README.md) • [README](../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
