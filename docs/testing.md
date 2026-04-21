# RapidCron 测试文档

## 测试概述

RapidCron 项目包含三种类型的测试：
- **单元测试**: 测试单个函数和模块的功能
- **集成测试**: 测试多个模块之间的交互
- **基准测试**: 测试性能和优化效果

## 单元测试

### Cron 解析器单元测试

位置: [src/scheduler/cron_parser.rs](../src/scheduler/cron_parser.rs)

#### 测试覆盖

- `test_cron_parser_new_valid_expressions`: 测试有效的 Cron 表达式
- `test_cron_parser_new_invalid_field_count`: 测试字段数量不正确的情况
- `test_cron_parser_new_invalid_syntax`: 测试语法错误
- `test_cron_parser_new_invalid_range`: 测试字段值超出范围
- `test_next_triggers_in_window_multiple_triggers`: 测试多个触发时间
- `test_next_triggers_in_window_no_triggers`: 测试无触发时间的情况
- `test_next_triggers_in_window_boundary`: 测试边界情况
- `test_next_triggers_in_window_complex_expression`: 测试复杂表达式
- `test_next_triggers_in_window_every_second`: 测试每秒执行
- `test_map_cron_error_syntax`: 测试语法错误映射
- `test_map_cron_error_range`: 测试范围错误映射
- `test_map_cron_error_internal`: 测试内部错误映射

## 集成测试

### Cron 解析器集成测试

位置: [tests/cron_parser_integration.rs](../tests/cron_parser_integration.rs)

#### 测试覆盖

- `test_cron_parser_integration_end_to_end`: 端到端 Cron 解析和触发时间计算
- `test_cron_parser_integration_complex_schedule`: 复杂调度表达式测试
- `test_cron_parser_integration_hourly_schedule`: 小时级调度测试
- `test_cron_parser_integration_daily_schedule`: 日级调度测试
- `test_cron_parser_integration_weekly_schedule`: 周级调度测试
- `test_cron_parser_integration_monthly_schedule`: 月级调度测试
- `test_cron_parser_integration_range_expression`: 范围表达式测试
- `test_cron_parser_integration_step_expression`: 步长表达式测试
- `test_cron_parser_integration_empty_window`: 空时间窗口测试
- `test_cron_parser_integration_very_long_window`: 长时间窗口测试

### 任务管理集成测试

位置: [tests/task_management.rs](../tests/task_management.rs)

#### 测试覆盖

- `test_create_task_request_to_task_command`: 创建命令任务
- `test_create_task_request_to_task_http`: 创建 HTTP 任务
- `test_create_task_request_empty_name`: 空名称验证
- `test_create_task_request_name_too_long`: 名称长度验证
- `test_create_task_request_invalid_cron`: 无效 Cron 表达式验证
- `test_create_task_request_invalid_timeout`: 无效超时时间验证
- `test_create_task_request_timeout_too_large`: 超时时间过大验证
- `test_create_task_request_invalid_max_retries`: 无效最大重试次数验证
- `test_create_task_request_http_without_url`: HTTP 任务缺少 URL 验证
- `test_create_task_request_command_without_command`: 命令任务缺少命令验证
- `test_create_task_request_with_dependency_ids`: 依赖任务 ID 测试
- `test_create_task_request_with_invalid_dependency_ids`: 无效依赖 ID 测试
- `test_create_task_request_default_enabled`: 默认启用状态测试
- `test_update_task_request_partial_update`: 部分更新测试
- `test_parse_object_id_valid`: 有效 ObjectId 解析
- `test_parse_object_id_invalid`: 无效 ObjectId 解析
- `test_parse_object_ids_valid`: 批量 ObjectId 解析
- `test_parse_object_ids_mixed`: 混合 ObjectId 解析
- `test_task_serialization`: 任务序列化测试
- `test_paginated_response_from_items`: 分页响应测试
- `test_api_response_success`: API 响应测试

### 任务执行集成测试

位置: [tests/task_execution.rs](../tests/task_execution.rs)

#### 测试覆盖

