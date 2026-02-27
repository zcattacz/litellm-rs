# Provider 优化完整计划（2026-02-26）

> 范围：`src/core/providers/**` 及其配置/路由/SDK 关联层
> 基线输入：`provider-development` skill、`docs/refactor/01-architecture.md`、`docs/refactor/09-provider-consistency.md`

---

## 1. 目标与原则

### 1.1 总体目标

在不改变对外 API 行为的前提下，完成 Provider 层“单一主路径”收敛：

1. **单一错误体系**：统一 `ProviderError` + 通用 HTTP 映射。
2. **单一配置语义**：`BaseConfig + provider-specific fields`。
3. **单一调度路径**：Catalog/Registry/Factory 一致，避免新旧并存分叉。
4. **单一 OpenAI-compatible 实现骨架**：减少重复请求转换、Header 构建、SSE 解析。
5. **可观测可回归**：每个阶段都可 `cargo check/test` 验证并可回滚。

### 1.2 约束原则

- 保持 **Trait Object + 统一错误** 架构（不回退 enum-dispatch 主路径）。
- 优先“低风险收敛”，避免一次性大爆炸重写。
- 只做 provider 体系去重，不扩展无关功能。
- 新增/重构路径必须有对应回归测试。

---

## 2. 当前核心问题（归并）

结合现有分析，问题可归并为 5 类：

1. **模型/类型重复定义**：`ProviderType`、`ProviderConfig`、`RouterConfig` 多份。
2. **基础设施重复实现**：`base_provider.rs` / `shared.rs` / `base/*` 职责重叠。
3. **错误映射重复**：大量 provider 自写 `map_http_error`，未复用统一 mapper。
4. **实现模式碎片化**：宏实现、手写实现、半抽象实现并存。
5. **工厂与注册路径不一致**：enum/dispatch/catalog/factory 覆盖范围不同步。

---

## 3. 目标架构（落地态）

### 3.1 Provider 分层

- **Layer A（Schema）**：统一配置/类型定义（canonical）。
- **Layer B（Base Runtime）**：连接池、重试、错误映射、SSE、认证策略。
- **Layer C（Provider Adapter）**：
  - OpenAI-compatible：统一宏 + hooks。
  - Custom protocol：手写实现（Anthropic/Bedrock/Vertex 等）。
- **Layer D（Registry/Factory）**：catalog-first，单入口创建 provider。

### 3.2 Provider 分类策略

- **Tier 1（OpenAI-compatible）**：优先使用统一宏路径。
- **Tier 2（轻度差异）**：宏 + patch hooks。
- **Tier 3（重度差异协议）**：保留手写实现。

---

## 4. 分阶段执行计划（P0 → P3）

> 执行状态（截至 2026-02-28）
>
> - P0：`completed`（inventory/tracker 文档与基线测试已稳定）
> - P1：`completed`（基础能力主路径收敛到 `base/*` + 统一 mapper）
> - P2：`completed`（Router strategy 已与 runtime canonical 模型对齐，legacy 未支持策略已移除）
> - P3：`in_progress`（Tier 1 catalog-first、streaming 分发去硬编码已落地；openai_like/together/watsonx 的 stream compat wrapper 已移除，Tier 2/3 继续规范化）
> - P4：`in_progress`（catalog/factory/dispatch 一致性守卫与 schema 重复检查守卫已接入 CI，legacy 兼容层删除仍在收尾）

## Phase P0：基线冻结与安全护栏（先做）

### 目标
建立重构“护栏”，避免边改边漂移。

### 任务
1. 生成 Provider 现状清单（目录、实现模式、错误映射方式、流式支持、工厂覆盖）。
2. 固化“必须保持行为一致”的关键路径测试（chat completion / streaming / error mapping）。
3. 定义迁移台账（每个 provider 的迁移状态）。

### 交付物
- `docs/refactor/2-26/provider-inventory.md`
- `docs/refactor/2-26/provider-migration-tracker.md`
- 基线测试报告（命令与结果）

### 验收标准
- 可回答“任一 provider 走哪条实现路径”。
- 基线测试全部通过。

---

## Phase P1：基础设施去重（高收益、低行为风险）

### 目标
先收敛底座，再迁移 provider，降低重复改动成本。

### 任务
1. 合并基础能力到 `base/`：
   - Header 构建统一为 `HeaderPair`。
   - HTTP 错误默认走统一 mapper。
   - 重试/超时策略统一入口（连接池或请求执行器）。
2. 明确 `base_provider.rs` 与 `shared.rs` 职责，删除完全重复能力。
3. 统一成本计算入口（避免多套 calculator 并存）。

### 涉及文件（核心）
- `src/core/providers/base_provider.rs`
- `src/core/providers/shared.rs`
- `src/core/providers/base/connection_pool.rs`
- `src/core/providers/base/config.rs`
- `src/core/providers/base/pricing.rs`

### 验收标准
- Header/HttpError/Retry/Cost 各有且仅有一条推荐实现路径。
- 旧实现仅保留薄适配层（并标记待删除）。

---

## Phase P2：配置与类型统一（降低耦合）

### 目标
消除 Provider 关键 schema 的多份定义。

