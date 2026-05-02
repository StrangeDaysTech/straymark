# DevTrail — Principios de diseño

**Versión:** 0.2.1 (anotaciones empíricas tras el experimento `/plan-audit` en Sentinel; rename Plan → Charter en vocabulario going-forward)
**Fecha:** 30 de abril de 2026
**Autor:** Jose Villaseñor Montfort — StrangeDaysTech
**Propósito:** Articular la filosofía del producto en forma compacta, para servir como referencia frente a decisiones de diseño y como criterio para decir no a features que no encajan.
**Documentos relacionados:** `devtrail-thesis-validation.md` (cuerpo de evidencia que motivó las anotaciones de v0.2).

---

## Por qué este documento existe

Los productos de software derivan, con el tiempo, hacia donde los empuja el incentivo más cercano. Si la presión es revenue, derivan hacia features que venden. Si la presión es adopción, derivan hacia features que viralizan. Si la presión es competencia, derivan hacia paridad con el competidor más visible. Cada una de esas derivas es individualmente racional y colectivamente destructiva: el producto pierde su forma original y se convierte en otro punto promediado del mercado.

Este documento existe para que DevTrail tenga un ancla explícita. Las decisiones futuras deben poder contrastarse contra estos principios. Una feature que viole un principio puede aún ser buena idea, pero requiere una conversación consciente sobre qué principio se está flexibilizando y por qué. La existencia del documento no garantiza disciplina; la pregunta consciente que genera, sí.

Los principios están ordenados por jerarquía: cuando dos entran en conflicto, gana el que viene antes.

