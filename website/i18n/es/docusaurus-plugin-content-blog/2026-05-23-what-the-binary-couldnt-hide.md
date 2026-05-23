---
slug: what-the-binary-couldnt-hide
title: Lo que el binario no pudo esconder
authors:
  - jose
tags:
  - straymark
  - charters
  - governance
  - polish-charter
  - sentinel
  - anti-patron
date: 2026-05-23T00:00:00.000Z
description: Diez gaps latentes aflorados en un solo polish Charter, y el anti-patrón que se ganó un nombre
---
*Un polish Charter que iba a ser limpieza cosmética afloró diez gaps de producción en seis horas — incluyendo dos features que estaban shippeadas a `main` y nunca habían funcionado durante diez días. El anti-patrón obtuvo nombre, el patrón se documentó como `fw-4.18.0`, y el helper de CLI se difirió a propósito.*

<!-- truncate -->

> *"Lo primero que el polish Charter intentó — bootear `./sentinel` desde `main` para correr el smoke de §5 — falló con dos panics distintos."*

Así abrió el Issue [#199](https://github.com/StrangeDaysTech/straymark/issues/199) el 22 de mayo, presentado por el adopter Sentinel como RFC. Para cuando el thread cerró cinco actualizaciones de comentario después, la cuenta había crecido de dos a diez. El polish Charter — el Charter de cierre de Etapa 2, planeado como un docket de auditorías WCAG y verificación del quickstart — había pasado seis horas haciendo otra cosa: cazando una clase de regresión latente que ninguna de las suites de tests per-Charter de los ocho Charters previos había cazado, y que ninguna de ellas *podía* haber cazado dado cómo estaban escritas.

Este post es la reconstrucción de por qué ocurrió, qué forma recurrente terminó habiendo debajo, y la decisión deliberada en el PR [#200](https://github.com/StrangeDaysTech/straymark/pull/200) de **nombrar el anti-patrón pero todavía no fabricar tooling para él**. El ciclo le es familiar a quien siguió los posts previos sobre [observación emergente](emergent-observation-design) y [evolución de cadenas](pattern-1-and-pattern-2-chain-evolution) — este es el tercer patrón en dos semanas que cristaliza vía el mismo arco: aflorar en N=1, nombrar el meta, diferir el tooling cross-proyecto hasta que N=2 lo valide.

## Los dos casos que pesaron

De los diez gaps que la sesión de polish afloró, ocho eran o bien deterioro ambiental de dependencias (un upgrade de `huma` introdujo un panic en parámetros `*string`; Go 1.22 endureció la semántica de wildcards de `http.ServeMux`) o bien drift del runbook (env vars faltantes en §boot, recetas de smoke que mezclaban modos mutuamente excluyentes, afirmaciones sobre el stdout de un fake provider que el fake nunca escribió). Cada uno merece un fix. Ninguno reescribe el manual.

Los dos que sí reescribieron el manual son distintos. Comparten forma.

**US3 Preference Center, shippeado el 12 de mayo.** Cuando los recipients de un correo enviado por Sentinel hacen click en el link tipo unsubscribe del footer, aterrizan en una URL JWT-in-path como `/preferences/<token>`. El handler es público-por-contrato a propósito: el JWT *es* la auth; no se espera un header `Authorization`. El doc-comment del handler lo decía. El test de integración, escrito en `humatest`, montaba el handler directamente a través del adapter de testing y confirmaba: dado un JWT válido en el path, el handler devuelve el HTML correcto. CI pasó. La feature se shipeó a `main`.

Lo que nunca pasó: nada que ejerciera la cadena de middleware de producción enfrente de ese handler. `internal/core/middleware/auth.go` tiene una lista `publicPrefixes` de prefijos de ruta que evaden el check de `Authorization`. `/preferences/` no estaba en ella. Durante diez días, cada recipient que hizo click en un link de unsubscribe recibió un `401 missing authorization header` del middleware antes de que el request alcanzara nunca al handler que sabía que no debía esperarlo. El Preference Center era alcanzable en tests, inalcanzable en producción.

**Instrumentos de observabilidad OTel del pipeline de envío, shippeados la misma semana.** Ocho metric instruments — counters, histograms, gauges — declarados en `internal/core/metrics/commshub.go`, registrados con el meter de OpenTelemetry al boot. El `Constitution Check §IV` del Charter declaró formalmente satisfecha la compuerta de observabilidad FR-039..042: la capa de API era correcta, los registros pasaban, los dashboards estaban planeados. Siete de esos ocho instrumentos tenían **cero call sites de `.Add()` / `.Record()` en todo el módulo commshub**. Declarados, registrados, nunca invocados. Los dashboards que ops construyó encima de esas series llevaban diez días recibiendo cero datos. El smoke manual del polish Charter — bootear un OTel Collector, generar unos cuantos sends, grep del output del collector buscando los nombres FR-039..042 — afloró esto al instante.

Ninguno de los dos casos es exótico. Ninguno es un bug duro. Ambos son instancias del mismo error mecánico: un artefacto se declaró en un lugar, y la implementación que se suponía que lo cableara vivía en otro lugar, y nada en el codebase ni en CI correlacionaba los dos.

## El nombre que el anti-patrón pedía

Un nombre importa cuando la misma forma aparece suficientes veces como para merecer uno. Para la tercera actualización del comentario en #199, el autor de Sentinel había contado cuatro subclases del mismo error subyacente:

| Sitio de declaración | Sitio de cableado | Verificación mecánica |
|---|---|---|
| env var documentada en el runbook operativo | `os.Getenv(...)` (o equivalente del stack) en código | cada env var documentada tiene al menos un consumidor |
| metric instrument declarado en un paquete de métricas | call site `.Record()` / `.Add()` en código de handler o worker | cada instrument declarado se registra al menos una vez |
| URL referenciada desde HTML renderizado/embebido (`<script src=...>`, `<link href=...>`) | ruta registrada en la misma superficie de API | cada `src=`/`href=` en HTML servido resuelve a una ruta registrada |
| ruta marcada pública-por-contrato (doc-comment, marker dedicado) | entrada en la lista de prefijos públicos del middleware de auth | cada handler público-por-contrato tiene una entrada de prefijo equivalente |

El unificador — el one-liner con el que abre el nuevo doc de gobernanza — es:

> *Cada artefacto de superficie declarado tiene al menos un sitio de cableado alcanzable desde un request real.*

El nombre del anti-patrón, que aterriza canónicamente en `dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md`, es **Declaración de superficie sin cableado.** Ese es el deliverable. El polish Charter es el vehículo de descubrimiento, no la tesis.

## Por qué los tests de integración omiten esto sistemáticamente

Esta parte vale la pena trabajarla porque le da forma al resto del post. El modo de falla común a las cuatro subclases es que el harness estándar de tests de integración — `humatest.NewTestAdapter` en Go, el equivalente en TypeScript, Python, Rust — monta los handlers *directamente* vía la API de testing. El handler bajo prueba está correctamente cableado por el fixture. El paso de composición de producción — donde el registro de rutas se encuentra con la cadena de middleware se encuentra con el inventario de env vars se encuentra con la tabla de assets embebidos — es lo que está roto. La luz verde de CI es genuina respecto a lo que afirma: el handler devuelve la respuesta correcta dado el request correcto. No dice nada sobre si el request puede alcanzar el handler en producción, o sobre si el artefacto del que el handler depende fue alguna vez cableado al runtime.

Hay una tentación de leer esto como una crítica a `humatest`. No lo es. `humatest` (y sus equivalentes) está haciendo exactamente lo que fue diseñado para hacer: dejar que pruebes la lógica del handler en aislamiento, rápido, sin parar todo el árbol de composición. Ese aislamiento es una feature. El costo del aislamiento es lo que acabamos de nombrar — y el costo solo se vuelve visible cuando algo fuera del aislamiento aflora una divergencia. El polish Charter es el método más barato para aflorar esa divergencia, porque hace lo que ningún fixture de test hace: bootea el binario real y corre la receta operativa documentada real contra él.

Esta es la afirmación load-bearing del nuevo pattern doc. No que los polish Charters sean valiosos por razones cosméticas. Que son **el único lugar** donde la composición de producción se ejerce end-to-end contra una especificación externamente legible (el runbook operativo). Si tratas el polish Charter como limpieza cosmética, también tratas esa capacidad de aflorar como cosmética. Los datos de Sentinel argumentan lo opuesto.

## La decisión: B′, no B

El RFC proponía tres opciones. La decisión en el PR #200 fue ninguna de ellas tal como estaban formuladas — llamémosla B′. Vale la pena decir por qué, porque la decisión estructural pesa más de lo que parece.

**La Option B del RFC** pedía un pattern doc bajo `docs/patterns/`. Ese directorio no existe en StrayMark hoy, y crearlo habría partido la convención de documentación del proyecto. Los patterns empíricos que ya viven en el canon — `FOLLOW-UPS-BACKLOG-PATTERN.md`, `CHARTER-CHAIN-EVOLUTION.md`, `EMERGENT-OBSERVATION-DESIGN.md`, `SPECKIT-CHARTER-BRIDGE.md` — todos viven en `dist/.straymark/00-governance/`. Comparten una infraestructura de mirror i18n (cada doc de gobernanza tiene hermanos en inglés, español y chino simplificado). Agregar una tercera superficie de documentación habría multiplicado el overhead de i18n y desparramado los patterns del proyecto entre dos hogares. Mantener el nuevo doc dentro de `00-governance/` reusaba la infraestructura existente con costo marginal cero.

**La Option C del RFC** pedía un helper CLI `straymark charter polish-checklist` o `straymark analyze declared-vs-wired`. El autor de Sentinel ya había prototipado uno (CHARTERs 25/26/27, los tres guards preparatorios de CI) y ofrecido sembrarlo upstream. La decisión aquí fue conservadora: diferir. No porque el prototipo no sea valioso — lo es — sino porque Sentinel es N=1. Las cuatro subclases fueron las que *un adopter, un stack* afloró. La quinta, sexta y séptima subclase que el framework tendría que anticipar para shippear un CLI cross-proyecto útil aún no existen, porque ningún segundo adopter las ha empujado. Ya hemos recorrido este camino. [`FOLLOW-UPS-BACKLOG-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md) shippeó como v0 en `fw-4.10.0` y vive ahí desde hace mes y medio, esperando que un segundo adopter lo valide antes de graduarse a un subcomando `straymark followups`. La misma compuerta aplica aquí. Las `## Open questions` del nuevo doc lo dicen explícitamente: "Cristalización como subcomando CLI `straymark analyze declared-vs-wired` … Gate: N=2 adopters."

**B′ es lo que aterrizó.** Un nuevo governance pattern doc en la ubicación canónica, en tres idiomas. Un séptimo bullet de `Format conventions` en el template de Charter apuntando a los autores hacia él cuando cierran una Etapa o Fase `Polish` de SpecKit. Una fila en la tabla `## Patterns` del QUICK-REFERENCE. Un bump `fw-4.18.0` con la cascada de footers a través de unos treinta docs. Ningún subcomando CLI. Ningún nuevo campo de frontmatter. Ningún cambio de schema. El anti-patrón tiene un nombre; el ritual de descubrimiento tiene un doc; el tooling espera.

## El arco que se repite

Hace tres semanas, el patrón de backlog de follow-ups cristalizó vía el mismo arco: un adopter afloró una necesidad, el patrón se nombró en v0, el helper CLI se difirió. Hace dos semanas, los patrones de evolución de cadena cristalizaron: Pattern 1 (pre-declare SpecKit refresh) y Pattern 2 (post-close audit-driven Batch N.4), aflorados desde el mismo adopter Sentinel, nombrados en `fw-4.16.0`, con `straymark charter refresh-suggest` como helper blando en vez de gate duro. Hace una semana, el meta — `EMERGENT-OBSERVATION-DESIGN.md` — codificó qué hacía posibles esas observaciones de entrada: cross-referencing formal más permiso cultural. Esta semana, este patrón.

Son cuatro cristalizaciones en un mes, todas desde el mismo adopter, todas siguiendo la misma forma: **aflorar en N=1 → nombrar el meta → fabricar tooling solo después de N=2**. La tentación cuando un patrón aflora vívidamente — y diez gaps de producción en seis horas es vívido — es saltearse el nombre e ir directo al tooling. La disciplina es lo opuesto. El nombre hace la mayor parte del trabajo. El tooling, cuando llega, está parametrizado por lo que al menos dos adopters han aflorado; construido sobre N=1, es una extrapolación de un stack y los modos de falla de un equipo, y tiende a osificar esas elecciones como defaults del framework que no generalizan.

Hay una pieza de esta iteración que sí es genuinamente nueva y vale la pena destacar. El nuevo pattern doc nombra una predicción falsable que el autor de Sentinel ya se comprometió a publicar: el polish Charter de la próxima Etapa, ejecutado contra los tres guards preparatorios de CI que acaban de aterrizar en CHARTERs 25/26/27 de Sentinel, debería aflorar aproximadamente 80% menos gaps. Si esa predicción se sostiene, el caso para graduar la Option C de "open question" a "subcomando CLI" obtiene respaldo cuantitativo. Si falla — si el siguiente polish Charter sigue aflorando diez gaps pero en una quinta subclase no anticipada — la falla en sí misma es el siguiente data point, y reescribe el spec antes de fabricar tooling. Cualquiera de los dos resultados es útil. Ninguno se ha observado aún. Esperamos.

## Una nota sobre lo que deliberadamente no se hizo

No hay campo `phase: polish` en el frontmatter de Charter. No hay `straymark charter polish-checklist <ID>`. No hay analyzer automatizado escaneando código Go (ni de ningún otro lenguaje) por símbolos declarados-pero-no-cableados. El pattern doc lista cada una de esas como candidata de evolución, gated explícitamente en N=2 o en el retrospectivo post-Etapa-3. Ninguna está presente en `fw-4.18.0`.

Esto es intencional y vale la pena decirlo en voz alta, porque la reacción natural ante un hallazgo vívido es sobre-construir la respuesta. Cada una de esas postergaciones intercambia exhaustividad de corto plazo por portabilidad de largo plazo. Un campo de frontmatter compromete al framework con un vocabulario que el próximo adopter puede no compartir. Un helper CLI compromete al framework con un runtime que refleja los modos de falla de un stack específico. Un analyzer compromete al framework con escanear un lenguaje cuyas convenciones pueden divergir de las del próximo adopter. Ninguno de esos compromisos debería hacerse con la señal de un solo dominio, por más limpia que sea la señal. El pattern doc en sí mismo puede revisarse; un subcomando CLI no puede desshipearse sin romper a los adopters.

## Qué te sugeriría mirar, si llegaste hasta acá

Si operás un codebase con tests de integración basados en mock adapter — que es la mayoría de los codebases — podés correr un ejercicio interno útil sin adoptar nada de StrayMark. Elegí la feature más reciente que tu equipo shipeó que tenga tanto (a) una declaración del lado de docs (env vars, specs OpenAPI, HTML embebido, metric instruments) como (b) un sitio de cableado que viva en un archivo distinto. Booteá el binario desde una shell limpia. Corré la receta operativa del runbook end-to-end. Si funciona al primer intento, tu codebase aún no tiene la deuda latente que este patrón caza — o sí la tiene y tuviste suerte en este caso. Si no funciona, contá los gaps. El número es la calibración que necesitás para saber si el ritual de polish-Charter-como-detección-de-deuda valdría su overhead en tu proyecto.

Si adoptaste StrayMark y estás cerrando una Etapa con handlers testeados a través de adapters estilo `humatest`, el nuevo pattern doc tiene las cuatro verificaciones de subclase scoped para vos. Presupuestá el polish Charter como L, no XS o S. Esperá Charters emergentes de follow-on, no scope creep de limpieza residual. Leé [`POLISH-CHARTER-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md) antes de scopear el trabajo, no después.

Y si sos adopter en un stack distinto a Go — TypeScript, Python, Rust, Elixir — las cuatro subclases son agnósticas de lenguaje, pero la forma concreta de verificación no lo es. Aflorar una quinta o sexta subclase en tu stack es exactamente la señal de segundo-adopter que el patrón necesita para graduarse de v0. El camino de "observación interesante en tu proyecto" a "nombrada en el canon de StrayMark" corre por el mismo canal por donde corrió #199: abrir un issue. El costo de abrirlo es bajo; el costo de dejar el patrón subvalidado es real.

---

*StrayMark `fw-4.18.0` — Issue [#199](https://github.com/StrangeDaysTech/straymark/issues/199) · PR [#200](https://github.com/StrangeDaysTech/straymark/pull/200) · tag [`fw-4.18.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.18.0). Anclas de Sentinel: [`AIDEC-2026-05-22-001`](https://github.com/StrangeDaysTech/sentinel/pull/93) · [PR de CHARTERs 25/26/27](https://github.com/StrangeDaysTech/sentinel/pull/94).*

*Este documento fue elaborado con asistencia de herramientas de IA generativa (Claude 4.7); toda la responsabilidad del contenido recae en el autor humano.*
