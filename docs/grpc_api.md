# rapidcron 分布式任务调度系统 gRPC 接口规格说明书（API Spec）
**文档版本**：V1.0
**编制日期**：2025年12月
**适用场景**：rapidcron 调度节点与执行节点、前端UI与调度节点间的跨语言通信
**核心协议**：gRPC + Protobuf 3.x（基于HTTP/2，支持双向流、认证、压缩）

## 一、接口基础规范
### 1.1 命名规范
| 维度         | 规范要求                                                                 |
|--------------|--------------------------------------------------------------------------|
| 服务名       | 大驼峰 + 模块名，例：`TaskService`、`NodeService`、`LogService`          |
| 接口方法名   | 小驼峰 + 动作 + 资源，例：`createTask`、`queryTaskList`、`getNextTrigger`|
| 消息体名     | 大驼峰 + 功能 + Request/Response，例：`CreateTaskRequest`、`QueryTaskListResponse` |
| 字段名       | 蛇形命名（snake_case），例：`task_id`、`cron_expr`、`exec_node`           |
| 枚举值       | 大写蛇形，例：`TASK_STATUS_PENDING`、`TASK_STATUS_SUCCESS`                |

### 1.2 通用约定
1. **错误码体系**：
   | 错误码范围 | 含义                 | 示例                |
   |------------|----------------------|---------------------|
   | 0          | 成功                 | `code: 0, msg: "success"` |
   | 1000-1999  | 参数错误             | `1001: 任务名称为空` |
   | 2000-2999  | 业务逻辑错误         | `2001: 任务已存在`   |
   | 3000-3999  | 系统错误             | `3001: 数据库连接失败` |
   | 4000-4999  | 权限错误             | `4001: 无任务创建权限` |
2. **数据类型**：
   - 时间字段：统一使用 `int64` 存储**毫秒级时间戳**（北京时间UTC+8），例：`1735689600000`（2025-12-01 00:00:00）；
   - 字符串字段：默认非空，长度限制通过 `max_length` 标注；
   - 枚举字段：必须定义默认值，避免空值。
3. **分页约定**：
   - 分页请求必含 `page`（页码，默认1）、`page_size`（页大小，默认20，最大100）；
   - 分页响应必含 `total`（总条数）、`list`（数据列表）、`page`、`page_size`。

