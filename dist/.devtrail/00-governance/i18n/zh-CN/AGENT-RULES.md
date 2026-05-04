# AI 代理规则 - DevTrail

> 本文档定义了所有 AI 代理在 DevTrail 管理的项目中工作时必须遵循的规则。

**语言**: [English](../../AGENT-RULES.md) | [Español](../es/AGENT-RULES.md) | 简体中文

---

## 1. 强制身份识别

### 会话开始时

每个代理必须以以下信息进行自我识别：
- 代理名称（例如：`claude-code-v1.0`、`cursor-v1.0`、`gemini-cli-v1.0`）
- 代理版本（如可用）

### 在每份文档中

在 frontmatter 中包含：
```yaml
agent: agent-name-v1.0
confidence: high | medium | low
```

---

## 2. 何时需要记录文档

### 强制要求 - 创建文档

| 场景 | 类型 | 备注 |
|------|------|------|
| 代码复杂度超过阈值 | AILOG | 运行 `devtrail analyze <changed-files> --output json`。如果 `summary.above_threshold > 0`，创建 AILOG（默认阈值：8）。**后备方案**：如果 CLI 不可用，应用 >20 行业务逻辑启发式规则 |
| 在 2 个以上技术方案之间做决策 | AIDEC | 记录备选方案 |
| 身份认证/授权/PII 相关变更 | AILOG + ETH | `risk_level: high`，ETH 需要审批 |
| 公共 API 或数据库 Schema 变更 | AILOG | `risk_level: medium+`，考虑 ADR |
| ML 模型或 AI 提示词变更 | AILOG | `risk_level: medium+`，需要人工审查 |
| 与外部服务集成 | AILOG | - |
| 添加/移除/升级安全关键依赖 | AILOG | 需要人工审查 |
| 影响 AI 系统生命周期的变更（部署、退役） | AILOG + ADR | 需要人工审查 |
| OTel 仪表化变更（spans、attributes、pipeline） | AILOG | 标签 `observabilidad`，参见 §9 |

### 禁止事项 - 不得记录

- 凭证、令牌、API 密钥
- 个人身份信息
- 任何类型的秘密信息

### 可选项 - 无需文档

- 格式变更（空格、缩进）
- 拼写纠正
- 代码注释
- 次要的样式变更

---

## 3. 自主权限

### 可自由创建

| 类型 | 描述 |
|------|------|
| AILOG | 已执行操作的日志 |
| AIDEC | 已做出的技术决策 |

### 创建草稿 → 需要人工审批

| 类型 | 描述 |
|------|------|
| ETH | 伦理审查 |
| ADR | 架构决策 |

### 提议 → 需要人工验证

| 类型 | 描述 |
|------|------|
| REQ | 系统需求 |
| TES | 测试计划 |

### 创建草稿 → 需要人工审批（新类型）

| 类型 | 描述 |
|------|------|
| SEC | 安全评估（`review_required: true` 始终为必需） |
| MCARD | 模型/系统卡片（`review_required: true` 始终为必需） |
| DPIA | 数据保护影响评估（`review_required: true` 始终为必需） |

### 可自由创建（新类型）

| 类型 | 描述 |
|------|------|
| SBOM | 软件物料清单（事实性清单） |

### 仅识别 → 人工确定优先级

| 类型 | 描述 |
|------|------|
| TDE | 技术债务 |
| INC | 事故总结 |

---

## 4. 何时请求人工审查

在以下情况下标记 `review_required: true`：

1. **低置信度**：`confidence: low`
2. **高风险**：`risk_level: high | critical`
3. **安全决策**：任何身份认证/授权相关变更
4. **不可逆变更**：迁移、删除
5. **用户影响**：影响用户体验的变更
6. **伦理问题**：隐私、偏见、无障碍性
7. **ML 模型变更**：模型参数、架构或训练数据的变更
8. **AI 提示词变更**：提示词或代理指令的修改
9. **安全关键依赖**：安全敏感包的添加、移除或升级
10. **AI 生命周期变更**：AI 系统的部署、退役或主要版本变更

---

## 5. 文档格式

### 使用模板

在创建文档之前，加载对应的模板：

```
.devtrail/templates/TEMPLATE-[TYPE].md
```

### 命名规范

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

### 存放位置

