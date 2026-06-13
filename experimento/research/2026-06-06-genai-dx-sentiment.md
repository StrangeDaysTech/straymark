---
title: "Sentimiento de desarrolladores frente a genAI — encuesta cwebber + contraste cuantitativo"
date: 2026-06-06
agent: claude-code-v2.1.167
confidence: high
type: research-note
review_required: false
status: final
---

# Sentimiento de desarrolladores frente a genAI: la encuesta de cwebber y el contraste cuantitativo

> Nota de investigación para el posicionamiento de StrayMark. No es un documento de gobernanza
> StrayMark (no registra cambios de código); es evidencia de mercado/DX.

## 1. Contexto y metodología

El 2026-06-05, Christine Lemmer-Webber (co-autora de ActivityPub, directora ejecutiva de
Spritely) publicó una encuesta en el Fediverso:

> *"The proliferation of genAI has made my life…"*
> — [social.coop/@cwebber/116698488515037678](https://social.coop/@cwebber/116698488515037678)

**Recolección**: API pública de Mastodon (`/api/v1/statuses/{id}` y `/context`) contra el
servidor de origen `social.coop`, 2026-06-06. Snapshots crudos en
[`data/cwebber-poll-status-2026-06-06.json`](data/cwebber-poll-status-2026-06-06.json) y
[`data/cwebber-poll-context-2026-06-06.json`](data/cwebber-poll-context-2026-06-06.json).
Se capturaron **56 de las 68 respuestas contabilizadas** (el resto: privadas o de instancias
no federadas con social.coop). Los 4 *quotes* del post no son accesibles sin autenticación
(limitación del endpoint `/quotes`), y el hilo es demasiado reciente para estar indexado en
buscadores — no se encontró discusión derivada externa al momento de la captura.

**Sesgo de muestra** (reconocido por la propia autora en su post de cierre): el Fediverso
concentra desarrolladores FOSS, críticos del big tech y población expulsada de plataformas
comerciales. Los porcentajes absolutos no son extrapolables a la población general de
desarrolladores; el valor del corpus es **cualitativo** — la articulación concreta de las
frustraciones — y la magnitud sí sorprendió incluso a quien conoce bien el medio:

> *"Poll closed. Interesting results. Well, I expected 'worse' to be much higher than 'better'
> […] but even given me knowing fedi pretty well, this is pretty damn stark"* — @cwebber, 2026-06-06
>
> *"I'm sure the results would be very different depending on where the poll is run. X-Twitter
> vs Bluesky vs Fedi are all going to give pretty significantly different responses, and so is
> my audience in particular. Still, kinda astounding in some ways"* — @cwebber, 2026-06-06

## 2. Resultados de la encuesta

4,712 votantes en 24 horas — la mayor participación que la autora ha tenido en una encuesta
(403 reblogs, 157 favs, 68 respuestas).

| Opción | Votos | % |
|---|---:|---:|
| **Worse** | 3,564 | **75.6%** |
| Better and worse | 705 | 15.0% |
| No difference | 327 | 6.9% |
| Better | 116 | 2.5% |

Lectura agregada: 90.6% de los votantes reporta algún componente "worse"; solo 17.5% reporta
algún componente "better".

## 3. Clasificación de problemáticas (56 respuestas públicas)

Las categorías no son excluyentes (una respuesta puede tocar varias). Ordenadas por
frecuencia y severidad. Citas verbatim en inglés.

### 3.1 Precariedad y destrucción de empleo (~10 respuestas — la más severa)

- Despidos directos: *"I no longer have a job. My joke of a retirement savings is either going
  to vanish when the bubble pops (again) or I'll have to use it to keep my family afloat"*
  (@drwho); *"I ain't got no job now. So worse"* (@kargas).
- **Represalia por disenso**: *"also got fired due to early criticism of it (found a new job
  about a year later but…)"* (@mirabilos); @crowbriarhexe responde *"🫂 same"*.
