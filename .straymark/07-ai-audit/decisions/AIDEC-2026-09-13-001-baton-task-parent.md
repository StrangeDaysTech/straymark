---
id: AIDEC-2026-09-13-001
title: Conservative explicit parent resolution for Baton tasks
status: review
created: 2026-09-13
agent: codex-cli-v0.154.0
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: [human_ai_config]
iso_42001_clause: []
tags: [baton, inheritance]
related: [07-ai-audit/agent-logs/AILOG-2026-09-13-001-baton-task-inheritance.md]
---

# AIDEC: Explicit task parent

## Context

Issue #427 requests the inheritance ratified in #332 without adding task declaration
slots. One spec may have multiple Charters; filenames and titles cannot assign a task.

## Alternatives

1. Pick the first/newest/active Charter or infer from titles: more coverage, but
   ambiguous ownership could silently recommend a cheaper model.
2. Require exactly one originating_spec relation to the existing sibling spec.md
   inside the project: bounded implementation with explicit evidence, but leaves
   multi-Charter specs and incomplete inventories undeclared.
3. Design a new task-to-Charter mapping: fuller solution, requires a separate schema
   decision and exceeds this first adopter contribution.

## Decision proposed and implemented in the PR

Use option 2. A malformed Charter makes the parent inventory incomplete, so inheritance
is disabled conservatively. Only work_verb and design_provenance flow to the child;
parent effort_estimate is not a measurement of task size. Missing or invalid verbs
remain unclassifiable in the existing classifier; no title fallback or task frontmatter.

## Consequences and review

Preserves deterministic and read-only recommendations. Known false negatives remain
visible as undeclared. Extending mappings or risk policy is separate work. This choice
is implemented for upstream review, not recorded as human acceptance or schema ratification.
