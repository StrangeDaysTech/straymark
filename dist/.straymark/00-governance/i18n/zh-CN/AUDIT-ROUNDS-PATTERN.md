# 审计轮次模式 - StrayMark

> 为需要多于一轮外部审计的 Charter 提供按轮命名。

**语言**: [English](../../AUDIT-ROUNDS-PATTERN.md) | [Español](../es/AUDIT-ROUNDS-PATTERN.md) | 简体中文

---

## 状态

**自 fw-4.35.0 / cli-3.33.0 起**（Issue #341）。可选且向后兼容：单轮 Charter 无需任何新增。

## 问题

外部审计子系统（`straymark charter audit`、`/straymark-audit-execute`、`/straymark-audit-review`）最初假设**每个 Charter 恰好一轮审计**。但多阶段 Charter 是一等概念，按阶段审计是它们的推荐做法。当一个 Charter 需要**多于一轮**外部审计（例如每个代码密集阶段一轮）时，扁平的单轮布局会在两方面失效：

1. **固定扁平路径互相覆盖。** 每轮都把 `audit-prompt.md`、`report-*.md` 和 `review.md` 写入同一个 `.straymark/audits/<CHARTER-ID>/` 目录，因此第二轮会静默覆盖第一轮的 prompt，而保留历史需要手动 `git mv` 归档。
2. **跨轮通配污染。** `report-*.md` 通配（`--merge-reports` 和 review skill 都用）是扁平且非递归的。如果上一轮的 reports 以任何仍匹配 `report-*.md` 的名字扁平存放，它们会被拉进**当前**轮的合并 `review.md` 和遥测中 — 混淆各轮。

## 模式：`--round <label>`

传入可选的轮次标签，将整个三件套命名到按轮的子目录下：

```bash
# 第 1 轮 — 安全阶段
straymark charter audit CHARTER-01 --prepare --round fase-1 --range <阶段1首个commit>..HEAD
# → .straymark/audits/CHARTER-01/fase-1/audit-prompt.md

# ...审计员将 reports 写入同一子目录，然后：
straymark charter audit CHARTER-01 --merge-reports --round fase-1 \
  --merge-into .straymark/charters/CHARTER-01.telemetry.yaml
```

标签必须是简单 slug（`[A-Za-z0-9._-]`，以字母数字开头，无路径分隔符或空格）— 它会成为目录名。

### 结果布局

```
.straymark/audits/CHARTER-01/
  fase-1/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-2/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-3/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
```

因为每轮位于自己的子目录且通配非递归，各轮永不互相覆盖（修复问题 1），合并恰好限定到当前轮的 reports（修复问题 2）。

### 传递标签

同一个 `--round <label>` 贯穿整个三件套 — CLI 的 `--prepare` 指引和各 skill 都会回显它：

- `/straymark-audit-prompt <CHARTER-ID>` → `charter audit --prepare --round <label>`
- `/straymark-audit-execute <CHARTER-ID> --round <label>` → 在子目录下读写
- `/straymark-audit-review <CHARTER-ID> --round <label>` → 仅合并该子目录

## 遥测：多轮共存

每个用 `--round` 合并的 `external_audit` 条目携带 `round:` 字段，使各轮在同一遥测文件中保持可区分：

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

`--merge-into` 会将新一轮**追加**到已填充的 `external_audit:` 块，而非报错 — 前提是轮次标签是新的。重新合并**同一**轮仍会被拒绝（同轮守卫防止静默重复）；在**没有**轮次标签的情况下合并到已填充块也会被拒绝（各轮必须保持可区分）。每轮使用新的 `--round <label>`。

## 向后兼容

完全省略 `--round`，一切行为与 fw-4.35.0 之前完全一致：`.straymark/audits/<CHARTER-ID>/` 下的扁平路径、无 `round:` 字段的单个 `external_audit` 块，以及 `--merge-into` 拒绝任何已填充数组。单轮 Charter（常见情形）无需改动。

## 相关

- [AGENT-RULES.md §12](AGENT-RULES.md) — 界定何时运行外部审计的审计检查点，以及稳定状态与多轮的条目。
- [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) — 姊妹 pattern doc（本文档所镜像的注册表布局约定）。

---

*StrayMark fw-4.35.0 | [Strange Days Tech](https://strangedays.tech)*