## 二、Protobuf 接口定义（核心）
```protobuf
syntax = "proto3";

package rapidcron;
option java_package = "com.rapidcron.grpc";
option java_outer_classname = "RapidCronProto";
option go_package = "./grpc;rapidcron";

// ========== 通用枚举定义 ==========
// 任务状态枚举
enum TaskStatus {
  TASK_STATUS_UNSPECIFIED = 0; // 默认值
  TASK_STATUS_PENDING = 1;     // 待执行
  TASK_STATUS_RUNNING = 2;     // 执行中
  TASK_STATUS_SUCCESS = 3;     // 执行成功
  TASK_STATUS_FAILED = 4;      // 执行失败
  TASK_STATUS_PAUSED = 5;      // 暂停
  TASK_STATUS_DISPATCH_FAILED = 6; // 分发失败
  TASK_STATUS_DEPEND_FAILED = 7;   // 依赖失败
}

// 执行结果枚举
enum ExecResult {
  EXEC_RESULT_UNSPECIFIED = 0;
  EXEC_RESULT_SUCCESS = 1;
  EXEC_RESULT_FAILED = 2;
}

// ========== 通用消息定义 ==========
// 通用响应头
message CommonResponseHeader {
  int32 code = 1; // 错误码，0为成功
  string msg = 2; // 错误信息，成功时为"success"
}

// 分页请求参数
message PageRequest {
  int32 page = 1 [default = 1]; // 页码，默认1
  int32 page_size = 2 [default = 20]; // 页大小，默认20，最大100
}

// 分页响应参数
message PageResponse {
  int64 total = 1; // 总条数
  int32 page = 2;  // 当前页码
  int32 page_size = 3; // 当前页大小
}

// ========== 任务服务接口 ==========
service TaskService {
  // 1. 创建任务
  rpc CreateTask (CreateTaskRequest) returns (CreateTaskResponse);
  
  // 2. 查询任务列表
  rpc QueryTaskList (QueryTaskListRequest) returns (QueryTaskListResponse);
  
  // 3. 查询单个任务详情
  rpc GetTaskDetail (GetTaskDetailRequest) returns (GetTaskDetailResponse);
  
  // 4. 更新任务
  rpc UpdateTask (UpdateTaskRequest) returns (CommonResponse);
  
  // 5. 删除任务
  rpc DeleteTask (DeleteTaskRequest) returns (CommonResponse);
  
  // 6. 启停/暂停任务
  rpc ChangeTaskStatus (ChangeTaskStatusRequest) returns (CommonResponse);
  
  // 7. 获取任务下N次触发时间
  rpc GetTaskNextTrigger (GetTaskNextTriggerRequest) returns (GetTaskNextTriggerResponse);
  
  // 8. 重试失败任务
  rpc RetryFailedTask (RetryFailedTaskRequest) returns (CommonResponse);
}

// 创建任务请求
message CreateTaskRequest {
  string task_name = 1 [(validate.rules).string = {min_len: 1, max_len: 64}]; // 任务名称，必填
  string cron_expr = 2 [(validate.rules).string = {min_len: 1}]; // Cron表达式，必填
  string exec_node = 3; // 执行节点ID，必填
  string task_content = 4 [(validate.rules).string = {max_len: 2048}]; // 任务内容（脚本/命令）
  repeated string depend_tasks = 5; // 依赖任务ID列表（最多2个）
  int32 shard_count = 6 [default = 1]; // 分片数量，默认1
  string creator = 7; // 创建人
}

// 创建任务响应
message CreateTaskResponse {
  CommonResponseHeader header = 1;
  string task_id = 2; // 任务唯一ID
}

// 查询任务列表请求
message QueryTaskListRequest {
  string task_id = 1; // 任务ID（可选）
  string task_name = 2; // 任务名称（模糊查询，可选）
  TaskStatus status = 3; // 任务状态（可选）
  string exec_node = 4; // 执行节点ID（可选）
  PageRequest page = 5; // 分页参数
}

// 任务列表项
message TaskListItem {
  string task_id = 1;
  string task_name = 2;
  string cron_expr = 3;
  TaskStatus status = 4;
  string exec_node = 5;
  int64 create_time = 6; // 创建时间（毫秒时间戳）
  int64 last_exec_time = 7; // 最后执行时间（毫秒时间戳）
}

// 查询任务列表响应
message QueryTaskListResponse {
  CommonResponseHeader header = 1;
  PageResponse page = 2;
  repeated TaskListItem list = 3;
}

// 查询单个任务详情请求
message GetTaskDetailRequest {
  string task_id = 1 [(validate.rules).string = {min_len: 1}]; // 任务ID，必填
}

// 任务详情
message TaskDetail {
  string task_id = 1;
  string task_name = 2;
  string cron_expr = 3;
  TaskStatus status = 4;
  string exec_node = 5;
  string task_content = 6;
  repeated string depend_tasks = 7;
  int32 shard_count = 8;
  string creator = 9;
  int64 create_time = 10;
  int64 update_time = 11;
  int64 last_exec_time = 12;
  int64 next_exec_time = 13; // 下次执行时间
}

// 查询单个任务详情响应
message GetTaskDetailResponse {
  CommonResponseHeader header = 1;
  TaskDetail task_detail = 2;
}

// 更新任务请求
message UpdateTaskRequest {
  string task_id = 1 [(validate.rules).string = {min_len: 1}]; // 任务ID，必填
  string task_name = 2 [(validate.rules).string = {min_len: 1, max_len: 64}]; // 任务名称
  string cron_expr = 3; // Cron表达式
  string exec_node = 4; // 执行节点ID
  string task_content = 5 [(validate.rules).string = {max_len: 2048}]; // 任务内容
  repeated string depend_tasks = 6; // 依赖任务ID列表
  int32 shard_count = 7; // 分片数量
}

// 通用响应
message CommonResponse {
  CommonResponseHeader header = 1;
}

// 删除任务请求
message DeleteTaskRequest {
  string task_id = 1 [(validate.rules).string = {min_len: 1}]; // 任务ID，必填
}

// 变更任务状态请求
message ChangeTaskStatusRequest {
  string task_id = 1 [(validate.rules).string = {min_len: 1}]; // 任务ID，必填
  TaskStatus target_status = 2; // 目标状态（仅支持PAUSED/PENDING）
}

// 获取任务下N次触发时间请求
message GetTaskNextTriggerRequest {
  string task_id = 1 [(validate.rules).string = {min_len: 1}]; // 任务ID，必填
  int32 count = 2 [default = 3]; // 获取次数，默认3，最大100
}

// 获取任务下N次触发时间响应
message GetTaskNextTriggerResponse {
  CommonResponseHeader header = 1;
  repeated int64 trigger_times = 2; // 触发时间列表（毫秒时间戳）
}

// 重试失败任务请求
message RetryFailedTaskRequest {
  string task_id = 1 [(validate.rules).string = {min_len: 1}]; // 任务ID，必填
}

// ========== 日志服务接口 ==========
service LogService {
  // 1. 查询任务执行日志
  rpc QueryTaskLog (QueryTaskLogRequest) returns (QueryTaskLogResponse);
  
  // 2. 导出任务日志（流式响应）
  rpc ExportTaskLog (ExportTaskLogRequest) returns (stream ExportTaskLogResponse);
}

// 查询任务日志请求
message QueryTaskLogRequest {
  string task_id = 1; // 任务ID（可选）
  string task_name = 2; // 任务名称（可选）
  int64 start_time = 3; // 开始时间（毫秒时间戳，可选）
  int64 end_time = 4; // 结束时间（毫秒时间戳，可选）
  ExecResult exec_result = 5; // 执行结果（可选）
  PageRequest page = 6; // 分页参数
}

// 任务日志项
message TaskLogItem {
  string log_id = 1;
  string task_id = 2;
  string task_name = 3;
  int64 trigger_time = 4; // 触发时间
  string exec_node = 5; // 执行节点
  int64 exec_start_time = 6; // 执行开始时间
  int64 exec_end_time = 7; // 执行结束时间
  int64 exec_cost = 8; // 执行耗时（毫秒）
  ExecResult exec_result = 9; // 执行结果
  string fail_reason = 10; // 失败原因
}

// 查询任务日志响应
message QueryTaskLogResponse {
  CommonResponseHeader header = 1;
  PageResponse page = 2;
  repeated TaskLogItem list = 3;
}

// 导出任务日志请求
message ExportTaskLogRequest {
  string task_id = 1; // 任务ID（可选）
  int64 start_time = 2; // 开始时间（必填）
  int64 end_time = 3; // 结束时间（必填）
  ExecResult exec_result = 4; // 执行结果（可选）
}

// 导出任务日志响应（流式）
message ExportTaskLogResponse {
  CommonResponseHeader header = 1;
  bytes log_data = 2; // 日志数据（CSV格式，分片传输）
  bool is_finished = 3; // 是否传输完成
}

// ========== 节点服务接口 ==========
service NodeService {
  // 1. 查询节点状态列表
  rpc QueryNodeStatus (QueryNodeStatusRequest) returns (QueryNodeStatusResponse);
  
  // 2. 下线异常节点
  rpc OfflineAbnormalNode (OfflineAbnormalNodeRequest) returns (CommonResponse);
}

// 节点类型枚举
enum NodeType {
  NODE_TYPE_UNSPECIFIED = 0;
  NODE_TYPE_SCHEDULER = 1; // 调度节点
  NODE_TYPE_EXECUTOR = 2;  // 执行节点
}

// 节点状态枚举
enum NodeStatus {
  NODE_STATUS_UNSPECIFIED = 0;
  NODE_STATUS_ONLINE = 1;
  NODE_STATUS_OFFLINE = 2;
  NODE_STATUS_ABNORMAL = 3; // 异常（高负载/连接失败）
}

// 查询节点状态请求
message QueryNodeStatusRequest {
  NodeType node_type = 1; // 节点类型（可选）
  NodeStatus node_status = 2; // 节点状态（可选）
}

// 节点状态项
message NodeStatusItem {
  string node_id = 1;
  NodeType node_type = 2;
  NodeStatus status = 3;
  string ip = 4; // 节点IP
  int32 port = 5; // 节点端口
  string cpu_usage = 6; // CPU使用率（如"20%"）
  string mem_usage = 7; // 内存使用率（如"30%"）
  int32 task_count = 8; // 承担任务数
  int64 last_heartbeat = 9; // 最后心跳时间（毫秒时间戳）
}

// 查询节点状态响应
message QueryNodeStatusResponse {
  CommonResponseHeader header = 1;
  repeated NodeStatusItem list = 2;
}

// 下线异常节点请求
message OfflineAbnormalNodeRequest {
  string node_id = 1 [(validate.rules).string = {min_len: 1}]; // 节点ID，必填
  NodeType node_type = 2; // 节点类型，必填
}
```

