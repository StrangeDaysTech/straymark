# Baton — Kit de forward-validation para adoptantes (Track C)

> **Para:** el equipo de un repo adoptante. **Desde:** StrayMark / experimento Baton.
> **Fecha:** 2026-08-02 · **Estado:** vigente — reemplaza la terna E1/E2/E3 de
> [`05-adopter-test-plan.md`](05-adopter-test-plan.md) para la fase post-graduación.
> **Naturaleza:** read-only, **recommend-only**. Nada de esto ejecuta un modelo,
> dispatcha un agente, abre red, ni muta tu repo. `straymark-baton route` exige `--dry-run`.
> **Trazabilidad:** Track C de [`PLAN-avance-post-calibracion.md`](PLAN-avance-post-calibracion.md) ·
> schema ratificado en [`06-work-verb-schema-ratification.md`](06-work-verb-schema-ratification.md) ·
> graduado al framework en **fw 4.38.0 / cli 3.40.0** (2026-07-29).

## 0. Qué cambió desde el plan original (05)

El plan 05 pedía tres experimentos (E1 corrección, E2 cobertura de señal, E3 costo real).
Dos de ellos quedaron **obsoletos por diseño**:

- **E2 ya no aplica.** La señal que pedía enriquecer (esfuerzo/riesgo/scope para subir
  confianza) fue reemplazada por algo mejor: el **`work_verb` declarado en autoría**
  (decisión #332). El *title-scan* fue descontinuado; el verbo declarado es la **única**
  señal autoritativa de clasificación.
- **E3 ya no aplica.** Los costos ilustrativos son suficientes para forward-validation
  (Track C del plan post-calibración).

Lo que queda — y es el corazón de Track C — es una **E1 simplificada y re-encuadrada**.
En el modelo antiguo, E1 preguntaba *¿acierta el clasificador?* (mecanismo de keywords).
Ese mecanismo ya no existe. Ahora la clasificación es determinística sobre lo declarado,
así que la pregunta real es:

> **¿Los autores declaran bien el verbo en producción?**

El oráculo sigues siendo tú, que hiciste el trabajo.

## 1. Precondiciones

- **Framework ≥ 4.38.0** (`straymark update` para refrescar plantillas) o plantillas
  con los campos `work_verb` / `design_provenance` (Charter, AILOG, follow-ups).
- El binario `straymark-baton`: descárgalo del release **`baton-*`** en
  [GitHub Releases](https://github.com/StrangeDaysTech/straymark/releases) —
  busca el asset para tu plataforma (`straymark-baton-v{version}-{target}.tar.gz`
  o `.zip`; targets: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`,
  `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`), extráelo y colócalo en tu
  `PATH`. Alternativa: compílalo desde el repo
  (`cargo build --release --manifest-path experiment-baton/Cargo.toml`).
- Tu repo con su `.straymark/` habitual.

Comprobación rápida (no muta nada):

```bash
straymark validate          # los campos son opcionales; nada debe romperse
straymark-baton classify <tu-repo> --out json | head
straymark-baton route    <tu-repo> --dry-run
git -C <tu-repo> status   # debe quedar intacto
```

## 2. Fase 1 — Adoptar el campo (costo ≈ 0)

Al crear unidades enrutables, declara el verbo. La plantilla lo pide como comentario;
descomentar y llenar:

| Unidad | Slot |
|---|---|
| Charter | Frontmatter: `work_verb:` / `design_provenance:` |
| AILOG (batch ledger) | Frontmatter del AILOG |
| Follow-up | Líneas `- **Work verb**:` / `- **Design provenance**:` |
| Task (SpecKit) | Sin slot propio — hereda del charter/spec padre |

### Vocabulario y reglas de decisión (resumen de la ratificación)

| `work_verb` | significado | tier resultante |
|---|---|---|
| `design` | Arquitectura/patrón **abierta**, autoría de spec | planner |
| `implement` | Lógica de servicio, fix con diagnóstico, query compleja, tooling, **contrato fundacional acotado** | implementer |
| `audit` | Revisión/contraste **independiente** contra la realidad | auditor |
| `operate` | Mecánico: tests, migraciones, scaffolding, docs, ceremonia, bulk | operator |

Las tres reglas que cierran la ambigüedad:

1. **Contrato fundacional = `implement`, NO `design`.** `design` se reserva a
   arquitectura abierta y autoría de spec.
2. **`implement` + `design_provenance: upstream` degrada a operator.** Si el pensamiento
   difícil ya se gastó aguas arriba y esta unidad solo instrumenta/cablea ese diseño,
   es mecánica aunque el output parezca sofisticado.
3. **No-trabajo → `operate`.** Higiene, meta-notas, cierre: no hay 5º valor.

`design_provenance` (`new` | `upstream`) solo es significativo para `implement`.

### Qué registra el validador

- Campo **opcional**: una unidad sin verbo es `undeclared` — estado honesto, no error.
  Se rutea conservadoramente a frontier y `route` emite un nudge.
- Un valor **fuera del vocabulario** → warning advisory de `straymark validate`,
  nunca bloqueante.

## 3. Fase 2 — Calibración E1 simplificada (tras 2–4 semanas de uso)

Cuando tengas un corpus con verbos declarados en producción real:

1. Genera las clasificaciones:
   ```bash
   straymark-baton classify <tu-repo> --out json > baton-classify.json
   straymark-baton route    <tu-repo> --dry-run --out json > baton-route.json
   ```
2. Toma una **muestra de ~20–30 unidades con verbo declarado**, repartida por
   granularidad y por verbo declarado.
3. Para cada una, contesta con tu juicio retrospectivo (quien hizo el trabajo):
   - ¿el `work_verb` declarado es el correcto?
   - si es `implement`: ¿el `design_provenance` declarado es el correcto?
4. Computa:
   - **concordancia del verbo** = aciertos / unidades declaradas muestreadas (objetivo **≥ 0.8**),
   - concordancia de `design_provenance` sobre el subconjunto `implement`,
   - **dirección de las discrepancias**: ¿se declaró más barato de lo que el trabajo
     realmente era? (implement declarado como operate, etc.) — esa es la dirección
     peligrosa; la dirección contraria (más caro) es segura.

**Plantilla de etiquetado** (`baton-calibration/labels.yml`, dentro de `.straymark/`):

```yaml
# Una entrada por unidad muestreada. Llena true_verb / true_provenance y notes.
- id: CHARTER-43-...
  granularity: charter
  declared_verb: implement
  declared_provenance: new
  true_verb:            # design | implement | audit | operate
  true_provenance:      # new | upstream (solo si true_verb = implement)
  notes:
- id: FU-012
  granularity: followup
  declared_verb: operate
  declared_provenance:
  true_verb:
  true_provenance:
  notes:
```

## 4. Fricciones que nos interesa conocer

- ¿Los autores declaran bien en el flujo normal, o el campo se ignora/se llena al azar?
- ¿El vocabulario (design/implement/audit/operate) **cubre** su trabajo, o hay trabajo
  real que no encaja en ningún valor?
- ¿`design_provenance` se usa con criterio, se ignora, o genera confusión?
- ¿Qué fracción del corpus nuevo queda `undeclared`? (visible en `route --dry-run`:
  la línea `N undeclared`)

## 5. Qué NO pedimos

- **No** E2 (enriquecimiento de señal) — ya está resuelto por `work_verb`.
- **No** E3 (costos reales) — los ilustrativos bastan para forward-validation.
- **No** instrumentación nueva — solo el hábito de declarar el verbo.
- **No** backfill del corpus legacy — undeclared es estado honesto (§4 de la ratificación).

## 6. Criterio de cierre (Track C done)

> Al menos un adoptante (idealmente 2+) reporta **concordancia ≥ 0.8** con `work_verb`
> declarado en producción real, sin discrepancias sistemáticas *hacia abajo*.

Con eso se cumple el gate #3 de graduación de Baton a `straymark-core` y se desbloquea
la decisión de Fase 3 (ejecución real bajo política).

## 7. Garantías

- **Read-only / recommend-only.** Ningún subcomando muta tu repo, ejecuta un modelo,
  dispatcha un agente ni abre red. `route` exige `--dry-run`. Verificable:
  `git status` intacto tras cualquier corrida.
- **Sin datos personales, sin inferencia de modelo.** Lectura de tus propios artefactos
  de gobernanza + aritmética de costo ilustrativa. (EU AI Act: no aplica — herramienta
  local de desarrollo.)
- **Tus datos no salen del repo.** El `labels.yml` vive en tu `.straymark/`; tú decides
  qué nos compartes (cifras, fracciones relativas, o solo el resumen).
