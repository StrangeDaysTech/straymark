---
slug: what-the-open-format-left-to-the-producer
title: Lo que el formato abierto le dejó al productor
authors:
  - jose
tags:
  - straymark
  - okf
  - knowledge-graph
  - governance
  - loom
  - interoperability
date: 2026-06-17T00:00:00.000Z
description: Google publicó el Open Knowledge Format — markdown, frontmatter YAML, un grafo de enlaces, un agente que los mantiene. StrayMark llegó a las mismas primitivas de forma independiente. La convergencia es real; la divergencia también — OKF construye cognición para la IA, StrayMark la construye para la IA y el ingeniero humano juntos.
---
*El 2026-06-12 Google Cloud publicó el Open Knowledge Format: un directorio de archivos markdown con frontmatter YAML, enlazados entre sí formando un grafo, escritos y mantenidos por agentes de IA, con un visualizador estático y una especificación de una página. Si esa descripción te provoca un déjà vu al leer este blog, debería: es la forma que StrayMark lleva meses entregando. Nosotros llegamos ahí desde nuestras propias ideas; OKF llegó acreditando el gist de Karpathy del que desciende todo el "patrón LLM-wiki". Dos caminos, las mismas primitivas. La reacción honesta no es ponerse a la defensiva — alguien con el alcance de Google acaba de validar la apuesta. Pero validar la forma no es estar de acuerdo sobre su propósito, y ahí vive la única pregunta interesante — la que su especificación responde en una sola frase.*

<!-- truncate -->

> *"OKF exige exactamente una sola cosa de cada concepto: un campo `type`. Todo lo demás se le deja al productor."* — la especificación de OKF v0.1

Esa frase es todo el post. Porque **todo lo demás —la parte que la especificación te devuelve a ti— es donde los dos proyectos dejan de ser la misma cosa.**

## La convergencia

Reduce OKF y StrayMark a su mecánica y la lista de piezas es idéntica. Una unidad
de conocimiento es un archivo markdown con un bloque de frontmatter YAML. El
frontmatter lleva un campo obligatorio — OKF lo llama `type`, nosotros lo
llamamos el tipo de documento. Los documentos se enlazan entre sí, y esos
enlaces forman un grafo que una herramienta renderiza. El corpus lo escriben y
mantienen al día los agentes de IA. Los bundles son solo archivos: comparables
con diff, alojables en git, legibles sin herramientas. Hay un generador que
siembra el corpus recorriendo un sistema existente. Hay un visualizador de grafos.

No copiamos a OKF y OKF no nos copió — y, para ser precisos, tampoco llegamos
aquí desde Karpathy. OKF acredita su gist (a su vez un guiño al Memex de Vannevar
Bush de 1945) como su nacedero; StrayMark alcanzó las mismas primitivas desde sus
propias ideas, sin esa genealogía. Y eso es justamente lo que hace que el solape
valga la pena señalarlo: cuando dos proyectos que parten de lugares distintos
aterrizan en markdown + frontmatter + grafo + agente, las primitivas no son una
moda. Son el sustrato correcto.

Así que no vamos a discutir con la mecánica de OKF. También es la nuestra. Vamos
a señalar el lugar donde piezas idénticas se ensamblan en dos máquinas que hacen
trabajos opuestos.

## Dos lectores, dos paradigmas

Este es el corazón del asunto, y no es una carencia de funcionalidad.

Hazle a cualquier corpus de conocimiento la pregunta más simple: *¿para quién es?*
OKF responde, con limpieza, *la IA.* Su trabajo es hacer que los datos de una
organización sean legibles para los agentes — quitarle a las personas la carga
cognitiva de "conocer el sistema" y codificarla en un grafo que el modelo pueda
instrumentar para sí mismo. Por eso el anuncio echa mano de BigQuery para hacer el
grafo consultable: el consumidor implícito que hace la consulta es el agente. Y
encaja con la corriente dominante de la industria ahora mismo — agentes autónomos
de larga duración, los que no duermen, los que toman un prompt de un párrafo y se
van a trabajar durante días. En ese mundo, *cuanto menos* necesite estar el humano
en el bucle, mejor se considera que el sistema rinde. OKF es buena infraestructura
para ese mundo.