## 三、接口实现与使用规范
### 3.1 代码生成
基于上述 Protobuf 文件，通过 gRPC 工具生成各语言的客户端/服务端代码：
```bash
# 生成Java代码
protoc --java_out=./src/main/java --grpc-java_out=./src/main/java rapidcron.proto

# 生成Go代码
protoc --go_out=./ --go-grpc_out=./ rapidcron.proto

# 生成Rust代码（使用tonic）
protoc --rust_out=./ --grpc-rust_out=./ rapidcron.proto
```

### 3.2 认证与授权
1. 所有 gRPC 接口需添加 Token 认证：
   - 在 RPC 请求的 `metadata` 中携带 `token` 字段（如 `token: "eyJhbGciOiJIUzI1NiJ9..."`）；
   - 服务端拦截器验证 Token 有效性，无效则返回 `code: 4001`（无权限）。
2. 权限控制：
   - 管理员：可调用所有接口；
   - 普通用户：仅可调用 `QueryTaskList`、`GetTaskDetail`、`QueryTaskLog` 等查询类接口。

### 3.3 错误处理
1. 服务端抛出业务异常时，需封装为 `CommonResponseHeader` 并返回对应错误码；
2. 客户端调用时，优先检查 `header.code`，非0则终止流程并提示 `header.msg`；
3. 网络异常（如连接超时）：客户端需实现重试逻辑（最多3次，间隔500ms/1000ms/2000ms）。

