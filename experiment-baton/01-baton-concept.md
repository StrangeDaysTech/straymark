# Baton — Documento conceptual fundacional

> **Versión:** 0.2 (decisiones de encuadre resueltas)
> **Fecha:** 24 de junio de 2026
> **Estado:** Borrador de trabajo — base para el Charter y los specs del experimento
> **Nombre oficial:** **Baton** (la batuta que dirige el conjunto; el relevo que traspasa la tarea al modelo correcto)
> **Reemplaza a:** `01-orchestrator-concept.md` (v0.1) y al borrador del consultor `StrayMark_gobernanza_orquestacion_SpecKit.docx`
> **Ámbito:** experimento `experiment-baton/`, hermano de Loom y okf, **diseñado para poder graduarse** al núcleo de StrayMark
> **Caso de referencia:** Sentinel, sin privilegio arquitectónico

**Principio rector**

> *StrayMark gobierna el trabajo; SpecKit expresa la intención; Baton enruta el cómputo al modelo adecuado para cada tarea; el repositorio y la evidencia determinan el estado aceptado.*

---

## 0. Procedencia y propósito

Existe un borrador previo (`StrayMark_gobernanza_orquestacion_SpecKit.docx`) redactado por un agente consultor sin visión completa de StrayMark. Mezclaba ideas correctas, invenciones de vocabulario y propuestas nuevas presentadas como infraestructura existente. La v0.1 de este documento (`01-orchestrator-concept.md`) lo reancló en el StrayMark real; esta v0.2 incorpora las decisiones de encuadre que tomamos y las ideas nuevas que surgieron en esa discusión.

### 0.1 Tabla de corrección (qué del borrador del consultor es qué)

| Concepto del borrador | Veredicto | Realidad |
|---|---|---|
| Charter, ciclo de vida, cierre con drift | ✅ Real | `declared → in-progress → closed`; `straymark charter drift/close` |
| AILOG, AIDEC, ADR | ✅ Reales | Tipos documentales del framework |
| Follow-ups (FU-NNN), TDE (`affects`) | ✅ Reales | Registry de primera clase + deuda técnica con scope de globs |
| `effort_estimate` (XS/S/M/L) | ⚠️ Real, malinterpretado | Es **tiempo humano**, no complejidad de cómputo ni tier de modelo |
| Auditoría multi-modelo independiente | ✅ Real (la semilla) | `straymark charter audit` ya orquesta auditores de familias distintas |
| **"Work Unit" / "WU-NNN"** | ❌ **Invención** | No existe en el repo — pero **sí necesitamos** una unidad mínima enrutable (ver §4.3) |
| **"streams" de Charter** | ❌ **Invención** | Solo existen *batches* |
| "Governance Graph" como event-ledger nuevo | 🔶 Innecesario por ahora | Reutilizamos los tipos documentales existentes (decisión §10.5) |
| `speckit reconcile`, `speckit-links.yaml` | 🔶 Propuesta nueva | Es el Coherence Bridge por construir (§4.1) |
| "StrayMark será la autoridad de ejecución multiagente" | 🔁 Reencuadrado | La ejecución/routing vive en Baton (experimento), no en el núcleo (§1.3) |

---

## 1. Por qué ahora

### 1.1 La presión económica

Se avizora el fin de las suscripciones mensuales y la migración a **pago por millón de tokens** (los proveedores chinos ya migraron; occidente parece cuestión de tiempo). Hoy una suscripción MAX (~100 USD/mes) está respaldada por un subsidio de consumo estimado en **400–600 USD/mes**. Si ese subsidio desaparece, el costo real del trabajo actual se vuelve potencialmente incosteable para un desarrollador independiente. Cambiar de proveedor es apuesta de alto riesgo; la alternativa accionable desde hoy es cambiar **cómo** se gasta el cómputo.

### 1.2 La oportunidad

