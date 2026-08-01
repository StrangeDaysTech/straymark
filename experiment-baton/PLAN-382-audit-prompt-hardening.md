# Plan — #382: Audit prompt hardening + verification quality

Source: [Issue #382](https://github.com/StrangeDaysTech/straymark/issues/382) — a 4-model
audit cycle missed a feature that was entirely unreachable (zero production callers).
The issue proposes one main rule + two secondary rules for the audit prompt, plus
documents a second-order finding about the implementer's broken self-verification.

---

## Track A: Audit prompt rules (EN + ES)

Three additions to `.straymark/audit-prompts/audit-prompt.md` and
`.straymark/audit-prompts/i18n/es/audit-prompt.md`.

### A1 — New Step 3: Enumerate callers of new public entry points (MANDATORY)

**What**: Insert a new mandatory step between current Step 2 (Verify each task)
and current Step 3 (Run verifications). Renumbers old 3→4, 4→5, 5→6.

**Rule text** (EN):

```
### Step 3 — Enumerate callers of new public entry points (MANDATORY)

For each public method, endpoint, or component the Charter ADDS (not modifies —
adds), run a call-site search across production code (excluding tests) and state
the count explicitly in your report.

- **Zero production callers** is a **High** finding by default, with no judgement
  required: the Charter added a capability that nothing reaches. Category:
  `implementation_gap`.
- **Non-zero count**: check that the callers are the INTENDED ones. An existing
  overload, a legacy path, or a synchronous sibling may still be winning — the
  new method exists and is called, but the call sites that matter still route
  through the old path. That is also a finding (High if the old path bypasses
  the Charter's purpose).
- **Where to search**: the full production source tree, not just the `git_range`.
  The callers that matter may live in files untouched by this Charter — that is
  precisely the defect this step catches.

> *Why this step exists.* A 4-auditor cycle missed a feature that was entirely
> unreachable: the Charter added `ResolverAsync` (the only method consulting
> elevation state), all 8 consumers still called the synchronous `Resolver`.
> Three auditors verified the mechanism (exists, is correct, has tests) without
> asking the adjacent question — who calls this? One grep would have caught it.
> This step makes that grep mandatory. (#382)
```

**Rule text** (ES): same structure, translated. Key terms:
- "call-site search" → "búsqueda de call-sites"
- "production callers" → "llamadores en producción"
- "implementation_gap" stays in English (schema key)

**Why this position**: after per-task verification (Step 2) and before running
commands (Step 4). It's a static-analysis check that doesn't require command
execution, so it works even when Step 4 is skipped.

### A2 — Enhance Step 2.6: consolidated test seam check

**What**: Add a paragraph to existing Step 2 sub-step 6 (Check verification
fidelity), after the existing text about proxy verification.

**Rule text** (EN, appended to sub-step 6):

```
  When a test is documented as "consolidated" or "merged" into another test,
  verify the replacement exercises the SAME SEAM, not merely the same unit.
  A Charter's own closing notes declaring "coverage equivalent" are a claim
  by the audited party — treat them as a hypothesis to verify, not as evidence.
  The audited party writes those notes; that is a structural conflict of
  interest the audit must correct for.
```

**Why here**: sub-step 6 already covers "verification fidelity" — checking claims
against reality. The consolidated-test pattern is a specific instance where the
claim comes from the Charter itself, making it easier to miss.

### A3 — Enhance Step 4 (renumbered): red gate enumeration

**What**: Add a paragraph to the renumbered Step 4 (was Step 3 — Run
verifications), after the existing text about running commands.

**Rule text** (EN, appended after the stack examples):

```
When a verification gate is red (a test fails, a lint check errors, a build
breaks), do not stop at reporting the failure. Enumerate what ONLY that gate
could have caught — what class of defect was it protecting against? A broken
guard test reported as a "config defect" without asking what it was protecting
is a missed finding. The gate's purpose is part of the finding's evidence.
```

**Why here**: this is about how to interpret command output, which is what
Step 4 (was 3) is about.

---

## Track B: AILOG + Charter verification quality

The second-order finding: the implementing agent built a verification method
that could not produce a red result (summed pass counts, never checked failures),
then trusted it 7 times. This is the implementer's side — the audit prompt
fixes the auditor's side.

### B1 — AILOG template Verification section

**Files**: `dist/.straymark/templates/TEMPLATE-AILOG.md`,
`dist/.straymark/templates/i18n/es/TEMPLATE-AILOG.md`,
`dist/.straymark/templates/i18n/zh-CN/TEMPLATE-AILOG.md`

**Change**: enhance the "Tests pass" checkbox with guidance about verification
method quality.

EN:
```markdown
- [ ] Tests pass — declare the exact command run. A verification that cannot
  produce a negative result is not verification: summing pass counts without
  checking failure output is a known anti-pattern (#382). Prefer the test
  runner's exit code or summary verdict over custom parsing.
```

ES:
```markdown
- [ ] Tests pass — declara el comando exacto ejecutado. Una verificación que
  no puede producir un resultado negativo no es verificación: sumar passes sin
  revisar fallos es un anti-patrón conocido (#382). Prefiere el exit code del
  runner o su veredicto final sobre parsing custom.
```

zh-CN:
```markdown
- [ ] 测试通过 — 声明执行的确切命令。无法产生阴性结果的验证不是验证：
  只统计通过数而不检查失败输出是已知的反模式 (#382)。
  优先使用测试运行器的退出码或总结判定，而非自定义解析。
```

### B2 — Charter template Verification section note

**File**: `dist/.straymark/templates/charter/charter-template.md`

**Change**: add a blockquote note after the "Local checks" section intro.

```markdown
> **Verification quality**: a check that cannot produce a red result is not
> a check. Summing pass counts without inspecting failure output is the
> canonical anti-pattern (#382). Prefer the test runner's own exit code or
> summary verdict over custom parsing of intermediate output.
```

---

## File inventory

| File | Track | Change |
|------|-------|--------|
| `.straymark/audit-prompts/audit-prompt.md` | A1+A2+A3 | New Step 3, enhance 2.6, enhance new Step 4 |
| `.straymark/audit-prompts/i18n/es/audit-prompt.md` | A1+A2+A3 | Same in Spanish |
| `dist/.straymark/templates/TEMPLATE-AILOG.md` | B1 | Enhance "Tests pass" checkbox |
| `dist/.straymark/templates/i18n/es/TEMPLATE-AILOG.md` | B1 | Same in Spanish |
| `dist/.straymark/templates/i18n/zh-CN/TEMPLATE-AILOG.md` | B1 | Same in Chinese |
| `dist/.straymark/templates/charter/charter-template.md` | B2 | Add verification quality note |
| `CHANGELOG.md` | — | Unreleased entry |

**7 files modified, 0 created.**

---

## Implementation order

1. **A1** (new Step 3 in EN audit prompt) — the main rule, biggest structural change
2. **A2 + A3** (EN enhancements) — same file, do together with A1
3. **A1+A2+A3 in ES** — mirror the EN changes
4. **B1** (AILOG templates EN/ES/zh-CN) — small, independent
5. **B2** (Charter template note) — small, independent
6. **CHANGELOG** — single entry covering all

## Verification

- No code changes — all changes are markdown templates. `cargo test` unaffected.
- Verify EN and ES audit prompts have matching structure (same steps, same numbering).
- Verify AILOG templates render correctly in all three locales.
- Grep for `#382` to confirm all references are present.

## Deferred

- **Blog post** about the case (requested in #382) — separate content task, not
  blocking the prompt fix.
- **Version bump** — after this PR merges, as planned.