| 类型 | 文件夹 |
|------|--------|
| AILOG | `.devtrail/07-ai-audit/agent-logs/` |
| AIDEC | `.devtrail/07-ai-audit/decisions/` |
| ETH | `.devtrail/07-ai-audit/ethical-reviews/` |
| ADR | `.devtrail/02-design/decisions/` |
| REQ | `.devtrail/01-requirements/` |
| TES | `.devtrail/04-testing/` |
| INC | `.devtrail/05-operations/incidents/` |
| TDE | `.devtrail/06-evolution/technical-debt/` |
| SEC | `.devtrail/08-security/` |
| MCARD | `.devtrail/09-ai-models/` |
| SBOM | `.devtrail/07-ai-audit/` |
| DPIA | `.devtrail/07-ai-audit/ethical-reviews/` |

### 标签和关联

在 frontmatter 中填写 `tags` 和 `related` 字段时：

**标签（Tags）：**
- 使用 kebab-case 关键词：`sqlite`、`api-design`、`gnome-integration`
- 每个文档 3 到 8 个标签，描述主题、技术或组件
- 标签支持在 `devtrail explore` 中进行搜索和分类

**关联（Related）：**
- 仅引用其他 **DevTrail 文档** — 使用文件名加 `.md` 扩展名
- 如果文档位于 `.devtrail/` 的子目录中，包含相对路径：`07-ai-audit/agent-logs/daemon/AILOG-2026-02-03-001-file.md`
- 如果文档在同一目录中，仅使用文件名即可
- **不要**在 `related` 中放置任务 ID（T001、US3）、Issue 编号或外部 URL — 请将这些放在文档正文中

---

## 6. 与人类的沟通

### 保持透明

- 解释决策背后的推理过程
- 记录考虑过的备选方案
- 在存在不确定性时坦诚承认

### 保持简洁

- 直奔主题
- 避免不必要的术语
- 适当使用列表和表格

### 保持主动

- 识别潜在风险
- 在明显时建议改进
- 提醒技术债务

---

## 7. 错误处理

如果代理犯了错误：

1. **记录**错误到 AILOG 中
2. **解释**出了什么问题
3. **提出**纠正方案
4. **标记** `review_required: true`

---

## 8. 文档更新

### 创建新文档 vs 更新现有文档

| 场景 | 操作 |
|------|------|
| 小幅修正 | 更新现有文档 |
| 重大变更 | 创建新文档 |
| 过时文档 | 标记为 `deprecated` |
| 完全替换 | 创建新文档 + 将旧文档标记为 `superseded` |

### 更新时

- 更新 frontmatter 中的 `updated` 字段
- 如果存在历史记录部分，添加备注
- 保持与关联文档的一致性

---

## 9. 可观测性（OpenTelemetry）

在使用 OpenTelemetry 的项目中工作时：

### 规则

- **不要**在 OTel 属性或日志中捕获 PII、令牌或秘密信息
- **记录**仪表化管道变更（新 spans、变更的 attributes、Collector 配置）到 AILOG 中，使用标签 `observabilidad`
- 在分布式项目中采用 OTel 时**创建** AIDEC 或 ADR — 记录采用决策和后端选择
- 当变更涉及 OTel 仪表化时，在 frontmatter 中**设置** `observability_scope`

### 文档触发条件

| 变更 | 文档 | 附加说明 |
|------|------|----------|
| 新 spans 或变更的 attributes | AILOG | 标签 `observabilidad` |
| OTel 后端选择 | AIDEC 或 ADR | 如果是分布式系统 |
| Collector 管道配置 | AILOG | 标签 `observabilidad` |
| 采样策略变更 | AIDEC | 记录理由 |
| 可观测性需求 | REQ | 使用可观测性需求部分 |
| 链路传播测试 | TES | 使用可观测性测试部分 |
| 包含链路证据的事故 | INC | 在时间线中包含 trace_id/span_id |
| 仪表化债务 | TDE | 标签 `observabilidad` |

---

## 10. 架构图（C4 模型）

在创建涉及架构变更的 ADR 文档时：

- **包含**适当层级的 Mermaid C4 图
- **使用** `C4Context` 用于系统级决策（谁使用系统、外部依赖）
- **使用** `C4Container` 用于服务/容器级决策（应用、数据库、消息队列）
- **使用** `C4Component` 用于内部模块决策（服务内的组件）
- **参见** `00-governance/C4-DIAGRAM-GUIDE.md` 获取语法参考和示例

> 图表对于次要决策是可选的。当决策改变系统边界、引入新服务或修改服务间通信时使用它们。

---

## 11. API 规范追踪

当变更修改 API 端点时：

- **验证**相应的 OpenAPI 或 AsyncAPI 规范已更新
- **引用**规范路径到 AILOG 或 ADR 中，使用 `api_spec_path` 字段（在 REQ 中）或 `api_changes` 字段（在 ADR 中）
- **记录**破坏性 API 变更到 ADR 中，设置 `risk_level: high`

---

## 12. 审计检查点（Charter 工作流）

