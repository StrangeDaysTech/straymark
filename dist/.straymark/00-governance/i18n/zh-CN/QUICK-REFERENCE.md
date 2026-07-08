# StrayMark - 快速参考

> AI 代理和开发者的单页参考。
>
> **这是一份派生文档** — DOCUMENTATION-POLICY.md 是权威来源。

**语言**: [English](../../QUICK-REFERENCE.md) | [Español](../es/QUICK-REFERENCE.md) | 简体中文

---

## 语言配置

**文件**：`.straymark/config.yml`

```yaml
language: en  # Options: en, es (default: en)
```

| 语言 | 模板路径 |
|------|----------|
| `en` | `.straymark/templates/TEMPLATE-*.md` |
| `es` | `.straymark/templates/i18n/es/TEMPLATE-*.md` |

---

## 命名约定

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

**示例**：`AILOG-2026-03-25-001-implement-oauth.md`

---

## 文档类型（12 种）

### 核心类型（8 种）

| 类型 | 名称 | 目录 | 代理自主权 |
|------|------|------|-----------|
| `AILOG` | AI 操作日志 | `07-ai-audit/agent-logs/` | 自由创建 |
| `AIDEC` | AI 决策 | `07-ai-audit/decisions/` | 自由创建 |
| `ETH` | 伦理审查 | `07-ai-audit/ethical-reviews/` | 仅草稿 |
| `ADR` | 架构决策 | `02-design/decisions/` | 需要审核 |
| `REQ` | 需求 | `01-requirements/` | 提议 |
| `TES` | 测试计划 | `04-testing/` | 提议 |
| `INC` | 事故事后分析 | `05-operations/incidents/` | 协助 |
| `TDE` | 技术债务 | `06-evolution/technical-debt/` | 识别 |

### 扩展类型（4 种）

| 类型 | 名称 | 目录 | 代理自主权 |
|------|------|------|-----------|
| `SEC` | 安全评估 | `08-security/` | 草稿 → 批准（始终） |
| `MCARD` | 模型/系统卡 | `09-ai-models/` | 草稿 → 批准（始终） |
| `SBOM` | 软件物料清单 | `07-ai-audit/` | 自由创建 |
| `DPIA` | 数据保护影响评估 | `07-ai-audit/ethical-reviews/` | 草稿 → 批准（始终） |

### 有界工作单元 — Charter

Charter **不是**文档类型——它包裹一个跨多个会话的实施块。文件名使用顺序前缀（`NN-slug.md`），而不是日期前缀。生命周期：`declared` → `in-progress` → `closed`。

| 概念 | 目录 | 代理自主权 |
|------|------|-----------|
| `Charter` | `.straymark/charters/`（声明式 `NN-slug.md` + 遥测 `NN-slug.telemetry.yaml`） | 通过 `charter new` 搭建脚手架；操作者拥有 trigger 与生命周期的转换 |

> 参见 `STRAYMARK.md` 第 15 节及 `.straymark/00-governance/SPECKIT-CHARTER-BRIDGE.md`，了解粒度启发式、生命周期与 SpecKit ↔ Charter 桥接。

### 一等公民注册表 — Follow-ups Backlog *(fw-4.21.0+)*

follow-ups backlog 同样**不是**文档类型 —— 它是一个单文件注册表,聚合跨 AILOG 的 `§Follow-ups` / `R<N> (new)` 条目。条目 id 为 `FU-NNN`;按触发类型分为五个 bucket;状态为 `open | in-progress | suspected-closed | closed | superseded | promoted`。计数器为 CLI-owned。

| 概念 | 文件 | 代理自主权 |
|------|------|-----------|
| `Follow-ups registry` | `.straymark/follow-ups-backlog.md`（schema: `follow-ups-backlog.schema.v1.json`,实验性） | 代理通过 `followups drift --apply` 提取（pre-commit）;操作者拥有 triage 与提升批准 |

```bash
straymark followups list / status / drift [--apply] / recount / promote FU-NNN
```

> 参见 `STRAYMARK.md` 第 16 节、`FOLLOW-UPS-BACKLOG-PATTERN.md` 及 AGENT-RULES.md §13,了解随框架发布的代理指令。

---

## 何时编写文档

| 场景 | 操作 |
|------|------|
| 复杂代码（`straymark analyze`；回退条件：>20 行） | AILOG |
| 多个备选方案间的决策 | AIDEC |
| 认证/授权/PII 变更 | AILOG + `risk_level: high` + ETH |
| 公共 API 或数据库模式变更 | AILOG + 考虑 ADR |
| 机器学习模型/提示词变更 | AILOG + 人工审核 |
| 安全关键依赖变更 | AILOG + 人工审核 |
| OTel 埋点变更 | AILOG + 标签 `observabilidad` |
| 跨多个会话的实施块（>1 天，跨多个阶段 >5 个任务） | 声明一个 **Charter**（`straymark charter new`） |
| 横向技术债务（先前 Charter 的遗留、横跨多个模块、需要专用 Charter、需要人工优先级） | **TDE** —— 与单 Charter 的 `R<N>` 不同；参见 AGENT-RULES.md §3 |
| 创建或修改了带有 `## Follow-ups` 或 `R<N> (new, not in Charter)` 条目的 AILOG | 在同一个 commit 中运行 `straymark followups drift --apply` —— 参见 AGENT-RULES.md §13 |

