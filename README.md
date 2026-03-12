# rapidcron

基于 Rust 的分布式任务调度系统，支持定时任务、任务依赖、自动重试和分布式执行。

## 核心特性

- **分布式架构** - 多节点负载均衡，高可用
- **任务管理** - Cron 表达式、任务依赖、自动重试
- **可靠执行** - RabbitMQ 消息队列 + MongoDB 持久化
- **服务发现** - etcd 自动注册与心跳检测
- **RESTful API** - 完整的任务管理和监控接口
- **日志追踪** - 执行日志、分发日志、触发方式记录

## 技术栈

Rust + Tokio + Axum + MongoDB + RabbitMQ + etcd

## 快速开始

### 前置要求

- Rust 1.70+
- MongoDB 4.4+
- RabbitMQ 3.8+
- etcd 3.5+

### 安装

```bash
# 克隆项目
git clone https://github.com/yourusername/rapidcron.git
cd rapidcron

# 配置（编辑 config.toml）
# 配置 MongoDB、RabbitMQ、etcd 连接信息

# 构建并启动调度器
cargo run --bin rapidcron

# 启动执行器（新终端）
cargo run --bin simple-executor
```

### API 示例

```bash
# 创建任务
curl -X POST http://localhost:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "示例任务",
    "type": "http",
    "schedule": "*/10 * * * *",
    "payload": {"url": "http://example.com/api"},
    "enabled": true
  }'

# 手动触发任务
curl -X POST http://localhost:8080/api/tasks/{task_id}/trigger

# 查看执行日志
curl http://localhost:8080/api/execution/logs

# 查看集群状态
curl http://localhost:8080/api/clusters/info
```

## 项目结构

```
src/
├── api/           # REST API（路由、处理器）
├── executor/       # 任务执行器（队列、重试）
├── scheduler/      # 定时调度器
├── coord/          # etcd 协调
├── storage/        # MongoDB 存储
└── types/          # 核心类型定义
```

## 文档

- [API 文档](docs/API.md) - 完整的 API 接口说明
- [OpenAPI 规范](docs/api.json) - Swagger/OpenAPI 定义

## 许可证

MIT
