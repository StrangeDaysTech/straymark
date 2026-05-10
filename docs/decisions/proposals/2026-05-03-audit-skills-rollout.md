# Plan de implementación — StrayMark audit skills + checkpoint

**Versión:** 0.2 (Fase 1 cerrada el 3 de mayo de 2026 con la release `fw-4.8.0` / `cli-3.9.0`; Fase 2 abierta — recolección de telemetría operativa)
**Fecha:** 3 de mayo de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Operacionalizar `straymark-audit-skills.md` v0.2 — aterrizar las 2 skills + checkpoint guidance en releases concretas, definir las condiciones de cierre por fase, y especificar la telemetría que se recaba durante el rollout para informar las decisiones de §9 (qué de la integración mayor de arborist se promueve a propuesta propia y qué se descarta).
**Documento padre:** `Propuesta/straymark-audit-skills.md` v0.2 (este plan no repite el diseño; lo asume).
**Documentos relacionados:** `Propuesta/straymark-cli-roadmap.md` v0.3 (gating de cristalización v0 → v1), `Propuesta/straymark-charter-telemetry.md` v0.3 (schema actual de telemetría).

---

## 0. Estructura del rollout

| Fase | Objetivo | Duración | Releases | Estado |
|---|---|---|---|---|
| 1 — Core implementation | Aterrizar las 2 skills + checkpoint en `fw-X.Y.Z` | 1.5–2 sem focused | 1 minor framework + posible patch CLI | ✅ **cerrada** (3 de mayo de 2026 — `fw-4.8.0` / `cli-3.9.0` shippeada en 1 día con 5 PRs) |
| 2 — Recolección de telemetría | Operar las skills en Sentinel + adoptante de frontend, recabar señal sobre uso real | 6–10 ciclos de Charter (~2-3 meses calendar) | sin releases (solo logs) | 🟡 **abierta** desde el cierre de Fase 1 |
| 3 — Decisiones §9 | Evaluar la data acumulada y decidir qué de los 4 gaps de §9 se promueve a propuesta propia | 1 sem de análisis | release subsiguiente solo si algo se promueve | ⬜ pendiente — gated en §2.4 |

Las fases son secuenciales para Fase 1 → Fase 2; Fase 3 puede ejecutarse parcialmente cuando haya señal suficiente sin esperar el N=10 (early-stop si la señal ya es clara).