No todo el trabajo necesita un modelo frontier. Commits, PRs, listas de tareas atómicas, clasificación — los hacen bien modelos como Sonnet o Gemini. Reservar frontier (Opus / GPT-5.5) para **diseño, arquitectura, adjudicación y diagnóstico abierto** puede reducir el costo por trabajo aceptado de forma drástica. La tesis: **enfocar el gasto por token al modelo apropiado para el tipo de tarea**, apoyándose en la disciplina de gobernanza que StrayMark ya produce.

### 1.3 Encuadre: experimento hermano, diseñado para graduar

En el pasado decidimos que **StrayMark no sería un orquestador de agentes ni un manejador de tuberías**. Baton respeta esa decisión **en esta fase**, pero con un matiz que tú añadiste y que cambia la disciplina de diseño:

- **Por ahora** Baton es un experimento hermano (como Loom y okf). Vive en `experiment-baton/`, con su propio ciclo, tags y *graduation gate*. El núcleo de StrayMark **no** adquiere conocimiento de modelos, tokens ni proveedores.
- **Pero se diseña para graduar.** Asumimos que Baton probablemente se integrará al núcleo. Eso impone, desde la Fase 0, **fronteras limpias**: lógica reutilizable en `core` (typed, sin I/O cuando sea posible), nada acoplado a un proyecto concreto, y un contrato estable entre "lo que StrayMark gobierna" y "lo que Baton enruta". Diseñar para graduar es más caro al inicio y mucho más barato después.

> **Decisión tomada (§10.1).** Lo registro explícitamente porque el "diseñar para graduar" es una restricción transversal que afecta cada decisión técnica posterior.

---

## 2. Los dos problemas reales

### 2.1 Driver económico — routing consciente de costo

Hoy **no existe en StrayMark ninguna noción de modelo, tier, presupuesto, token o costo** (verificado: cero ocurrencias en el código). Todo se ejecuta con el mismo modelo, sea un commit trivial o un rediseño arquitectónico. Baton construye, desde cero, la capa que clasifica tareas y las enruta al tier adecuado — **sujeta a una restricción económica dura** que se detalla en §4.2.

### 2.2 Driver estructural — la deriva entre intención (SpecKit) y estado (StrayMark/Loom)

Este es el problema que Loom hizo visible y que debemos resolver de raíz.

**Lo que ocurre hoy, verificado contra el código:**

- Un Charter puede declarar `originating_spec: specs/004/spec.md`, pero StrayMark **solo valida que el archivo exista**; **nunca parsea su contenido** (`validate_spec_path()`). El vínculo Charter ↔ SpecKit es nominal.
- La proyección de arquitectura de Loom (`core/src/architecture/projection.rs`) deriva el estado de cada componente desde señales de gobernanza (charters, drift, AILOGs, TDEs, inventario en disco), pero **nunca consulta los specs de SpecKit**. `architecture generate` mina directorios de código, tablas *Affected Components* y diagramas C4 de ADRs; **no lee `spec.md`, `plan.md` ni `tasks.md`**.

**Consecuencia:** la arquitectura intencionada (SpecKit) y la arquitectura emergente (StrayMark/Loom) son **dos planos que nadie reconcilia**. La intención se diluye en la implementación y la divergencia es **invisible** porque ningún artefacto los compara. Loom *visualiza* el plano emergente para que el humano ponga restricciones, pero no *importa la intención de SpecKit* — y por eso no basta.

#### Caso canónico — la telemetría mockup de Sentinel

En Sentinel estamos en fase de front-end y descubrimos que **toda la telemetría está en modo mockup**. ¿Por qué? Un agente emitió un fragmento que produce telemetría falsa porque **ignoraba que faltaba implementar un módulo completo encargado de proveer esa telemetría** — módulo que sí estaba en el plan global de módulos interrelacionados de SpecKit. Ese plan se olvidó; el agente improvisó mockups donde los necesitó. Ahora el front-end se construye sobre cimientos falsos.

