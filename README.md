# rapidcron 分布式任务调度系统

## 介绍
rapidcron 是一个基于 Rust 和 Tokio 的分布式任务调度系统，旨在高性能和可靠性方面提供卓越的表现。它支持多种任务类型，包括定时任务、周期任务和一次性任务，并且可以通过 REST API 进行管理和监控。

## 核心特性

- **分布式架构**: 支持多节点部署，实现任务的负载均衡和高可用
- **基于 etcd 的服务发现**: 使用 etcd 进行服务注册与发现，支持自动心跳和健康检查
- **基于 RabbitMQ 的任务队列**: 确保任务的可靠传递和执行
- **基于 MongoDB 的数据存储**: 持久化存储任务和执行记录
- **RESTful API**: 提供完整的 API 接口，方便集成和管理
- **Cron 表达式支持**: 支持标准的 Cron 表达式，灵活定义任务执行时间
- **任务依赖管理**: 支持任务间的依赖关系，确保任务按正确顺序执行
- **任务重试机制**: 自动重试失败的任务，提高系统可靠性

## 技术栈

- **编程语言**: Rust
- **异步运行时**: Tokio
- **服务发现**: etcd
- **消息队列**: RabbitMQ
- **数据存储**: MongoDB
- **Web 框架**: Axum
- **日志系统**: Tracing

## 项目结构

```
rapidcron/
├── src/
│   ├── api/            # API 模块
│   │   ├── routes/     # API 路由
│   │   ├── handlers/   # API 处理器
│   │   └── models/     # API 数据模型
│   ├── config/         # 配置管理
│   ├── coord/          # 协调模块 (etcd 服务发现)
│   ├── error/          # 错误处理
│   ├── executor/       # 执行器模块
│   │   ├── task_queue/ # 任务队列
│   │   └── retry/      # 重试逻辑
│   ├── logging/        # 日志配置
│   ├── scheduler/      # 调度器模块
│   ├── storage/        # 存储模块
│   │   └── mongo/      # MongoDB 实现
│   ├── types/          # 核心类型定义
│   ├── bin/            # 可执行文件
│   │   └── simple-executor.rs # 简单执行器
│   └── main.rs         # 主程序
├── docs/               # 文档
├── config.toml         # 配置文件
├── Cargo.toml          # Cargo 配置
└── README.md           # 项目说明
```

## 安装与部署

### 前置依赖

- Rust 1.70+ (推荐使用 rustup 安装)
- MongoDB 4.4+
- RabbitMQ 3.8+
- etcd 3.5+

### 安装步骤

1. **克隆代码库**
   ```bash
   git clone https://github.com/yourusername/rapidcron.git
   cd rapidcron
   ```

2. **配置环境**
   编辑 `config.toml` 文件，根据实际环境配置 MongoDB、RabbitMQ 和 etcd 的连接信息。

3. **构建项目**
   ```bash
   cargo build --release
   ```

4. **启动服务**
   ```bash
   ./target/release/rapidcron
   ```

5. **启动执行器**
   ```bash
   cargo run --bin simple-executor --port 8081
   ```

## API 文档

完整的 API 文档请参考 [docs/API.md](docs/API.md)。

## 使用示例

### 创建定时任务

```bash
curl -X POST http://localhost:8000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "示例任务",
    "description": "这是一个示例任务",
    "schedule": "0 * * * *",
    "enabled": true,
    "timeout_seconds": 300,
    "max_retries": 3
  }'
```

### 手动触发任务

```bash
curl -X POST http://localhost:8000/api/tasks/{task_id}/trigger \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 查看任务列表

```bash
curl http://localhost:8000/api/tasks
```

### 查看集群信息

```bash
curl http://localhost:8000/api/cluster
```

## 监控与维护

### 日志管理

系统使用 Tracing 进行日志记录，日志配置可在 `config.toml` 中设置。

### 健康检查

执行器提供了健康检查接口：

```bash
curl http://localhost:8081/health
```

## 贡献指南

欢迎贡献代码和提出问题！请遵循以下步骤：

1. Fork 代码库
2. 创建特性分支
3. 提交更改
4. 推送分支
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。
