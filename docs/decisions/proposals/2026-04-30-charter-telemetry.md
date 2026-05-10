# StrayMark — Esquema de telemetría de ejecución de Charters

**Versión:** 0.3 (rename Plan → Charter; refinado tras seis ciclos del experimento `/plan-audit` en Sentinel)
**Fecha:** 30 de abril de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Recopilar datos comparables sobre la ejecución real de Charters en proyectos adoptantes, para validar la hipótesis del patrón antes de cristalizarlo en un schema formal.
**Evidencia preliminar disponible:** ver `straymark-thesis-validation.md` §3 para el cuerpo de datos generado por Sentinel (5 telemetrías YAML, 6 reportes de auditoría dual, 5 AILOGs narrativos).

---

## Por qué este documento existe

Los Charters (descritos en `que-es-un-charter.md` y en `straymark-cloud-proposal.md` §4.5) son un patrón emergente del uso real de StrayMark. Antes de elevarlos a entidad de primera clase del producto y cristalizarlos en un schema formal, conviene observarlos en ejecución a través de varios proyectos y recopilar datos que respondan preguntas concretas sobre su utilidad real, sus puntos fuertes y sus modos de falla.

Este documento define un shape de telemetría manual que se aplica al cierre de cada Charter ejecutado. La aplicación es deliberadamente ligera (5–10 minutos por Charter) para no introducir fricción que sesgue los datos. Los campos están elegidos para responder preguntas específicas del proyecto de investigación, no para ser exhaustivos.

Este esquema es instrumental: existe para producir conocimiento, no para definir el producto. Cuando ese conocimiento esté disponible, el schema formal del Charter podrá absorber lo que corresponda y descartar lo demás.

**Nota terminológica.** Lo que este documento llama "Charter" se llamó "Plan" en el experimento Sentinel (PLAN-01..06). Los datos empíricos citados abajo se refieren a esos Plans históricos por su nombre original; el shape del schema y los ejemplos prospectivos usan el nombre going-forward "Charter". Justificación del rename en `que-es-un-charter.md` §2.

---

## Preguntas que la telemetría debe responder

La instrumentación está diseñada para responder, con datos cuantitativos y cualitativos, las siguientes preguntas. Bajo cada pregunta se añade la **respuesta preliminar de Sentinel** cuando el experimento aportó datos relevantes.

1. **¿La sección "Pre-trabajo antes de planning" cumple su propósito?** ¿Cuánto del pre-trabajo declarado se hace efectivamente antes de la sesión de planning? ¿Cuánto se descubre durante? Si el descubrimiento durante es alto, el campo no está cumpliendo su función.

   *Respuesta preliminar de Sentinel:* sin evidencia directa — los seis ciclos no usaron sesiones de planning formales separadas de la ejecución. El campo `additional_context_loaded_manually` (PLAN-06: 6 archivos cargados durante implementación) es proxy del descubrimiento durante. Pendiente proyecto con sesión de planning explícita.

2. **¿Los AILOGs de origen siguen siendo útiles cuando el Charter se activa?**

   *Respuesta preliminar de Sentinel:* sí, validado en PLAN-06 (`originating_ailogs.AILOG-2026-04-25-015`, marcado `still_relevant_at_execution: true`). El AILOG de origen documentaba el switch de fuente RPM que generó el sesgo de baseline; sin él, el use-case del Plan no estaría articulado.

3. **¿Las estimaciones de esfuerzo son fiables?**

   *Respuesta preliminar de Sentinel:* el campo en TIEMPO (XS/S/M/L) es predictivo, el de líneas no lo es. PLAN-01..05 todos cumplieron tiempo 1.0x; el drift de líneas varió de 1.0x a 8.1x. PLAN-06 dio drift de tiempo 1.33x (40 min real vs 30 estimado), atribuido al rigor de format v3 (drift script + AILOG estructurado). Conclusión: retirar la estimación en líneas; mantener TIEMPO como única métrica de esfuerzo. Aplicado en TEMPLATE v2 (Sentinel) y propagado al schema v0.x abajo.