### 任务
1. 确定 `ProviderType` canonical 定义，其他层做 `From/TryFrom`。
2. 确定 `ProviderConfig` canonical schema：
   - 通用字段归 `BaseConfig`
   - provider 特有字段留本地
3. 统一 Router 侧映射：提取 `ProviderConfig -> RuntimeDeploymentConfig` 单一转换器。
4. 清理重复 `GatewayConfig`/`RouterConfig` 定义（保留主模型 + 适配层）。

### 涉及文件（核心）
- `src/config/models/provider.rs`
- `src/sdk/config.rs`
- `src/core/types/config/provider.rs`
- `src/core/router/config.rs`
- `src/config/models/router.rs`
- `src/core/router/gateway_config.rs`

### 验收标准
- 新增 provider 字段只需修改一处 canonical schema。
- Router 不再手工拼装散落字段。

---

## Phase P3：Provider 实现路径收敛（主工程量）

### 目标
将大多数 OpenAI-compatible provider 迁入统一实现骨架。

### 任务
1. 设定“统一宏 + hook”模板，迁移 Tier 1/Tier 2 provider。
2. 清理“仅类型别名 error.rs”“重复 map_http_error”类噪音文件。
3. 统一流式处理：
   - OpenAI-compatible 默认使用 `base/sse.rs`。
   - 非标准协议保留自定义 transformer。
4. 工厂路径统一：catalog-first + factory/registry/dispatch 同步生成。

### 涉及文件（核心）
- `src/core/providers/macros.rs`
- `src/core/providers/mod.rs`
- `src/core/providers/registry/catalog.rs`
- `src/core/providers/registry/mod.rs`
- `src/core/providers/*/provider.rs`（批量迁移）
- `src/core/providers/*/error.rs`（批量清理）
- `src/core/providers/*/streaming.rs`（批量收敛）

### 验收标准
- OpenAI-compatible provider 中 ≥80% 使用统一骨架。
- dispatch/factory/registry 覆盖清单一致，无“实现了但不可调度”provider。
- `chat_completion_stream` 不再硬编码小名单。

---

## Phase P4：收尾与删除兼容层

### 目标
移除临时适配层，完成“单路径”。

### 任务
1. 删除已无引用的 legacy 兼容代码。
2. 文档更新：新增 provider 必须遵循统一模板。
3. 加入 CI 守卫：
   - Provider 覆盖一致性检查（catalog vs factory vs dispatch）
   - 禁止新增重复 schema（lint/脚本检查）

### 验收标准
- 无死代码兼容层。
- CI 可阻止重复设计回流。

---

## 5. PR 切分建议（可执行）

- **PR-1（P0）**：基线清单 + 测试护栏（不改行为）
- **PR-2（P1）**：基础设施去重（base/shared）
- **PR-3（P2）**：配置与类型统一
- **PR-4~N（P3）**：按 provider 批次迁移（每批 8~15 个）
- **最终 PR（P4）**：删除兼容层 + CI 守卫

> 每个 PR 保持“可回滚、可单独通过测试、可独立审阅”。

---

## 6. 风险与应对

### 高风险点
1. **错误语义变化**：同状态码映射结果变化会影响上游重试策略。
2. **流式兼容性**：SSE chunk 细节不一致会导致前端解析异常。
3. **工厂覆盖缺口**：迁移中可能出现“目录存在但不可创建”。

### 应对策略
- 先写兼容测试再迁移。
- 关键 provider（OpenAI/Anthropic/Azure/Bedrock/Groq）作为金丝雀分批验证。
- 每批迁移后执行全量 `cargo check` + 目标集成测试。

---

## 7. 度量指标（Definition of Done）

### 结构指标
- Provider 实现模式由 5+ 种收敛到 2 种（统一宏 / 手写特例）。
- 重复配置模型减少到“1 canonical + 适配转换”。
- `error.rs` 噪音文件显著减少（仅保���有真实差异者）。

### 质量指标
- `cargo check --all-features` 通过。
- `cargo test --all-features` 通过。
- provider 覆盖一致性检查通过（catalog/factory/dispatch 对齐）。

### 维护性指标
- 新增一个 OpenAI-compatible provider 的改动文件数明显下降。
- “新增字段需改多处”的场景减少到单点修改。

---

## 8. 里程碑建议

- **M1**：完成 P0 + P1，底座统一。
- **M2**：完成 P2，schema 与路由映射统一。
- **M3**：完成 P3，主量 provider 收敛。
- **M4**：完成 P4，兼容层删除 + CI 守卫上线。

---

## 9. 后续执行顺序（建议）

1. 先做 P0（台账和护栏），避免盲改。
2. 立即做 P1（收益最大、风险可控）。
3. P2 与 P3 并行推进：
   - 一条线做 schema 统一；
   - 一条线按 provider 批次迁移。
4. 最后 P4 清理与封板。

---

## 10. 附：本计划与现有文档映射

- 架构重复问题：`docs/refactor/01-architecture.md`
- Provider 一致性问题：`docs/refactor/09-provider-consistency.md`
- 本文定位：**把“问题清单”转为“可执行重构计划”**
