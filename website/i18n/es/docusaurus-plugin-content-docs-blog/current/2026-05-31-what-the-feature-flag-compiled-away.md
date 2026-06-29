---
slug: what-the-feature-flag-compiled-away
title: Lo que el feature flag compiló fuera
authors:
  - jose
tags:
  - straymark
  - charters
  - governance
  - polish-charter
  - anti-patron
  - adopters
  - lnxdrive
date: 2026-05-31T00:00:00.000Z
description: Un segundo adopter, en otro lenguaje, validó el anti-patrón "declaración de superficie sin cableado" — y el helper de CLI que diferimos a propósito por fin shippeó
---
*Una semana después de nombrar un anti-patrón y declinar deliberadamente construir tooling para él, un segundo proyecto — en otro lenguaje, sobre otro stack — lo afloró de nuevo, en una forma más afilada: no un gap nuevo, sino la regresión de una mitigación de seguridad ya entregada, escondida en código muerto tras un feature flag que nunca se definió. Esa fue la señal N=2 que el framework decía estar esperando. El helper diferido shippeó.*

<!-- truncate -->

> *"Compiló limpio — el módulo GOA era código muerto, y los proxies de zbus se validan en runtime, no en tiempo de compilación, así que nada lo marcó. Activar la feature por primera vez incluso afloró un error de tipo latente que nunca había compilado."*