4. **¿Los triggers observables se disparan limpiamente?**

   *Respuesta preliminar de Sentinel:* en 5/5 Plans ejecutados el trigger declarado del roadmap **no disparó**; el signal real fue del meta-experimento (validación del flujo) o cierre de backlog. Conclusión: el campo `trigger.declared_kind` es prescriptivo (cuándo SHOULD ejecutarse), no predictivo. Esto sugiere refinar el shape del Charter para distinguir trigger declarado de trigger efectivo, capturando ambos.

5. **¿La ejecución de un Charter genera nuevos Charters, y en qué proporción?**

   *Respuesta preliminar de Sentinel:* sostenible. PLAN-05 generó 0 Plans nuevos (los follow-ups F4/F5/F6 cierran como tareas menores en commit follow-up al PR, no como Plans). PLAN-06 generó 0 Plans nuevos. Ratio observado: 0 follow-up Plans por Plan ejecutado, dentro del rango sostenible (0-1) que el supuesto pedía.

6. **¿La calidad de las decisiones del agente mejora cuando hay contexto del Charter cargado vs cuando no?**

   *Respuesta preliminar de Sentinel:* sí parcialmente — los AILOGs ricos exportan signal pública y los auditores externos convergen a calibración mayor (Copilot 9.25, Gemini 9.5 en PLAN-05) cuando hay TEMPLATE v2 + AILOG rico. Pero los gaps F4 (evaluator_test no implementado) y F5 (hallucination arquitectónica) muestran que el agente puede no documentar lo que olvida. La conclusión refinada: la disciplina hace los gaps *visibles* a auditores externos, no los elimina. Ver `straymark-thesis-validation.md` §4.2.

---

## Esquema de telemetría por Charter

El campo se agrega al cierre de cada Charter ejecutado, idealmente en el frontmatter del markdown del Charter o en un archivo asociado `.straymark/charters/CHARTER-XXX.telemetry.yaml` (en Sentinel los archivos análogos viven como `.straymark/plans/PLAN-XXX.telemetry.yaml`).