- `test_task_instance_status_transitions`: 任务实例状态转换测试
- `test_task_instance_failed_status`: 失败状态测试
- `test_task_instance_manual_trigger`: 手动触发测试
- `test_execution_result_success`: 成功结果测试
- `test_execution_result_failure`: 失败结果测试
- `test_execution_log_creation`: 执行日志创建测试
- `test_execution_log_failure`: 失败日志测试
- `test_task_instance_serialization`: 任务实例序列化测试
- `test_execution_log_serialization`: 执行日志序列化测试
- `test_task_status_equality`: 任务状态相等性测试
- `test_triggered_by_equality`: 触发方式相等性测试
- `test_task_instance_duration_calculation`: 任务实例持续时间计算
- `test_task_instance_retry_count_increment`: 重试计数递增测试
- `test_execution_log_output_summary_truncation`: 输出摘要截断测试

### 重试逻辑集成测试

位置: [tests/retry_logic.rs](../tests/retry_logic.rs)

#### 测试覆盖

- `test_retry_strategy_fixed`: 固定延迟策略测试
- `test_retry_strategy_exponential`: 指数退避策略测试
- `test_retry_strategy_linear`: 线性退避策略测试
- `test_retry_config_default`: 默认配置测试
- `test_retry_strategy_exponential_max_delay`: 指数退避最大延迟测试
- `test_retry_strategy_exponential_with_small_max`: 小最大延迟指数退避测试
- `test_retry_strategy_linear_with_large_increment`: 大增量线性退避测试
- `test_retry_strategy_fixed_with_large_delay`: 大延迟固定策略测试
- `test_retry_strategy_exponential_base_delay_zero`: 零基础延迟指数退避测试
- `test_retry_strategy_linear_initial_delay_zero`: 零初始延迟线性退避测试

## 基准测试

### Cron 解析器基准测试

位置: [benches/cron_parser_bench.rs](../benches/cron_parser_bench.rs)

#### 基准测试

- `bench_cron_parser_new`: 不同复杂度 Cron 表达式解析性能
- `bench_cron_parser_next_triggers_simple`: 简单表达式触发时间计算
- `bench_cron_parser_next_triggers_complex`: 复杂表达式触发时间计算
- `bench_cron_parser_next_triggers_every_second`: 每秒执行触发时间计算
- `bench_cron_parser_next_triggers_hourly`: 小时级触发时间计算
- `bench_cron_parser_next_triggers_daily`: 日级触发时间计算
- `bench_cron_parser_next_triggers_weekly`: 周级触发时间计算
- `bench_cron_parser_next_triggers_monthly`: 月级触发时间计算
- `bench_cron_parser_batch_parsing`: 批量解析性能

### 任务创建基准测试

位置: [benches/task_creation_bench.rs](../benches/task_creation_bench.rs)

#### 基准测试

- `bench_create_task_request_command`: 命令任务创建性能
- `bench_create_task_request_http`: HTTP 任务创建性能
- `bench_create_task_request_with_dependencies`: 带依赖任务创建性能
- `bench_create_task_request_complex_schedule`: 复杂调度任务创建性能
- `bench_create_task_request_batch`: 批量任务创建性能
- `bench_task_serialization`: 任务序列化性能
- `bench_task_deserialization`: 任务反序列化性能

### 任务查询基准测试

位置: [benches/task_query_bench.rs](../benches/task_query_bench.rs)

#### 基准测试

- `bench_paginated_response_from_items_small`: 小数据集分页性能
- `bench_paginated_response_from_items_medium`: 中等数据集分页性能
- `bench_paginated_response_from_items_large`: 大数据集分页性能
- `bench_task_instance_serialization`: 任务实例序列化性能
- `bench_task_instance_deserialization`: 任务实例反序列化性能
- `bench_task_serialization`: 任务序列化性能
- `bench_task_deserialization`: 任务反序列化性能
- `bench_parse_object_id`: ObjectId 解析性能
- `bench_parse_object_ids_batch`: 批量 ObjectId 解析性能
- `bench_api_response_creation`: API 响应创建性能

### 重试计算基准测试

位置: [benches/retry_calculation_bench.rs](../benches/retry_calculation_bench.rs)

#### 基准测试

