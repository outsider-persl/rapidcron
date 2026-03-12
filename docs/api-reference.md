# RapidCron API 文档

## 基本信息

- **API 基础 URL**: `http://localhost:8080/api`
- **内容类型**: `application/json`
- **响应格式**: JSON

## 通用响应结构

所有 API 响应都遵循以下格式：

```json
{
  "success": true,
  "data": { ... },
  "message": null
}
```

- `success`: 请求是否成功
- `data`: 响应数据（成功时）
- `message`: 错误消息（失败时）

---

## API 接口列表

### 1. 获取统计信息

**接口地址**: `GET /tasks/stats`

**描述**: 获取系统统计信息，包括任务总数、实例总数等

**请求参数**: 无

**响应示例**:

```json
{
  "success": true,
  "data": {
    "total_tasks": 10,
    "enabled_tasks": 8,
    "total_instances": 100,
    "pending_instances": 5,
    "running_instances": 2,
    "success_instances": 90,
    "failed_instances": 3
  },
  "message": null
}
```

---

### 2. 获取任务列表

**接口地址**: `GET /tasks`

**描述**: 分页获取任务列表，支持按启用状态、名称、任务类型筛选

**请求参数**:

| 参数名    | 类型    | 必填 | 默认值 | 描述                     |
| --------- | ------- | ---- | ------ | ------------------------ |
| enabled   | boolean | 否   | -      | 是否只返回启用的任务     |
| name      | string  | 否   | -      | 任务名称（模糊查询）     |
| task_type | string  | 否   | -      | 任务类型（command/http） |
| page      | integer | 否   | 1      | 页码                     |
| page_size | integer | 否   | 20     | 每页数量                 |

**请求示例**:

```bash
GET /api/tasks?enabled=true&name=test&page=1&page_size=20
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "_id": "507f1f77bcf86cd799439011",
        "name": "test-task",
        "description": "测试任务",
        "type": "command",
        "schedule": "0/5 * * * * *",
        "enabled": true,
        "payload": {
          "command": "echo 'Hello World'",
          "timeout_seconds": 30
        },
        "timeout_seconds": 30,
        "max_retries": 3,
        "created_at": "2026-02-25T11:00:00.000Z",
        "updated_at": "2026-02-25T11:00:00.000Z"
      }
    ],
    "total": 10,
    "page": 1,
    "page_size": 20,
    "total_pages": 1
  },
  "message": null
}
```

---

### 3. 创建任务

**接口地址**: `POST /tasks`

**描述**: 创建新的定时任务

**请求参数**:

| 参数名          | 类型    | 必填 | 描述                                     |
| --------------- | ------- | ---- | ---------------------------------------- |
| name            | string  | 是   | 任务名称                                 |
| description     | string  | 否   | 任务描述                                 |
| schedule        | string  | 是   | Cron 表达式（6 字段：秒 分 时 日 月 周） |
| task_type       | string  | 否   | 任务类型（command/http）                 |
| command         | string  | 否   | 命令（当 task_type 为 command 时使用）   |
| url             | string  | 否   | URL（当 task_type 为 http 时使用）       |
| enabled         | boolean | 否   | 是否启用                                 |
| timeout_seconds | integer | 否   | 超时时间（秒）                           |
| max_retries     | integer | 否   | 最大重试次数                             |
| dependency_ids  | array   | 否   | 依赖任务 ID 列表                         |

**请求示例**:

```json
{
  "name": "test-task",
  "description": "测试任务",
  "schedule": "0/5 * * * * *",
  "task_type": "command",
  "command": "echo 'Hello World'",
  "enabled": true,
  "timeout_seconds": 30,
  "max_retries": 3,
  "dependency_ids": []
}
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "_id": "507f1f77bcf86cd799439011",
    "name": "test-task",
    "description": "测试任务",
    "type": "command",
    "schedule": "0/5 * * * * *",
    "enabled": true,
    "payload": {
      "command": "echo 'Hello World'",
      "timeout_seconds": 30
    },
    "timeout_seconds": 30,
    "max_retries": 3,
    "created_at": "2026-02-25T11:00:00.000Z",
    "updated_at": "2026-02-25T11:00:00.000Z"
  },
  "message": null
}
```

