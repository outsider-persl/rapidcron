# RapidCron 项目目录结构

## 项目概览

RapidCron 是一个基于 Rust 的分布式任务调度系统，支持定时任务调度、任务依赖管理、服务注册与发现等功能。

```
rapidcron/
├── docs/                          # 文档目录
│   ├── API.md                     # API 文档
│   ├── SIMPLE_EXECUTOR.md          # Simple Executor 文档
│   ├── api.json                   # API JSON 规范
│   ├── database_design.md          # 数据库设计文档
│   ├── 基于Rust的分布式任务调度系统设计与实现.docx
│   ├── 基于Rust的分布式任务调度系统设计与实现.md
│   └── PROJECT_STRUCTURE.md         # 本文档
│
├── scripts/                        # 脚本目录
│   └── init_mongo.js              # MongoDB 初始化脚本
│
├── src/                           # 源代码目录
│   ├── api/                       # API 模块
│   │   ├── clusters.rs              # 集群管理 API
│   │   ├── mod.rs                  # API 模块入口
│   │   ├── routes.rs               # 路由定义
│   │   └── tasks.rs                # 任务管理 API
│   │
│   ├── bin/                        # 可执行程序
│   │   └── simple-executor.rs      # Simple Executor 程序
│   │
│   ├── coord/                      # 协调模块
│   │   ├── etcd.rs                 # etcd 服务注册与发现
│   │   └── mod.rs                  # 协调模块入口
│   │
│   ├── executor/                   # 执行器模块
│   │   ├── mod.rs                  # 执行器模块入口
│   │   ├── retry_logic.rs          # 重试逻辑
│   │   └── task_queue.rs            # 任务队列
│   │
│   ├── logging/                    # 日志模块
│   │   └── mod.rs                  # 日志模块入口
│   │
│   ├── scheduler/                  # 调度器模块
│   │   ├── cron_parser.rs          # Cron 表达式解析器
│   │   ├── dispatcher.rs           # 任务分发器
│   │   ├── mod.rs                  # 调度器模块入口
│   │   └── sorter.rs               # 任务依赖排序器
│   │
│   ├── storage/                    # 存储模块
│   │   ├── mod.rs                  # 存储模块入口
│   │   └── mongo.rs                # MongoDB 数据源
│   │
│   ├── config.rs                   # 配置管理
│   ├── error.rs                    # 错误处理
│   ├── lib.rs                      # 库入口
│   ├── main.rs                     # 主程序入口
│   └── types.rs                    # 类型定义
│
├── .gitignore                      # Git 忽略文件
├── Cargo.lock                      # Cargo 锁文件
├── Cargo.toml                      # Cargo 配置文件
├── README.md                       # 项目说明文档
├── config.toml                     # 配置文件
└── docker-compose.yml              # Docker Compose 配置
```

## 核心模块说明

### 1. API 模块 (`src/api/`)

负责提供 HTTP API 接口，包括任务管理、集群管理等功能。

**主要文件：**
- `clusters.rs` - 集群管理 API，获取节点信息、状态监控
- `tasks.rs` - 任务管理 API，创建、更新、查询任务
- `routes.rs` - 路由定义，组合所有 API 路由
- `mod.rs` - 模块入口，导出公共接口

**核心功能：**
- 任务 CRUD 操作
- 任务实例查询
- 集群节点状态监控
- 手动触发任务执行

### 2. 协调模块 (`src/coord/`)

负责服务注册与发现，基于 etcd 实现。

**主要文件：**
- `etcd.rs` - etcd 客户端封装，实现服务注册、KeepAlive、服务发现
- `mod.rs` - 模块入口

**核心功能：**
- 服务注册（带 Lease 机制）
- 自动 KeepAlive（TTL/3 频率）
- 服务发现（获取所有注册的服务）
- 服务注销（主动撤销租约）

**关键特性：**
- 基于 etcd Lease 机制实现无状态故障检测
- 自动心跳续期，防止服务被误剔除
- KeepAlive 失败自动退出，感知服务下线

### 3. 执行器模块 (`src/executor/`)

负责任务执行和重试逻辑。

**主要文件：**
- `task_queue.rs` - 任务队列管理，连接 RabbitMQ
- `retry_logic.rs` - 重试逻辑，支持多种重试策略
- `mod.rs` - 模块入口

**核心功能：**
- 任务队列消费（RabbitMQ）
- 任务执行（Command/HTTP）
- 失败任务重试
- 重试策略管理（指数退避、固定延迟等）

### 4. 调度器模块 (`src/scheduler/`)

负责任务调度和分发。

**主要文件：**
- `cron_parser.rs` - Cron 表达式解析，计算下次触发时间
- `dispatcher.rs` - 任务分发器，扫描待执行任务并分发
- `sorter.rs` - 任务依赖排序器，处理任务依赖关系
- `mod.rs` - 模块入口

