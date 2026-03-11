# 项目结构

## 目录结构

```
rapidcron/
├── src/
│   ├── api/            # API 模块
│   │   ├── routes/     # API 路由定义
│   │   ├── handlers/   # API 请求处理器
│   │   └── models/     # API 数据模型
│   ├── config/         # 配置管理
│   ├── coord/          # 协调模块 (etcd 服务发现)
│   ├── error/          # 错误处理
│   ├── executor/       # 执行器模块
│   │   ├── task_queue/ # 任务队列实现
│   │   └── retry/      # 任务重试逻辑
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

## 模块说明

### 1. api/ - API 模块

负责处理 HTTP 请求，提供 RESTful API 接口。

- **routes/**: 定义 API 路由和端点
- **handlers/**: 处理 API 请求，实现业务逻辑
- **models/**: 定义 API 数据模型和请求/响应结构

### 2. config/ - 配置管理

负责加载和管理应用配置。

### 3. coord/ - 协调模块

负责服务注册与发现，基于 etcd 实现。

### 4. error/ - 错误处理

定义统一的错误类型和错误处理逻辑。

### 5. executor/ - 执行器模块

负责任务的执行和管理。

- **task_queue/**: 实现基于 RabbitMQ 的任务队列
- **retry/**: 实现任务重试逻辑

### 6. logging/ - 日志配置

负责配置和初始化日志系统。

### 7. scheduler/ - 调度器模块

负责任务的调度和分发。

### 8. storage/ - 存储模块

负责数据持久化，基于 MongoDB 实现。

### 9. types/ - 核心类型定义

定义系统中使用的核心数据类型。

### 10. bin/ - 可执行文件

包含项目的可执行文件。

## 核心流程

1. **服务启动流程**:
   - 加载配置
   - 初始化日志
   - 连接 MongoDB
   - 连接 etcd 并注册服务
   - 连接 RabbitMQ 并初始化任务队列
   - 启动任务调度器
   - 启动 API 服务器

2. **任务调度流程**:
   - 调度器定期扫描待执行的任务
   - 为任务创建实例
   - 将任务实例发送到 RabbitMQ 队列
   - 执行器从队列中获取任务并执行
   - 执行结果更新到 MongoDB

3. **服务发现流程**:
   - 服务启动时注册到 etcd
   - 定期发送心跳保持服务活跃
   - 其他服务通过 etcd 发现可用节点
   - 服务关闭时从 etcd 注销

## 技术栈

- **编程语言**: Rust
- **异步运行时**: Tokio
- **Web 框架**: Axum
- **服务发现**: etcd
- **消息队列**: RabbitMQ
- **数据存储**: MongoDB
- **日志系统**: Tracing

## 依赖关系

- **api** 依赖 **types**, **storage**, **coord**
- **scheduler** 依赖 **types**, **storage**, **executor**
- **executor** 依赖 **types**, **storage**
- **storage** 依赖 **types**
- **coord** 依赖 **types**
