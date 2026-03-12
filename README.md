# RapidCron

基于 Rust 的分布式定时任务调度系统，支持高可用、可扩展的任务调度和执行。

## 特性

- **分布式架构**: 支持多节点部署，自动负载均衡
- **高可用**: 基于 etcd 的服务注册与发现，RabbitMQ 持久化队列
- **任务类型**: 支持 Command 和 HTTP 两种任务类型
- **灵活调度**: 支持标准 6 字段 Cron 表达式
- **重试机制**: 支持固定延迟、指数退避、线性退避三种重试策略
- **可观测性**: 详细的执行日志、分发日志和集群监控
- **REST API**: 完整的 HTTP API，支持任务管理和监控

## 快速开始

### 环境要求

- Rust 1.75+
- MongoDB 4.4+
- RabbitMQ 3.8+
- etcd 3.4+

### 安装依赖

使用 Docker Compose 快速启动依赖服务：

```bash
docker-compose up -d
```

### 配置

编辑 `config.toml` 文件，配置数据库、消息队列和服务发现等参数。

### 运行

#### 启动调度器

```bash
cargo run --bin rapidcron
```

#### 启动执行器

```bash
cargo run --bin simple-executor -- --port 8081
```

### 创建任务

```bash
curl -X POST http://localhost:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-task",
    "description": "测试任务",
    "schedule": "0/5 * * * * *",
    "task_type": "command",
    "command": "echo \"Hello World\"",
    "enabled": true,
    "timeout_seconds": 30,
    "max_retries": 3
  }'
```

## 配置说明

### 服务器配置

```toml
[server]
name = "rapidcron"
host = "127.0.0.1"
http_port = 8080
grpc_port = 50051
```

### 数据库配置

```toml
[database]
uri = "mongodb://localhost:27017"
database_name = "rapidcron"
username = "mongo"
password = "mongo@12"
```

### 消息队列配置

```toml
[rabbitmq]
host = "localhost"
port = 5672
username = "guest"
password = "guest"
queue_name = "rapidcron-tasks"
```

### etcd 配置

```toml
[etcd]
host = "localhost"
port = 2379
service_prefix = "rapidcron/services"
heartbeat_interval_secs = 10
offline_threshold_secs = 30
dead_threshold_secs = 60
```

### 分发器配置

```toml
[dispatcher]
scan_interval_secs = 30
max_concurrent_tasks = 10
log_retention_days = 30
```

### 重试配置

```toml
[retry]
scan_interval_secs = 60
batch_size = 100
default_max_retries = 3
default_strategy = "exponential"
exponential_base_delay = 5
exponential_max_delay = 300
```

## API 文档

完整的 API 文档请参考 [api-reference.md](docs/api-reference.md)

### 主要接口

- `GET /api/tasks/stats` - 获取统计信息
- `GET /api/tasks` - 获取任务列表
- `POST /api/tasks` - 创建任务
- `GET /api/tasks/{id}` - 获取任务详情
- `PUT /api/tasks/{id}` - 更新任务
- `DELETE /api/tasks/{id}` - 删除任务
- `POST /api/tasks/{id}/enable` - 启用任务
- `POST /api/tasks/{id}/disable` - 禁用任务
- `POST /api/tasks/{id}/trigger` - 手动触发任务
- `GET /api/tasks/instances` - 获取任务实例列表
- `GET /api/clusters/info` - 获取集群信息
- `GET /api/execution/logs` - 获取执行日志
- `GET /api/dispatch/logs` - 获取分发日志

## 文档

- [架构文档](docs/architecture.md) - 系统架构设计
- [项目结构](docs/project-structure.md) - 项目目录结构说明
- [数据库设计](docs/database-schema.md) - 数据库表结构和索引
- [API 参考](docs/api-reference.md) - 完整的 API 文档
- [测试文档](docs/testing.md) - 测试指南

## 技术栈

- **语言**: Rust (edition 2024)
- **异步运行时**: Tokio
- **Web 框架**: Axum
- **数据库**: MongoDB
- **消息队列**: RabbitMQ
- **服务发现**: etcd
- **Cron 解析**: cron crate
- **日志**: tracing
- **序列化**: serde

## 许可证

MIT License