**核心功能：**
- Cron 表达式解析
- 任务扫描（定时扫描待执行任务）
- 任务分发（发送到 RabbitMQ）
- 任务依赖管理（DAG 拓扑排序）

### 5. 存储模块 (`src/storage/`)

负责数据持久化，基于 MongoDB。

**主要文件：**
- `mongo.rs` - MongoDB 数据源封装
- `mod.rs` - 模块入口

**核心功能：**
- MongoDB 连接管理
- 任务 CRUD 操作
- 任务实例查询
- 执行日志记录

### 6. 日志模块 (`src/logging/`)

负责日志初始化和配置。

**主要文件：**
- `mod.rs` - 日志模块入口

**核心功能：**
- 基于 tracing 的日志系统
- 支持多种日志格式（JSON、文本）
- 日志文件轮转

### 7. 核心文件

**`src/config.rs`** - 配置管理
- 从 TOML 文件加载配置
- 配置结构定义
- 配置验证

**`src/error.rs`** - 错误处理
- 自定义错误类型
- 错误转换
- 错误传播

**`src/types.rs`** - 类型定义
- 任务相关类型（Task, TaskInstance, TaskStatus 等）
- API 响应类型
- 集群节点类型

**`src/main.rs`** - 主程序入口
- 配置加载
- 组件初始化
- 服务启动
- 优雅关闭

**`src/lib.rs`** - 库入口
- 模块导出
- 公共接口定义

## 可执行程序

### 1. rapidcron（主服务）

主调度服务，负责任务调度、API 服务、集群管理等功能。

**启动流程：**
1. 加载配置
2. 初始化日志
3. 连接 MongoDB
4. 连接 etcd
5. 注册服务
6. 初始化任务队列
7. 启动调度器
8. 启动重试管理器
9. 启动 API 服务

### 2. simple-executor（执行器）

简单的任务执行器，用于测试和演示。

**启动流程：**
1. 加载配置
2. 初始化日志
3. 连接 etcd
4. 注册服务
5. 连接 RabbitMQ
6. 启动任务消费
7. 启动 HTTP 服务（健康检查、节点信息）

## 配置文件

### config.toml

主配置文件，包含以下配置项：

- `server` - 服务器配置（主机、端口）
- `logging` - 日志配置（级别、格式、输出）
- `database` - 数据库配置（URI、数据库名）
- `rabbitmq` - RabbitMQ 配置（主机、端口、队列名）
- `etcd` - etcd 配置（主机、端口、服务前缀）
- `dispatcher` - 调度器配置（扫描间隔、并发数）
- `retry` - 重试配置（扫描间隔、批次大小）
- `metrics` - 指标配置（启用状态、端口）
- `service` - 服务配置（服务名、元数据）

### docker-compose.yml

Docker Compose 配置，用于本地开发环境，包括：
- MongoDB
- RabbitMQ
- etcd

## 数据库设计

### MongoDB 集合

- `tasks` - 任务定义
- `task_instances` - 任务实例
- `execution_logs` - 执行日志

详细设计参见 [database_design.md](./database_design.md)

## API 设计

### RESTful API

- `GET /api/tasks` - 获取任务列表
- `POST /api/tasks` - 创建任务
- `GET /api/tasks/:id` - 获取任务详情
- `PUT /api/tasks/:id` - 更新任务
- `DELETE /api/tasks/:id` - 删除任务
- `POST /api/tasks/:id/trigger` - 手动触发任务
- `GET /api/tasks/:id/instances` - 获取任务实例列表
- `GET /api/cluster` - 获取集群信息

详细 API 文档参见 [API.md](./API.md)

## 技术栈

### 核心技术

- **Rust** - 主要编程语言
- **Tokio** - 异步运行时
- **Axum** - Web 框架
- **MongoDB** - 数据库
- **RabbitMQ** - 消息队列
- **etcd** - 服务注册与发现

### 主要依赖

- `tokio` - 异步运行时
- `axum` - Web 框架
- `mongodb` - MongoDB 客户端
- `lapin` - RabbitMQ 客户端
- `etcd-client` - etcd 客户端
- `cron` - Cron 表达式解析
- `tracing` - 日志框架
- `serde` - 序列化/反序列化

## 开发指南

### 构建项目

```bash
cargo build
```

### 运行主服务

```bash
cargo run --bin rapidcron
```

### 运行 Simple Executor

```bash
cargo run --bin simple-executor
```

### 运行测试

```bash
cargo test
```

### 使用 Docker Compose

```bash
docker-compose up -d
```

## 部署

### 生产环境部署

1. 配置 `config.toml`
2. 启动 MongoDB、RabbitMQ、etcd
3. 运行主服务
4. 根据需要启动多个 Executor

### Docker 部署

参见 [基于Rust的分布式任务调度系统设计与实现.md](./基于Rust的分布式任务调度系统设计与实现.md)

## 许可证

[待补充]

## 贡献指南

[待补充]

## 联系方式

[待补充]