```yaml
charter_telemetry:
  # Identificación
  charter_id: "CHARTER-2026-XX-YY-001"
  charter_title: "string corto"
  closed_at: "2026-XX-YY"

  # Origen y activación
  originating_ailogs:
    - ailog_id: "AILOG-2026-04-24-010"
      still_relevant_at_execution: true   # bool
      relevance_notes: "texto opcional explicando si seguía relevante"
  trigger:
    declared_kind: "event_trigger | date | metric_threshold | infrastructure_milestone"
    declared_description: "texto del Charter"
    fired_at: "2026-XX-YY"
    fire_clarity: "clear | ambiguous | manually_decided"
    fire_clarity_notes: "texto opcional"

  # Pre-trabajo
  pre_work:
    items_declared: 3                      # int — cuántos items listaste como pre-trabajo
    items_completed_before_planning: 2     # int — cuántos hiciste antes de la sesión
    items_skipped: 0                       # int — cuántos no se hicieron (declarar por qué)
    items_discovered_during_planning: 1    # int — cuántos pre-trabajos no listados aparecieron en sesión
    pre_work_quality: "high | medium | low"  # tu juicio cualitativo
    pre_work_notes: "texto libre"

  # Sesión de planning (si la hubo)
  planning_session:
    occurred: true                         # bool — algunos Charters se ejecutan sin sesión formal
    duration_minutes: 45                   # int
    participants: 1                        # int
    decisions_made: 3                      # int — decisiones de diseño/scope tomadas
    decisions_deferred: 0                  # int — decisiones que requirieron más investigación
    notes: "texto libre"

  # Ejecución (campo de líneas retirado — Sentinel demostró que no es predictivo)
  execution:
    started_at: "2026-XX-YY"
    finished_at: "2026-XX-YY"
    estimated_effort: "M (~1.5h)"          # XS/S/M/L con tiempo aproximado en h
    actual_effort: "M (~1.5h)"             # tiempo observado
    estimation_drift_factor: 1.0           # ratio actual/estimated en TIEMPO únicamente
    estimation_drift_reason: "texto"

  # Calidad del trabajo del agente durante la ejecución
  agent_quality:
    sessions_count: 1                      # int — sesiones de Claude Code u otro agente
    hallucinations_caught: 2               # int — veces que el agente inventó algo y lo pillaste
    hallucination_categories:
      - "API inexistente"
      - "comportamiento de librería incorrecto"
    decisions_contradicting_prior_adrs: 1  # int
    contradiction_notes: "texto si las hubo"
    context_loaded_was_sufficient: true    # bool
    additional_context_loaded_manually: 0  # int — archivos extra cargados manualmente durante implementación
    r_n_plus_one_emergent_count: 0         # riesgos emergentes nombrados durante ejecución
                                            # (validados en 4/4 ciclos de Sentinel como señal cross-agent)
    skill_prompts_used: ["/plan-audit", "/plan-audit-review"]

  # Auditoría externa — array, no objeto único.
  # Sentinel validó que dual-audit (Copilot + Gemini) genera calibración cross-modelo
  # y que el delta de categorización entre auditores es señal valiosa.
  external_audit:
    - auditor: "copilot-v1.0.37"
      findings_total: 5                    # int
      findings_by_category:
        hallucination: 0
        implementation_gap: 2
        real_debt: 2
        false_positive: 1
      audit_quality: "high | medium | low" # juicio del calibrador humano
      audit_notes: "texto libre"
    - auditor: "gemini-cli-v1"
      findings_total: 4
      findings_by_category:
        hallucination: 1
        implementation_gap: 2
        real_debt: 0
        false_positive: 1
      audit_quality: "high | medium | low"
      audit_notes: "texto libre"

  # Resultado y follow-ups
  outcome:
    completed_as_planned: true             # bool
    scope_changes: "ninguno | menor | mayor"
    scope_change_notes: |                  # codificación F1...FN consistente con claude-analisis
      F1 (IG): descripción del primer drift declarado-vs-implementado.
      F2 (hallucination): el Charter asumió X pero la realidad es Y.
      F3 (FP): riesgo R<N> sobre-anticipado, no se materializó.
    new_followups_generated: 2             # int — AILOGs con follow-ups nuevos
    new_charters_created: 1                # int — Charters nuevos derivados
    charters_invalidated: 0                # int — otros Charters que este Charter hizo obsoletos
    associated_stage_id: "STAGE-2026-XX-YY-NNN"

  # Friction points y wins (cualitativos)
  qualitative:
    format_iteration: "v3"                 # qué versión del TEMPLATE se usó
                                            # (auto-evolutivo: cada Charter refina el formato del próximo)
    friction_points:
      - "El trigger 'cuando aterrice un UI de operadores' fue ambiguo; tuve que decidir manualmente"
      - "Pre-trabajo subestimado: faltaba investigar comportamiento de RBAC"
    wins:
      - "Tener el AILOG de origen ahorró 30 min de re-discovery del contexto"
      - "El agente no contradijo ADR-007 porque estaba cargado en contexto inicial"
    overall_satisfaction: 4                # int 1-5 — qué tan útil fue el formato del Charter en este caso
    would_repeat_format: true              # bool
    proposed_format_changes: "texto — alimenta la próxima versión del TEMPLATE"
```

---

## Convenciones de uso

**Cuándo se llena.** Al cierre de cada Charter ejecutado, en el mismo commit que cierra el Stage asociado. Si el Charter se cancela o se vuelve obsoleto sin ejecutarse, también vale registrar telemetría con `outcome.completed_as_planned: false` y notas explicando por qué.

**Cuánto detalle.** Suficiente para que un análisis posterior pueda detectar patrones, no tanto que la instrumentación introduzca fricción que distorsione los datos. Los campos cuantitativos (`int`, `bool`, `ratio`) son obligatorios; los cualitativos (texto libre) son opcionales pero altamente valorados — frecuentemente el insight está en las notas, no en los números.