StrayMark está construido sobre una apuesta distinta acerca de hacia dónde va
todo esto. No el humano sacado del bucle — el humano con un lugar defendible
*dentro* de él. El corpus existe para que un ingeniero pueda saber qué se decidió
y por qué, qué está en marcha, qué queda pendiente y hacia dónde se dirige todo —
sin tener que cargar el código entero en la cabeza a cada paso, y sin quedar
reducido a estampar su sello sobre un resultado que en realidad no puede evaluar.
Es la postura que Scott Hanselman y otros vienen defendiendo bajo el nombre de
**ingeniería aumentada por IA** (*AI-augmented engineering*): el humano no es
desplazado, es *reubicado* en la parte que sí es ingeniería — el juicio, la
dirección, la rendición de cuentas — y ahí es potenciado por la IA. El polo
opuesto al vibe coding, y al agente-conductor totalmente autónomo que decide por
sí mismo qué hace, hacia dónde va y cuándo se detiene.

Así que los dos proyectos construyen cognición para lectores distintos. **OKF
construye cognición para el agente. StrayMark construye cognición para el dúo** —
el agente y el ingeniero humano a la vez — y trabaja para mantener a ambos
orientados sobre el mismo mapa. Loom, nuestro visualizador (experimental) de
grafos y arquitectura, es la señal más clara: sus overlays de estado y su vista de
arquitectura 3D son *apoyos didácticos para la ubicación espacio-temporal de una
persona* en un proyecto — estás aquí, esto está activo, esto carga deuda, hacia
aquí se dirige el trabajo. Eso no se construye para un agente. Se construye para
el humano del dúo.

Léelo con cuidado: nada de esto va de frenar a la IA. Un mapa no te frena — es
cómo avanzas rápido sin perderte. El objetivo es hacer la IA útil en un contexto
donde el humano conserva el control y sigue *sabiendo*: el qué, el porqué, el qué
sigue, el qué queda abierto.

## Lo que se le dejó al productor

Ahora la frase de la especificación se lee distinto. OKF es, por intención,
*mínimamente opinado*. Estandariza el sobre —cómo se da forma a un concepto, cómo
funcionan los enlaces, cómo se entrega un bundle— y se detiene. *Todo lo demás se
le deja al productor.* Para el trabajo de OKF —datos legibles para agentes— esa es
exactamente la decisión correcta; el sobre es la contribución.

Pero "todo lo demás" no es un cajón de sastre de extras opcionales. Es la **capa
entera de cara al humano**: qué tipos de documento vale la pena conservar (un ADR
—una decisión y su porqué; un AILOG —lo que el agente realmente hizo; una Charter
—lo que se planificó; una TDE —deuda asumida a conciencia), bajo qué ciclo de vida
(las charters derivan respecto al código, los seguimientos se capturan y se
promueven, las auditorías corren sobre modelos independientes, el cumplimiento se
mapea a la EU AI Act y a la ISO 42001), con qué garantías. Esa capa no son datos
para que un agente los consuma. Es **conocimiento —no solo información— puesto
frente a una persona para que pueda tomar decisiones.** StrayMark es máximamente
opinado sobre exactamente eso. *Es* el "todo lo demás", a propósito.

La forma más limpia de sostener ambos en la cabeza:

> **OKF describe el sistema para que una IA pueda entenderlo. StrayMark describe
> cómo se construyó el sistema para que un humano pueda confiar en él — y seguir al
> mando de él.**

¿Podrías extender OKF hacia afuera hasta que hiciera el trabajo de StrayMark —
añadirle tipos, un ciclo de vida, un visualizador de cara al humano? Sí. Pero
estarías reconstruyendo, externamente, algo que ya existe y que ya lo hace bien.
La relación honesta no es de competencia ni de reemplazo. Es que ambos miran en
direcciones opuestas: uno hacia los datos y el agente que los lee, el otro hacia
el proceso y el humano que responde por él. Un equipo puede usar ambos. No se
solapan — y donde parecen hacerlo, StrayMark ya cubre el terreno.

## La parte que la especificación no estandarizó, y nosotros sí

Hay una versión más afilada y concreta de la misma divergencia, escondida en el
modelo de enlaces de OKF.

En OKF, los enlaces son **sin tipo**. La especificación es explícita: *"el tipo
específico de relación … lo transmite la prosa circundante, no el enlace en sí."*
Un enlace de A a B significa *relacionados, de algún modo* — lee el párrafo.
Razonable para un formato mínimo cuyo lector es un modelo que puede leer el
párrafo.