---

### 4. 获取任务详情

**接口地址**: `GET /tasks/{id}`

**描述**: 根据任务 ID 获取任务详细信息

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 任务 ID |

**请求示例**:

```bash
GET /api/tasks/507f1f77bcf86cd799439011
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "_id": "507f1f77bcf86cd799439011",
    "name": "test-task",
    "description": "测试任务",
    "type": "command",
    "schedule": "0/5 * * * * *",
    "enabled": true,
    "payload": {
      "command": "echo 'Hello World'",
      "timeout_seconds": 30
    },
    "timeout_seconds": 30,
    "max_retries": 3,
    "created_at": "2026-02-25T11:00:00.000Z",
    "updated_at": "2026-02-25T11:00:00.000Z"
  },
  "message": null
}
```

---

### 5. 更新任务

**接口地址**: `PUT /tasks/{id}`

**描述**: 更新任务信息

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 任务 ID |

**请求参数**: 同创建任务（所有参数都是可选的）

**请求示例**:

```json
{
  "name": "updated-task",
  "enabled": false
}
```

**响应示例**: 同获取任务详情

---

### 6. 删除任务

**接口地址**: `DELETE /tasks/{id}`

**描述**: 软删除任务（设置 deleted_at 字段）

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 任务 ID |

**请求示例**:

```bash
DELETE /api/tasks/507f1f77bcf86cd799439011
```

**响应示例**:

```json
{
  "success": true,
  "data": null,
  "message": "任务删除成功"
}
```

---

### 7. 启用任务

**接口地址**: `POST /tasks/{id}/enable`

**描述**: 启用指定的任务

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 任务 ID |

**请求示例**:

```bash
POST /api/tasks/507f1f77bcf86cd799439011/enable
```

**响应示例**:

```json
{
  "success": true,
  "data": null,
  "message": "任务启用成功"
}
```

---

### 8. 禁用任务

**接口地址**: `POST /tasks/{id}/disable`

**描述**: 禁用指定的任务

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 任务 ID |

**请求示例**:

```bash
POST /api/tasks/507f1f77bcf86cd799439011/disable
```

**响应示例**:

```json
{
  "success": true,
  "data": null,
  "message": "任务禁用成功"
}
```

---

### 9. 手动触发任务

**接口地址**: `POST /tasks/{id}/trigger`

**描述**: 手动触发任务执行，创建任务实例

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 任务 ID |

**请求参数**:

| 参数名         | 类型    | 必填 | 描述                                        |
| -------------- | ------- | ---- | ------------------------------------------- |
| scheduled_time | integer | 否   | 计划执行时间（Unix 时间戳），为空则立即执行 |

**请求示例**:

```json
{
  "scheduled_time": null
}
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "_id": "507f1f77bcf86cd799439012",
    "task_id": "507f1f77bcf86cd799439011",
    "scheduled_time": "2026-02-25T11:00:00.000Z",
    "status": "pending",
    "executor_id": null,
    "start_time": null,
    "end_time": null,
    "retry_count": 0,
    "result": null,
    "created_at": "2026-02-25T11:00:00.000Z"
  },
  "message": null
}
```

---

### 10. 获取任务实例列表

**接口地址**: `GET /tasks/instances`

**描述**: 分页获取任务实例列表，支持按任务 ID 和状态筛选

**请求参数**:

| 参数名    | 类型    | 必填 | 默认值 | 描述                                                 |
| --------- | ------- | ---- | ------ | ---------------------------------------------------- |
| task_id   | string  | 否   | -      | 任务 ID                                              |
| status    | string  | 否   | -      | 任务状态（pending/running/success/failed/cancelled） |
| page      | integer | 否   | 1      | 页码                                                 |
| page_size | integer | 否   | 20     | 每页数量                                             |

**请求示例**:

```bash
GET /api/tasks/instances?status=success&page=1&page_size=20
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "_id": "507f1f77bcf86cd799439012",
        "task_id": "507f1f77bcf86cd799439011",
        "scheduled_time": "2026-02-25T11:00:00.000Z",
        "status": "success",
        "executor_id": "executor-1",
        "start_time": "2026-02-25T11:00:01.000Z",
        "end_time": "2026-02-25T11:00:02.000Z",
        "retry_count": 0,
        "result": {
          "output": "Hello World",
          "error": null,
          "exit_code": 0
        },
        "created_at": "2026-02-25T11:00:00.000Z"
      }
    ],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  },
  "message": null
}
```

---

### 11. 获取任务实例详情

**接口地址**: `GET /tasks/instances/{id}`

**描述**: 根据实例 ID 获取任务实例详细信息

**路径参数**:

| 参数名 | 类型   | 必填 | 描述    |
| ------ | ------ | ---- | ------- |
| id     | string | 是   | 实例 ID |

**请求示例**:

```bash
GET /api/tasks/instances/507f1f77bcf86cd799439012
```

**响应示例**: 同获取任务实例列表中的单个实例

---

### 12. 获取集群信息

**接口地址**: `GET /clusters/info`

**描述**: 获取所有集群节点信息，包括节点名称、状态、CPU、内存和活跃任务数

**请求参数**: 无

**请求示例**:

```bash
GET /api/clusters/info
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "nodes": [
      {
        "node_name": "executor-1",
        "node_id": "executor-1",
        "host": "localhost",
        "port": 8081,
        "status": "active",
        "cpu_usage": 45.5,
        "memory_usage": 28.75,
        "memory_total": 8,
        "active_tasks": 3,
        "metadata": "executor"
      },
      {
        "node_name": "executor-2",
        "node_id": "executor-2",
        "host": "localhost",
        "port": 8082,
        "status": "active",
        "cpu_usage": 30.2,
        "memory_usage": 22.5,
        "memory_total": 8,
        "active_tasks": 1,
        "metadata": "executor"
      }
    ],
    "total_nodes": 2,
    "active_nodes": 2
  },
  "message": null
}
```

**响应字段说明**:

| 字段名       | 类型    | 描述       |
| ------------ | ------- | ---------- |
| nodes        | array   | 节点列表   |
| total_nodes  | integer | 总节点数   |
| active_nodes | integer | 活跃节点数 |

**节点信息字段说明**:

| 字段名       | 类型    | 描述                        |
| ------------ | ------- | --------------------------- |
| node_name    | string  | 节点名称                    |
| node_id      | string  | 节点 ID                     |
| host         | string  | 主机地址                    |
| port         | integer | 端口号                      |
| status       | string  | 节点状态（active/inactive） |
| cpu_usage    | float   | CPU 使用率（百分比）        |
| memory_usage | float   | 内存使用率（百分比）        |
| memory_total | integer | 内存总量（GB）              |
| active_tasks | integer | 活跃任务数                  |
| metadata     | string  | 元数据                      |

---

### 13. 获取执行日志列表

**接口地址**: `GET /execution/logs`

**描述**: 分页获取执行日志列表，支持按任务ID、实例ID、状态和触发方式筛选

**请求参数**:

| 参数名       | 类型    | 必填 | 默认值 | 描述                                                 |
| ------------ | ------- | ---- | ------ | ---------------------------------------------------- |
| task_id      | string  | 否   | -      | 任务 ID                                              |
| instance_id  | string  | 否   | -      | 实例 ID                                              |
| status       | string  | 否   | -      | 任务状态（pending/running/success/failed/cancelled） |
| triggered_by | string  | 否   | -      | 触发方式（scheduler/manual）                         |
| page         | integer | 否   | 1      | 页码                                                 |
| page_size    | integer | 否   | 20     | 每页数量                                             |

**请求示例**:

```bash
GET /api/execution/logs?status=failed&page=1&page_size=20
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "_id": "69b1121135cf369be666ca8b",
        "task_id": "670000000000000000000002",
        "task_name": "Test Error Task",
        "instance_id": "69b1120e9ea2fa76dc90516f",
        "scheduled_time": "2026-03-11T07:02:57.250Z",
        "start_time": "2026-03-11T07:02:57.250Z",
        "end_time": "2026-03-11T07:02:57.255Z",
        "status": "failed",
        "duration_ms": 5,
        "output_summary": "HTTP 500 Internal Server Error 失败",
        "error_message": "HTTP 错误: 500 Internal Server Error",
        "triggered_by": "scheduler"
      }
    ],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  },
  "message": null
}
```

