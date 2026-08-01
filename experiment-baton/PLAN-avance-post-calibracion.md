# Baton — Plan de avance post-calibración

> **Fecha:** 2026-07-29 · **Estado:** propuesta de trabajo · **Base:** calibración Sentinel E1/E2/E3 + schema work_verb ratificado
> **Trazabilidad:** #332 (decisión title-scan), #331 (evidencia adopter), 06-work-verb-schema-ratification.md (schema ratificado)

## Principio rector

El schema `work_verb`/`design_provenance` está **ratificado** (06-work-verb-schema-ratification.md). La secuencia que des-arriesga la línea completa es:

```
graduar schema al framework  →  forward-validate en adopters  →  (N=2) generalizar codescan  →  Fase 3 (ejecución real)
```

Cada paso desbloquea el siguiente. No saltar pasos.

---

## Track A — Graduación del schema work_verb al framework

**Origen:** #332 paso 2 · **Bloquea:** forward-validation, adopción general
**Esfuerzo estimado:** M · **Dependencias:** ninguna

### A1. Campos de primera clase en plantillas

Agregar `work_verb` (y opcionalmente `design_provenance`) a las plantillas que generan unidades enrutables:

| Plantilla | Cambio |
|-----------|--------|
| `dist/.straymark/templates/charter/charter-template.md` | Agregar `work_verb: <design\|implement\|audit\|operate>` con comment explicativo |
| `dist/.straymark/templates/ailog/TEMPLATE-AILOG.md` | Agregar en la sección de batch ledger o como campo por-batch |
| `dist/.straymark/templates/followups/` | Agregar `**Work verb**:` al formato de entry |

**Decisión pendiente:** ¿batch/task declaran verbo? El prototipo los deja undeclared (no tienen frontmatter propio). La ratificación §4.2 los mapea a `operate` por default. Documentar esta decisión.

### A2. Schema validation en `straymark validate`

- Agregar `work_verb` al Charter schema (ya existe advisory `check_charter_work_verb` en validation.rs — promover a validación formal)
- Emitir warning cuando una unidad no declara verbo: "unidad inclasificable para routing — declara work_verb"
- El warning es **anti-noise**: no se emite para el corpus legacy 100%-undeclared (postura §5 ratificación)

### A3. Nudge de higiene

Cuando `straymark-baton route` reporta `undeclared_fraction > 0`, sugerir:
```
hint: N unidades sin work_verb declarado. Agrégalo en el frontmatter para routing preciso.
```

### A4. Documentación

- Actualizar `docs/adopters/CLI-REFERENCE.md` con los nuevos campos
- Actualizar QUICK-REFERENCE.md con la tabla verbo→tier
- CHANGELOG entry

### A5. Version bump

Framework: `fw-X.Y.Z` (minor — nuevos campos opcionales en schemas)

**Done when:** `straymark init` genera plantillas con `work_verb`, `straymark validate` marca undeclared, CHANGELOG actualizado.

---

## Track B — Fixes internos de código

**Esfuerzo estimado:** S-M cada uno · **Independientes entre sí**

### B1. #319 — Producer-side route keying (huma)

**Problema:** En Go con huma, el response struct se define ~75 líneas después del route registration. Nearest-anchor lo bindea al route equivocado → producer=None → C2/C3 no pueden disparar.

**Fix:** Bind route literal de `huma.Get(api, "/path", h.handlerMethod)` al handler symbol, luego key el output struct via naming convention (`<handler>Output`). Conservador: solo cuando la convención resuelve unívocamente.

**Archivos:** `experiment-baton/src/codescan.rs` (parse_go, extract_file)

**Done when:** En un repo huma-style, el response struct keys al route correcto y C2/C3 disparan end-to-end.

### B2. #315 — EPIPE/SIGPIPE handling

**Problema:** CLI crash con EPIPE/SIGPIPE (broken pipe cuando output es pipeado a `head` o similar).

**Fix:** Instalar signal handler o usar `sigpipe` crate. Standard Rust CLI pattern.

**Archivos:** `experiment-baton/src/main.rs`

### B3. #314 — Component→path mapping para C1

**Problema:** C1 (intended-not-implemented) es low-confidence porque mina nombres de `.specify/memory/` filenames. Falsos positivos: conceptos arquitectónicos que nunca fueron módulos.

**Fix:** Permitir mapping explícito component→globs en `.specify/memory/` frontmatter o reuse de `model.yml` component ids. Con anchor explícito, C1 sube de info/low a trustworthy.