- Colapso de profesiones completas: @sknob, traductor técnico freelance, cerró su negocio tras
  20 años — *"Most of them [clients] wrote back to me privately to tell me how much they hated
  genAI, imposed on them by their bosses"*.
- Obsolescencia anticipada: *"I will probably be unemployed and unemployable soon"* (@datarama).
- Mercado degradado: *"AI has somehow made the impossibility of getting a stable job even MORE
  impossible"* (@marisadoom); *"job market is atrocious so there was no chance for me to
  negociate doing my job part time"* (@kincat).

### 3.2 Colapso de la confianza informacional (~9)

- Búsqueda degradada: *"search engines don't search"* (@amici); resultados *"60% correct and
  40% bullshit"* que obligan a verificación constante (@vfrmedia); *"more recaptchas and less
  relevant search results"* (@Paul).
- Hipervigilancia cognitiva: *"My paranoia was well-managed, but now my brain identifies every
  statement as a lie. I feel like I'm burning out from information"* (@kaylee).
- Desconfianza en medios visuales: *"I don't trust the art I see on mainstream platforms
  anymore"* (@kincat); manipulación de imagen/video (@CStamp).
- Erosión epistémica social: *"I have lost so much respect for folks in my life that use LLM
  chatbots to answer questions instead of looking for actual verified information, or just
  thinking about stuff"* (@greenarchist); slop en tareas universitarias (@dlakelan).
- Confusión terminológica deliberada: *"The conflation of ML, GenAI, and LLM has made it
  impossible to tell whether or not the 'AI' any particular project is using [is] the planet
  burning bullshit machinery or some actually useful tech"* (@greenarchist).

### 3.3 Imposición corporativa / pérdida de agencia (~7)

- *"AI is hardly the first technology I've avoided, but it's the technology I've had to work
  the hardest to avoid"* (@Steve — la respuesta con más favs del hilo: 29).
- Presión laboral para adoptarla (@OrdRadical, @Xarizzar); *"the companies that try to shove it
  down everyone's throats"* (@deBaer); evitar la IA cuesta esfuerzo activo (@CStamp).
- **Supresión del disenso**: *"I'm tired of pretending I like genAI because dissent of our
  current direction is considered a grave sin today"* (@ladytel); *"I'm so sick of getting
  gaslit over AI nonsense. I don't mind the tech... it's the mobbing that's sick!"* (@promovicz).

### 3.4 Costos materiales de hardware (~7 — inesperadamente prominente)

- Crisis de RAM/flash/SSD/GPU por demanda de datacenters: *"i can't afford a ram upgrade
  anymore"* (@thed4rknss); *"RAM and flash crisis"* (@valpackett); *"all these costs in SSDs,
  Graphics Cards, etc… rising"* (@Xarizzar); compras de equipo pospuestas (@gregtitus).
- Agravado por software peor: *"more programs are vibe-coded electron-based RAM black holes
  with useless AI chatbots"* (@Gabriel).

### 3.5 Degradación del tejido comunitario y del discurso técnico (~6)

- *"All my communities have gone to shit and people I used to respect have latched on to pure
  ignorance instead of the ample well-founded criticisms available for the technology"* (@Kye).
- Monopolización temática: la discusión de IA *"has driven out every other interesting topic"*
  (@gregtitus); *"stealing all the oxygen from the actual work"* (@erisceleste); también fuera
  de tech — *"Everyone in philosophy has been trying to convince people they are now an expert
  in AI"* (@locha).
- Polarización que quema incluso a neutrales (@reallylazybear).

### 3.6 Salud mental (~5)

- *"my mental health is worse than it has ever been, and almost everything I used to enjoy now
  feels meaningless. I don't believe that there is any place for someone like me in the future
  they're building, and society rarely treats superfluous people kindly"* (@datarama).
- Daño a terceros: *"a friend of mine fell into chatbot hole went mildly psychotic, went
  bankrupt, and fled the country"* (@dlakelan).
