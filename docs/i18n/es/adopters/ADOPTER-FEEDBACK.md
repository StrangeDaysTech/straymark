# StrayMark - Retroalimentación de Adoptantes

**Cómo anunciar tu adopción y enviar telemetría y hallazgos al proyecto upstream.**


---

## Tabla de Contenidos

1. [Por qué importa la retroalimentación](#por-que-importa-la-retroalimentacion)
2. [Dos canales](#dos-canales)
3. [Qué es la telemetría — y dónde vive](#que-es-la-telemetria-y-donde-vive)
4. [Cómo compartirla](#como-compartirla)
5. [La puerta N=1 → N=2](#la-puerta-n1--n2)
6. [Referencia rápida](#referencia-rapida)

---

## Por qué importa la retroalimentación {#por-que-importa-la-retroalimentacion}

StrayMark no recolecta nada automáticamente. No hay endpoint remoto, ni baliza opt-in, ni tablero de
adopción. El framework evoluciona con la **evidencia que los adoptantes deciden enviar** — lo que
significa que la calidad del siguiente release es función directa de lo que reportan los proyectos
reales.

La mayoría de los patrones publicados entre fw-4.13 y fw-4.19 surgieron de un único adoptante
([Sentinel](https://github.com/StrangeDaysTech/sentinel)) que alimentó hallazgos a lo largo de muchos
Charters. El framework mejora cuando más proyectos hacen lo mismo, desde más dominios.

## Dos canales {#dos-canales}

La retroalimentación tiene dos naturalezas distintas, así que tiene dos hogares:

| | **Anuncio** | **Hallazgos** |
|---|---|---|
| **Dónde** | Discussions → categoría **Adopters** | **Issues** (plantilla `Adopter feedback / upstream finding`) |
| **Cuándo** | Una vez, al adoptar | De forma continua, conforme descubres cosas |
| **Qué** | Tu proyecto, stack, versiones, a qué te comprometes | Una brecha, fricción, bug o candidato a patrón concreto — respaldado con telemetría |
| **Ciclo de vida** | Queda abierto como tu registro de adopción | Se cierra cuando se atiende |

Abre primero la [discusión de Adopters](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters);
luego **cruza** cada Issue de hallazgo de vuelta a ella. Ese enlace es lo que liga un hallazgo a un
adoptante conocido y a su contexto N.

## Qué es la telemetría — y dónde vive {#que-es-la-telemetria-y-donde-vive}

Cuando cierras un Charter (`straymark charter close`), StrayMark registra telemetría estructurada en
`.straymark/charters/CHARTER-NN.telemetry.yaml` — precisión de estimación (en tiempo, no en líneas),
comportamiento del agente, resultados de auditoría externa, cambios de alcance, y wins/fricciones
cualitativos. La forma está definida por
[`charter-telemetry.schema.v0.json`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/schemas/charter-telemetry.schema.v0.json).

**Este archivo se queda en tu repositorio.** No se transmite a ningún lado. Compartirlo upstream es
siempre un acto deliberado y manual de tu parte.

## Cómo compartirla {#como-compartirla}

1. **Decide qué es relevante.** Rara vez se necesita el archivo completo — la parte útil suele ser un
   bloque (un drift de `effort`, una delta de `external_audit`, una lista de
   `qualitative.friction_points`) que respalda una afirmación concreta.
2. **Anonimiza.** Quita lo sensible — nombres internos, secretos, rutas de repos privados — antes de
   que salga de tu repo.
3. **Adjúntalo a un hallazgo.** Pega el extracto en el Issue *Adopter feedback / upstream finding*, en
   el campo de telemetría (renderizado como YAML), y enlaza tu discusión de adopción.

El mantenedor puede, con tu consentimiento, anonimizar y agregar hallazgos de varios proyectos en
posts de blog o documentación — pero solo lo que tú hayas decidido publicar.

## La puerta N=1 → N=2 {#la-puerta-n1--n2}

StrayMark cristaliza patrones por **conteo de validaciones independientes**:

- **N=1** — un patrón visto en un solo proyecto/dominio. Documentado, pero se mantiene **manual**.
- **N=2** — una segunda validación independiente, idealmente en un **dominio distinto**. Esta es la
  puerta que justifica **automatizar** el patrón en el CLI.

Una app de escritorio en Rust validando un patrón observado primero en un backend de Go es un N=2
mucho más fuerte que otro backend de Go. Así que cuando anuncies — y cuando reportes hallazgos — di si
estás validando un patrón existente, y desde qué dominio. Ese único dato suele ser lo más valioso que
aporta un adoptante.

## Referencia rápida {#referencia-rapida}

| Quieres… | Haz esto |
|---|---|
| Anunciar tu adopción | Abre una [discusión de Adopters](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters) |
| Reportar una brecha / fricción / patrón | Abre un Issue con la plantilla *Adopter feedback / upstream finding* |
| Respaldar un hallazgo con datos | Pega un extracto anonimizado de `charter_telemetry:` en el Issue |
| Entrar al registro | Anuncia primero; un mantenedor te agrega a [`ADOPTERS.md`](https://github.com/StrangeDaysTech/straymark/blob/main/ADOPTERS.md) |

Ver también: [Guía de Adopción](ADOPTION-GUIDE.md) · [Flujos de Trabajo Recomendados](WORKFLOWS.md) · [Referencia del CLI](CLI-REFERENCE.md)

---

*StrayMark — Porque cada cambio cuenta una historia.*

[Strange Days Tech](https://strangedays.tech)