---

### 14. 获取执行日志详情

**接口地址**: `GET /execution/logs/{id}`

**描述**: 根据日志 ID 获取执行日志详细信息

**路径参数**:

| 参数名 | 类型   | 必填 | 描述        |
| ------ | ------ | ---- | ----------- |
| id     | string | 是   | 执行日志 ID |

**请求示例**:

```bash
GET /api/execution/logs/69b1121135cf369be666ca8b
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "_id": "69b1121135cf369be666ca8b",
    "task_id": "670000000000000000000002",
    "task_name": "Test Error Task",
    "instance_id": "69b1120e9ea2fa76dc90516f",
    "scheduled_time": "2026-03-11T07:02:57.250Z",
    "start_time": "2026-03-11T07:02:57.250Z",
    "end_time": "2026-03-11T07:02:57.255Z",
    "status": "failed",
    "duration_ms": 5,
    "output_summary": "HTTP 500 Internal Server Error 失败",
    "error_message": "HTTP 错误: 500 Internal Server Error",
    "triggered_by": "scheduler"
  },
  "message": null
}
```

---

### 15. 获取分发日志列表

**接口地址**: `GET /dispatch/logs`

**描述**: 分页获取分发日志列表，支持按扫描时间范围、是否有错误等条件筛选

**请求参数**:

| 参数名     | 类型    | 必填 | 默认值 | 描述                         |
| ---------- | ------- | ---- | ------ | ---------------------------- |
| start_time | string  | 否   | -      | 扫描开始时间（ISO 8601格式） |
| end_time   | string  | 否   | -      | 扫描结束时间（ISO 8601格式） |
| has_error  | boolean | 否   | -      | 是否有错误                   |
| page       | integer | 否   | 1      | 页码                         |
| page_size  | integer | 否   | 20     | 每页数量                     |

**请求示例**:

```bash
GET /api/dispatch/logs?has_error=false&page=1&page_size=20
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "_id": "69b1121135cf369be666ca8c",
        "scan_time": "2026-03-11T07:02:57.250Z",
        "scan_window_start": "2026-03-11T07:02:47.250Z",
        "scan_window_end": "2026-03-11T07:02:57.250Z",
        "total_tasks": 5,
        "enabled_tasks": 3,
        "dispatched_instances": 8,
        "error_message": null
      },
      {
        "_id": "69b1121135cf369be666ca8d",
        "scan_time": "2026-03-11T07:02:47.250Z",
        "scan_window_start": "2026-03-11T07:02:37.250Z",
        "scan_window_end": "2026-03-11T07:02:47.250Z",
        "total_tasks": 5,
        "enabled_tasks": 3,
        "dispatched_instances": 8,
        "error_message": "连接数据库失败"
      }
    ],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "total_pages": 5
  },
  "message": null
}
```

---

### 16. 获取分发日志详情

**接口地址**: `GET /dispatch/logs/{id}`

**描述**: 根据日志 ID 获取分发日志详细信息

**路径参数**:

| 参数名 | 类型   | 必填 | 描述        |
| ------ | ------ | ---- | ----------- |
| id     | string | 是   | 分发日志 ID |

**请求示例**:

```bash
GET /api/dispatch/logs/69b1121135cf369be666ca8c
```

**响应示例**:

```json
{
  "success": true,
  "data": {
    "_id": "69b1121135cf369be666ca8c",
    "scan_time": "2026-03-11T07:02:57.250Z",
    "scan_window_start": "2026-03-11T07:02:47.250Z",
    "scan_window_end": "2026-03-11T07:02:57.250Z",
    "total_tasks": 5,
    "enabled_tasks": 3,
    "dispatched_instances": 8,
    "error_message": null
  },
  "message": null
}
```

---

## 数据模型

### Task（任务）

