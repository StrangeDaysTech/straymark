---
slug: the-grep-that-was-read-backwards
title: El grep que se leyó al revés
authors:
  - jose
tags:
  - straymark
  - audit
  - governance
  - verification
  - multi-model
draft: false
date: 2026-07-29T00:00:00.000Z
description: Un ciclo de auditoría con cuatro modelos revisó un Charter que añadía un método público nuevo — el único método que hacía funcionar una feature. Tres de cuatro auditores reportaron que no había hallazgos críticos ni altos. La feature tenía cero llamadores en producción, 3.376 tests en verde, y no funcionaba de punta a punta. El defecto era detectable mecánicamente con un solo grep; un auditor corrió exactamente ese grep y leyó el resultado al revés. El post-mortem produjo una regla nueva en el prompt de auditoría — y un hallazgo más incómodo sobre la verificación del propio implementador, que no podía dar rojo.
---

*Un ciclo de auditoría con cuatro modelos no vio un defecto que dejaba la feature auditada completamente inservible. Tres de cuatro auditores reportaron "sin hallazgos críticos ni altos". El que lo encontró lo calificó como Medio. El defecto era detectable mecánicamente con un solo grep — un auditor corrió exactamente ese grep, obtuvo la salida correcta, y la leyó al revés. La feature tenía 3.376 tests en verde y no funcionaba de punta a punta, porque el Charter añadió un método público nuevo y nada lo llamaba. Cuando le preguntamos al auditor más minucioso qué había pasado, su respuesta valió más que el hallazgo: un modo de fallo con nombre, reproducible, compartido por tres de los cuatro auditores. Este post va de ese modo de fallo, del fix de una sola regla que produjo, y del descubrimiento de segundo orden — que el método de verificación del propio implementador era estructuralmente incapaz de dar rojo, una verificación que sumaba pases y nunca miraba los fallos, confiada siete veces seguidas.*

<!-- truncate -->

## El caso

El proyecto es un adopter privado — mantendremos los identificadores genéricos. Un Charter implementó una feature de elevación break-glass: un operador con permisos elevados temporales debería poder abrir expedientes que su rol base normalmente no alcanza. El Charter anterior había construido la maquinaria de resolución de elevación y la había dejado sin usar. La sección de contexto de este Charter declaraba su propósito sin rodeos: *"el tipo de elevación ya está escrito y testeado, y hoy nadie lo construye. Este Charter lo pone en uso."*

El Charter añadió un método resolver asíncrono — el único método del codebase que consulta el estado de elevación — a la interfaz del resolver. Se escribieron tests. Pasaron 3.376 tests. Corrió la auditoría.

Nada llama al método nuevo. Los ocho consumidores siguen llamando al resolver síncrono, que resuelve solo por rol y devuelve *ninguno* para el rol de operador al que la feature existe para servir. Un operador con una elevación activa no puede abrir un solo expediente. La feature tiene 3.376 tests en verde y no funciona.

Dos greps cuentan la historia entera:

```
$ grep -rn "resolver.Resolve\(" src/
Page1.razor:124    Page2.razor:66     Page3.razor:86
Page4.razor:120    Page5.razor:223    Page6.razor:347
Modal.razor:159    Controller.cs:311

$ grep -rn "ResolveAsync" src/
(sin resultados)
```

Ocho sitios de llamada, todos enrutando por el camino viejo. Cero por el nuevo. El método asíncrono existe, es correcto, tiene tests — y es inalcanzable.

## Qué hicieron los cuatro auditores

| Auditor | Hallazgos | ¿Lo encontró? |
|---|---|---|
| gpt-5-6-terra-high | 2 | sí — calificado Medio (subcalibrado) |
| kimi-k3 | 15 | no |
| glm-5.2 | 8 | no — escribió explícitamente "sin hallazgos críticos ni altos" |
| qwen3-8-max-preview | 3 | no |

Tres de cuatro auditores detectaron *síntomas* y no llegaron a la causa:

- Un test guard de paridad de DI fallando de forma determinista — es decir, el fixture E2E no podía arrancar la feature. Encontrado por dos auditores. Reportado como defecto de configuración.
- Un hueco de cobertura: el test de expiración verificaba a nivel de servicio, no al nivel HTTP/cookie que el Charter declaraba. Encontrado por uno.
- La *forma* exacta del defecto — "el llamador decide el valor de autorización y la capa de abajo lo acepta" — identificada con precisión, en otro archivo. Encontrado por uno.

Todo síntoma era real. Todo síntoma era un hilo que llevaba a la misma raíz. Nadie tiró.

## El post-mortem que importó más que el hallazgo

Le preguntamos a kimi-k3 — el auditor con el reporte más minucioso y más citas de evidencia — qué había pasado. Su respuesta es el artefacto más útil de todo el ciclo, porque el fallo es reproducible y tiene nombre.

