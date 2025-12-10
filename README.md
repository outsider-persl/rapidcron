# rapidcron: 基于Rust的分布式任务调度系统

## 项目简介
`rapidcron`是一个轻量、高性能的分布式任务调度系统，支持Cron表达式定时、任务依赖编排、分片执行、失败重试等核心能力，通过Rust实现核心层保证高并发与低资源消耗，同时提供跨语言gRPC接口（Java/Vue/Python等客户端均可调用）。


## 技术栈
- **核心语言**：Rust（Tokio异步运行时）
- **通信协议**：gRPC（跨语言接口）
- **存储组件**：MongoDB（任务/日志存储）、etcd（分布式锁/服务注册）
- **消息队列**：RabbitMQ（任务异步队列）
- **客户端支持**：Java/Vue3/Python/Rust（多语言调用）


## 快速启动

### 1. 依赖环境准备
确保已安装以下组件并启动：
- MongoDB（>=6.0）
- etcd（>=3.5）
- RabbitMQ（>=3.10）


### 2. 配置文件
在项目根目录创建`config.toml`（参考配置）：
```toml
[server]
grpc_port = 50051  # gRPC服务端口

[mongodb]
uri = "mongodb://localhost:27017"
db_name = "rapidcron"

[etcd]
endpoints = ["http://localhost:2379"]

[rabbitmq]
uri = "amqp://guest:guest@localhost:5672/%2f"
task_queue = "rapidcron_task_queue"
```


### 3. 编译与启动
```bash
# 编译（开发模式）
cargo build

# 启动服务
cargo run -- --config config.toml
```


## 模块结构
```
rapidcron/
├── src/
│   ├── main.rs                # 程序入口（启动gRPC服务、初始化组件）
│   ├── lib.rs                 # 核心库导出（供测试/其他模块调用）
│   ├── config/                # 配置模块（加载、解析）
│   ├── scheduler/             # 任务调度核心（Cron解析、依赖编排、任务分发）
│   ├── executor/              # 任务执行模块（分片、重试、幂等性）
│   ├── storage/               # 存储模块（MongoDB/etcd交互）
│   ├── messaging/             # 消息队列模块（RabbitMQ生产者/消费者）
│   ├── grpc/                  # gRPC通信模块（服务实现+protobuf定义）
│   └── common/                # 通用工具（错误、日志、指标）
└── tests/                     # 集成测试（调度/执行器功能）
```


## 核心能力示例（gRPC接口）
通过gRPC调用`rapidcron`的核心接口（以任务注册为例）：

### 1. Protobuf接口定义（`task_scheduler.proto`）
```protobuf
syntax = "proto3";

service TaskScheduler {
  // 注册定时任务
  rpc RegisterTask (RegisterTaskRequest) returns (RegisterTaskResponse);
  // 触发任务执行
  rpc TriggerTask (TriggerTaskRequest) returns (TriggerTaskResponse);
  // 查询任务状态
  rpc QueryTaskStatus (QueryTaskStatusRequest) returns (QueryTaskStatusResponse);
}

message RegisterTaskRequest {
  string task_id = 1;
  string cron_expr = 2;       // Cron表达式（如"0/5 * * * *"）
  repeated string dependencies = 3;  // 依赖的前置任务ID
}

message RegisterTaskResponse {
  bool success = 1;
  string message = 2;
}
```


### 2. 客户端调用示例（Java）
```java
// 连接gRPC服务
ManagedChannel channel = ManagedChannelBuilder.forAddress("localhost", 50051)
    .usePlaintext()
    .build();
TaskSchedulerGrpc.TaskSchedulerBlockingStub stub = TaskSchedulerGrpc.newBlockingStub(channel);

// 注册任务
RegisterTaskRequest request = RegisterTaskRequest.newBuilder()
    .setTaskId("task_001")
    .setCronExpr("0/5 * * * *")
    .addDependencies("task_000")
    .build();
RegisterTaskResponse response = stub.registerTask(request);
System.out.println("注册结果：" + response.getSuccess());
```


## 项目定位
本项目为**分布式任务调度领域的轻量级解决方案**，适用于定时任务、批量任务、依赖任务等场景，通过Rust的高性能特性保证系统的高并发与稳定性，同时提供跨语言接口降低接入成本。