**Cierre de Fase 1 (3 de mayo de 2026):** las 5 PRs (#96, #97, #98, #99, #100) mergeadas el mismo día. Tags `fw-4.8.0` y `cli-3.9.0` push, release workflows ejecutados. Adoptantes obtienen el flujo nuevo via `straymark update-framework` / `straymark update-cli`. Detalle por PR en `straymark-audit-skills.md` §0.

---

## 1. Fase 1 — Core implementation

### 1.1 Secuencia de PRs

Hereda `straymark-audit-skills.md` §7 con verificación concreta por PR.

**PR 1 — Skill `straymark-audit-prompt` (3 plataformas) + tests**

Archivos a crear:
- `dist/.claude/skills/straymark-audit-prompt/SKILL.md`
- `dist/.gemini/skills/straymark-audit-prompt/SKILL.md`
- `dist/.agent/workflows/straymark-audit-prompt.md`

Verificación operativa (ejecutar antes de mergear):

```bash
# El skill no rompe la instalación
cargo install --path cli/
straymark init /tmp/test-audit-prompt
ls /tmp/test-audit-prompt/.claude/skills/straymark-audit-prompt/SKILL.md
ls /tmp/test-audit-prompt/.gemini/skills/straymark-audit-prompt/SKILL.md
ls /tmp/test-audit-prompt/.agent/workflows/straymark-audit-prompt.md

# El skill produce un Charter ejercitable
cd /tmp/test-audit-prompt
straymark charter new --type M --title "test audit-prompt"
# (simular invocación de skill manualmente — leer SKILL.md y seguir instrucciones)
# Verificar que el `cargo run -- charter audit CHARTER-01` se ejecuta limpio
```

Test fixture en `cli/tests/audit_skill_test.rs`: verificar que el archivo de skill existe en el manifest y se copia tras `init`.

Condición de cierre PR 1:
- 3 archivos de skill creados.
- Test fixture verde.
- `dist-manifest.yml` no requiere cambios (las 3 carpetas ya están listadas).
- CHANGELOG.md `### Added (Framework)` sumado.

**PR 2 — Skill `straymark-audit-review` (3 plataformas) + auto-merge YAML + tests**

Archivos a crear: análogo a PR 1 (3 SKILL.md).

Lógica clave en el cuerpo del skill (descrita en `straymark-audit-skills.md` §3.2):
1. Verificar `audit/charters/<id>/auditor-{primary,secondary}.md` existen.
2. Ejecutar `straymark charter audit <id> --calibrate`.
3. Generar calibrator response usando el agente principal.
4. Ejecutar `straymark charter audit <id> --finalize`.
5. **Mergear** YAML en `.straymark/charters/<id>.telemetry.yaml`.

Verificación operativa:

```bash
# Flujo end-to-end con responses fixtures
cd /tmp/test-audit-prompt
straymark charter audit CHARTER-01  # PREPARE
cp tests/fixtures/auditor-primary.md audit/charters/CHARTER-01/
cp tests/fixtures/auditor-secondary.md audit/charters/CHARTER-01/
# (simular invocación de straymark-audit-review manualmente)
# Resultado esperado: calibrator-reconciler.md generado, telemetry.yaml mergeado
test -f audit/charters/CHARTER-01/calibrator-reconciler.md
grep -A5 'external_audit:' .straymark/charters/CHARTER-01.telemetry.yaml
```

**El auto-merge requiere ajustes al CLI** (probable patch `cli-3.8.2` o `cli-3.9.0`):
- Hoy `--finalize` solo imprime el YAML a stdout. Para que la skill pueda mergearlo confiablemente, conviene un flag `--finalize --emit-json` que estructure la salida como JSON parseable, o un flag `--finalize --merge-into <telemetry-path>` que el CLI haga directamente el merge (más limpio — la skill solo invoca, no parsea).
- Decisión recomendada: agregar `--merge-into` al CLI en este mismo PR. Mantiene el patrón "CLI orquesta, skill UX inline".

Condición de cierre PR 2:
- 3 archivos de skill creados.
- Patch del CLI con `--merge-into` (si se acuerda esa ruta).
- Test integration end-to-end con fixture en `cli/tests/audit_review_e2e_test.rs`.
- CHANGELOG.md `### Added (CLI)` y `### Added (Framework)`.

**PR 3 — Checkpoint guidance en `AGENT-RULES.md` (3 langs) + tests de fixture**

Archivos a editar:
- `dist/.straymark/00-governance/AGENT-RULES.md` (EN canónico)
- `dist/.straymark/00-governance/i18n/es/AGENT-RULES.md`
- `dist/.straymark/00-governance/i18n/zh-CN/AGENT-RULES.md`

Sección nueva con título "Audit checkpoint" que codifica:
- Los 4 triggers booleanos (in-progress + tasks done + drift OK + close no invocado).
- Forma del mensaje SÍ/NO (texto literal a usar por el agente).
- Heurísticas SÍ/NO de §4.3 incluyendo la complejidad arborist con graceful-degradation.
- Reglas: emit una sola vez por Charter, recordar decisión del developer.

Verificación:

```bash
# La sección existe en las 3 langs
for lang in '' 'i18n/es/' 'i18n/zh-CN/'; do
  grep -c '^## Audit checkpoint\|^## 审计检查点\|^## Checkpoint de auditoría' \
    dist/.straymark/00-governance/${lang}AGENT-RULES.md
done
# Cada uno debe reportar 1
```

Test fixture en `cli/tests/governance_test.rs`: post-`init` la sección existe en cada `AGENT-RULES.md`.

Condición de cierre PR 3:
- 3 archivos editados con paridad estructural.
- Test verde.
- `release-framework.yml` versionado en footers.

**PR 4 — Documentación adopter (paralelo a PR 3, no critical-path)**

Archivos a editar (cada uno × 3 langs):
- `docs/adopters/WORKFLOWS.md` — diagrama del loop incluye checkpoint.
- `docs/adopters/CLI-REFERENCE.md` — nueva sección `## Skills` listando los 9 skills (7 + 2 nuevos), con ejemplo de invocación y archivos producidos por cada uno. La sección `### straymark charter audit` (CLI) gana párrafo "Skill alternative".
- `docs/adopters/ADOPTION-GUIDE.md` — sección "External audit (optional)".
- `dist/.straymark/00-governance/QUICK-REFERENCE.md` — tabla de skills +2 filas.
- `README.md` — solo si la tabla de skills existe en el README.

Verificación:

```bash
for lang in '' 'i18n/es/' 'i18n/zh-CN/'; do
  echo "=== ${lang:-en} ==="
  grep -c 'straymark-audit-prompt\|straymark-audit-review' \
    docs/${lang}adopters/CLI-REFERENCE.md
done
# Cada uno debe reportar ≥ 2
```

Condición de cierre PR 4:
- 9 archivos editados (3 docs × 3 langs).
- Paridad estructural verificada (las 3 langs documentan los mismos skills).
- Sin breaking changes en links existentes.

**PR 5 — Bump versión + CHANGELOG + tag release**

Decisión de bump:
- `fw-X.Y.0` (minor — nuevas skills + nueva guidance en governance).
- `cli-X.Y.0` (minor) **solo si PR 2 incluyó el flag `--merge-into`**. Si la skill parsea el stdout actual en su lugar, no hay bump CLI.

Procedimiento estándar (`CLAUDE.md` Release Workflow). Tag release. Verificar que los 4 binarios oficiales se publican y que `straymark update-framework` baja el nuevo fw.

### 1.2 Estimación

5 PRs × ~3-6 h = 21-28 h focused. Calendarizable en 2 semanas con ciclos de revisión.

PR 1, 2, 3 son secuenciales (PR 2 depende de la decisión de schema en PR 1; PR 3 puede arrancar en paralelo con PR 2 desde su rama).

PR 4 puede arrancar tras PR 3 estabilizar el shape de los skills y el wording del checkpoint. PR 5 cierra la fase.

### 1.3 Condiciones de cierre Fase 1

- ✅ Los 5 PRs mergeados a main, tag release publicado. *Cumplido — PRs #96, #97, #98, #99, #100 mergeados el 3 de mayo de 2026; tags `fw-4.8.0` y `cli-3.9.0` pushed; release workflows ejecutados.*
- ✅ Las 3 langs en paridad estructural (sección AGENT-RULES + skills documentation + WORKFLOWS). *Cumplido — verificado por el test fixture `cli/tests/checkpoint_guidance_test.rs::audit_checkpoint_section_three_langs_share_load_bearing_anchors` y por inspección visual en PR #98 + #99.*
- ✅ Verificación end-to-end ejecutada en un proyecto sandbox: crear Charter, invocar `straymark-audit-prompt` simulado, pegar respuestas fixture, invocar `straymark-audit-review` simulado, verificar telemetry mergeada. *Cumplido — el test integration `cli/tests/charter_audit_test.rs::audit_merge_into_appends_external_audit_to_telemetry` ejercita el flujo completo (PREPARE → fixtures de auditor → CALIBRATE → fixture de calibrator → FINALIZE --merge-into → assertion sobre telemetry mergeada con indent 2, ambos auditores presentes, charter id real, keys pre-existentes preservadas).*
- ✅ 0 issues abiertas en GitHub atribuibles al rollout. *Cumplido al momento del cierre.*
- ✅ `straymark-audit-skills.md` v0.1 → v0.2: §0 nuevo "Estado de implementación" con la release shippeada (mismo patrón que `straymark-cli-roadmap.md` §0). *Cumplido en este mismo bump — ver `straymark-audit-skills.md` §0.*

**Nota de cierre:** Fase 1 ejecutada en 1 día calendar (5 PRs secuenciales el 3 de mayo de 2026), substancialmente más rápido que la estimación de 1.5-2 semanas focused. El throughput se debe a que el diseño en `straymark-audit-skills.md` ya tenía las decisiones cristalizadas (D1/D2/D3) y la heurística arborist con graceful-degradation explícita — no hubo decisiones pendientes durante la implementación. Sirve como evidencia operativa adicional para principio #6 (cuando la propuesta es bien escrita, la implementación es ejecución, no diseño).

---

## 2. Fase 2 — Recolección de telemetría

El propósito de esta fase es **acumular evidencia operativa** que informe las decisiones de §9 (qué de la integración mayor de arborist se promueve). Sin un punto explícito de recolección, las decisiones se tomarían por intuición — exactamente lo que principio #12 pretende prevenir.

### 2.1 Qué se recaba

Por cada invocación de `straymark-audit-review` (= cada cierre de ciclo audit completo), la skill escribe un registro JSONL append en `.straymark/audit-telemetry/log.jsonl` con shape:

```json
{
  "schema_version": "v0",
  "charter_id": "CHARTER-04",
  "charter_status_at_review": "in-progress",
  "timestamp": "2026-05-15T14:30:00Z",
  "checkpoint": {
    "emitted": true,
    "recommendation": "yes",
    "reasons_fired": ["security_surface", "cog_complexity_2x"],
    "developer_decision": "accepted"
  },
  "complexity": {
    "feature_available": true,
    "max_cog_complexity_in_diff": 24,
    "files_over_2x_threshold": 2,
    "threshold_used": 8
  },
  "diff_size": {
    "files_changed": 17,
    "lines_added": 542,
    "lines_deleted": 88
  },
  "audit": {
    "auditor_primary_model": "claude-sonnet-4-6",
    "auditor_secondary_model": "gemini-2.5-pro",
    "calibrator_model": "claude-opus-4-7",
    "findings_total": 8,
    "findings_by_category": {
      "hallucination": 0,
      "implementation_gap": 3,
      "real_debt": 4,
      "false_positive": 1
    },
    "calibrator_status": {
      "agreed": 3,
      "disputed": 2,
      "unique_primary": 1,
      "unique_secondary": 1,
      "rejected": 1
    }
  },
  "auto_merge": {
    "succeeded": true,
    "telemetry_path": ".straymark/charters/CHARTER-04.telemetry.yaml"
  }
}
```

**Campos clave para §9:**

- `complexity.max_cog_complexity_in_diff` ↔ `audit.findings_by_category.implementation_gap` — establece o refuta la correlación: ¿funciones complejas atraen más implementation gaps?
- `checkpoint.reasons_fired` con `cog_complexity_2x` ↔ `checkpoint.developer_decision` — valida si la heurística de complejidad es predictiva de decisiones SÍ del developer.
- `complexity.feature_available: false` cuenta — cuántos adoptantes corren CLI sin `analyze` activo (informa si forzar el feature haría daño).
- `diff_size.lines_added` ↔ `audit.findings_total` — establece o refuta: ¿los Charters con diff grande (proxy de "el operador no pudo auditar archivo por archivo") se benefician más del audit externo en términos de findings descubiertos? Esta correlación es la evidencia central que justifica o descarta §9(b) — un warning en `validate --include-charters` cuando un Charter grande cierra sin audit. Campo objetivo, derivado de `git diff --stat` sobre el rango, no requiere supervisión humana ni API clients.

**Anti-objetivo:** este log NO se sube a un servicio externo, NO contiene PII, NO sale del repo del adoptante. Es un artefacto local que **el adoptante consulta voluntariamente** y comparte fragmentos relevantes en el feedback periódico (issues, conversaciones). Honra el principio de StrayMark-as-local-tool.

### 2.2 Cómo se evita el sesgo de datos

- **Recolección por defecto**, no opt-in. Si el adoptante quiere desactivar, agrega `audit.telemetry: false` en `.straymark/config.yml` (no se incluye un flag CLI para evitar fricción visible).
- **Sin envío automático**. Si después de N ciclos hace falta evidencia agregada, el operador del adoptante corre `straymark metrics --export-audit-telemetry > audit-telemetry.json` y lo comparte en un issue. La friction de tener que exportar manualmente garantiza intención.
- **Schema explícito** (`audit-telemetry.schema.v0.json` en `dist/.straymark/schemas/`) versionado. Cambios al shape son breaking changes documentados.

### 2.3 Quién opera Fase 2

- **Sentinel** continúa siendo el dominio principal. Cada Charter cerrado vía el nuevo flujo (`straymark-audit-prompt` → `straymark-audit-review`) genera un registro.
- **Adoptante de frontend** (cuando aterrice) genera registros desde el primer Charter. **Crítico para inter-stack:** la heurística de complejidad debe validarse en TypeScript/JavaScript, no solo en Go.
- Cada 2-3 ciclos, el operador exporta el log a un issue de GitHub con el label `audit-telemetry-snapshot`. Eso construye un timeline público auditado (cumple principio #2).

### 2.4 Métricas de salida Fase 2

Fase 2 cierra cuando se cumple **al menos una** de estas condiciones:

- **Suficiencia cuantitativa.** Acumulados ≥ 8 registros de los cuales ≥ 3 son del adoptante de frontend (no Sentinel).
- **Suficiencia de señal.** La correlación `max_cog_complexity_in_diff` ↔ `implementation_gap_count` es estadísticamente clara (Spearman ρ > 0.5 con p < 0.1) **o** claramente nula (ρ < 0.2 con p < 0.1) — en cualquiera de los dos extremos hay evidencia para decidir.
- **Suficiencia narrativa.** Al menos 2 incidentes operativos donde un campo arborist hubiera sido decisivo (o claramente irrelevante) — convertirlos en mini-postmortems en AILOGs y cerrar la fase con esa evidencia.
- **Hard timeout.** 4 meses calendar tras cierre de Fase 1, independiente de la evidencia. Se entra a Fase 3 con la data que haya, aunque sea inconclusa.

---

## 3. Fase 3 — Decisiones §9

### 3.1 Marco de decisión

Por cada uno de los 4 gaps de `straymark-audit-skills.md` §9, se decide **explícitamente** entre:

- **Promote** — escribir propuesta dedicada (e.g., `Propuesta/straymark-charter-complexity-delta.md`) y entrar al ciclo de roadmap.
- **Defer** — seguir observando; reabrir cuando una nueva señal lo justifique.
- **Discard** — la data demostró que el gap no aporta valor; queda registrado en este documento como decidido y cerrado.

La decisión es **separada por gap** — los 4 no se mueven en bloque.

### 3.2 Criterios concretos por gap

**(a) Campo `agent_quality.complexity_delta` en charter-telemetry**

- **Promote si:** existe correlación clara complejidad ↔ findings (§2.4 suficiencia de señal positiva). El campo tiene valor predictivo.
- **Promote si (alternativo):** ≥ 3 incidentes donde el operador habría usado el campo retrospectivamente (e.g., "este Charter fue problemático, ¿qué tan complejo era el código?") y el dato no estaba.
- **Discard si:** correlación clara nula y cero incidentes operativos.

**(b) Warning en `validate --include-charters` por funciones sobre 2× threshold sin AILOG**

- **Promote si:** durante Fase 2, el log muestra ≥ 2 casos donde un Charter cerró con `max_cog_complexity_in_diff > 2× threshold` y NO había AILOG referenciando esa complejidad — y al menos 1 de esos 2 produjo finding `real_debt` o `implementation_gap`.
- **Defer si:** los casos existen pero no produjeron findings (la complejidad sin AILOG no fue señal de problema en práctica).
- **Discard si:** durante Fase 2 hubo 0 casos del patrón.

**(c) Métricas longitudinales en `straymark metrics`**

- **Promote si:** ≥ 1 adoptante (Sentinel o frontend) reporta haber consultado manualmente la evolución de complejidad y encuentra fricción de no tenerlo en el comando.
- **Defer si:** nadie lo consulta voluntariamente. Las métricas longitudinales solo aportan valor si se miran; no se construye reporting que nadie lee.
- **Discard:** no aplica — la data se acumula gratis vía PR (a) si aterriza, así que "discard" estricto es improbable.

**(d) Sección "Code complexity surface" auto-generada en `charter new --from-spec/--from-ailog`**

- **Promote si:** el adoptante de frontend reporta que al crear Charters nuevos, la información de complejidad pre-existente del scope ayuda a estimar effort más realistamente.
- **Defer si:** el adoptante reporta que la sección auto-generada se ignora en práctica (lo borraron del template, no lo leyeron).
- **Discard si:** la sección introdujo sesgo (Charters más estrechos de lo conveniente porque "ese archivo es complejo, no lo toco" — anti-patrón documentado).

### 3.3 Output de Fase 3

Un commit a `Propuesta/` con:

- **`Propuesta/straymark-audit-skills.md` v0.2** — §9 actualizado para cada gap: estado (promote/defer/discard), evidencia que sostiene la decisión, link a propuesta nueva si aplica.
- **`Propuesta/straymark-audit-skills-implementacion.md` v0.2** — §3 actualizado con la data acumulada y las decisiones tomadas.
- **Si algún gap se promueve:** nueva propuesta dedicada en `Propuesta/straymark-<gap-name>.md` con su propio plan.

---

## 4. Próximos pasos post-cierre

Cuando Fase 3 cierre:

1. **Si ningún gap se promueve** — la integración arborist queda como hoy: alimenta `straymark analyze` (standalone) y la heurística opcional del checkpoint en §4.3. Documentar explícitamente en `Propuesta/straymark-audit-skills.md` §9 que se evaluó y se decidió no expandir, con la evidencia. Esta es una decisión legítima — no expandir es respuesta válida.

2. **Si uno o más gaps se promueven** — cada uno entra al ciclo de roadmap en `Propuesta/straymark-cli-roadmap.md` v0.4 (cuando se escriba) con su propia secuencia de PRs, exit criteria, y subsection en §0 si aplica.

3. **Cristalización v0 → v1 de las skills mismas** — independiente de §9, las skills `straymark-audit-prompt` y `straymark-audit-review` cumplen su gate de v1 estable cuando satisfacen §10 de la propuesta padre. Eso puede pasar antes, durante, o después de Fase 3 — son ortogonales.

4. **Evolución del schema de telemetría audit** — `audit-telemetry.schema.v0.json` puede evolucionar a v1 cuando la data acumulada muestre campos faltantes (e.g., tiempo invertido por audit, costo en USD reportado por el adoptante, satisfacción cualitativa). Cualquier cambio breaking incrementa el `schema_version` en cada registro.

---

## 5. Decisiones operativas que NO requieren propuesta

Algunas evoluciones menores se acuerdan acá y se aplican sin propuesta dedicada:

- **Ajustes al wording del checkpoint** en `AGENT-RULES.md` (lenguaje, ejemplos, tono): patch directo cuando un adoptante reporta fricción.
- **Tuning de los multiplicadores de la heurística arborist** (hoy `2× threshold`): ajustar a `1.5×` o `3×` si la data de Fase 2 muestra que `2×` produce demasiados/pocos triggers. Cambio mínimo, sin breaking change.
- **Adición de modelos a la tabla de familias** (`dist/.straymark/audit-prompts/model-families.yaml` si se crea — es un follow-up del CLI roadmap §5.2): no requiere propuesta, es housekeeping.
- **Bumps menores del feature `analyze` por upgrades de arborist**: housekeeping.

---

*Este plan es operativo, no exhaustivo. Cualquier desviación material durante implementación (PR que se parte en dos, fase de telemetría que se acelera, decisión §9 que se toma early) se documenta amending este archivo, no creando uno nuevo. v0.2 cierra Fase 1; v0.3 se escribirá al cerrar Fase 2 (recolección de telemetría completada según §2.4); v0.4 al cerrar Fase 3 (decisiones §9 tomadas).*
