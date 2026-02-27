# Provider Migration Tracker

> 用途：按批次跟踪 Provider 收敛进度（配合 `provider-optimization-plan.md`）
> 更新：2026-02-28

---

## 1) 批次总览

| Batch | 范围 | 目标 | 状态 | 负责人 | PR |
|---|---|---|---|---|---|
| B1 | Tier 1（首批 8-15 个） | 统一宏路径 + 错误映射 | DONE | main 历史收敛 | 已合入 |
| B2 | Tier 1（次批 8-15 个） | 统一配置 + 流式处理 | DONE | main 历史收敛 | 已合入 |
| B3 | Tier 2（hooks） | 宏 + patch hooks | IN_PROGRESS | main | 持续中 |
| B4 | Tier 3（特例） | 手写实现规范化 | IN_PROGRESS | main | 持续中 |
| B5 | 收尾 | 删除兼容层 + CI 守卫 | IN_PROGRESS | main | 守卫已接入，兼容层继续收敛 |

---

## 2) Provider 级跟踪表（当前关键样本）

> 状态：`TODO` / `IN_PROGRESS` / `DONE` / `BLOCKED`

| Provider | Tier | 当前实现模式 | 目标实现模式 | Config Canonical | Error Canonical | Streaming Canonical | Registry/Factory 对齐 | 测试通过 | 状态 | 备注 |
|---|---|---|---|---|---|---|---|---|---|---|
| groq | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | 已走 catalog-first |
| xai | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | 已走 catalog-first |
| openrouter | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | 已走 catalog-first |
| deepseek | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | 已走 catalog-first |
| moonshot | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | 已走 catalog-first |
| aiml_api | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B1 测试覆盖 |
| anyscale | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B1 测试覆盖 |
| bytez | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B1 测试覆盖 |
| comet_api | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B1 测试覆盖 |
| compactifai | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B2 测试覆盖 |
| aleph_alpha | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B2 测试覆盖 |
| yi | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B2 测试覆盖 |
| lambda_ai | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B2 测试覆盖 |
| ovhcloud | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B3 样本覆盖 |
| maritalk | 1 | catalog -> OpenAILike | 统一骨�� | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B3 样本覆盖 |
| siliconflow | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B3 样本覆盖 |
| lemonade | 1 | catalog -> OpenAILike | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | B3 样本覆盖 |
| openai | 3 | 手写 | 手写规范化 | ☑ | ☑ | ☑ | ☑ | ☑ | IN_PROGRESS | `from_config_async` 直接分支 |
| anthropic | 3 | 手写 | 手写规范化 | ☑ | ☑ | ☑ | ☑ | ☑ | IN_PROGRESS | 非 OpenAI 协议 |
| bedrock | 3 | 手写 | 手写规范化 | ☑ | ☑ | ☑ | ☑ | ☑ | IN_PROGRESS | SigV4 特例 |
| azure | 2 | 混合 | 宏+hooks / 手写规范化 | ☑ | ☑ | ☑ | ☑ | ☑ | IN_PROGRESS | 已移除 `AzureError` 兼容别名，统一 `ProviderError` |
| cloudflare | 2 | 直接分支 | 手写规范化 | ☑ | ☑ | ☑ | ☑ | ☑ | IN_PROGRESS | `from_config_async` 直接分支 |
| vllm | 1 | catalog(local, skip_api_key) | 统一骨架 | ☑ | ☑ | ☑ | ☑ | ☑ | DONE | 已支持无 key 创建（env + validate） |

---

## 3) 验证记录（2026-02-28）

- `cargo test test_provider_type_from_display_consistency --lib` ✅
- `cargo test test_b1_first_batch_create_provider_from_name --lib` ✅
- `cargo test test_b2_second_batch_create_provider_from_name --lib` ✅
- `cargo test test_b3_third_batch_create_provider_from_name --lib` ✅
- `cargo test test_create_provider_tier1_catalog_creates_openai_like --lib` ✅
- `cargo test --test lib integration::provider_factory_tests` ✅（14 passed）
- `cargo test test_catalog_entries_are_supported_selectors --lib` ✅
- `cargo test test_catalog_entries_are_creatable_via_factory --lib` ✅
- `cargo test core::router::gateway_config::tests:: --lib` ✅
- `bash scripts/guards/check_schema_duplicates.sh` ✅
- `cargo test core::providers::azure:: --lib` ✅（399 passed）
- `cargo test core::providers::openai_like::streaming::tests:: --lib` ✅
- `cargo test core::providers::together:: --lib` ✅
- `cargo test core::providers::watsonx:: --lib` ✅
- `cargo check --all-features` ✅
- `cargo test --all-features --lib` ✅（10213 passed）
- `cargo test --all-features --test lib` ✅（126 passed）

---

## 4) 未完成项（下一步）

1. **B3/B4 深化**：继续将 Tier 2/3 provider 的错误映射、流式和配置路径规范化，减少局部差异实现。
2. **B5 收尾**：schema 重复检查守卫已接入；继续删除剩余 legacy 兼容层。
3. 将本跟踪表扩展到全量 provider（当前先覆盖关键路径与代表性样本）。