**Archivos:** `experiment-baton/src/speckit/memory.rs`, `experiment-baton/src/coherence.rs`

**Done when:** Componente con mapping explícito → C1 high-confidence sin falsos positivos.

---

## Track C — Forward-validation (adopter)

**Origen:** #332 paso 3 · **Bloquea:** decisión de Fase 3
**Requiere:** Track A completo (schema graduado)

### Qué esperamos del adoptante

Una vez el schema esté graduado al framework (Track A), necesitamos que el adoptante:

1. **Adopte el campo en su flujo.** Al crear Charters/AILOGs/follow-ups, declarar `work_verb` en el frontmatter. La plantilla lo pide; el autor lo llena (costo ≈0 tokens).

2. **Corra la calibración E1 simplificada.** Después de N semanas de uso (sugerencia: 2-4), correr:
   ```bash
   straymark-baton classify <repo> --out json > classify.json
   ```
   Y etiquetar una muestra (~20-30 unidades) con la clase verdadera. Reportar precisión por clase y confianza.

3. **Reporte de fricciones.** ¿Los autores declaran bien? ¿El vocabulario (design/implement/audit/operate) cubre su trabajo? ¿`design_provenance` se usa o se ignora?

### Cómo lo pueden lograr

El adoptante no necesita herramientas nuevas:

- La plantilla graduada (Track A) ya pide el campo
- `straymark validate` marca undeclared como warning
- `straymark-baton classify` produce el JSON para etiquetar
- El formato de reporte es el mismo `labels.yml` del plan de calibración original

### Qué NO pedimos

- No pedimos E2 (enriquecimiento de señal) — ya está hecho con work_verb
- No pedimos E3 (costo real) — los costos ilustrativos son suficientes para forward-validation
- No pedimos instrumentación nueva — solo el hábito de declarar el verbo

**Done when:** Al menos un adoptante (idealmente 2+) reporta precisión high+medium ≥0.8 con work_verb declarado en producción real.

---

## Track D — Generalización codescan (bloqueado N=2)

**Origen:** #321 · **Bloquea:** graduación de Baton a core
**Trigger:** segundo adoptante con stack ≠ Go⇄TS

### Estado

Codescan es Go⇄TS-specific (walk .go/.ts/.tsx, Go=producer, TS=consumer). El core (normalized endpoint como ContractId) ya es language-agnostic. El trigger para generalizar es N=2 (un segundo adoptante real con stack diferente).

### Cuando se dispare

1. Introducir `LanguageAdapter` seam (trait o config-driven, siguiendo el patrón #279 de Loom)
2. Land adapter para el nuevo stack como primera generalización real
3. Go/TS extractors quedan como dos adapters detrás del seam

### Preparación (no bloqueante, se puede hacer antes)

- #319 (huma route keying) es consistente con la tesis y puede hacerse ya — mejora el adapter Go existente
- Documentar el seam esperado en `experiment-baton/specs/` para que el trabajo futuro tenga diseño previo

**No actuar hasta N=2.** Generalizar desde un solo ejemplo bakes assumptions.

---

## Track E — Issues relacionados (no-bloqueantes)

| Issue | Tipo | Acción sugerida |
|-------|------|-----------------|
| #335 | Design note | Cerrar con comment (el principio ya está registrado en #332 y 06-ratification) |
| #306 | Question | Dejar abierto como question de diseño; no requiere acción inmediata |
| #304 | Adopter feedback | Mantener abierto hasta que #319 cierre (C2/C3 end-to-end en huma) |

---

## Secuencia propuesta

```
Semana 1-2:  Track A (graduación schema) + B2 (#315 EPIPE fix)
Semana 2-3:  B1 (#319 huma keying) + B3 (#314 C1 mapping)
Semana 3+:   Track C (forward-validation con adopter) — esperar resultados
N=2:         Track D (generalización codescan)
```

Tracks A y B son paralelizables. Track C depende de A. Track D espera trigger externo.

---

## Graduation gate (recordatorio)

Baton se gradúa a `straymark-core` cuando:

1. ✅ Concepto validado en un oracle real (Sentinel — done, CHARTER-01/02/03 closed)
2. ⬜ Schema work_verb graduado al framework (Track A)
3. ⬜ Forward-validation con ≥1 adopter (Track C)
4. ⬜ Codescan generalizado (Track D, N=2)
5. ⬜ Loom overlay integrado (parcial — overlay.rs existe, falta integración con servidor)

El gate #1 está cumplido. Los demás son trabajo planificado arriba.
