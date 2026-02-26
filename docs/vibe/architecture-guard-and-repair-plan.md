# LiteLLM-RS 设计问题防再发与修复方案

## 目标
- 消除已发现的 7 类设计问题（路由分叉、配置漂移、Provider 装配不一致等）。
- 建立“代码-配置-文档”一致性守卫，避免同类问题再次发生。

## 范围与约束
- 向后兼容：`required`（优先保持现有 API 路径可用）。
- 提交策略：`per_step`（每步改完即验证并提交）。
- 验证范围：
  - 步骤级：`cargo check` + 定向测试。
  - 最终：`cargo check` + 关键路由/配置相关测试。
- 基线：当前工作区从提交 `03f4fe3` 开始，无预存脏改动。

## 根因归类
1. 多入口装配：Provider 初始化有多条路径，且行为不一致。
2. 架构过渡期未收敛：`ProviderRegistry` 与 `UnifiedRouter` 并存但未统一请求链路。
3. 合同漂移：示例配置与真实 schema 长期未做一致性校验。
4. 失败语义设计不清：启动 fallback、异步任务初始化失败处理不明确。
5. 默认值语义冲突：如 CORS “空列表”注释与实现相反。

## 防再发机制（Guardrails）
1. 单一路径守卫
- 规则：服务启动只允许通过 `create_provider(...)` 构建 Provider。
- 检查：禁止在 server 初始化路径直接调用 `Provider::from_config_async`。

2. 路由一致性守卫
- 规则：模型路由优先走 `UnifiedRouter`（当可用时），`ProviderRegistry` 仅作兼容回退。
- 检查：AI 路由的 provider 选择 helper 统一入口。

3. 配置合同守卫
- 规则：`config/gateway.yaml.example` 必须与 `src/config/models/*` 字段一致。
- 检查：新增/修改字段时同步更新 example，并在 PR 中强制审阅。

4. 安全默认值守卫
- 规则：注释、默认值、运行时行为必须同义；默认值优先安全。
- 检查：CORS/TLS/Auth 等安全配置变更需补充测试。

5. 启动与后台任务可观测守卫
- 规则：初始化失败必须可见（error/warn），不可静默吞错。
- 检查：关键服务（pricing 等）初始化结果必须记录日志。

## 执行步骤（按风险优先级）
1. Provider 装配收敛
- 内容：
  - `enabled=false` 不初始化。
  - 使用配置名注册，避免同类型 provider 覆盖。
  - Server 初始化改为统一调用 `create_provider`。
- 完成标准：同类型多实例可并存且可按前缀选择。

2. UnifiedRouter 真正接线
- 内容：
  - 建立 `GatewayRouterConfig -> RouterConfig` 映射。
  - 启动时构建 `UnifiedRouter`。
  - 模型型请求（chat/completions/embeddings）优先走统一路由选择。
- 完成标准：`router.strategy` 配置对线上选择路径生效。

3. 配置与示例对齐
- 内容：修正 `gateway.yaml.example` 的字段结构与命名，移除过时示例。
- 完成标准：按 example 可直接被当前 schema 解析通过。

4. 启动语义和默认安全值修复
- 内容：
  - 移除误导性的“失败后默认配置启动”语义。
  - 修复 CORS 默认值/注释冲突。
  - pricing 初始化与刷新失败补充日志。
- 完成标准：失败行为可预测且日志可诊断。

5. 回归验证
- 命令：`cargo check` + 配置/路由相关定向测试。
- 完成标准：全部通过并输出 commit 映射。

## 验收标准
- 关键问题关闭：
  - Provider 键覆盖问题关闭。
  - UnifiedRouter 不再是“挂名字段”。
  - example 与 schema 一致。
  - CORS 默认语义明确且安全。
- 文档化：本文件与最终变更摘要可追溯。

## 执行记录（本轮）
1. `6339a11` `docs(vibe): add architecture guard and stepwise repair plan`
- 新增本方案文档。

2. `d630143` `fix(server): unify provider bootstrap and register by configured name`
- `enabled=false` 不再初始化。
- 使用配置名注册 provider，避免同类型覆盖。
- Server 启动统一走 `create_provider(...)`。

3. `0e45597` `feat(router): wire unified router into runtime model selection`
- 接入 `GatewayRouterConfig -> RouterConfig` 映射。
- 启动时构建 `UnifiedRouter`。
- chat/completions/embeddings/images 路由选择支持 UnifiedRouter。

4. `0b41376` `fix(config): align gateway example schema and deterministic startup config loading`
- 重写 `config/gateway.yaml.example` 与当前 schema 对齐。
- 启动配置改为 file -> env -> error，去除误导 fallback。

5. `52a594e` `fix(security): tighten CORS default semantics and log pricing init status`
- CORS 空 origins 改为默认收敛语义（不再等价 allow all）。
- pricing 初始加载与自动刷新启动补充可观测日志。

## 回归命令（本轮）
- `cargo check`
- `cargo test core::router::gateway_config::tests --lib`
- `cargo test config::models::gateway::tests::test_gateway_config_validate_success --lib -- --exact`
- `cargo test config::models::server::tests::test_cors_config_allows_all_origins_empty --lib -- --exact`
- `cargo test config::models::server::tests::test_cors_config_validate_all_origins_with_credentials --lib -- --exact`