| 字段名          | 类型    | 描述                                     |
| --------------- | ------- | ---------------------------------------- |
| _id             | string  | 任务 ID                                  |
| name            | string  | 任务名称                                 |
| description     | string  | 任务描述                                 |
| type            | string  | 任务类型（command/http）                 |
| schedule        | string  | Cron 表达式（6 字段：秒 分 时 日 月 周） |
| enabled         | boolean | 是否启用                                 |
| payload         | object  | 任务载荷                                 |
| timeout_seconds | integer | 超时时间（秒）                           |
| max_retries     | integer | 最大重试次数                             |
| created_at      | string  | 创建时间                                 |
| updated_at      | string  | 更新时间                                 |

### TaskInstance（任务实例）

| 字段名         | 类型    | 描述                                                 |
| -------------- | ------- | ---------------------------------------------------- |
| _id            | string  | 实例 ID                                              |
| task_id        | string  | 任务 ID                                              |
| scheduled_time | string  | 计划执行时间                                         |
| status         | string  | 实例状态（pending/running/success/failed/cancelled） |
| executor_id    | string  | 执行器 ID                                            |
| start_time     | string  | 开始执行时间                                         |
| end_time       | string  | 结束执行时间                                         |
| retry_count    | integer | 重试次数                                             |
| result         | object  | 执行结果                                             |
| triggered_by   | string  | 触发方式（scheduler/manual）                           |
| created_at     | string  | 创建时间                                             |

### ExecutionResult（执行结果）

| 字段名    | 类型    | 描述     |
| --------- | ------- | -------- |
| output    | string  | 输出内容 |
| error     | string  | 错误信息 |
| exit_code | integer | 退出码   |

### ExecutionLog（执行日志）

| 字段名         | 类型    | 描述                                                 |
| -------------- | ------- | ---------------------------------------------------- |
| _id            | string  | 日志 ID                                              |
| task_id        | string  | 任务 ID                                              |
| task_name      | string  | 任务名称                                             |
| instance_id    | string  | 实例 ID                                              |
| scheduled_time | string  | 计划执行时间                                         |
| start_time     | string  | 开始执行时间                                         |
| end_time       | string  | 结束执行时间                                         |
| status         | string  | 执行状态（pending/running/success/failed/cancelled） |
| duration_ms    | integer | 执行时长（毫秒）                                     |
| output_summary | string  | 输出摘要                                             |
| error_message  | string  | 错误消息                                             |
| triggered_by   | string  | 触发方式（scheduler/manual）                         |

### DispatchLog（分发日志）

| 字段名               | 类型    | 描述             |
| -------------------- | ------- | ---------------- |
| _id                  | string  | 日志 ID          |
| scan_time            | string  | 扫描时间         |
| scan_window_start    | string  | 扫描窗口开始时间 |
| scan_window_end      | string  | 扫描窗口结束时间 |
| total_tasks          | integer | 总任务数         |
| enabled_tasks        | integer | 启用的任务数     |
| dispatched_instances | integer | 分发的实例数     |
| error_message        | string  | 错误消息         |

---

## Cron 表达式说明

RapidCron 使用标准的 6 字段 Cron 表达式：

```
秒 分 时 日 月 周
*  *  *  *  *  *
```

| 表达式           | 描述               |
| ---------------- | ------------------ |
| `0/5 * * * * *`  | 每 5 秒执行一次    |
| `0 */10 * * * *` | 每 10 分钟执行一次 |
| `0 0 * * * *`    | 每小时执行一次     |
| `0 0 0 * * *`    | 每天执行一次       |
| `0 0 0 * * 1`    | 每周一执行一次     |
| `0 0 0 1 * *`    | 每月 1 号执行一次  |

---

## 错误码

| 错误码 | 描述           |
| ------ | -------------- |
| 400    | 请求参数错误   |
| 404    | 资源不存在     |
| 500    | 服务器内部错误 |

---

## 导入到 Apifox

1. 打开 Apifox
2. 点击"导入"
3. 选择"OpenAPI"
4. 上传 `docs/api.json` 文件
5. 点击"确定"完成导入

或者直接复制本 Markdown 文档的内容到 Apifox 中手动创建接口。