- Pérdida de vocación: *"it finished killing my motivation to work with computers"* (@kincat);
  *"Even if it didn't also ruin my mood — which it does — it adds friction & removes
  effectiveness & quality in my otherwise awesome job"* (@jwcph).

### 3.7 Calidad de servicios / errores automatizados (~4)

- Errores en servicios críticos: *"my water company, and my health insurance vendor — each
  recently sent me wildly inaccurate erroneous bills. While I can't prove it, I suspect that
  they had applied some sort of Ai 'solution' to their accounting databases"* (@Guillotine_Jones).
- Chatbots de soporte con *"cheap and useless models"* (@RezzaBuh).
- **Deuda técnica anticipada**: *"clients generating genuinely useful tools for themselves […]
  delivered as black boxes. I'm bracing for some weird maintenance work in the future"*
  (@Snelldunks).

### 3.8 Captura corporativa de la investigación (~3)

- La más amarga, de una persona con maestría en ML: *"These are interesting research topics.
  The millisecond it became usable to make billionaires richer it turned into what we have
  now"* (@kincat). En la misma línea: @fortyseven, @tbortels.

### 3.9 Medio ambiente (~2)

- *"the environment globally is being damaged by crazy investment in datacenters"* (@dlakelan);
  *"the planet burning bullshit machinery"* (@greenarchist).

### 3.10 Lo positivo reconocido (minoritario, siempre con reservas)

- El caso de uso mejor articulado — análisis de logs de seguridad a escala: *"its great at
  pattern matching so its saved me from having to read terrabytes of logs looking for an
  attackers IP address and patterns […] Its removed parts of my workload that were tedious but
  well enough defined that I could explain them in a way I could have honestly hired my 7 year
  old to do"* (@kusuriya).
- Vibecoding de herramientas personales y Deep Research (@RezzaBuh); recuperar capacidad de
  búsqueda (@Kye); voz femenina sintética en juegos tras 15 años evitando usar la propia (@maho).
- El "better" más irónico: @targetdrone votó "better" porque la genAI fue insumo de su decisión
  de **jubilarse anticipadamente** — *"I'm loving not having to deal with genAI at work anymore!!!"*

## 4. Contraste cuantitativo (fuentes 2025–2026, verificadas contra fuente primaria)

