# Patrón de Polish Charter — StrayMark

> El Charter de cierre de una Etapa (o Fase `Polish` de SpecKit) es la compuerta load-bearing para detectar un anti-patrón recurrente — **"Declaración de superficie sin cableado"** — que las suites de pruebas de los Charters de user-story sistemáticamente no pueden atrapar.

**Idiomas**: [English](../../POLISH-CHARTER-PATTERN.md) | Español | [简体中文](../zh-CN/POLISH-CHARTER-PATTERN.md)

---

## Estado

**v1 — validado en N=2 dominios independientes.** Dos ejes, reportados por separado a propósito para no confundirlos:

- **Dominios independientes: 2.** `StrangeDaysTech/sentinel` (backend Go, CHARTER-19 → CHARTER-27, 2026-05-22) y `StrangeDaysTech/lnxdrive` (daemon Rust de sincronización en Linux + escritorio GTK, 2026-05, [hallazgo #209](https://github.com/StrangeDaysTech/straymark/issues/209)). Una app de escritorio en Rust validando un patrón visto primero en un backend Go es la señal cross-domain fuerte que exige la [compuerta de N-status](../../../ADOPTERS.md).
- **Ocurrencias: 3.** Sentinel afloró las subclases originales (1–4); LNXDrive afloró una ocurrencia cualitativamente nueva — una *regresión cross-component de una mitigación ya entregada* (subclase 5, abajo).

La compuerta N=2 para la cristalización en el CLI (reflejada de [`FOLLOW-UPS-BACKLOG-PATTERN.md`](FOLLOW-UPS-BACKLOG-PATTERN.md)) **ya está cruzada**. La convención + anti-patrón nombrado siguen siendo el núcleo portable; la verificación mecánica gradúa a un subcomando `straymark analyze declared-vs-wired` (set-difference dirigido por config, alcance v0 entregado en cli-3.17.x+ — ver [Preguntas abiertas](#preguntas-abiertas)). Los adopters aún pueden reproducir el descubrimiento localmente con un Charter polish dedicado y (opcionalmente) guards de CI local al proyecto.

---

## Cuándo aplica este patrón

El Charter de cierre de una Etapa (en vocabulario SpecKit: la Fase `Polish` que sigue a Foundation + N Fases de user-story — ver [`SPECKIT-CHARTER-BRIDGE.md`](SPECKIT-CHARTER-BRIDGE.md)) suele tratarse como limpieza cosmética: auditorías WCAG, correcciones de copy, residuo diferido-pero-no-bloqueante. La señal empírica de la implementación de referencia es más fuerte — cuando el polish Charter es el **primer lugar** donde el binario end-to-end se ejerce contra el runbook operativo documentado, aflora una clase de regresiones latentes que los harnesses de tests de integración con adapters mockeados (p.ej. `humatest`, `gomock`, buses de eventos en memoria) sistemáticamente evaden.

Adopta este patrón (trata el polish Charter como una compuerta load-bearing de detección de deuda, no como limpieza cosmética) cuando se cumpla **cualquiera** de estas condiciones:

- La Etapa entregó ≥3 handlers cuyos tests de integración usan un adapter mockeado que evade el camino de boot de la app compuesta (el router real, la cadena de middleware real, el inventario real de env-vars).
- La Etapa introdujo ≥1 artefacto de superficie cuyo sitio de declaración y sitio de cableado viven en **archivos distintos o módulos distintos** (p.ej. un instrumento métrico declarado en `metrics/` y registrado en `handlers/`; una env var documentada en un quickstart y leída en un fake adapter; un embed HTML que referencia una ruta registrada en otro lado).
- El runbook operativo (`quickstart.md` §boot, §smoke, §verification) documenta comandos que nunca han sido ejecutados end-to-end contra el binario construido desde el HEAD de la Etapa.
- La Etapa cierra una Fase `Polish` de SpecKit en la que Fases anteriores difirieron trabajo vía anotaciones como `(T103 polish)`.

Por debajo de esos umbrales (Etapa sin tests de integración con mock adapter, sin separación cross-module declaración/cableado, runbook ejercido continuamente en CI), las suites de tests per-Charter por sí solas suelen ser suficientes — adoptar este patrón antes de necesitarlo agrega overhead de polish Charter sin retorno.

---

## Forma

### El anti-patrón nombrado: Declaración de superficie sin cableado

El deliverable central del patrón es un **anti-patrón nombrado** que los polish Charters afloran consistentemente y que es reducible a una verificación mecánica:

> **Declaración de superficie sin cableado** — cuando una parte del contrato de una feature (docs, API pública, registro de métrica, cuerpo de plantilla HTML, marcador de ruta pública) anuncia un comportamiento que otra parte (consumidor de env-var, invocación de handler, llamada de registro de instrumento, registrador de ruta, lista de prefijos) se suponía debía implementar pero no lo hizo. El sitio de declaración y el sitio de cableado viven lejos entre sí en el codebase. Ni el tooling ni el proceso de review correlacionan ambos. CI prueba cada lado en aislamiento y pasa en verde.

El polish Charter es el **vehículo de descubrimiento**: es el método más barato para aflorar esta clase de regresión porque ejerce el runbook documentado end-to-end contra el binario, no contra un harness de tests que ha sido cableado directamente al sitio de declaración.

### Cuatro subclases generalizadas

El anti-patrón se presenta en al menos cuatro subclases. Cada una mapea a una verificación mecánica que puede codificarse en CI (per-proyecto hoy; candidata a tooling CLI cross-proyecto en una futura v1 — ver [Preguntas abiertas](#preguntas-abiertas)). La lista es intencionalmente agnóstica de lenguaje y runtime; las instanciaciones concretas varían por stack:

| # | Sitio de declaración | Sitio de cableado | Verificación mecánica |
|---|---|---|---|
| 1 | Env var documentada en runbook operativo (`quickstart.md`, `deploy/README.md`) | Consumidor de env-var en código de aplicación (`os.Getenv`, `process.env`, `ENV[]`) | Cada env var documentada tiene al menos un sitio consumidor |
| 2 | Instrumento métrico / símbolo de observabilidad declarado en un paquete de métricas | Sitio de llamada de registro / incremento en código de handler o worker | Cada instrumento declarado tiene al menos un sitio de llamada de registro |
| 3 | URL referenciada desde HTML renderizado o plantilla embebida (`<script src="/...">`, `<link href="/...">`) | Ruta registrada con la misma superficie de API | Cada `src=`/`href=` en HTML servido resuelve a una ruta registrada |
| 4 | Ruta marcada pública-por-contrato (doc-comment del handler, marcador dedicado) | Entrada en la lista de prefijos públicos / paths públicos del middleware de auth | Cada handler público-por-contrato tiene una entrada de prefijo equivalente |
| 5 | Método proxy IPC/RPC declarado client-side (proxy D-Bus, stub gRPC, cliente REST) — **especialmente uno reintroducido tras una mitigación que eliminó el método del servidor** | Interfaz del servidor / daemon que realmente implementa el método | Cada método proxy declarado resuelve a un método de interfaz implementado; un cambio de API cross-component debe actualizar **todos** los consumidores |

El one-liner unificador a través de las subclases es:

> **Cada artefacto de superficie declarado tiene al menos un sitio de cableado alcanzable desde un request real.**

Los adopters que extiendan la lista (nuevos pares declaración↔cableado que la implementación de referencia aún no haya aflorado) están invitados a contribuir subclases adicionales vía issue o PR.

### Subclase 5 nombrada: regresión de mitigación entregada vía un consumidor downstream no actualizado

LNXDrive afloró la subclase 5 como una *regresión de una mitigación ya entregada, a través de una frontera de componentes* — un datapoint más afilado que un gap nuevo. El productor (un daemon D-Bus) había cerrado un riesgo de seguridad eliminando un método portador de tokens y entregando un reemplazo token-safe. Un componente separado (un cliente GTK de preferencias, compilado vía un build system distinto) **seguía llamando al método eliminado** y obteniendo tokens client-side — exactamente el comportamiento que la mitigación había eliminado.

Dos factores que se componen lo hicieron invisible a cada backstop existente:

- **Ceguera cross-frontera.** Productor y consumidor viven en crates distintas, construidas por toolchains distintas (Cargo vs Meson), unidas solo en runtime sobre el bus. Los proxies zbus/D-Bus se validan en *runtime*, no en tiempo de compilación — así que los propios tests del daemon pasaron, el cliente compiló limpio, y ninguna suite de tests abarcaba el contrato.
- **Código muerto tras feature-gate.** La llamada obsoleta vivía tras un `#[cfg(feature = "goa")]` cuya feature `Cargo.toml` nunca definió. Compilaba *fuera* por completo — código muerto que derrota tanto a CI como a la revisión, ya que ninguno ejerce una feature indefinida. Activar la feature por primera vez incluso afloró un error de tipo latente que nunca había compilado: prueba concreta de que el camino nunca estuvo cableado.

La señal legible que lo atrapó fue la **verificación de contrato ex-ante** del polish/auditoría — un diff de los métodos proxy declarados del cliente contra la interfaz implementada del daemon. Esto generaliza la verificación mecánica de "cada artefacto de superficie declarado tiene un sitio de cableado" a su corolario cross-component: **un cambio de API del lado productor debe actualizar, o al menos contemplar, cada consumidor declarado de esa API.** La disciplina de Charter que operacionaliza esto vive en [la guía de la plantilla](#relacionado) (#209.c): una mitigación que toca una API cross-component lista *todos* los consumidores en `## Archivos a modificar`, para que un cambio del productor no pueda huérfanar silenciosamente a un consumidor.

### Por qué los tests de integración las omiten

La causa común a través de las cuatro subclases es que el harness estándar de tests de integración monta handlers directamente vía la API de testing (`humatest.NewTestAdapter`, equivalente en otros stacks), evadiendo el paso de composición donde declaración y cableado se unen. El handler bajo prueba está cableado correctamente *por el fixture de test*; la composición de producción es lo que está roto. La señal verde de CI refleja "el handler se comporta correctamente dado un request" — no "el request puede alcanzar al handler" ni "el artefacto declarado es alcanzable desde producción".

El smoke manual del polish Charter (`./binary && curl <recipe-documentada>`) reintroduce el paso de composición, y aflora el gap en la primera instancia de subclase que toca.

---

## Walkthrough de adopción

Para un adopter que cierra una Etapa por primera vez usando este patrón:

1. **Declara un polish Charter** scoped explícitamente a (a) ejecutar el runbook operativo documentado end-to-end contra `./binary` construido desde el HEAD de la Etapa, y (b) verificar cada una de las cuatro subclases anteriores contra los artefactos que la Etapa introdujo. Presupuesta el Charter como **L** (no XS/S/M) — la evidencia empírica de la implementación de referencia es ~10 gaps aflorados por sesión de polish de primera vez.
2. **Espera Charters emergentes de follow-on**, no scope creep residual. Cada gap que el polish Charter aflora obtiene un Charter dedicado de follow-on (p.ej. fix de boot del server, fix de middleware de auth, implementación de fake provider, cableado de llamada de registro de instrumento). El polish Charter no los absorbe — los triagea.
3. **Actualiza el runbook operativo atómicamente** con gaps de documentación (env vars faltantes en §boot, formas de smoke que no coinciden con la implementación, comportamientos reclamados-pero-ausentes en fake adapters). El runbook es la especificación de test; si está mal, tanto el binario como los docs pierden alineación.
4. **Al cierre de Etapa, presenta una retrospectiva** ([`AIDEC`](../../../docs/contributors/WHAT-IS-A-CHARTER.md) o equivalente) que clasifica los gaps aflorados por causa raíz: deterioro ambiental de dependencias, drift de documentación, o "declaración de superficie sin cableado". El corte más limpio es load-bearing para predecir cuáles guards de CI (si alguno) habrían atrapado cada clase en tiempo de PR.
5. **Opcionalmente aterriza guards de CI** para las subclases más prevalentes en la Etapa. La implementación de referencia aterrizó tres: un test de boot de cadena completa (atrapa subclases 3+4 de la variedad runtime), un analizador declared-vs-wired (subclases 1+2 estáticamente; 3+4 dinámicamente), y un test de smoke del runbook operativo (atrapa drift del runbook). El analizador es el más portable; el test de boot es project-specific en forma.

Para un adopter en Etapas subsiguientes: mismo flujo, con la predicción de que la cuenta de gaps por polish Charter cae conforme los guards de CI locales del proyecto maduran y conforme los ingenieros internalizan las cuatro subclases.

---

## Implementación de referencia

`StrangeDaysTech/sentinel` CHARTER-19 (polish Charter, mayo 2026) → CHARTER-27 (guards de CI post-AIDEC):

- La sesión del polish Charter afloró **10 gaps latentes distintos** en ~6 horas, generando 5 Charters de follow-on (CHARTERs 20/21/22/23/24) más 3 follow-ups diferidos. Dos de los gaps eran features que habían sido entregadas a producción y nunca funcionalmente funcionaron (US3 Preference Center con 401-loop durante 10 días; 7 instrumentos OTel declarados y nunca registrados durante 10 días).
- La retrospectiva de causa raíz es [AIDEC-2026-05-22-001](https://github.com/StrangeDaysTech/sentinel/pull/93) ("adopt polish-Charter-as-debt-detection pattern + 3 preparatory CI guards for Etapa 3"). Clasifica los 10 gaps por categoría (deterioro ambiental, drift de documentación, declaración de superficie sin cableado) y commitea a Sentinel a aterrizar tres guards de CI antes de abrir la siguiente Etapa.
- Los tres guards de CI aterrizaron como [sentinel#94](https://github.com/StrangeDaysTech/sentinel/pull/94) (CHARTERs 25/26/27): test de boot de cadena completa, analizador multipass declared-vs-wired (subclase 2 totalmente cableada; subclases 1+3+4 stubbed para follow-up), test de smoke del runbook operativo.

La implementación de referencia incluye una predicción falsable: el polish Charter de la siguiente Etapa aflorará ~80% menos gaps. La validación de esa predicción (o las nuevas categorías de gap aflorados si la predicción falla) es el trigger empírico natural para revisitar la graduación `v0 → v1` de este patrón.

La discusión RFC originante es [straymark#199](https://github.com/StrangeDaysTech/straymark/issues/199), preservando la cadena empírica a través de cinco actualizaciones de comentario conforme la sesión del polish Charter se desplegó.

---

## Preguntas abiertas

Estas no están resueltas en v0. Revisiones futuras de este patrón, o un helper CLI, pueden abordarlas:

- **Cristalización como subcomando CLI `straymark analyze declared-vs-wired`** — *compuerta N=2 cruzada; alcance v0 resuelto.* Con LNXDrive validando el patrón en un segundo dominio, el framework entrega un v0 de **set-difference dirigido por config**: el operador provee un glob+regex del lado declarado y un glob+regex del lado cableado (el grupo de captura del regex es el nombre del símbolo), y el comando reporta los símbolos declarados-pero-no-cableados (`D \ W`). Esto es mecánicamente tratable en *cualquier* stack precisamente porque el conocimiento específico del stack vive en los regexes del adopter, no en el CLI — y atrapa directamente la subclase 5 (nombres de método del proxy D-Bus client-side vs nombres de método de interfaz server-side). **Diferido a una revisión posterior:** variantes basadas en AST de las subclases 1–4 (docs de env-var, instrumentos métricos, embeds HTML, marcadores de ruta pública), que necesitan parsers por stack; y las verificaciones runtime/dinámicas (boot de cadena completa, resolución de rutas), que son inherentemente project-local.
- **Completitud de enumeración de subclases**. Las cuatro subclases fueron las que la implementación de referencia afloró. Candidatas adicionales: columna de base de datos declarada en una migración pero nunca leída/escrita por código de aplicación; feature flag declarado pero nunca verificado; campo protobuf definido pero nunca serializado. Cada subclase adicional necesita al menos un afloramiento empírico de un adopter para entrar al canon.
- **Integración con `straymark charter close --polish-checklist`**. Un subcomando polish-específico podría aflorar el checklist canónico (ejecuta runbook end-to-end; verifica que cada artefacto declarado tenga un sitio de cableado; verifica que el inventario de env-var coincida con los requisitos reales del binario; verifica que las herramientas CLI referenciadas en el runbook existan). Compuerta: después de que el subcomando CLI `declared-vs-wired` aterrice, ya que el último ítem del checklist lo invocaría.
- **Guías de instanciación por stack**. Las cuatro subclases son agnósticas de lenguaje; la forma concreta de verificación (`analysis.Pass` de Go, walker de AST de TypeScript, módulo `ast` de Python, etc.) no lo es. Una revisión futura del patrón puede alojar implementaciones de referencia por stack como docs hermanas.
- **Calibración del presupuesto de esfuerzo**. La implementación de referencia observó ~10 gaps por polish Charter de primera vez. La predicción es que esto cae fuertemente conforme los guards locales del proyecto maduran. Una v1 de este patrón puede publicar guía de presupuesto derivada de datapoints N≥2 (XS/S/M/L por madurez de Etapa).

---

## Créditos

Originado vía [issue #199](https://github.com/StrangeDaysTech/straymark/issues/199) por el adopter Sentinel (N=1). Base empírica: cadena CHARTER-19 → CHARTER-27 en `StrangeDaysTech/sentinel`, retrospectiva [AIDEC-2026-05-22-001](https://github.com/StrangeDaysTech/sentinel/pull/93).

Cristalizado a **v1 (N=2)** vía [hallazgo #209](https://github.com/StrangeDaysTech/straymark/issues/209) por el adopter LNXDrive (escritorio Rust, segundo dominio independiente), que contribuyó la subclase 5 (regresión de mitigación entregada vía un consumidor downstream no actualizado) y disparó el subcomando `analyze declared-vs-wired`. El hallazgo compañero [#210](https://github.com/StrangeDaysTech/straymark/issues/210) agregó la disciplina de reconocimiento en `charter new` y la regla de validación `CHARTER-FILES-EXIST`. Autor: José Villaseñor Montfort.

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude 4.7); toda responsabilidad por el contenido recae en el autor humano.*

---

## Relacionado

- [SPECKIT-CHARTER-BRIDGE.md](SPECKIT-CHARTER-BRIDGE.md) — define la Fase `Polish` de SpecKit a la que este patrón le adjunta semántica load-bearing.
- [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) — patrón v0 hermano validado en el mismo adopter; comparte la compuerta de graduación N=1 → N=2 para cristalización CLI.
- [EMERGENT-OBSERVATION-DESIGN.md](EMERGENT-OBSERVATION-DESIGN.md) — meta-patrón que el rol de detección de deuda del polish Charter instancia en la superficie de cierre de Etapa.
- [AGENT-RULES.md](AGENT-RULES.md) — directivas agent-side que gobiernan cómo las superficies de follow-up (`R<N> (new, not in Charter)`, promoción a TDE) fluyen desde hallazgos del polish Charter al backlog de gobernanza más amplio.

---

*StrayMark fw-4.20.0 | [Strange Days Tech](https://strangedays.tech)*