在与人共同实现 Charter 时，Agent **主动**在工作流的特定时刻提议外部多模型审计。该检查点是**软性**的——它从不阻塞 `charter close`，也不会升级到强制执行。外部审计在设计上是 opt-in 的（成本，对操作员主要纪律的信任）。

### 何时发出检查点

当**四个**触发条件同时为真时，**每个 Charter 仅发出一次**检查点：

1. Charter 处于 `in-progress` 或 `declared` 状态（非 `closed`）。
2. Charter 的 `## Tasks` 节中所有任务被标记为 `[x]` 已完成（或 Agent 刚完成最后一个）。
3. `devtrail charter drift <CHARTER-ID>` 退出码为 0（无未计入的漂移）。
4. Developer **尚未**调用 `devtrail charter close <CHARTER-ID>`，也未提及关闭意图。

如果 developer 在同一 Charter 的之前轮次中拒绝了审计，**不要在同一对话的后续轮次中重新发出**。

### 检查点消息的形式

按以下格式渲染消息（替换 `<CHARTER-ID>` 和推荐理由）：

```
到达 <CHARTER-ID> 的检查点。实现已完成，drift check OK，
仅待执行 `devtrail charter close`。

此时你可以运行外部审计（典型为 2 个不同族的 LLM + 1 个校准器），
该审计会对实现产出跨模型 findings。

我的建议：[是 / 否]，因为：
  - <基于 Charter、AILOGs 或 diff 的具体原因>

如果决定审计：
  运行 /devtrail-audit-prompt <CHARTER-ID>，我会在此处直接展示
  两个 prompts。当你保存了外部审计员的回复到规范路径后，运行
  /devtrail-audit-review <CHARTER-ID>，我会在本地校准并将
  findings 合并到 Charter 遥测中。

如果决定不审计：
  准备好后继续 `devtrail charter close <CHARTER-ID>`。外部审计
  完全可选——DevTrail 的声明性 Charter + drift check + AILOG
  纪律已为周期提供了足够严格的关闭信心。
```

### 是/否推荐的启发式

这些是启发式，不是硬性规则——你接近上下文，可与 adopter 一起细化。

**当满足以下任一条件时推荐"是"**：

- Charter 触及安全关键面（auth、RLS、secret 处理、IAM）。
- Charter 引入了 developer 之前未共同实现过的新组件（非重构）。
- 某关联的 AILOG 记录了 `R<N>`，其 `confidence: low | medium` 且 `risk_level: medium` 或更高。
- Developer 将 Charter 标记为 `effort_estimate: L` 且这是 adopter 的第一个 Charter。
- Developer 在 Charter trigger 中**明确**要求跨模型验证。
- **结构性复杂度信号** *(仅当 CLI 编译启用了 `analyze` feature 时可用，对官方二进制为真)*：`range` 中的 diff 引入或修改了至少一个函数，其认知复杂度超过 `.devtrail/config.yml` 中配置的 `complexity.threshold`（默认 `8`）的 **2 倍**（即 ≥ `17`）。一个新的密集函数恰好是两个跨族审计员能捕获 implementation gap、单一模型会遗漏的场景。**优雅降级**：如果二进制缺少 `analyze` feature，静默跳过此信号——不警告，不提及缺失。

**当以下条件**全部**满足时推荐"否"**：

- Charter 是重构或文档变更（无新行为）。
- `effort_estimate` 为 `XS` 或 `S`。
- 所有关联 AILOGs 的 `confidence` 均为 `high`，无涌现的 `R<N+1>` 风险。
- Charter 的 `risk_level` 为 `low`（或未设置）。

**默认情况（无明显信号）**：推荐**"否"**，使用中性措辞（"我没有看到具体信号能正当化两个额外模型的成本；准备好就关闭吧"）。外部审计的成本是真实的——不要靠惯性推荐"是"来虚胖采用。

### 行为规则

- 检查点在同一 Charter 内一旦 developer 回复就**永不**重复。
- 检查点**不**阻塞任何后续操作。如果 developer 忽略它并运行 `charter close`，close 正常进行——没有强制执行，将来也不会有（这是 v0+v1 永久设计决策；见 `Propuesta/devtrail-audit-skills.md` §2.2）。
- 检查点**不**计入任何质量度量。`devtrail metrics` 中没有"已审计 Charter 百分比"KPI——按设计，避免产生虚胖审计计数的激励。
- 如果 developer 接受审计，接下来的两个 skills（`/devtrail-audit-prompt` 然后 `/devtrail-audit-review`）会推进工作流。

---

*DevTrail v4.7.1 | [Strange Days Tech](https://strangedays.tech)*
