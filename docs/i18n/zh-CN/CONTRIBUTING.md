# 为 StrayMark 做贡献

感谢你有兴趣为 StrayMark 做贡献！本文档提供贡献者的指南和相关信息。

**语言**: [English](../../../CONTRIBUTING.md) | [Español](../es/CONTRIBUTING.md) | 简体中文

## 目录

- [行为准则](#行为准则)
- [贡献者许可协议 (CLA)](#贡献者许可协议-cla)
- [如何贡献？](#如何贡献)
- [开发环境设置](#开发环境设置)
- [Pull Request 流程](#pull-request-流程)
- [风格指南](#风格指南)
- [文档标准](#文档标准)

---

## 行为准则

本项目遵循我们的[贡献者契约行为准则](../../../CODE_OF_CONDUCT.md)（[简体中文](CODE_OF_CONDUCT.md)）。参与即表示你同意遵守此准则。

简而言之：在所有互动中保持尊重、包容和建设性。骚扰、歧视和恶意行为不被容忍。请在贡献前阅读[完整行为准则](CODE_OF_CONDUCT.md)。

---

## 贡献者许可协议 (CLA)

本项目要求所有贡献者在 Pull Request 合并前签署**贡献者许可协议 (CLA)**。我们使用 [CLA Assistant](https://cla-assistant.io/) 管理此流程。

### 工作流程

1. 当你提交第一个 Pull Request 时，CLA Assistant 会自动发布评论，要求你签署 CLA。
2. 点击评论中的链接查看并签署协议。
3. CLA 只需签署一次——涵盖你对本项目的所有未来贡献。
4. 签署后，CLA Assistant 将更新 PR 检查状态，你的贡献即可进入审查流程。

如果你对 CLA 有疑问，请开启一个[讨论](https://github.com/StrangeDaysTech/straymark/discussions)。

---

## 如何贡献？

### 报告 Bug

在创建 Bug 报告之前，请检查现有 Issue 以避免重复。

**报告 Bug 时，请包含：**

- 清晰、描述性的标题
- 重现行为的步骤
- 预期行为
- 实际行为
- 截图（如适用）
- 环境详情（操作系统、AI 平台等）

### 建议功能

欢迎功能建议！请包含：

- 功能的清晰描述
- 它解决的问题
- 可能的实现方案
- 你考虑过的替代方案

### 改进文档

文档改进非常受重视：

- 修正错别字或不清晰的措辞
- 添加示例
- 改进说明
- 翻译为其他语言

### 提交代码

代码贡献应该：

- 修复 Bug 或实现功能
- 包含适当的测试（如适用）
- 遵循项目的风格指南
- 根据需要更新文档

---

## 开发环境设置

### 前提条件

- **Git**
- **文本编辑器**（推荐 VS Code）
- **Rust 工具链**（用于 CLI 开发——通过 [rustup.rs](https://rustup.rs/) 安装）
- **Node.js 20+**（可选，用于 markdownlint）

### 设置步骤

1. **Fork 仓库**

   在 [GitHub 仓库页面](https://github.com/StrangeDaysTech/straymark) 点击 "Fork"。

2. **克隆你的 Fork**
   ```bash
   git clone https://github.com/your-username/straymark.git
   cd straymark
   ```

3. **安装 pre-commit 钩子**
   ```bash
   echo 'straymark validate --staged' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

4. **安装开发工具（可选）**
   ```bash
   # Markdown 检查
   npm install -g markdownlint-cli
   ```

5. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

6. **进行修改并验证**
   ```bash
   straymark validate
   ```

---

## Pull Request 流程

### 提交前

- [ ] 成功运行 `straymark validate`
- [ ] 根据需要更新文档
- [ ] 将自己添加到 CONTRIBUTORS.md（如适用）
- [ ] 编写清晰的 PR 描述

### PR 标题格式

使用约定式提交格式：

```
type(scope): description

Examples:
feat(templates): add template for security reviews
fix(validation): correct regex for file naming
docs(readme): clarify installation steps
chore(ci): update GitHub Actions workflow
```

**类型：**
- `feat` - 新功能
- `fix` - Bug 修复
- `docs` - 文档变更
- `chore` - 维护任务
- `refactor` - 代码重构
- `test` - 测试添加或修复

### PR 描述模板

```markdown
## 摘要
变更的简要描述

## 动机
为什么需要这个变更？

## 变更内容
- 变更 1
- 变更 2

## 测试
这些变更是如何测试的？

## 检查清单
- [ ] `straymark validate` 通过
- [ ] 文档已更新
- [ ] 未包含敏感信息
```

### 审查流程

1. 维护者将审查你的 PR
2. 处理任何请求的变更
3. 批准后，维护者将合并

---

## 风格指南

### Markdown

- 使用 ATX 风格标题（`#`、`##` 等）
- 使用带语言标识符的围栏代码块
- 使用表格展示结构化数据
- 实际操作中尽量保持行宽在 120 字符以内
- 使用空行分隔章节

### YAML Front-matter

```yaml
---
id: TYPE-YYYY-MM-DD-NNN
title: Clear, descriptive title
status: draft | accepted | deprecated
created: YYYY-MM-DD
# Additional fields as needed
---
```

### 文件命名

StrayMark 文档：
```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

- 描述部分使用小写字母
- 使用连字符分隔单词
- 描述保持简洁但清晰

### 脚本中的代码

- 使用清晰的变量名
- 为复杂逻辑添加注释
- 遵循 shell/PowerShell 最佳实践

---

## 文档标准

### 添加新的文档类型

如果你要提议新的文档类型：

1. **创建模板**
   - 将 `TEMPLATE-NEWTYPE.md` 添加到 `dist/.straymark/templates/`
   - 遵循现有模板模式

2. **更新治理文档**
   - `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md`
   - `dist/.straymark/00-governance/AGENT-RULES.md`
   - `dist/.straymark/QUICK-REFERENCE.md`

3. **更新 Agent 配置**
   - `dist/dist-templates/directives/`（分发模板）

4. **更新验证**
   - 在 CLI 验证逻辑中添加新类型（`cli/src/commands/validate.rs`）
   - `dist/.github/workflows/docs-validation.yml`

5. **记录变更**
   - 创建 ADR 说明新类型
   - 根据需要更新 README

### 编写模板

模板应包含：

- 完整的 YAML front-matter，包含所有字段
- 清晰的章节标题
- 占位文本说明每个章节应填写的内容
- 在有帮助的地方提供示例

### 编写治理文档

- 表述清晰无歧义
- 使用表格展示参考信息
- 包含示例
- 保持规则可操作

---

## CLI 开发

StrayMark CLI 使用 Rust 编写，位于 `cli/` 目录。

### 编译

```bash
cd cli
cargo build
```

### 运行测试

```bash
cd cli
cargo test
```

### Release 构建

```bash
cd cli
cargo build --release
```

Release 二进制文件经过 LTO 优化并精简以获得最小体积。

### 架构

```
cli/src/
├── main.rs              # 入口点 + clap CLI 定义
├── commands/
│   ├── mod.rs           # 子命令路由
│   ├── init.rs          # straymark init [path]
│   ├── update.rs        # straymark update (组合)
│   ├── update_framework.rs # straymark update-framework
│   ├── update_cli.rs    # straymark update-cli
│   ├── remove.rs        # straymark remove [--full]
│   ├── status.rs        # straymark status [path]
│   └── about.rs         # straymark about
├── config.rs            # 配置和校验和管理
├── download.rs          # GitHub Releases API（前缀过滤）
├── inject.rs            # 指令文件注入（标记）
├── manifest.rs          # dist-manifest.yml 解析
├── platform.rs          # 操作系统/架构检测，用于二进制下载
├── self_update.rs       # CLI 二进制自更新逻辑
└── utils.rs             # 辅助函数（哈希、颜色、路径）
```

> **注意**：Framework 和 CLI 使用独立版本管理（`fw-*` 和 `cli-*` 标签）。详情参见 [CLI 参考手册](adopters/CLI-REFERENCE.md#版本管理)。

---

## 有疑问？

如果你对贡献有疑问：

1. 查看现有 [Issues](https://github.com/StrangeDaysTech/straymark/issues)
2. 查看 [Discussions](https://github.com/StrangeDaysTech/straymark/discussions)
3. 开启新的 Discussion 提问
4. 为特定 Bug 或功能开启 Issue

---

## 致谢

贡献者将在以下位置获得认可：

- GitHub 贡献者图表
- 重要贡献的发布说明
- CONTRIBUTORS.md（针对持续贡献者）

感谢你帮助改进 StrayMark！

---

*StrayMark — 因为每一次变更都值得被记录。*

[Strange Days Tech](https://strangedays.tech)
