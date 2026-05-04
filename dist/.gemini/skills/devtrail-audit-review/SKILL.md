---
name: devtrail-audit-review
description: Calibrate the responses of two external auditors and merge findings into the Charter telemetry. Counterpart to /devtrail-audit-prompt — invoke after the operator has saved the auditor responses.
---

# DevTrail Audit Review Skill

Reconcile the responses of two external auditors and produce the calibrator analysis. When the Charter has telemetry, append the consolidated `external_audit:` array directly into the YAML so the operator does not have to copy/paste.

## When to invoke

After running `/devtrail-audit-prompt <CHARTER-ID>`, having pasted both prompts into LLMs of different families, and saved the responses to:

- `audit/charters/<CHARTER-ID>/auditor-primary.md`
- `audit/charters/<CHARTER-ID>/auditor-secondary.md`

This skill produces the calibrator response and merges findings into telemetry.

## Instructions

### 1. Verify auditor responses exist

```bash
ls audit/charters/<CHARTER-ID>/auditor-primary.md \
   audit/charters/<CHARTER-ID>/auditor-secondary.md
```

If either file is missing, instruct the operator to run `/devtrail-audit-prompt <CHARTER-ID>` first and exit.

### 2. Resolve the calibrator prompt (CALIBRATE step)

```bash
devtrail charter audit <CHARTER-ID> --calibrate
```

The CLI:
- Validates `auditor-primary.md` and `auditor-secondary.md` against `audit-output.schema.v0.json`. If validation fails, surface the error to the operator and exit — the auditor response frontmatter must follow the schema documented inside each prompt.
- Writes `audit/charters/<CHARTER-ID>/prompts/calibrator-reconciler.prompt.md`.

### 3. Run the calibrator inline (this conversation IS the calibrator)

The calibrator-reconciler prompt is designed to run in any model family — the agent currently in this conversation can produce the calibrator response directly. Heterogeneity inter-family is required for the auditor pair, NOT for the calibrator (the calibrator's task is definitional, not discovery — see `Propuesta/devtrail-cli-roadmap.md` §5.2 for the rationale).

Read the resolved prompt:

```bash
cat audit/charters/<CHARTER-ID>/prompts/calibrator-reconciler.prompt.md
```

Produce the response **following the prompt's output schema exactly** — the frontmatter is required (`audit_role: calibrator-reconciler`, `calibrator: <model-id>`, `auditors_reconciled`, `findings_consolidated`, `findings_by_status`). Save to:

- `audit/charters/<CHARTER-ID>/calibrator-reconciler.md`

### 4. Finalize and merge into telemetry

Determine the telemetry path: `.devtrail/charters/<CHARTER-ID>.telemetry.yaml` (canonical form, `<CHARTER-ID>` without slug).

```bash
test -f .devtrail/charters/<CHARTER-ID>.telemetry.yaml
```

**Branch A — telemetry exists** (operator has already run `devtrail charter close`):

```bash
devtrail charter audit <CHARTER-ID> --finalize \
  --merge-into .devtrail/charters/<CHARTER-ID>.telemetry.yaml
```

The CLI validates all 3 outputs and appends `external_audit:` to the telemetry YAML directly. **Do NOT** edit the YAML by hand afterwards — `git diff` will show what changed; the operator reviews before commit.

If the CLI rejects the merge because `external_audit:` already exists (re-audit guard, v0 does not support re-merge), surface the message to the operator. Manual append of new findings is the v0 fallback for that case.

**Branch B — telemetry does NOT exist** (Charter not yet closed):

```bash
devtrail charter audit <CHARTER-ID> --finalize > /tmp/external-audit-block.yaml
mkdir -p audit/charters/<CHARTER-ID>
mv /tmp/external-audit-block.yaml audit/charters/<CHARTER-ID>/external-audit-pending.yaml
```

Tell the operator: "The Charter is not yet closed. The findings are saved in `audit/charters/<CHARTER-ID>/external-audit-pending.yaml`. When you run `devtrail charter close <CHARTER-ID>`, paste the `external_audit:` block from that file into the telemetry when prompted, or merge it manually after close completes."

### 5. Print summary

After step 4, print to the operator:

```
Audit review complete for <CHARTER-ID>.

  Auditors reconciled:
    - auditor-primary.md   (<N> findings, <model-id>)
    - auditor-secondary.md (<M> findings, <model-id>)
  Calibrator:               <calibrator-model-id> (<K> findings consolidated)

  external_audit YAML:
    [merged into .devtrail/charters/<CHARTER-ID>.telemetry.yaml]
    or
    [pending in audit/charters/<CHARTER-ID>/external-audit-pending.yaml]

  Run `git diff .devtrail/charters/<CHARTER-ID>.telemetry.yaml` to review.
```

## Notes

- This skill **does** invoke an LLM (the calibrator runs in the current conversation), unlike `/devtrail-audit-prompt` which is purely orchestration. The distinction is intentional: the calibrator is family-agnostic, so the agent driving the conversation is a valid calibrator.
- The skill is **idempotent for steps 1-3** (re-running regenerates the calibrator response if you delete `calibrator-reconciler.md`). It is **NOT idempotent for step 4 in Branch A** — once telemetry has `external_audit:`, the CLI rejects re-merge to prevent silent duplication.
- The auto-merge re-emits the YAML using the existing `charter_telemetry:` shape; cosmetic formatting of the merged file matches the close.rs output. Comments in the original telemetry YAML, if any, are preserved (the CLI uses string-level append, not full re-serialization).
- The `audit_notes:` field in the merged block points at the canonical paths `audit/charters/<CHARTER-ID>/auditor-{primary,secondary}.md`. If you renamed those files, fix the field manually after merge.
