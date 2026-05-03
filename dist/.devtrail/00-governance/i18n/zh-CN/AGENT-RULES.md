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

*DevTrail v4.6.0 | [Strange Days Tech](https://strangedays.tech)*