Esa frase viene del Issue [#209](https://github.com/StrangeDaysTech/straymark/issues/209), presentado el 31 de mayo por [LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) — un daemon de sincronización en la nube para Linux y un cliente de escritorio GTK escritos en Rust, FUSE y D-Bus y systemd hasta el fondo. Es el segundo proyecto en el [registro de adopters de StrayMark](https://github.com/StrangeDaysTech/straymark/blob/main/ADOPTERS.md), y el primero en un dominio genuinamente distinto al backend Go de Sentinel. Esa diferencia es el punto entero de este post.

Ocho días antes, en [*Lo que el binario no pudo esconder*](what-the-binary-couldnt-hide), documentamos un anti-patrón — **declaración de superficie sin cableado** — que un polish Charter en Sentinel afloró diez veces en seis horas. Lo nombramos, shippeamos el doc del patrón como `fw-4.18.0`, y **diferimos el helper de CLI a propósito**, con la razón dicha sin rodeos: Sentinel es N=1. Las cuatro subclases eran las que un adopter, un stack había aflorado. El tooling construido sobre los modos de fallo de un solo dominio tiende a osificar esas decisiones en defaults del framework que no generalizan. Las `## Open questions` del doc del patrón fijaron la compuerta en una línea: *"Cristalización como subcomando CLI `straymark analyze declared-vs-wired` … Compuerta: N=2 adopters."*

Este post es lo que pasó cuando llegó N=2 — y por qué la ocurrencia fue un dato más fuerte que la docena que nombró el patrón.

## La regresión que se compiló a sí misma fuera

El Charter `CHARTER-01-road-to-v0-1-0-alpha-1` de LNXDrive tenía una Fase 1 que cerró un riesgo de seguridad, `RISK-002`. Vale la pena enunciar la forma del fix con precisión, porque la regresión es su imagen espejo.

El flujo de autenticación originalmente tenía al daemon exponiendo un método D-Bus, `Auth.CompleteAuthWithTokens`, que cargaba tokens OAuth a través del bus desde el cliente. Tokens en el bus es el riesgo. La Fase 1 lo cerró de la forma correcta: **eliminó** ese método y shippeó `Auth.CompleteAuthViaGOA(account_path)` en su lugar — ahora el daemon obtiene los tokens él mismo desde GNOME Online Accounts, daemon-side, y nunca cruzan el bus. La mitigación shippeó. Los tests del daemon pasaron. `RISK-002` quedó cerrado.

Luego la Fase 3 auditó el panel de preferencias GTK4 — un crate separado, compilado vía Meson en vez del build Cargo del daemon. El panel **seguía llamando a `complete_auth_with_tokens`**, y seguía obteniendo tokens client-side. El consumidor nunca había sido actualizado cuando el productor cambió su contrato. La mitigación había regresado a través de una frontera de componentes, en la mitad del sistema que el trabajo de Fase 1 nunca tocó.

Aquí está por qué se mantuvo invisible en el hueco entre Fase 1 y Fase 3 — y por qué ningún test, ningún compilador, ni ningún revisor lo cazó:

- **La llamada muerta estaba tras un feature flag que no existía.** El camino de código GOA del cliente vivía bajo `#[cfg(feature = "goa")]`. `Cargo.toml` nunca definió una feature `goa`. Así que el módulo entero compilaba *fuera* — era código muerto. El crate compilaba en verde porque el camino roto nunca se construía en absoluto. CI no ejerce una feature indefinida; tampoco la revisión de código, que lee la línea y sigue. Cuando el trabajo del panel en Fase 3 por fin activó la feature por primera vez, afloró un **error de tipo latente que nunca había compilado** — la prueba concreta e infalsificable de que el camino nunca estuvo cableado ni una vez.
- **La frontera era triple.** Productor y consumidor viven en crates distintas, construidas por toolchains distintas (Cargo para el daemon, Meson para el panel), unidas solo en runtime sobre D-Bus. Y los proxies de zbus se validan en *runtime*, no en tiempo de compilación — así que incluso con la feature encendida, el proxy declarando `complete_auth_with_tokens` habría compilado contra una interfaz del daemon que ya no lo ofrecía, y solo habría fallado cuando una llamada real golpeara un bus real.

Quita el Rust y los detalles de D-Bus y tienes la misma equivocación mecánica que nombró el post de Sentinel: un artefacto declarado en un lugar (el método proxy del cliente), la implementación que debería respaldarlo viviendo en otro lugar (la interfaz del daemon), y nada — ni el compilador, ni CI, ni la revisión — correlacionando los dos. El sitio de declaración y el sitio de cableado viven lejos entre sí, y el hueco entre ellos es donde la regresión se escondió.

## Por qué esto fue más afilado que los diez gaps que nombraron el patrón

Los casos de Sentinel eran gaps *nuevos*: un Preference Center que hacía 401-loop durante diez días, siete instrumentos OTel declarados y nunca registrados. Features que shippearon y nunca funcionaron funcionalmente. Suficientemente malo.

El caso de LNXDrive es cualitativamente peor de una forma que importa para el canon del patrón. Es una **regresión de una mitigación ya entregada, ya auditada.** `RISK-002` se cerró correctamente. El fix era real. Y silenciosamente derivó de vuelta fuera de cumplimiento — no porque alguien reintrodujera el método riesgoso, sino porque el *consumidor* de la API nunca se actualizó cuando el *productor* la cambió. Lo que hizo legible la regresión no fue un test. Fue la **verificación de contrato ex-ante** de la auditoría: un diff de los métodos proxy que el cliente declara contra la interfaz que el daemon realmente implementa.

Esto se gana la subclase 5 en el doc del patrón, y un nombre propio: **regresión de mitigación entregada vía un consumidor downstream no actualizado.** Las cuatro subclases originales eran variaciones de "una superficie declarada nunca se cableó". La quinta agrega un filo más cortante: "una superficie declarada *sí* estaba cableada, el cableado se eliminó como un fix, y un componente distinto siguió llamando al contrato viejo".

Una palabra sobre el conteo, porque StrayMark cristaliza patrones por conteo de validación y vale la pena ser honesto con la aritmética. Hay dos ejes, y el doc del patrón ahora los reporta por separado para que no se confundan:

- **Dominios independientes: 2.** Sentinel (backend Go) y LNXDrive (escritorio Rust). Una app de escritorio en Rust validando un patrón visto primero en un backend Go es la señal cross-domain fuerte — mucho más fuerte de lo que habría sido otro backend Go. Este es el eje que abre la automatización en el CLI.
- **Ocurrencias: 3.** El afloramiento original de Sentinel, más un drift de documentación temprano de Fase 1 dentro de LNXDrive, más esta regresión cross-component.

La compuerta es el eje de dominio, y ahora está cruzada.

## La compuerta que dijimos que estábamos esperando

En el post de Sentinel escribí que la reacción natural ante un hallazgo vívido es sobre-construir la respuesta, y que la disciplina es lo opuesto: nombrar el meta, diferir el tool hasta que al menos dos adopters hayan aflorado su forma. También señalé el riesgo específico — *"Un helper de CLI compromete al framework a un runtime que refleja los modos de fallo de un stack específico."*

Ese riesgo es exactamente lo que el segundo dominio permite esquivar. El helper que shippeó — `straymark analyze declared-vs-wired`, en [`cli-3.18.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.18.0) — no codifica Go, ni Rust, ni D-Bus, ni HTTP. Es un **set-difference dirigido por config.** Le das un lado *declarado* y un lado *cableado*, cada uno como un par `(glob, regex)`, y el grupo de captura de cada regex es el nombre del símbolo. Reporta los símbolos que aparecen del lado declarado y no del lado cableado:

```bash
straymark analyze declared-vs-wired \
  --declared-glob "client/**/*.rs"     --declared-pattern 'fn (\w+)' \
  --wired-glob    "daemon/**/*.rs"      --wired-pattern   'fn (\w+)'
```

Corrido contra el layout de LNXDrive, eso marca `complete_auth_with_tokens` — declarado en el proxy del cliente, ausente de la interfaz del daemon — y sale con código distinto de cero. El conocimiento específico del stack vive enteramente en los dos regexes del adopter, commiteados una vez como un perfil con nombre en `.straymark/config.yml`. El framework provee la maquinaria de set-difference; el adopter provee el significado de "declarado" y "cableado" para su stack. Ese es el diseño que N=1 no podría haber producido: con un dominio, no puedes distinguir qué partes de tu tool son esenciales y cuáles son accidentales a ese stack. El segundo dominio es lo que las separa.

Este es el argumento entero a favor de la compuerta, observado en la práctica en vez de afirmado. Si hubiéramos shippeado el helper en N=1 — digamos, un analizador que caminara ASTs de Go buscando instrumentos OTel declarados-pero-no-registrados — habría sido inútil para un proyecto Rust hablando sobre D-Bus, y habríamos gastado un ciclo de release generalizando algo que construimos demasiado pronto. Esperar nos costó ocho días y nos compró un tool cuya superficie v0 es honestamente cross-stack.

## El hallazgo compañero, y el backstop más barato

LNXDrive presentó un segundo issue el mismo día, [#210](https://github.com/StrangeDaysTech/straymark/issues/210), y es el más callado pero más ampliamente útil de los dos. Observó que la sección `## Archivos a modificar` del Charter había sido escrita contra código *asumido*, no código *leído* — y estaba mal repetidamente. `RISK-002` declaró un nuevo `dbus_iface.rs` y un tipo opaco `SessionHandle`; ninguno existía, y el fix shippeó en el ya presente `service.rs`. `ISSUE-002` nombró `lnxdrive-config/src/parser.rs`; no existe tal crate, y el parser real es `lnxdrive-core/src/config.rs`. Un ítem de endurecimiento de CI apuntaba a un engine cuyo workflow vivía en un path de subdirectorio que GitHub Actions ignora silenciosamente — nunca había corrido. Cada fila se volvió una "corrección de premisa" documentada durante la ejecución.

Eso es el framework funcionando como backstop: la declaración ex-ante es lo que hizo *legible* el drift posterior. Pero la causa raíz está upstream del drift — es un error de autoría, una ruta que nunca existió, no una implementación que divergió. Esos son fallos distintos y confundirlos agrega ruido. Así que `cli-3.17.0` agregó una regla de validación, `CHARTER-FILES-EXIST`: cuando una fila de `## Archivos a modificar` nombra una ruta que no está en disco y no está tagueada como recién creada, `straymark validate --include-charters` avisa. Es solo-aviso, pure-Rust (así funciona sin el bash que el chequeo de drift necesita), y — este es el punto que pidió #210 — vive en un *comando distinto* a `charter drift`. Una ruta que nunca existió es una mala declaración del Charter que se arregla en el lugar; una ruta declarada y no modificada es drift de implementación que se reconcilia. Dos fallos, dos códigos de regla, dos comandos. La plantilla de Charter ahora también le dice a los autores, en el comentario sobre `## Archivos a modificar`, que lean cada ruta antes de declararla — y, cuando un Charter toca una API cross-component, que listen *todos* los consumidores, no solo el productor. Esa última línea es la disciplina que habría cazado la regresión GOA en tiempo de autoría, antes de escribir cualquier código.

## Lo que aún deliberadamente no hicimos

La misma contención que la vez pasada, en un lugar distinto. `analyze declared-vs-wired` shippea **solo** la subclase 5 — el chequeo proxy-vs-interfaz, el que LNXDrive realmente afloró y el que es mecánicamente tratable a través de stacks sin más que dos regexes. Las variantes basadas en AST de las subclases 1 a 4 — caminar código buscando consumidores de env-var, sitios de registro de métricas, resolución de rutas HTML, prefijos de ruta pública — siguen diferidas, y el doc del patrón aún las lista bajo `## Open questions`. Necesitan parsers por stack, y un set-difference sobre capturas de regex no es eso. Los chequeos dinámicos — bootear el binario, resolver rutas en runtime — siguen siendo inherentemente project-local. Cruzamos la compuerta que el chequeo dirigido-por-config necesitaba; no usamos el cruce como licencia para construir todo lo que el patrón podría eventualmente justificar.

Hay una deferral más pequeña que vale la pena nombrar también. Consideramos un campo de frontmatter duro `recon: confirmed` en los Charters — una casilla que el autor marca para afirmar que leyó el árbol antes de declararlo. No lo shippeamos. Un campo requerido rompe la creación de Charters no-interactiva y dirigida-por-skill, y crea una superficie de mentira: los agentes lo marcarán reflexivamente. El nudge suave en la salida de `charter new` más el backstop mecánico `CHARTER-FILES-EXIST` hace el mismo trabajo sin el modo de fallo. Si la versión suave resulta insuficiente a través de más adopters, esa es la señal para revisitar — la misma lógica de compuerta, un nivel más abajo.

## Si has leído hasta aquí

El ejercicio portable del post anterior sigue en pie, y el segundo dominio lo extiende. Si corres un sistema partido entre un productor y un consumidor unidos en runtime — un daemon y un cliente, un servicio y un SDK, un servidor y un stub generado — anota los métodos que el consumidor *declara* que llamará, y los métodos que el productor *realmente implementa*, y diffea los dos conjuntos. No necesitas StrayMark para hacerlo; `grep` y `comm` te llevan casi todo el camino. El set-difference en una dirección es la regresión de la que trata este post. El set-difference en la otra dirección es superficie muerta que puedes borrar. De cualquier forma el número es información que no tenías.

Y si operas sobre un stack que aún no hemos visto — y en N=2, eso sigue siendo la mayoría de los stacks — el camino de "observación interesante en mi proyecto" a "nombrado en el canon de StrayMark" corre por el mismo canal por el que pasaron #199 y #209: abre un issue. La subclase 6 está ahí afuera en algún codebase ahora mismo, declarada en un archivo y cableada en otro, esperando al adopter que será quien la aflore.

---

*StrayMark `fw-4.20.0` + `cli-3.17.0` (Release A) y `cli-3.18.0` (Release B) — Issues [#209](https://github.com/StrangeDaysTech/straymark/issues/209) · [#210](https://github.com/StrangeDaysTech/straymark/issues/210) · PRs [#211](https://github.com/StrangeDaysTech/straymark/pull/211) · [#212](https://github.com/StrangeDaysTech/straymark/pull/212). Patrón: [`POLISH-CHARTER-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md) (v1, N=2). Predecesor: [*Lo que el binario no pudo esconder*](what-the-binary-couldnt-hide).*

*Este documento fue producido con asistencia de herramientas de IA generativa (Claude Opus 4.8); toda responsabilidad por el contenido recae en el autor humano.*