Pero ve a leer el hilo de comentarios bajo el gist original de Karpathy —la
genealogía de la que OKF desciende— y encontrarás a los propios críticos del
patrón nombrando sus modos de fallo. Una respuesta (la de pursultani) hace el
señalamiento: el patrón LLM-wiki tiende por defecto a *reconciliar*
contradicciones, resolviendo silenciosamente las tensiones en una única historia
convergente — y en muchos dominios una contradicción *porta información* y debería
**preservarse**, no aplanarse. El arreglo propuesto: aristas tipadas en el
frontmatter, para que una relación pueda decir *contradice*, *reemplaza*, *es una
alternativa a* — y no solo *se relaciona con*.

Preservar una contradicción en lugar de aplanarla es, precisamente, un movimiento
de humano-en-el-bucle: una tensión que se mantiene visible es una tensión sobre la
que se le puede pedir a una persona que *decida*. StrayMark ya funciona así.
`supersedes` es una arista tipada: este ADR reemplaza a aquel, y el viejo queda en
disco, inmutable, como historia preservada. `alternatives_documented` registra los
caminos no tomados. El concepto entero de una **Transversal Debt Entry** existe
para anotar una tensión sin resolver y mantenerla frente a alguien, en lugar de
taparla. Otra respuesta en ese gist —una implementación llamada *memwiki*— se
construyó específicamente para combatir la *"Amnesia del Agente,"* agentes que
olvidan decisiones arquitectónicas. Ese es, palabra por palabra, el problema que
StrayMark fue fundado para resolver — al que llegamos por cuenta propia, y que el
hilo casualmente corrobora. El hilo no es la hoja de ruta de un competidor. Es un
catálogo de los modos de fallo del patrón, y varios de ellos son cosas que
tratamos como requisitos desde el primer día — porque construíamos para un lector
que tiene que vivir con las consecuencias: una persona.

## Qué vamos a hacer al respecto

Dos cosas, ninguna a la defensiva.

Primero, **interoperar.** Como las primitivas son compartidas, la distancia de un
corpus StrayMark a un bundle OKF conforme es pequeña y en su mayoría mecánica — un
mapa de nombres de tipo, y reescribir nuestras referencias de frontmatter tipadas
como enlaces markdown relativos al bundle, algo que el trabajo de resolución de
referencias detrás de Loom ya hace. Un proyecto StrayMark debería poder emitir un
bundle OKF de su registro de gobernanza, para que cualquier cosa que hable OKF
—incluido el propio Knowledge Catalog de Google— pueda leerlo. Y debería ir y
volver sin pérdidas: OKF promete preservar las claves de frontmatter que no
reconoce, y nuestros metadatos de gobernanza son exactamente esas claves. Nos
encontramos con OKF en su propio sobre y no perdemos nada del nuestro.

Segundo, **Loom ya renderiza esto.** Loom ingiere markdown-con-frontmatter y
construye un grafo; un bundle OKF es exactamente esa misma entrada. La diferencia
es lo que Loom hace con él — overlays de estado, analíticas de grafo, una vista de
arquitectura 3D, todo apuntado a orientar a un humano. Apuntar Loom a cualquier
bundle OKF es un pequeño adaptador, no una reescritura. Y es también la
demostración más directa de la tesis: toma un corpus diseñado para que lo lea un
agente, y renderízalo de modo que una persona pueda pararse dentro de él.

## Si has leído hasta aquí

El ejercicio portable esta vez no va de formatos de archivo. Es una pregunta que
hacerse sobre cualquier "conocimiento" que tu equipo guarde para su IA: **¿para
quién es?** Si la respuesta es *el agente solo* —para que pueda trabajar más
tiempo sin ti— entonces un sobre mínimo y estándar como OKF es casi todo lo que
necesitas, y ese sobre está a punto de convertirse en un commodity, lo cual es
bueno para todos. Si la respuesta es *tú y el agente juntos* —para que una persona
siga orientada, al mando, y capaz de tomar las decisiones que le corresponden—
entonces el sobre nunca fue la parte difícil. La parte difícil es decidir qué debe
recordar tu proyecto, en qué tipos, bajo qué ciclo de vida, y a quién se supone
que todo eso mantiene en el bucle. Un organismo de estándares puede entregarte el
sobre. Deliberadamente no puede entregarte una postura sobre el lugar del humano
en el trabajo. *Eso* siempre iba a quedar en manos del productor.

---

*Open Knowledge Format v0.1: [especificación](https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf) · [anuncio de Google Cloud](https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing) · [el gist de Karpathy](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) del que desciende OKF. El análisis de StrayMark vive en `experiment-okf/` en el repo.*

*Este documento se produjo con la asistencia de herramientas de IA generativa (Claude Opus 4.8); toda la responsabilidad sobre el contenido recae en el autor humano.*