Es el mismo mecanismo que el caso PolicyEngine (donde funciones de un módulo dedicado se dispersaron en otros, rompiendo la arquitectura clean): **un agente que no consultó el plano global de SpecKit toma decisiones locales que contradicen el diseño, y ninguna herramienta lo alerta.** Estos no son descuidos humanos aislados — son un hueco estructural.

#### El ángulo de integración

SpecKit expone un sistema de **extensiones/hooks** que podemos aprovechar. La meta operativa no es "leer SpecKit una vez", sino lograr una **integración fuerte y continua** entre los productos de diseño de SpecKit (plan global, módulos interrelacionados, decisiones arquitectónicas) y los planes de implementación/seguimiento de StrayMark, de modo que **un agente nunca implemente a ciegas un mockup donde el plan exige un módulo real**.

> **Item de investigación (§10.3):** falta verificar qué ofrece hoy el sistema de hooks/extensiones de SpecKit. No tengo grounding confirmado; hay que investigarlo antes de diseñar el adaptador.

### 2.3 Por qué son un solo experimento, y en qué orden

Un router más barato que opera sobre **intención olvidada** solo automatiza la deriva más rápido. Por eso: **coherencia primero, routing después**. El Coherence Bridge entrega valor (cazar la deriva de Sentinel) **antes** de tocar un solo modelo.

---

## 3. Lo que StrayMark YA es (cimientos verificados)

| Pieza real | Qué es | Para qué le sirve a Baton |
|---|---|---|
| **Charter** | Unidad acotada: alcance, archivos, riesgos, verificación, cierre. `declared → in-progress → closed` | Frontera de un lote a gobernar — pero **demasiado amplio** para enrutar (ver §4.3) |
| **Batch / Batch Ledger** | Registro ex-post de batches completados en Charters multi-sesión | Granularidad sub-Charter; candidato a unidad enrutable |
| **`straymark charter audit`** | Orquesta auditores externos de familias distintas, consolida findings, opt-in | **El precedente de orquestación multi-modelo que ya existe** |
| **`effort_estimate` (XS/S/M/L)** | Estimación de **tiempo humano** | Eje inicial de clasificación, insuficiente por sí solo |
| **AILOG / AIDEC / ADR** | Bitácora de ejecución / decisión ligera / decisión arquitectónica formal | Cadena de evidencia y decisión que el router respeta y cita |
| **Follow-ups (FU-NNN)** | Registry de trabajo diferido: buckets, severidad, dedup por hash | Backlog estructurado del que salen tareas enrutables |
| **TDE + `affects`** | Deuda técnica con scope de globs; `promoted_from_followup` | Señal de "dónde hay deuda" que ya alimenta el overlay de Loom |
| **`analyze` (cognitiva/ciclomática)** | Complejidad por función vía `arborist-metrics` (13 lenguajes) | Clasifica **código existente**, no tareas de IA — base parcial |
| **Proyección de arquitectura (Loom)** | `model.yml` + estado por componente (`Active/Implemented/HasDebt/WiringGap/Uncharted`) | El plano "qué construimos realmente" — un lado de la reconciliación |
| **`charter drift` + `glob_match`** | Detección declared-vs-actual; matcher compartido | Garantía de consistencia que el Bridge reutiliza, no duplica |

**Lo que NO existe** (construcción nueva): cualquier registro de modelos/tiers/presupuestos/tokens/costo/routing; y cualquier lectura del *contenido* de SpecKit.

---

## 4. Visión de Baton

Tres piezas, construidas en orden.

### 4.1 Coherence Bridge — vive en `core`

Cierra la brecha del §2.2. Ingiere el *contenido* de SpecKit (plan global, módulos interrelacionados, tareas, decisiones/C4), lo vincula con la gobernanza por **IDs explícitos** (no por similitud textual) y produce un **diagnóstico de deriva reconciliable**:

- Módulos/decisiones de SpecKit sin componente correspondiente en el `model.yml` de Loom → **la alerta que habría cazado tanto el PolicyEngine como la telemetría mockup**.
- Tareas de SpecKit sin Charter ni evidencia; charters cerrados con tareas de origen ambiguas; estado declarado incompatible con la realidad implementada.

**Decisión tomada (§10.6):** el Bridge vive en `core` porque ramifica hacia **Loom** (extiende la proyección con un tercer plano: *intención vs. gobernanza vs. código*) y hacia **Baton** (el router consulta la coherencia antes de enrutar). Nunca reescribe SpecKit en silencio: emite diagnóstico y, a lo sumo, un patch revisable.

### 4.2 Cost-Aware Router — empieza por clasificar y recomendar

**Decisión tomada (§10.2):** Baton arranca **clasificando y recomendando**, no ejecutando. Queremos asegurar que la clasificación se haga bien antes de delegar ejecución. Una vez sólido, seguiremos con ejecución de agentes (que requerirá archivos de configuración y, posiblemente, interfaces web de configuración y vigilancia — ver §5).

Generaliza el patrón de `charter audit` hacia el ciclo completo: clasificar → recomendar tier → (luego) ejecutar bajo política → verificar → escalar por señales.

| Rol | Trabajo típico | Tier inicial |
|---|---|---|
| Planner / Architect | Descomposición compleja, arquitectura, trade-offs, criterios | Frontier |
| Implementer | Implementación acotada sobre una unidad enrutable | Económico → frontier según clasificación |
| Auditor | Contraste independiente (ya es `charter audit`) | Familias distintas |
| Operator | Commits, PRs, docs, limpieza, preparación de contexto | Económico / local |

#### Principio económico de diseño (restricción dura)

> **El costo de clasificar, atomizar y enrutar no debe igualar ni superar el ahorro de no cargar todo a un modelo frontier.** Si la complejidad de la solución empareja el costo del sistema sin ella, la solución no existe. Toda decisión de granularidad y de "cuánto modelo gastar en decidir a qué modelo enrutar" se mide contra este techo.

Corolario: la clasificación debe ser **barata** (reglas, heurísticas, señales ya computadas por StrayMark — `effort_estimate`, complejidad de `analyze`, riesgo del Charter, estado de la proyección) antes de recurrir a un LLM clasificador, y solo escalar a un clasificador caro cuando el ahorro esperado lo justifique.

### 4.3 La unidad mínima enrutable

Descartamos el "Work Unit" inventado **como vocabulario falso**, pero el problema que nombraba es real: **necesitamos una unidad mínima clasificable**. El Charter es demasiado amplio — contiene etapas heterogéneas (planeación, codificación, auditoría, remediación, cierre, limpieza) que pertenecen a tiers distintos. El batch a veces también mezcla.

Dos caminos vivos (sub-decisión de diseño, §10.4):

- **(a) Normalizar batches para que sean enrutables.** Planear los batches con la intención de que su carga sea homogénea y fácilmente clasificable (un batch = un tipo de trabajo = un tier). Ventaja: no inventa vocabulario nuevo; reusa lo existente. Riesgo: impone disciplina de planeación que no siempre se cumple.
- **(b) Introducir una sub-unidad real dentro del batch** (un "work unit" *legítimo*, esta vez como concepto diseñado, no alucinado) como la unidad clasificable. Ventaja: granularidad fina sin forzar la forma de los batches. Riesgo: **el overhead de atomizar/enrutar a ese nivel puede violar el principio económico de §4.2.**

Ambos caminos se evalúan contra el techo económico. Probablemente la respuesta sea híbrida y empírica (medir en Sentinel qué granularidad paga).

---

## 5. Plataforma compartida de control (dirección emergente)

Cuando Baton pase a ejecución, necesitará **configuración y vigilancia** — posiblemente interfaces web. En lugar de que cada experimento monte su propio servidor, surge una dirección mayor:

> **Reingeniería del servidor web interno de Loom hacia un host autónomo de módulos.** Un dashboard interno sobre el cual se montan, como extensiones, los módulos experimentales (Loom, Baton) y los ya graduados. Loom dejaría de "ser el servidor" para convertirse en "un módulo del servidor".

Esto encaja con el "diseñar para graduar" del §1.3: la plataforma es el vehículo natural de graduación de varios experimentos a la vez. Es una línea de trabajo **propia**, mayor que Baton, y probablemente merezca su **propio nombre y su propio Charter** cuando la abordemos (un candidato a explorar: *Podium* — el estrado que sostiene tanto al director como a los módulos). Por ahora solo la registramos como dirección, sin comprometer alcance.

---

## 6. Anclajes reutilizables vs. construcción nueva

**Se reutiliza (no reinventar):**
- El patrón **config-driven** ya probado (`architecture:` en `config.yml` #279, `complexity.threshold`) para declarar tiers, reglas de routing y presupuestos.
- **`charter audit`** como esqueleto de orquestación multi-modelo.
- **`glob_match` / `drift`** compartidos — el Bridge consume el mismo matcher (consistencia garantizada).
- **Los tipos documentales existentes** (charters, AILOGs, follow-ups, TDEs, ADRs) como persistencia. **Decisión tomada (§10.5):** mientras no surja una necesidad específica de un documento nuevo, usamos la gama que StrayMark ya ofrece.
- **`effort_estimate`, `analyze`, la proyección de Loom** como ejes iniciales de clasificación barata.

**Se construye desde cero:**
1. Esquema de clasificación de tareas (atómica/mecánica vs. diseño/diagnóstico; ejes de riesgo, verificabilidad, superficie, confidencialidad).
2. Registro de modelos y política de routing (declarativo, con fallback y escalamiento por señales).
3. Telemetría económica (costo por unidad/Charter aceptado; tasa de escalamiento; % local/intermedio/frontier).
4. Ingestión de SpecKit como **adaptador versionado** (vía su sistema de hooks/extensiones — pendiente de investigar).

---

## 7. Plan incremental

Coherencia primero; clasificar-y-recomendar antes que ejecutar; plataforma como track habilitante.

| Fase | Alcance | Resultado | Toca modelos |
|---|---|---|---|
| **0 — Encuadre** ✅ | Este documento; Charter + specs de Baton; fronteras `core` para graduar | Contrato conceptual estable | No |
| **1 — Coherence Bridge (read-only)** ✅ **HECHA (2026-06-25)** | Adaptador SpecKit → IntentModel + procedencia → motor C1–C4 → overlay para Loom; dogfood en Sentinel | Caza la deriva #304-class read-only; gate cumplido. Ver `03-sentinel-dogfood-report.md` y `CHARTER-01` | No |
| **2 — Clasificación + router en seco** | Esquema de clasificación barato; router que **recomienda** (no ejecuta); telemetría de "qué iría a qué tier"; experimentar granularidad de unidad enrutable en Sentinel | Visibilidad económica + validación del principio de §4.2, sin riesgo | Solo telemetría |
| **3 — Ejecución bajo política** | Registro de modelos; archivos de config; ejecución aislada; tracking de costo real; (posible) interfaz web de config/vigilancia | Ahorro medible de costo por resultado aceptado | Sí |
| **Track P — Plataforma** | Reingeniería del servidor interno hacia host de módulos (Loom + Baton) | Vehículo de graduación | — |
| **4 — Router empírico** | Métricas históricas; costo esperado por ruta; recomendaciones aprendidas | Optimización por evidencia | Sí |

**Graduation gate (borrador):** *Fase 1 caza al menos una deriva arquitectónica real en Sentinel que la revisión humana habría pasado por alto; Fase 2 demuestra que ≥X% del trabajo es enrutable a tier económico **sin que el overhead de clasificación borre el ahorro** (principio §4.2) y sin pérdida de calidad.*

---

## 8. Gobernanza, seguridad y confianza (conservado del borrador)

- **Independencia invertida:** Baton gobierna/observa Sentinel aun cuando Sentinel esté roto. StrayMark nunca depende de que el proyecto gobernado esté operativo.
- **Mínimo privilegio por unidad enrutable**; push/PR son gates independientes del permiso de editar.
- **Verificación determinista primero:** build, tests, tipos, contratos y `drift` sobre juicios probabilísticos.
- **Independencia de auditores real, no por instrucción** (identidad provista por gateway; cero findings con baja cobertura ≠ aprobación).
- **Evidencia antes que autoafirmación:** el estado deriva de eventos verificables. **Semántica de aceptación (decisión §10.7):** `accepted` significa "con evidencia"; los estados intermedios viven en el sidecar de gobernanza.

---

## 9. No objetivos

- **No** mutar el núcleo de StrayMark en orquestador en esta fase (Baton se diseña graduable, con fronteras limpias para una integración futura ordenada).
- **No** reemplazar SpecKit ni imponer un formato nuevo.
- **No** marcar trabajo como aceptado solo porque los tests pasan; **no** eliminar el juicio humano en alto riesgo.
- **No** obligar a un proveedor de modelos ni a un runtime particular.
- **No** inferir vínculos SpecKit↔gobernanza con LLM cuando hay IDs explícitos.
- **No** dejar que el overhead de la solución empareje el costo de no tenerla (§4.2).

---

## 10. Decisiones tomadas y sub-decisiones abiertas

### Tomadas (esta sesión)

| # | Decisión | Resolución |
|---|---|---|
| 1 | Encuadre | Experimento hermano **diseñado para graduar** al núcleo |
| 2 | Alcance de ejecución | **Clasificar y recomendar primero**; ejecución después (config + posibles interfaces web) |
| 5 | Persistencia | **Reutilizar** los tipos documentales existentes hasta que surja necesidad específica |
| 6 | Coherence Bridge | Vive en **`core`**; ramifica a Loom y Baton |
| 7 | Semántica de aceptación | `accepted` **con evidencia**; estados intermedios en el sidecar |
| 8 | Identidad/costo de modelos | **Diferido** (no hay contrato unificado de proveedores; planear con cuidado) |

### Sub-decisiones de diseño abiertas

| # | Pregunta | Notas |
|---|---|---|
| 3 | **Integración SpecKit** | **Investigar** el sistema de hooks/extensiones de SpecKit. Meta: que un agente nunca improvise un mockup donde el plan global exige un módulo real |
| 4 | **Unidad enrutable** | (a) normalizar batches vs. (b) sub-unidad real dentro del batch. Evaluar empíricamente en Sentinel contra el techo económico (§4.2) |
| P | **Nombre de la plataforma** | El host de dashboards (Loom + Baton como módulos) necesitará nombre y Charter propios. Candidato a explorar: *Podium* |
| E | **Presupuesto de overhead** | Cuantificar el techo de §4.2: ¿cuánto puede costar la clasificación como fracción del ahorro esperado? |

---

## 11. Próximo paso recomendado

1. **Investigar el sistema de hooks/extensiones de SpecKit** (sub-decisión #3) — destranca el diseño del Coherence Bridge.
2. Con eso, redactar el **Charter del experimento** (`experiment-baton/CHARTER-01-coherence-bridge.md`) y el spec (`experiment-baton/specs/001-coherence-bridge/`), siguiendo el patrón de `experiment-loom/`, enfocados en la **Fase 1** (coherencia, read-only, sin modelos).

El Coherence Bridge es el primer entregable porque es el problema más doloroso y ya validado (telemetría mockup + PolicyEngine), entrega valor sin tocar modelos, y evita que un router opere sobre intención incoherente.

---

*Fin del documento — base de trabajo, v0.2.*