**Versión 0.2 — anotaciones empíricas.** Esta versión incorpora aprendizajes del primer experimento que estresa los principios contra evidencia: Sentinel `/plan-audit` durante 6 ciclos (PLAN-01..06), tres iteraciones de formato (v1 → v2 → v3) y dos auditorías externas duales por ciclo. Las anotaciones por principio referencian ese cuerpo de evidencia y se concentran en los principios que el experimento ejercitó directamente (#6, #9, #12). La jerarquía y la redacción de los doce principios no cambian.

---

## 1. La herramienta sirve al oficio, no al producto

DevTrail existe para que el ingeniero que trabaja con agentes de IA produzca trabajo del que se sienta orgulloso. Esa es la métrica final. Cualquier feature que mejore métricas de adopción, retención o revenue a costa de degradar la calidad del trabajo del usuario es una feature que viola el principio fundacional.

En la práctica, esto significa que las decisiones se toman con la pregunta *"¿esto ayuda al ingeniero a hacer mejor ingeniería?"*, no con la pregunta *"¿esto crece el producto?"*. Las dos preguntas a veces convergen y a veces no. Cuando divergen, gana la primera.

## 2. El usuario primario es el ingeniero senior orquestando agentes

No el VP Engineering. No el CISO. No el compliance officer. El usuario primario es el ingeniero con criterio técnico fuerte que usa agentes de IA para trabajar en proyectos que no podría abordar solo, y que necesita disciplina cognitiva externalizada para que el agente no introduzca caos sistémico.

Las features se diseñan para esa persona. Los flujos se optimizan para esa persona. La documentación se escribe para esa persona. Cuando otras personas (compliance officers, managers, auditores) eventualmente usan el producto, lo hacen sobre una base que respeta primero al ingeniero. La inversión de esta jerarquía es la causa más común de productos de developer tooling que se vuelven malos.

## 3. Open source estricto, sin asteriscos en el núcleo

El framework, el CLI y el TUI son y serán siempre open-source bajo licencia permisiva. Sin features capadas en el OSS para empujar al pago. Sin telemetría obligatoria. Sin requerimiento de cuenta para uso completo. Sin "community edition" que sea inferior a una "enterprise edition" en capacidades core.

Cuando exista una capa comercial (Cloud, hosting, servicios), debe ofrecer valor genuino aditivo, no rescatar funcionalidades que se sacaron del OSS para crear demanda artificial. La diferencia entre open-core honesto y open-core como bait-and-switch es la honestidad sobre qué es valor adicional vs qué es la herramienta misma.

## 4. El cumplimiento regulatorio es un side effect, no el producto

ISO/IEC 42001, EU AI Act, NIST AI RMF, GDPR son frameworks útiles que dan estructura a la documentación. DevTrail los soporta como capa opcional porque son valiosos para usuarios que los necesitan. Pero el producto no es "una herramienta de compliance". El producto es una herramienta de ingeniería que produce, como subproducto, evidencia compatible con esos frameworks.

Esta distinción importa porque el comprador de compliance y el ingeniero que usa la herramienta tienen necesidades distintas. Cuando el producto se posiciona como compliance, las decisiones de diseño se sesgan hacia el comprador de compliance y degradan el flujo del ingeniero. Mantener el cumplimiento como side effect protege la calidad del flujo principal.

## 5. Schema-driven antes que feature-driven

Las entidades centrales del producto (Stage Closure Bundle, Charter, Document) se definen primero como schemas formales con versionado estricto. Las features se construyen contra esos schemas. Esto tiene tres beneficios estructurales: permite que múltiples implementaciones coexistan sin coordinarse, hace que la evolución sea controlada y reversible, y obliga a pensar el contrato antes que la implementación.

Las consecuencias prácticas son que cambios breaking en schemas son raros y deliberados, que los schemas son documentación de primera clase del producto, y que features que requieren violar un schema requieren primero proponer la evolución del schema.

## 6. Disciplina cognitiva sobre productividad bruta

DevTrail no compite con herramientas que prometen *"haz código 10x más rápido con IA"*. Compite, si compite con algo, contra el caos que el código rápido con IA genera en proyectos serios. La métrica que importa no es velocidad bruta del agente sino calidad sostenida del sistema producido.

Esto significa que features que añaden fricción justificada (validaciones, gates, requerimientos de documentación, pre-trabajo antes de planning) son aceptables y a veces deseables. Features que prometen quitar fricción a costa de la calidad del razonamiento del agente son sospechosas por defecto.

**Anotación v0.2 — distinción virtud vs ceremonia.** Sentinel reveló que no toda fricción del flujo DevTrail tiene el mismo carácter. La fricción es *virtuosa* cuando externaliza signal pública: nombrar `R<N+1> (nuevo, no en Plan)` permite que auditores externos heterogéneos lo capturen (`AILOG-020` §Additional Notes); `check-plan-drift.sh` ejecutado al cierre detecta drifts archivos-declarados-vs-modificados con cero falsos positivos en 2/2 tests empíricos (`AILOG-022` §Summary); la auditoría dual Copilot+Gemini calibra cross-modelo y captura gaps que un solo auditor pierde (`PLAN-05.telemetry.yaml`). La fricción es *ceremonia atacable* cuando solo genera triage manual o prescribe sin reflexividad: el TEMPLATE prescribiendo una firma arquitectónica que choca con la convención del módulo destino (F2 en PLAN-05) o el drift script alertando sobre R<N> ya documentados en AILOG (`AILOG-022` R2). La primera se mantiene; la segunda es bug del formato, no virtud del principio. Ver `devtrail-thesis-validation.md` §5 para el desarrollo y los criterios accionables.

## 7. Local-first, Cloud como amplificador

El CLI funciona completo, sin red, en el repo local. Esa es la primera experiencia y la base sobre la que todo se construye. Cloud existe para amplificar el valor (agregación cross-repo, evidencia firmada, búsqueda semántica histórica, colaboración de equipo) pero nunca para reemplazar al CLI o convertirlo en cliente delgado.

Un usuario que nunca conecta al Cloud debe poder obtener la mayor parte del valor del producto. Esto protege la herramienta contra modos de falla del Cloud, contra cambios de modelo de negocio, y contra la deriva clásica de productos que empiezan local y terminan exigiendo conexión obligatoria.

## 8. La memoria del proyecto vive en el repo, no en una base de datos externa

AILOGs, ADRs, AIDECs, Charters y bundles viven como archivos versionados junto al código. La razón es estructural: la memoria del proyecto debe sobrevivir al producto que la generó. Si DevTrail desaparece mañana, los `.devtrail/` siguen siendo legibles, parseable y útiles porque son markdown estructurado y JSON conformante a schemas públicos.

Esta decisión renuncia a algunas capacidades que serían más fáciles con backend centralizado (por ejemplo, búsqueda full-text instantánea sin sincronización), pero gana confiabilidad de largo plazo y control del usuario sobre sus propios datos. La asimetría es deliberada y permanente.

## 9. Simplicidad sobre capacidad

Cuando dos diseños cumplen el mismo objetivo, gana el más simple. Cuando una feature es genuinamente útil pero requiere complejidad significativa, conviene esperar hasta que el patrón de uso esté validado en al menos tres proyectos reales antes de cristalizarla en el producto.

Esto se aplica tanto al producto como al lenguaje del producto. Los nombres de comandos, tipos de documento y conceptos del framework se eligen para ser inmediatamente comprensibles, no para sonar técnicamente sofisticados. Si un concepto requiere explicación larga para un usuario nuevo, probablemente está mal diseñado.

**Anotación v0.2 — bash antes que framework cuando bash basta.** Sentinel demostró que `scripts/check-plan-drift.sh` (~145 líneas bash con awk + grep + git) cubre el caso "drift de archivos declarados vs modificados" con cero falsos positivos sobre dos tests empíricos. La decisión consciente de no implementarlo en Go inicialmente está documentada en `AILOG-022` §Decision: "Bash es zero-build, sin dependencies, inspeccionable in situ; el costo de mantenimiento es bajo". Cuando se porte al CLI Rust en fase 2 del roadmap (`devtrail-cli-roadmap.md`), preservar la simplicidad bash en lo posible — invocar el shell desde Rust o reimplementar con cuidado de no inflar la lógica. La complejidad solo se cristaliza cuando el patrón de uso lo demande, no por defecto.

## 10. Honestidad sobre lo que la herramienta no hace

DevTrail no evalúa modelos. No es un LLM gateway. No reemplaza al juicio del ingeniero. No previene todas las alucinaciones. No certifica automáticamente cumplimiento regulatorio. Las páginas del producto, la documentación y la comunicación pública son explícitas sobre estos límites.

La razón es práctica: usuarios que adoptan la herramienta con expectativas realistas la mantienen. Usuarios que la adoptan creyendo que hace cosas que no hace la abandonan frustrados. La honestidad temprana sobre los límites es mejor inversión de marketing que las promesas amplias.

## 11. La comunidad cuida la herramienta, no al revés

Los usuarios de DevTrail son ingenieros que típicamente saben más que el equipo del producto sobre sus propios casos de uso. Las contribuciones, issues y feedback se tratan con respeto profesional, y las decisiones de roadmap se toman incorporando esa visión. No de forma democrática (el producto necesita una visión coherente), pero sí como input serio.

Esto no significa hacer todo lo que la comunidad pide. Significa explicar cuándo se dice no y por qué, mantener la conversación visible y pública, y aceptar que el producto será mejor por la presión inteligente de quienes lo usan en serio que por las decisiones aisladas del equipo.

## 12. La velocidad del producto es la velocidad del aprendizaje

DevTrail no debe avanzar más rápido de lo que aprendemos sobre cómo se usa realmente. Cristalizar features prematuramente, antes de tener datos de uso real en al menos tres proyectos distintos, genera costos altos de mantenimiento sobre features que pueden ser equivocadas. La paciencia de esperar evidencia es disciplina, no inacción.

Esto se traduce en una preferencia explícita por: prototipos antes que features, observación antes que cristalización, schemas estables sobre features cambiantes, y evolución incremental sobre rewrites grandes.

**Anotación v0.2 — el espíritu del N≥3 frente a Sentinel.** Sentinel es un solo proyecto, un solo dominio (Go backend), un solo autor. Aplicado literalmente, el principio bloquearía cualquier cristalización a partir de su evidencia. Aplicado al espíritu — observación rigurosa antes que cristalización, evolución incremental con datos reales — Sentinel cubre tres ejes de diversidad estructural: (a) escala variable (XS, S, M en cinco ciclos), (b) iteración del formato bajo presión empírica (v1 → v2 → v3, cada una derivada del ciclo anterior), (c) calibración cross-modelo de auditores (Copilot + Gemini convergiendo a 9.25/9.5 en PLAN-05). Estos seis ciclos × tres formatos × tres escalas son evidencia equivalente al N≥3 que el principio busca proteger.

Lo que sigue sin diversificar es el dominio. Para preservar el principio en su forma operacional: cualquier schema que se cristalice a partir de Sentinel se publica como `v0.json` con marca *experimental*. La transición a `v1.0` estable requiere validación con un segundo proyecto en otro dominio (frontend, ML pipeline, infra-as-code). Cualquier feature que dependa de un supuesto sin evidencia (como el #4 "aprobaciones rara vez binarias", que Sentinel no ejercitó) queda explícitamente bloqueada hasta tener un proyecto multi-actor. Ver `devtrail-thesis-validation.md` §6 y §8 para el desarrollo del argumento y la lista de gaps específicos por supuesto.

**Patrón meta-meta detectado en Sentinel — auto-evolución del formato.** El experimento reveló que el formato se mejora a sí mismo: cada Plan ejecutado en Sentinel (y, going-forward en DevTrail, cada Charter ejecutado) genera datos que refinan el formato del próximo (`AILOG-022` §Additional Notes). Esto sugiere ampliar el principio en futuras versiones: la herramienta evoluciona consigo misma; cada uso es input para la próxima versión. Por ahora se documenta como observación, no como principio adicional, hasta que un segundo proyecto confirme el patrón.

---

## Cómo usar este documento

Cuando aparezca una decisión de diseño no trivial, conviene contrastarla explícitamente contra los doce principios y registrar el resultado. Tres patrones útiles:

**Decisiones que satisfacen todos los principios**: proceder normalmente.

**Decisiones que entran en conflicto con un principio**: documentar explícitamente cuál principio se está flexibilizando, por qué, y qué evidencia llevaría a revertir la decisión. Este registro debe quedar en un AIDEC o ADR.

**Decisiones que violan principios fundacionales (1, 2, 3, 4)**: no se toman sin revisar primero si el principio sigue siendo correcto. Si lo sigue siendo, la decisión se descarta. Si no, se actualiza el principio antes de tomar la decisión, no después.

Los principios se revisan formalmente cada seis meses con la evidencia acumulada. Cambios entre revisiones formales son posibles pero deben ser deliberados y documentados.

---

*Este documento es un guardarraíl, no un manual. Su utilidad se mide en cuántas veces nos hace pausar antes de tomar una decisión que después habríamos lamentado.*
