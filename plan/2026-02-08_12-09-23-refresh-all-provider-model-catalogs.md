---
mode: plan
cwd: /Users/lifcc/Desktop/code/AI/gateway/litellm-rs
task: Refresh all provider model catalogs and pricing metadata to latest official versions
complexity: complex
planning_method: builtin
created_at: 2026-02-08T04:09:23Z
---

# Plan: Refresh All Provider Model Catalogs

## 0. 目标与约束

- 目标: 在不引入设计分叉、不删除兼容别名的前提下，分批更新所有厂商模型清单、能力和定价映射。
- 约束:
  - 保持向后兼容，不删除现有旧 ID。
  - 每一批改动可独立验证和回滚。
  - 每批完成后必须记录验证结果。

## 1. 全量步骤（文件级）

### Step P1 - 建立厂商盘点与分层优先级

- 状态: `pending`
- 目标:
  - 建立 Tier 1/2/3 厂商清单并冻结本轮范围与非目标。
- 预计改动文件:
  - `docs/`（新增盘点文档，具体路径待定）
  - `src/core/providers`
- 步骤级测试命令:
  - `cargo check`
- 完成标准:
  - Tier 分层及范围边界可追踪、可审阅。

### Step P2 - 建立官方来源映射与抓取方式

- 状态: `pending`
- 目标:
  - 为每个厂商记录官方来源与抓取方式（可脚本化/需人工核对）。
- 预计改动文件:
  - `docs/`（来源映射表，具体路径待定）
- 步骤级测试命令:
  - `cargo check`
- 完成标准:
  - 每个厂商都有可复现的数据来源路径。

### Step P3 - 统一模型元数据更新规范

- 状态: `pending`
- 目标:
  - 统一模型 ID、别名、上下文窗口、能力、默认推荐模型和定价字段规范。
- 预计改动文件:
  - `src/utils/ai/models/pricing.rs`
  - `src/utils/ai/models/utils.rs`
  - `src/utils/ai/tokens.rs`
- 步骤级测试命令:
  - `cargo check`
- 完成标准:
  - 元数据字段规则可复用且一致。

### Step P4 - Wave 1 核心厂商更新

- 状态: `pending`
- 目标:
  - 更新 Anthropic、Gemini/Vertex、Azure OpenAI、xAI、DeepSeek、Mistral、Cohere、Qwen、Meta Llama。
- 预计改动文件:
  - `src/core/providers`
- 步骤级测试命令:
  - `cargo check`
  - `cargo test --lib`
- 完成标准:
  - 核心厂商模型与定价更新可编译、可验证。

### Step P5 - Wave 2 平台与聚合厂商更新

- 状态: `pending`
- 目标:
  - 更新 OpenRouter、VLLM/Hosted VLLM、Ollama、GitHub/Copilot、Snowflake/Watsonx 等入口。
- 预计改动文件:
  - `src/core/providers`
- 步骤级测试命令:
  - `cargo check`
  - `cargo test --lib`
- 完成标准:
  - 聚合平台模型映射与定价规则一致。

### Step P6 - Wave 3 长尾厂商更新

- 状态: `pending`
- 目标:
  - 逐个补齐其余 providers 的模型与别名，不移除旧 ID。
- 预计改动文件:
  - `src/core/providers`
- 步骤级测试命令:
  - `cargo check`
  - `cargo test --lib`
- 完成标准:
  - 长尾厂商覆盖率达到本轮定义范围。

### Step P7 - 按厂商验证矩阵与分批提交

- 状态: `pending`
- 目标:
  - 对每个厂商执行编译、目标测试、定价回归、模型识别回归，并单厂商单提交。
- 预计改动文件:
  - `src/core/cost/calculator.rs`
  - `src/core/cost/utils.rs`
- 步骤级测试命令:
  - `cargo check`
  - `cargo test --tests`
- 完成标准:
  - 每批变更都有可审查验证记录。

### Step P8 - 全量回归与最终对账

- 状态: `pending`
- 目标:
  - 完成全局回归并输出新增/变更/弃用模型清单与风险项。
- 预计改动文件:
  - `docs/`
  - `CHANGELOG.md`
- 步骤级测试命令:
  - `cargo check`
  - `cargo test --lib`
  - `cargo test --tests`
- 完成标准:
  - 全量回归通过，迁移说明完整。

## 2. 执行日志（每步完成后追加）

- Step P1: `pending`
- Step P2: `pending`
- Step P3: `pending`
- Step P4: `pending`
- Step P5: `pending`
- Step P6: `pending`
- Step P7: `pending`
- Step P8: `pending`

## 3. 风险与注意事项

- 官方文档动态页面、反爬或权限限制可能导致抓取不稳定。
- 厂商可能仅提供别名或 region-specific ID，存在冲突映射风险。
- 模型发布时间与定价发布时间可能不同步，可能出现窗口期不一致。
- 一次性大改会降低可审查性，必须坚持按厂商分批推进。

## 4. 参考路径

- `src/core/providers`
- `src/core/providers/openai/models.rs`
- `src/core/providers/bedrock/model_config.rs`
- `src/core/providers/bedrock/utils/cost.rs`
- `src/core/cost/calculator.rs`
- `src/core/cost/utils.rs`
- `src/utils/ai/models/pricing.rs`
- `src/utils/ai/models/utils.rs`
- `src/utils/ai/tokens.rs`