### 3.4 性能优化
1. 批量接口（如 `QueryTaskList`）：服务端实现结果缓存（缓存时间5秒），避免高频查询数据库；
2. 流式接口（如 `ExportTaskLog`）：分片传输数据（每片≤1MB），避免内存溢出；
3. 超时设置：所有接口默认超时时间5秒，批量/导出接口可延长至30秒。

## 四、接口测试规范
### 4.1 测试维度
| 测试类型       | 测试内容                                                                 |
|----------------|--------------------------------------------------------------------------|
| 功能测试       | 验证接口入参校验、业务逻辑、响应结果是否符合定义                         |
| 性能测试       | 单接口QPS≥100，批量查询接口响应时间≤500ms                                |
| 异常测试       | 测试空参数、非法Cron表达式、节点离线等异常场景的错误码返回是否准确       |
| 安全测试       | 验证未授权访问、Token失效/伪造、参数注入等场景的防护效果                 |
| 兼容性测试     | 验证Java/Go/Rust客户端调用同一服务端接口的兼容性                         |

### 4.2 测试用例示例（CreateTask接口）
| 用例ID | 测试场景               | 入参示例                          | 预期结果                          |
|--------|------------------------|-----------------------------------|-----------------------------------|
| TC001  | 正常创建任务           | task_name="测试任务"，cron_expr="0/5 * * * * *" | code=0，返回task_id               |
| TC002  | 任务名称为空           | task_name=""，cron_expr="0/5 * * * * *"       | code=1001，msg="任务名称不能为空" |
| TC003  | 非法Cron表达式         | task_name="测试任务"，cron_expr="60 * * * * *" | code=1002，msg="Cron表达式非法：秒数超出0-59范围" |

## 五、附录
### 5.1 修订记录
| 版本 | 修订日期 | 修订内容 | 修订人 |
|------|----------|----------|--------|
| V1.0 | 2025-12-09 | 初始版本，定义任务/日志/节点核心gRPC接口 | outsider |

### 5.2 参考文档
1. Protobuf 3.x 官方文档：https://protobuf.dev/
2. gRPC 官方文档：https://grpc.io/docs/
3. gRPC 错误处理最佳实践：https://grpc.io/docs/guides/error/