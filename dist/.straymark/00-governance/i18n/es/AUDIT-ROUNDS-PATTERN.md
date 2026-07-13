# Patrón de Rondas de Auditoría - StrayMark

> Namespacing por ronda para Charters que necesitan más de una ronda de auditoría externa.

**Idiomas**: [English](../../AUDIT-ROUNDS-PATTERN.md) | Español | [简体中文](../zh-CN/AUDIT-ROUNDS-PATTERN.md)

---

## Estado

**Desde fw-4.35.0 / cli-3.33.0** (Issue #341). Opcional y retrocompatible: los Charters de una sola ronda no necesitan nada nuevo.

## El problema

El subsistema de auditoría externa (`straymark charter audit`, `/straymark-audit-execute`, `/straymark-audit-review`) asumía originalmente **exactamente una ronda de auditoría por Charter**. Pero los Charters multi-fase son un concepto de primera clase, y la auditoría acotada por fase es la práctica recomendada para ellos. Cuando un Charter necesita **más de una** ronda de auditoría externa (p. ej. una por fase con mucho código), el layout plano de una sola ronda se rompe de dos formas:

1. **Los paths planos fijos se sobrescriben.** Cada ronda escribía `audit-prompt.md`, `report-*.md` y `review.md` en el mismo directorio `.straymark/audits/<CHARTER-ID>/`, así que una segunda ronda pisaba silenciosamente el prompt de la primera, y preservar el historial requería un baile manual de archivado con `git mv`.
2. **Contaminación cross-ronda del glob.** El glob `report-*.md` (tanto en `--merge-reports` como en la skill de review) es plano y no-recursivo. Si los reports de una ronda previa quedaban planos con cualquier nombre que aún matcheara `report-*.md`, eran arrastrados al `review.md` consolidado y a la telemetría de la ronda **actual** — mezclando rondas.

## El patrón: `--round <label>`

Pasa una etiqueta de ronda opcional para namespacear la tríada completa bajo un subfolder por ronda:

```bash
# Ronda 1 — fase de seguridad
straymark charter audit CHARTER-01 --prepare --round fase-1 --range <primer-commit-fase-1>..HEAD
# → .straymark/audits/CHARTER-01/fase-1/audit-prompt.md

# ...los auditores escriben sus reports en el mismo subfolder, luego:
straymark charter audit CHARTER-01 --merge-reports --round fase-1 \
  --merge-into .straymark/charters/CHARTER-01.telemetry.yaml
```

La etiqueta debe ser un slug simple (`[A-Za-z0-9._-]`, empieza con alfanumérico, sin separadores de path ni espacios) — se convierte en un nombre de directorio.

### Layout resultante

```
.straymark/audits/CHARTER-01/
  fase-1/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-2/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-3/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
```

Como cada ronda vive en su propio subfolder y el glob es no-recursivo, las rondas nunca se sobrescriben (arregla el problema 1) y el merge se acota exactamente a los reports de la ronda actual (arregla el problema 2).

### Threading de la etiqueta

El mismo `--round <label>` fluye por toda la tríada — la guía de `--prepare` del CLI y las skills lo repiten:

- `/straymark-audit-prompt <CHARTER-ID>` → `charter audit --prepare --round <label>`
- `/straymark-audit-execute <CHARTER-ID> --round <label>` → lee/escribe bajo el subfolder
- `/straymark-audit-review <CHARTER-ID> --round <label>` → consolida solo ese subfolder

## Telemetría: múltiples rondas coexisten

Cada entrada de `external_audit` mergeada con `--round` lleva un campo `round:`, así que las rondas quedan distinguibles dentro de un mismo archivo de telemetría:

```yaml
charter_telemetry:
  external_audit:
    - auditor: "gpt-5-2-codex"
      round: "fase-1"
      findings_total: 5
      # ...
    - auditor: "claude-sonnet-5"
      round: "fase-2"
      findings_total: 2
      # ...
```

`--merge-into` **hace append** de una nueva ronda a un bloque `external_audit:` ya poblado en lugar de rechazar — siempre que la etiqueta de ronda sea nueva. Re-mergear la **misma** ronda sigue siendo rechazado (el guard de misma-ronda previene la duplicación silenciosa); mergear en un bloque poblado **sin** etiqueta de ronda también se rechaza (las rondas deben quedar distinguibles). Usa un `--round <label>` nuevo por ronda.

## Retrocompatibilidad

Omite `--round` por completo y todo se comporta exactamente como antes de fw-4.35.0: paths planos bajo `.straymark/audits/<CHARTER-ID>/`, un solo bloque `external_audit` sin campo `round:`, y `--merge-into` rechazando cualquier array poblado. Los Charters de una sola ronda (el caso común) no necesitan cambio.

## Relacionado

- [AGENT-RULES.md §12](AGENT-RULES.md) — el Punto de Control de Auditoría que enmarca cuándo correr una auditoría externa, más los bullets de estado-estable y multi-ronda.
- [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) — pattern doc hermano (la convención de layout de registro que este espeja).

---

*StrayMark fw-4.35.0 | [Strange Days Tech](https://strangedays.tech)*