| Fuente | Hallazgo | Conexión con el corpus |
|---|---|---|
| [Stack Overflow Developer Survey 2025](https://stackoverflow.blog/2025/12/29/developers-remain-willing-but-reluctant-to-use-ai-the-2025-developer-survey-results-are-here/) | Adopción ~80% en workflows, pero confianza en exactitud cayó de 40% → **29%**; favorabilidad 72% → **60%** YoY; frustración #1 (45%): *"AI solutions that are almost right, but not quite"*; **66%** pasa más tiempo arreglando código IA "casi correcto"; 75% recurre a otra persona cuando no confía en la IA. | Corrobora §3.2 (confianza) y §3.7 (calidad). La paradoja adopción-alta/confianza-baja explica el resentimiento de §3.3: se usa por imposición, no por convicción. |
| [DORA 2025 (Google Cloud)](https://cloud.google.com/blog/products/ai-machine-learning/announcing-the-2025-dora-report) | *"AI doesn't fix a team; it amplifies what's already there."* 90% usa IA en el trabajo; relación **positiva con throughput** pero **negativa con estabilidad** de entrega; 30% reporta poca o ninguna confianza en código generado. Sin sistemas de control (testing automatizado, version control maduro, feedback rápido), más volumen de cambio = más inestabilidad. | Corrobora §3.7 (black boxes, errores en producción) y fundamenta la respuesta de gobernanza: el problema no es la aceleración sino la ausencia de controles aguas abajo. |
| [METR RCT, jul 2025](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/) | RCT con 16 devs experimentados de open source, 246 issues (~2h c/u), Cursor Pro + Claude 3.5/3.7 Sonnet: **19% más lentos con IA**, habiendo predicho +24% de velocidad y creyendo aún +20% *después* de experimentar la lentitud. | La brecha percepción-realidad (39 puntos) valida la sospecha cualitativa de §3.2: ni los propios usuarios pueden auto-reportar el impacto con fiabilidad → se necesita registro objetivo, no percepción. |

Tendencia 2026 (secundarias, direccionales): la distrust activa siguió creciendo tras el corte
2025 ([LeadDev](https://leaddev.com/technical-direction/trust-in-ai-coding-tools-is-plummeting),
[Stack Overflow blog 2026-02](https://stackoverflow.blog/2026/02/18/closing-the-developer-ai-trust-gap/)).

## 5. Síntesis: mapeo a StrayMark

El hallazgo central: **la frustración dominante no es con la capacidad técnica de los modelos,
sino con la imposición sin consentimiento, la opacidad y la ausencia de rendición de cuentas.**
Varias respuestas distinguen explícitamente el ML como campo legítimo (§3.8: *"These are
interesting research topics"*) de su despliegue extractivo actual. Es una crisis de
*gobernanza*, no (solo) de tecnología.

| Problemática | ¿StrayMark la atiende? | Cómo |
|---|---|---|
| §3.2 Confianza informacional | ✅ directa | Trazabilidad agente→cambio (AILOG), confianza declarada, registro verificable vs. percepción (METR demuestra que el auto-reporte falla) |
| §3.7 Black boxes / deuda técnica | ✅ directa | AIDEC/ADR registran alternativas y racional; declared-vs-wired ataca "shipped but broken"; DORA: los controles aguas abajo son el factor diferencial |
| §3.3 Imposición / falta de agencia | ✅ parcial | `review_required` + gates humanos devuelven agencia al operador; documentar disenso es legítimo (AIDEC con alternativas rechazadas) |
| §3.1 Empleo | ⚠️ indirecta | StrayMark documenta *qué* hizo la IA y *quién* es responsable — insumo para accountability laboral, no solución |
| §3.5 Discurso comunitario | ⚠️ indirecta | Evidencia objetiva (telemetría ex-post de Charters) baja la temperatura del debate percepción-contra-percepción |
| §3.4 Hardware, §3.6 Salud mental, §3.9 Ambiente | ❌ fuera de alcance | Problemas de economía política del sector, no de gobernanza documental |

Implicación de posicionamiento: StrayMark no debe venderse como "herramienta para usar más IA"
sino como **infraestructura de consentimiento y accountability** para equipos a los que la IA
ya les llegó — el 80–90% adopción con 29–33% de confianza *es* el mercado: gente obligada a
convivir con una herramienta de la que desconfía y sin registro objetivo de lo que hace.

## 6. Limitaciones

- Muestra auto-seleccionada del Fediverso (sesgo anti-big-tech reconocido por la autora).
- 12 de 68 respuestas no capturadas (privadas / no federadas); 4 quotes inaccesibles sin auth.
- Sin discusión derivada externa al momento de captura (hilo de <48h, no indexado).
- Las fuentes "tendencia 2026" del §4 son secundarias; las tres fuentes principales sí fueron
  verificadas contra primaria el 2026-06-06.

## Fuentes

- Post original: <https://social.coop/@cwebber/116698488515037678> (snapshot en `data/`)
- Stack Overflow Developer Survey 2025: <https://stackoverflow.blog/2025/12/29/developers-remain-willing-but-reluctant-to-use-ai-the-2025-developer-survey-results-are-here/> · <https://survey.stackoverflow.co/2025/ai>
- DORA 2025: <https://cloud.google.com/blog/products/ai-machine-learning/announcing-the-2025-dora-report>
- METR: <https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/>
- Tendencia 2026: <https://leaddev.com/technical-direction/trust-in-ai-coding-tools-is-plummeting> · <https://stackoverflow.blog/2026/02/18/closing-the-developer-ai-trust-gap/>
