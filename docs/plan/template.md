# Plan Template (Lint-Compatible)

本模板用于创建新的执行计划文件，默认满足 `plan_lint.py` 的结构要求。

## 0. 元信息

- 任务: `<task-name>`
- 仓库: `<absolute-path>`
- 兼容性策略: `<required | not required>`
- 提交策略: `<per_step | milestone | final_only>`

## 1. 全量步骤（文件级）

### Step A1 - `<step-title-1>`

- 状态: `pending`
- 目标:
  - `<what this step achieves>`
- 预计改动文件:
  - `<path/to/file1>`
  - `<path/to/file2>`
- 具体调整:
  - `<change item 1>`
  - `<change item 2>`
- 步骤级测试命令:
  - `cargo check`
  - `<targeted-test-command>`
- 完成标准:
  - `<observable done criteria>`

### Step A2 - `<step-title-2>`

- 状态: `pending`
- 目标:
  - `<what this step achieves>`
- 预计改动文件:
  - `<path/to/file>`
- 具体调整:
  - `<change item>`
- 步骤级测试命令:
  - `cargo check`
  - `<targeted-test-command>`
- 完成标准:
  - `<observable done criteria>`

### Step A3 - `<step-title-3>`

- 状态: `pending`
- 目标:
  - `<what this step achieves>`
- 预计改动文件:
  - `<path/to/file>`
- 具体调整:
  - `<change item>`
- 步骤级测试命令:
  - `cargo check`
  - `<targeted-test-command>`
- 完成标准:
  - `<observable done criteria>`

## 2. 执行日志（每步完成后追加）

- Step A1: `pending`
- Step A2: `pending`
- Step A3: `pending`

### Log Step A1

- 状态: `pending`
- 状态变更: `pending -> in_progress -> pending`
- 实际改动文件:
  - `<path/to/file>`
- 测试命令:
  - `<command>` ✅
- 结果:
  - `<result summary>`

### Log Step A2

- 状态: `pending`
- 状态变更: `pending -> in_progress -> pending`
- 实际改动文件:
  - `<path/to/file>`
- 测试命令:
  - `<command>` ✅
- 结果:
  - `<result summary>`

### Log Step A3

- 状态: `pending`
- 状态变更: `pending -> in_progress -> pending`
- 实际改动文件:
  - `<path/to/file>`
- 测试命令:
  - `<command>` ✅
- 结果:
  - `<result summary>`

## 3. 使用说明

- 每次仅允许一个步骤处于 `in_progress`。
- 若步骤完成，更新步骤状态为 `completed`，并同步更新“执行日志索引行”中的状态。
- 若步骤阻塞，状态设为 `blocked` 并在对应 `Log Step` 写明阻塞原因与替代验证。
- 新增步骤时，必须同时新增：
  - `### Step <ID> - ...`
  - 执行日志索引 `- Step <ID>: \`pending\``
  - 对应 `### Log Step <ID>`