- `bench_retry_strategy_fixed`: 固定延迟策略计算性能
- `bench_retry_strategy_exponential`: 指数退避策略计算性能
- `bench_retry_strategy_linear`: 线性退避策略计算性能
- `bench_should_retry_with_error`: 重试判断性能（有错误）
- `bench_should_retry_exceeded`: 重试判断性能（超过最大次数）
- `bench_calculate_retry_delay_fixed`: 固定延迟计算性能
- `bench_calculate_retry_delay_exponential`: 指数退避延迟计算性能
- `bench_calculate_retry_delay_linear`: 线性退避延迟计算性能
- `bench_batch_retry_calculation`: 批量重试计算性能

## 运行测试

### 运行所有单元测试##

```bash
cargo test
```

### 运行特定模块的单元测试##

```bash
cargo test cron_parser
cargo test task_management
cargo test task_execution
cargo test retry_logic
```

### 运行集成测试##

```bash
cargo test --tests
```

### 运行特定集成测试##

```bash
cargo test --test cron_parser_integration
cargo test --test task_management
cargo test --test task_execution
cargo test --test retry_logic
```

### 运行基准测试##

```bash
cargo bench
```

### 运行特定基准测试##

```bash
cargo bench --bench cron_parser_bench
cargo bench --bench task_creation_bench
cargo bench --bench task_query_bench
cargo bench --bench retry_calculation_bench
```

### 基准测试运行模式（带参数）

项目使用 Criterion 作为基准测试框架，可以通过命令行参数控制采样次数和预热时间。推荐三种模式：

```bash
# 快速模式：20 个样本，1 秒预热（开发调试用）
cargo bench --bench cron_parser_bench -- --sample-size 20 --warm-up-time 1

# 标准模式：80 个样本，3 秒预热（日常性能回归用）
cargo bench --bench cron_parser_bench -- --sample-size 80 --warm-up-time 3

# 精准模式：200 个样本，8 秒预热（深入性能分析用）
cargo bench --bench cron_parser_bench -- --sample-size 200 --warm-up-time 8
```

> 说明：上面的示例以 `cron_parser_bench` 为例，替换为其它 bench 名称即可。

## 使用自动测试脚本

项目提供了一个交互式测试脚本，帮助你选择测试类型、范围以及基准测试模式，并自动将测试结果输出到 `logs` 目录下。

### 脚本位置

- 脚本文件: `scripts/run_tests.sh`
- 日志目录: `logs/`

首次使用前，确保脚本有执行权限（仓库通常已经设置好，如需手动设置）：

```bash
chmod +x scripts/run_tests.sh
```

### 运行脚本

```bash
./scripts/run_tests.sh
```

运行后，脚本会按步骤提示你选择：

1. **测试类型**
   - `单元测试`
   - `集成测试`
   - `基准测试`
2. **测试范围**
   - 单元 / 集成测试:
     - `全部`
     - `指定模块`（输入如 `cron_parser`、`task_management` 等）
   - 基准测试:
     - `全部基准测试`
     - `单个基准测试文件`（输入如 `cron_parser_bench`、`task_creation_bench` 等）
3. **基准测试模式（仅在选择“基准测试”时出现）**
   - `快速模式`：20 样本，1 秒预热
   - `标准模式`：80 样本，3 秒预热
   - `精准模式`：200 样本，8 秒预热

脚本会根据你的选择构造相应的 `cargo` 命令并执行。

### 日志输出规则

- 每次运行脚本，都会在 `logs` 目录下生成一个日志文件。
- 日志文件命名格式为：

  ```text
  测试模块-年月日时分秒.log
  ```

  实际例子：

  - `单元测试-全部-20260312153045.log`
  - `单元测试-cron_parser-20260312153210.log`
  - `集成测试-cron_parser_integration-20260312154001.log`
  - `基准测试-cron_parser_bench-fast-20260312154530.log`

- 文件内容包含：
  - 测试描述
  - 实际执行的 `cargo` 命令
  - 开始时间与结束时间
  - 测试过程输出（标准输出和标准错误）
  - 退出码（是否成功）

## 运行测试并显示输出

```bash
cargo test -- --nocapture
```

### 运行测试并生成覆盖率报告

需要安装 `cargo-tarpaulin`:

```bash
cargo install cargo-tarpaulin

cargo tarpaulin --out Html
```

## 相关文档

- [架构文档](architecture.md)
- [项目结构](project-structure.md)
- [API 参考](api-reference.md)
- [数据库设计](database-schema.md)