**不要记录**：凭据、令牌、PII、机密信息。

---

## 最低元数据

```yaml
---
id: AILOG-2026-03-25-001
title: Brief description
status: accepted
created: 2026-03-25
agent: agent-name-v1.0
confidence: high | medium | low
review_required: true | false
risk_level: low | medium | high | critical
# 可选法规字段（按上下文启用）：
# eu_ai_act_risk: not_applicable
# nist_genai_risks: []
# iso_42001_clause: []
# observability_scope: none
---
```

---

## 需要人工审核

在以下情况下标记 `review_required: true`：

- `confidence: low`
- `risk_level: high | critical`
- 安全决策
- 不可逆变更
- 机器学习模型或提示词变更
- 安全关键依赖变更
- 文档类型：ETH、ADR、REQ、SEC、MCARD、DPIA

---

## 目录结构

```
.straymark/
├── 00-governance/               ← 策略，AI-GOVERNANCE-POLICY.md
├── 01-requirements/             ← REQ
├── 02-design/decisions/         ← ADR
├── 03-implementation/           ← 指南
├── 04-testing/                  ← TES
├── 05-operations/incidents/     ← INC
├── 06-evolution/technical-debt/ ← TDE
├── 07-ai-audit/
│   ├── agent-logs/              ← AILOG
│   ├── decisions/               ← AIDEC
│   └── ethical-reviews/         ← ETH, DPIA
├── 08-security/                 ← SEC
├── 09-ai-models/                ← MCARD
├── charters/                    ← Charter（NN-slug.md + NN-slug.telemetry.yaml）
├── follow-ups-backlog.md        ← Follow-ups 注册表（FU-NNN 条目,自 fw-4.21.0 起为一等公民）
└── templates/                   ← 模板（包括 charter/ 子目录 + follow-ups-backlog.md）
```

---

## 工作流

```
1. 评估    → 这是否需要文档记录？
       ↓
2. 加载    → 对应的模板
       ↓
3. 创建    → 使用正确的命名约定
       ↓
4. 标记    → 如适用则标记 review_required
```

---

## 级别

### 置信度
| 级别 | 操作 |
|------|------|
| `high` | 继续执行 |
| `medium` | 记录替代方案 |
| `low` | `review_required: true` |

### 风险
| 级别 | 示例 |
|------|------|
| `low` | 文档、格式 |
| `medium` | 新功能 |
| `high` | 安全、API |
| `critical` | 生产环境、不可逆 |

---

## 法规对齐

| 标准 | 关键文档 |
|------|----------|
| ISO/IEC 42001:2023 | AI-GOVERNANCE-POLICY.md（核心） |
| EU AI Act | ETH（风险分类）、INC（事件报告） |
| NIST AI RMF / 600-1 | ETH（12 个 GenAI 风险类别）、AILOG |
| GDPR | ETH（数据隐私）、DPIA |
| ISO/IEC 25010:2023 | REQ（质量）、ADR（质量影响） |
| OpenTelemetry | 可选 — 参见 OBSERVABILITY-GUIDE |
| C4 Model | ADR 图表 — 参见 C4-DIAGRAM-GUIDE |

---

## 技能（Claude Code）

| 命令 | 用途 |
|------|------|
| `/straymark-status` | 检查文档状态和合规性 |
| `/straymark-new` | 创建任意类型文档（交互式） |
| `/straymark-ailog` / `/straymark-aidec` / `/straymark-adr` | AILOG / AIDEC / ADR 的快速快捷方式 |
| `/straymark-mcard` / `/straymark-sec` | Model Card / SEC 评估的交互流程 |
| `/straymark-charter-new` | 搭建一个 Charter（声明式事前工作单元） |
| `/straymark-followups` *(fw-4.22.0+)* | 维护 follow-ups backlog 注册表 —— “有什么待办？”、提交前 drift、关闭后分诊/promote |
| `/straymark-audit-prompt CHARTER-XX` *(fw-4.9.0+，在 fw-4.9.0 中重构)* | 外部多模型审计 — 在规范路径写入统一 prompt |
| `/straymark-audit-execute [CHARTER-XX]` *(fw-4.9.0+)* | 在审计员 CLI 中运行 — 读取 prompt，使用 tool use 审计，写入 report |
| `/straymark-audit-review CHARTER-XX` *(fw-4.9.0+，在 fw-4.9.0 中扩展)* | 合并 N 个 reports 为 review.md（6 节）+ YAML 合并入遥测 |

---

## 模式

| 模式 | 文档 |
|------|------|
| Follow-ups backlog（一等公民注册表 + 原生 `followups` CLI） *(fw-4.10.0+,自 fw-4.21.0+ 起为一等公民)* | [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) |
| Polish Charter 作为债务检测("声明了表层但未接线"反模式) *(fw-4.18.0+)* | [POLISH-CHARTER-PATTERN.md](POLISH-CHARTER-PATTERN.md) |

---

*StrayMark fw-4.34.0 | [Strange Days Tech](https://strangedays.tech)*