**1. Corrió la búsqueda correcta y leyó el resultado al revés.**

> *"Corrí un grep cuya salida mostraba solo líneas de inyección — la ausencia de la llamada asíncrona era visible ahí, y lo leí como 'inyección confirmada' en vez de 'sin llamadas, confirmado'. Grepé por la presencia del cableado y leí el resultado como confirmatorio cuando era desconfirmatorio."*

No falló nada salvo la lectura de un resultado negativo. El grep devolvió líneas de sitios de inyección (el cableado del contenedor de DI) y el auditor las leyó como prueba de que el método se *usaba*, cuando solo probaban que estaba *disponible*. Presencia de cableado ≠ presencia de llamadas.

**2. Verificó el mecanismo en vez de la adopción.**

> *"Verifiqué el mecanismo (existe, es correcto) en vez de la adopción (se usa)."*

Había leído el cuerpo del método — reportó un hallazgo sobre una caché faltante *dentro del nuevo método asíncrono*. La pregunta contigua — ¿quién llama a esto? — nunca se hizo. Este es el modo de fallo en su forma más pura: el auditor estaba *dentro del método correcto*, verificó su corrección, y nunca levantó la vista para ver si algo llegaba hasta ahí.

**3. Trató el contexto del Charter anterior como narrativa, no como afirmaciones falsables.**

> *"El prompt tiene 10.598 líneas y absorbí el material de los AILOGs originales como narrativa de fondo, sin convertir cada uno en una hipótesis falsable ('este Charter debe volver verdadero X — ¿dónde?'). Ese es el fallo de proceso que puedo nombrar con precisión: traté el contexto del Charter anterior como contexto, no como una lista de afirmaciones-pendientes-de-volverse-verdaderas."*

El prompt *sí* contenía la información necesaria. El Charter declara que su propio propósito es "ponerlo en uso". Verificar "puesto en uso" significa verificar que los sitios de llamada cambiaron. La afirmación estaba ahí; se leyó como trasfondo, no como aserción chequeable.

**4. Su diagnóstico estructural.**

> *"Cada capa estaba testeada y la costura entre capas no — el resolver funciona (16 tests), las páginas funcionan (bUnit con el permiso inyectado), los servicios rechazan (14 tests). Tres de cuatro auditores verificaron capas; el defecto vivía en la elección de sobrecarga dentro de archivos que no estaban en el diff."*

Esta es la parte que generaliza: auditar un diff no basta cuando un Charter afirma poner en uso código existente. Los ocho archivos con la llamada equivocada se modificaron en el Charter *anterior*.

## La convergencia mide lo visible, no lo severo

Esta es la segunda ocurrencia consecutiva de un patrón que vale la pena nombrar. En un ciclo de auditoría previo, dos de tres auditores validaron una garantía citando un test sin abrir su cuerpo — y el test estaba vacío. Aquí, tres auditores convergieron en el test guard en rojo (visible, mecánico, fácil de reportar) mientras cero de los tres encontraron el defecto que dejaba la feature inservible.

La convergencia entre auditores es la señal que el ciclo de auditoría de StrayMark está diseñado para producir. Pero la convergencia mide lo que varias mentes *pueden ver*, y lo que pueden ver está sesgado sistemáticamente hacia lo *chequeable*: tests que fallan, errores de configuración, tipos que no cuadran. El defecto que más importa — una capacidad que nada alcanza — es invisible para todo auditor que no haga la pregunta explícitamente. Hacerla no requiere juicio; requiere una regla.

## El fix: una regla, sin juicio de por medio

Le ofrecimos al auditor cuatro remedios candidatos. Eligió este sin dudar:

> *"(a), sin duda, y no es sabiduría a toro pasado: el resolver asíncrono es un método público nuevo; 'enumera sus llamadores' es un grep que ya tenía en la mano y que devuelve '0 en producción' — hallazgo Alto mecánico, sin juicio."*

La regla, ahora parte del prompt de auditoría como paso obligatorio:

> **Enumera los llamadores de los puntos de entrada públicos nuevos.** Por cada método público, endpoint o componente que el Charter AÑADE, corre una búsqueda de sitios de llamada sobre el código de producción (excluyendo tests) y declara el conteo explícitamente. Cero llamadores en producción es un hallazgo Alto por defecto — el Charter añadió una capacidad que nada alcanza. Cuando el conteo no es cero, comprueba que los llamadores sean los previstos — una sobrecarga existente o un camino legacy pueden seguir ganando.

La regla resulta atractiva precisamente porque no requiere juicio. Convierte una ausencia invisible en un conteo explícito. "Cero llamadores" es un hallazgo que se escribe solo.

Del mismo post-mortem salieron otras dos reglas secundarias:

1. **Chequeo de costura en tests consolidados.** Cuando un test se documenta como "consolidado" en otro, verifica que el reemplazo ejercite la misma costura, no meramente la misma unidad. En este ciclo, el auditor investigó exactamente esto y lo dejó caer porque las notas de cierre del Charter declaraban la cobertura equivalente. No lo era. Las notas de cierre del propio Charter apagaron una línea de indagación de un auditor — y esas notas las escribe la parte auditada. Ese es un conflicto de interés estructural que la auditoría debe corregir.

2. **Enumeración de gates en rojo.** Cuando una gate de verificación está en rojo, enumera qué solo esa gate podía haber cazado. El test de paridad de DI roto se reportó como defecto de configuración; nadie preguntó qué estaba protegiendo.

## El hallazgo de segundo orden: verificación que no puede fallar

El agente implementador declaró "3.376 tests pasando, 0 fallos" en el AILOG, en el cuerpo del PR y en siete mensajes de commit. Un test guard llevaba fallando de forma determinista desde el batch 4. El comando de verificación era:

```bash
dotnet test ... | grep -E "Passed!|Failed!" | awk -F'Passed: *' '{s+=$2} END {print s}'
```

Sumaba los conteos de pases de los cinco proyectos de test sin comprobar si alguno reportaba fallos. Un proyecto en rojo se sumaba y nunca afloraba.

Podría decirse que esta es la mitad más útil del caso. Un agente construyó un método de verificación que *no podía dar rojo*, y luego confió en él siete veces seguidas. La auto-verificación jamás habría cazado esto. La auditoría sí — eventualmente, a través de otro auditor, sobre otro hallazgo, leyendo la salida del test guard que el propio pipeline del implementador venía tragándose en silencio.

El fix aquí no está en el prompt de auditoría sino en la plantilla de AILOG — el documento que escribe el implementador. La casilla "Tests pasan" ahora exige declarar el comando exacto que se corrió, con una guía explícita: *una verificación que no puede producir un resultado negativo no es verificación.* Sumar conteos de pases sin mirar la salida de fallos es el anti-patrón canónico.

## Qué se liberó

Publicado como [`fw-4.38.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.38.0) / [`cli-3.40.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.40.0), rastreado en [#382](https://github.com/StrangeDaysTech/straymark/issues/382):

**Prompt de auditoría v1.2** (EN + ES). Tres añadidos:

- **Nuevo Paso 3 obligatorio** — enumera los llamadores de los puntos de entrada públicos nuevos. Cero llamadores en producción = Alto, sin juicio de por medio. Si no es cero: verifica que los llamadores sean los previstos.
- **Paso 2.6 reforzado** — chequeo de costura en tests consolidados: las notas de cierre del propio Charter son una afirmación de la parte auditada, no evidencia.
- **Paso 4 reforzado** — enumeración de gates en rojo: ¿qué estaba protegiendo la gate rota?

**Plantillas de AILOG + Charter** (EN/ES/zh-CN). La casilla "Tests pasan" ahora exige declarar el comando exacto que se corrió. Una verificación que no puede producir un resultado negativo no es verificación.

## La versión portátil

Hazle a tu proceso de revisión — humano o automatizado — esta pregunta: *por cada capacidad pública nueva, ¿alguien comprueba que algo la llame?* No que exista, no que sea correcta, no que tenga tests. Que algo la *llame*. La respuesta suele ser no, porque la pregunta parece demasiado simple para hacerla. Lo es. Ese es el punto. Los chequeos más simples son los que nadie escribe, y los que nadie escribe son los que se saltan todos los revisores, humanos o máquinas, en cada ciclo, para siempre — hasta que un Charter añade una feature que nada alcanza, tres auditores reportan verde, y lo único que lo caza es la suerte.

Escribe el chequeo. Vuélvelo mecánico. "Cero llamadores en producción" es un número, no un juicio. Si tu prompt de auditoría, tu checklist de code review o tu gate de CI no produce ese número para cada punto de entrada público nuevo, estás corriendo el mismo proceso que produjo 3.376 tests en verde sobre una feature que no funciona.

---

*Caso documentado en [#382](https://github.com/StrangeDaysTech/straymark/issues/382) — cuatro reportes de auditor, revisión consolidada, post-mortem del auditor y artefactos fuente completos en el directorio `.straymark/audits/` del adopter privado. Enviado en [`fw-4.38.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/fw-4.38.0) / [`cli-3.40.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.40.0). Relacionados: [Lo que un millón de aserciones no puede zanjar](what-a-million-assertions-cant-settle) (diversidad entre familias como seguro contra puntos ciegos), [Quién creyó la auditoría que era](who-the-audit-thought-it-was) (atribución de la auditoría).*

*Este documento se produjo con asistencia de herramientas de IA generativa; toda la responsabilidad del contenido recae en el autor humano.*