**Quién interpreta.** Inicialmente el autor del proyecto (Jose) interpreta su propia telemetría. Cuando haya datos de tres o más proyectos distintos, se puede empezar a comparar entre ellos. La interpretación cruzada genera mejor evidencia que la auto-interpretación.

**Privacidad y publicación.** La telemetría se queda en el repo; no se publica automáticamente. Cuando los datos se usen para escribir sobre el patrón (blog, documentación, paper), se anonimiza lo necesario y se publica sólo lo que el autor consciente.

---

## Datos retrospectivos disponibles desde fases 8–12 del MVP

**Estado actual:** este ejercicio sigue pendiente. El experimento `/plan-audit` post-MVP cubre el frente *prospectivo* (Plans 01-06 ejecutados con telemetría completa) pero no el retrospectivo de las fases 8-12. Cuando se materialice, los datos pre/post-skills aportarán otro eje de validación al supuesto #2 de la tesis ("notas estructuradas reducen modos de falla"). Ver `straymark-thesis-validation.md` §4.2.

Además de los Plans en ejecución, el proyecto Sentinel tiene historial de auditorías y evaluaciones desde las fases 8 a 12 del MVP. Ese historial es valioso porque captura una transición natural: introducción de las skills de auditoría que generan prompts contextuales para Géminis. La hipótesis observable es que las auditorías post-skills tienen mejor calidad (menos falsos positivos, mejor detección de gaps reales, menos fricción en remediación) que las pre-skills.

Para extraer ese conocimiento, conviene aplicar a las fases 8–12 una versión retrospectiva del esquema de telemetría, llenando con la mejor información disponible. Específicamente, conviene capturar:

- Para cada fase: número de hallazgos del auditor, distribución por categoría (hallucination, gap, deuda, falso positivo), tiempo de remediación, número de re-auditorías necesarias.
- Marcar la fase exacta en la que se introdujo cada iteración de las skills, para poder hacer comparación pre/post.
- Versionado de las skills usadas en cada fase, si está disponible.
- Notas cualitativas sobre cómo se sintió cada fase: qué funcionó, qué no, qué cambiarías.

Este ejercicio retrospectivo puede hacerse en una sola sesión de 2–3 horas de revisión del historial. El output sería un archivo `sentinel-mvp-audit-retrospective.md` que sirve como input para análisis comparativo y como contenido editorial futuro.

---

## Cómo este documento evoluciona

Después de ejecutar Charters en proyectos adoptantes y aplicar la telemetría retrospectiva a las fases 8–12 de Sentinel, la información acumulada permitirá:

1. **Validar o invalidar campos del esquema.** Si un campo nunca aporta señal, se elimina. Si aparece consistentemente un dato no capturado, se agrega.
2. **Detectar patrones reales.** Por ejemplo, si los Charters con triggers de tipo `event_trigger` tienen ejecuciones más limpias que los de tipo `infrastructure_milestone`, eso es señal accionable para el diseño del producto.
3. **Generar narrativa basada en datos.** Cuando llegue el momento de escribir sobre StrayMark (blog, documentación, conversaciones de venta), las afirmaciones estarán respaldadas por evidencia concreta, no por intuición.
4. **Cristalizar el schema formal del Charter.** Sentinel produjo, a través del ciclo `/plan-audit` v1 → v2 → v3, un borrador empírico de TEMPLATE.md y un script `check-plan-drift.sh` validados con cero falsos positivos sobre dos tests empíricos. Ese borrador es el material de entrada para `charter.schema.v0.json` y `charter-telemetry.schema.v0.json` (ambos marcados *experimental* hasta validación con un segundo proyecto en otro dominio — ver `straymark-thesis-validation.md` §6 para el argumento N≈2-3). El schema `v1.0` estable requiere un eje de validación adicional: dominio distinto a Go backend.

---

*Este es un instrumento de aprendizaje, no un compromiso de producto. Su valor se mide en cuánto conocimiento accionable produce sobre el patrón de Charters, no en su completitud o sofisticación.*
