### database_design.md
## tasks collection
| 字段              | 类型              | 必填 | 说明                              |
| ----------------- | ----------------- | ---- | --------------------------------- |
| `_id`             | ObjectId          | ✅    | 主键                              |
| `name`            | string            | ✅    | 任务名（未删除时唯一）            |
| `description`     | string \| null    | ❌    | 任务描述                          |
| `dependency_ids`  | array of ObjectId | ❌    | 依赖的任务 ID 列表                |
| `type`            | string            | ✅    | `"command"` 或 `"http"`           |
| `schedule`        | string            | ✅    | Cron 表达式                       |
| `enabled`         | bool              | ✅    | 是否启用                          |
| `payload`         | object            | ✅    | 任务参数（含 `url`/`command` 等） |
| `timeout_seconds` | int \| null       | ❌    | 超时秒数                          |
| `max_retries`     | int \| null       | ❌    | 最大重试次数                      |
| `created_at`      | date              | ✅    | 创建时间                          |
| `updated_at`      | date              | ✅    | 最后更新时间（不含删除）          |
| `deleted_at`      | date \| null      | ❌    | 软删除时间，`null` 表示未删除     |

### indexes
- `name`（唯一索引，条件为 `deleted_at: null`）
- `dependency_ids`（多键索引）
- `enabled:1`（部分索引，条件为 `enabled: true` 和 `deleted_at: null`）
- `deleted_at:1`（索引加速软删除查询）
  
## task_instances collection
| 字段             | 类型           | 必填 | 说明                                                             |
| ---------------- | -------------- | ---- | ---------------------------------------------------------------- |
| `_id`            | ObjectId       | ✅    | 主键                                                             |
| `task_id`        | ObjectId       | ✅    | 关联 `tasks._id`                                                 |
| `scheduled_time` | date           | ✅    | 计划执行时间                                                     |
| `status`         | string         | ✅    | `"pending"`, `"running"`, `"success"`, `"failed"`, `"cancelled"` |
| `executor_id`    | string \| null | ❌    | 执行节点 ID                                                      |
| `start_time`     | date \| null   | ❌    | 实际开始时间                                                     |
| `end_time`       | date \| null   | ❌    | 实际结束时间                                                     |
| `retry_count`    | int            | ✅    | 重试次数（从 0 开始）                                            |
| `result`         | object \| null | ❌    | 执行结果（含 output/error）                                      |
| `created_at`     | date           | ✅    | 实例创建时间                                                     |

### indexes
- `task_id:1, scheduled_time:-1`（复合索引，优化任务实例查询）
- `status:1`（索引加速状态查询）
- `scheduled_time:1`（索引加速定时任务扫描）
- `end_time:1`（索引加速历史查询）

## execution_logs collection
| 字段             | 类型           | 必填 | 说明                        |
| ---------------- | -------------- | ---- | --------------------------- |
| `_id`            | ObjectId       | ✅    | 主键                        |
| `task_id`        | ObjectId       | ✅    | 关联 `tasks._id`            |
| `task_name`      | string         | ✅    | 冗余任务名                  |
| `instance_id`    | ObjectId       | ✅    | 关联 `task_instances._id`   |
| `scheduled_time` | date           | ✅    | 计划执行时间                |
| `start_time`     | date           | ❌    | 实际开始时间                |
| `end_time`       | date           | ✅    | 实际结束时间                |
| `status`         | string         | ✅    | 最终状态                    |
| `duration_ms`    | long           | ✅    | 执行耗时（毫秒）            |
| `output_summary` | string \| null | ❌    | 输出摘要（截断）            |
| `error_message`  | string \| null | ❌    | 错误信息                    |
| `triggered_by`   | string         | ✅    | `"scheduler"` 或 `"manual"` |

### indexes 
- `task_id:1, end_time:-1`（复合索引，优化历史查询）
- `scheduled_time:1`（索引加速定时任务扫描）
- `status:1,end_time:-1`（复合索引，优化状态查询）
- `triggered_by:1,end_time:-1`（复合索引，优化触发方式查询）