# Propuesta a StrayMark — verbo declarado en autoría (tercera vía de clasificación)

> **De:** adopter Sentinel · **Para:** StrayMark / experimento Baton ·
> **Motiva:** E1 ([E1-findings.md](E1-findings.md)) + #328 · **Fecha:** 2026-06-26.
> Feedback de adoptante (plan §6), no un cambio aplicado.

## El problema de raíz: clasificar *después* tira información gratis

Baton clasifica las unidades **después** de creadas, reconstruyendo el tipo de
trabajo desde el substring del título. E1 demostró que esa reconstrucción falla justo
donde importa: el verbo del trabajo está enterrado o es ambiguo, y el cue se ancla a
tokens incidentales (`audit.go` filename → auditor; "**Wire**" → implementer;
"(commit `hash`)" → operator; "make **test**-live" → operator; "coverage" de fechas →
operator). El defecto no es de afinación de keywords — es **temporal**: se intenta
adivinar a posteriori algo que era **certero y gratis en el momento de autoría**.

Cuando el operador (humano + agente) crea el charter/batch, conoce dos cosas que el
clasificador después tiene que adivinar:

1. **El verbo real** del trabajo (diseñar / implementar / auditar / operar).
2. La **carga cognitiva residual**: ¿es una *decisión nueva* (optimizar, diagnosticar,
   diseñar) o solo *instrumentar un diseño ya hecho aguas arriba*? (Una query compleja
   que solo se transcribe de un diseño previo es mecánica, aunque parezca sofisticada.)

## Las dos vías que ya estamos sopesando, y su límite

- **(a) Determinista por título** (lo actual): barato, pero ciego al verbo real →
  high+medium 0.57, 4 errores hacia abajo (E1). Insuficiente y, peor, *inseguro*.
- **(b) Score de complejidad por IA** por unidad: certero, pero **la inferencia puede
  costar tantos o más tokens que el ahorro de routing** — se come su propio beneficio.

## La tercera vía: campo de verbo normalizado declarado

Entre (a) y (b): **capturar el verbo como campo de gobernanza de primera clase en el
momento de autoría**, no inferirlo en `classify`.

```yaml
# frontmatter de charter / batch / (task SpecKit)
work_verb: implement        # design | implement | audit | operate
design_provenance: new      # new | upstream   (la dimensión de carga residual)
```

Vocabulario controlado, mapeado a tier (es el `cue_class` actual de Baton, pero
**declarado** en vez de inferido):

| work_verb | ejemplos | tier |
|---|---|---|
| `design` | decidir patrón, escalabilidad, autoría de spec/arquitectura | planner |
| `implement` | lógica de servicio, fix con diagnóstico, query compleja/optimización, tooling nuevo | implementer |
| `audit` | revisar, verificar, contrastar contra realidad | auditor |
| `operate` | tests/test-infra/migración mecánica, scaffolding, docs, cierre, bulk | operator |

**`design_provenance` captura tu insight más profundo:** una unidad con verbo
aparente `implement` pero `design_provenance: upstream` (solo instrumenta un diseño
hecho en otra etapa) **rutea a operator**. Es lo que separó batch-1/CHARTER-28
(upstream → operator) de batch-2/CHARTER-03 (new → implementer) en E1.

## Por qué resuelve la restricción de costo

- **Costo marginal ≈ 0 tokens.** No es inferencia en `classify`: es un enum que el
  autor fija mientras ya escribe el scope/título. El ahorro de routing queda intacto.
- **Cierra exactamente los fallos de E1:** sin inferencia de substring, no hay
  falsos positivos de filename/metadata/polisemia/verbo-vs-objeto.
- **Auditable y corregible:** un verbo mal puesto se arregla en un campo, no
  re-afinando tablas de keywords para todo el corpus.

## Cambio mínimo en Baton (no tira lo determinista)

`signals::signals_for` ya lee `effort_estimate`/`risk_level`/`severity` del
frontmatter. Añadir: si existe `work_verb`, **es la cue autoritativa**; si no,
**fallback a `scan_cues(title)`** (lo actual) para las 762 unidades legacy. El
title-scan se degrada de oráculo a *prior*. `classify.rs` no cambia su mapa
cue→tier. Es un PR pequeño y de alta palanca.

Complementos opcionales, alineados con el plan de adoptante:
- **Nudge de clasificabilidad** (plan §5.5): un check estilo `coherence` marca
  "unidad sin `work_verb` → baja confianza; declara el verbo". Convierte la baja
  confianza en una acción de higiene, no en ruido.
- **Calibración periódica** (este mismo loop E1): valida que los verbos declarados no
  deriven, con el título como cross-check débil.

## Riesgos y mitigaciones

- **Deriva/gaming del verbo** (el autor pone el verbo cómodo): vocabulario pequeño +
  título como prior de contraste + calibración periódica.
- **Unidades heterogéneas** (charter con diseño + parte mecánica): declarar al grano
  más fino que sea homogéneo (verbo por batch); permitir verbo primario + modificador.
- **Carga en autoría:** es precisamente la tesis de #328 — la gobernanza debe
  registrar señal estructurada de primera clase, y el autor es la fuente más barata y
  mejor informada. No es trabajo extra desechable: mejora auditorías, métricas y
  triage, además de Baton.

## Síntesis

E1 mostró que el título es una señal contaminada y que el verbo real (+ la provenance
del diseño) es lo que decide el tier. Baton ya *construye* intent/provenance (B2) pero
el clasificador no lo usa. La tercera vía no pide IA cara ni se conforma con el
título: pide que el verbo se **declare donde se conoce gratis** — en autoría — y deja
lo determinista como red de seguridad. Es la forma más barata de la lección de #328.
