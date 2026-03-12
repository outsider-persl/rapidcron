# RapidCron 项目结构

## 目录结构

```
rapidcron/
├── src/                          # 源代码目录
│   ├── main.rs                   # 主程序入口（调度器 + API 服务器）
│   ├── lib.rs                    # 库入口
│   ├── config.rs                 # 配置管理
│   ├── error.rs                  # 错误类型定义
│   ├── types.rs                  # 数据类型定义
│   ├── logging/                  # 日志模块
│   │   └── mod.rs
│   ├── scheduler/                # 调度器模块
│   │   ├── mod.rs
│   │   ├── dispatcher.rs        # 任务分发器
│   │   └── cron_parser.rs        # Cron 表达式解析器
│   ├── executor/                 # 执行器模块
│   │   ├── mod.rs
│   │   ├── task_queue/           # 任务队列
│   │   │   ├── mod.rs
│   │   │   └── task_queue.rs
│   │   └── retry/                # 重试管理
│   │       ├── mod.rs
│   │       └── retry_logic.rs
│   ├── coord/                    # 协调器模块
│   │   ├── mod.rs
│   │   └── etcd.rs               # etcd 服务注册与发现
│   ├── storage/                  # 存储层模块
│   │   ├── mod.rs
│   │   └── mongo.rs              # MongoDB 数据源
│   ├── api/                      # API 层模块
│   │   ├── mod.rs
│   │   ├── models/               # API 模型
│   │   │   ├── mod.rs
│   │   │   └── api_state.rs      # API 状态
│   │   ├── routes/               # 路由定义
│   │   │   ├── mod.rs
│   │   │   └── routes.rs
│   │   └── handlers/             # 请求处理器
│   │       ├── mod.rs
│   │       ├── tasks.rs          # 任务管理
│   │       ├── clusters.rs       # 集群信息
│   │       ├── execution.rs      # 执行日志
│   │       ├── dispatch.rs       # 分发日志
│   │       └── auth.rs           # 认证
│   └── bin/                      # 可执行程序
│       └── simple-executor.rs    # 简单执行器
├── docs/                         # 文档目录
│   ├── api-reference.md          # API 参考文档
│   ├── architecture.md           # 架构文档
│   ├── database-schema.md        # 数据库设计
│   ├── project-structure.md      # 项目结构
│   ├── testing.md                # 测试文档
│   └── thesis.md                 # 论文文档
├── scripts/                      # 脚本目录
│   ├── init_mongo.js             # MongoDB 初始化脚本
│   └── api.json                  # API 配置
├── Cargo.toml                    # Rust 项目配置
├── Cargo.lock                    # 依赖锁定文件
├── config.toml                   # 应用配置文件
├── docker-compose.yml            # Docker Compose 配置
├── README.md                     # 项目说明
└── LICENSE                       # 许可证
```

## 模块说明

### 核心模块

#### main.rs
主程序入口，负责：
- 加载配置
- 初始化日志
- 连接 MongoDB
- 连接 etcd
- 初始化 RabbitMQ 任务队列
- 启动任务分发器
- 启动重试调度器
- 启动 API 服务器
- 处理优雅关闭

#### lib.rs
库入口，导出所有公共模块。

#### config.rs
配置管理模块，定义：
- `Config`: 总配置结构
- `ServerConfig`: 服务器配置
- `DatabaseConfig`: 数据库配置
- `RabbitMQConfig`: 消息队列配置
- `EtcdConfig`: etcd 配置
- `DispatcherConfig`: 分发器配置
- `RetryConfig`: 重试配置
- `LoggingConfig`: 日志配置
- `ServiceConfig`: 服务配置
- `AuthConfig`: 认证配置

#### error.rs
错误类型定义，包含：
- `Error`: 统一错误枚举
  - `Database`: 数据库错误
  - `Serialization`: 序列化错误
  - `Io`: IO 错误
  - `Scheduling`: 调度错误
  - `Execution`: 执行错误
  - `Validation`: 验证错误
  - `CronFieldCount`: Cron 字段数错误
  - `CronSyntax`: Cron 语法错误
  - `CronTimeRange`: Cron 时间范围错误
  - `CronInternal`: Cron 内部错误
  - `Etcd`: etcd 错误
  - `MessageQueue`: 消息队列错误
- `Result<T>`: 结果类型别名

#### types.rs
数据类型定义，包含：
- `TaskType`: 任务类型枚举
- `TaskStatus`: 任务状态枚举
- `TriggeredBy`: 触发方式枚举
- `TaskPayload`: 任务载荷枚举
- `Task`: 任务结构
- `TaskInstance`: 任务实例结构
- `ExecutionResult`: 执行结果结构
- `ExecutionLog`: 执行日志结构
- `DispatchLog`: 分发日志结构
- `CreateTaskRequest`: 创建任务请求
- `UpdateTaskRequest`: 更新任务请求
- `TriggerTaskRequest`: 触发任务请求
- `PaginatedResponse<T>`: 分页响应
- `ApiResponse<T>`: API 响应
- `StatsResponse`: 统计响应
- `ClusterNode`: 集群节点
- `ClusterResponse`: 集群响应
- `LoginRequest`: 登录请求
- `LoginResponse`: 登录响应
- `UserInfo`: 用户信息

### 调度器模块 (scheduler/)

#### dispatcher.rs
任务分发器，核心功能：
- 定期扫描启用的任务
- 计算任务触发时间
- 创建任务实例
- 发布任务到队列
- 任务实例去重
- 调度日志记录和清理

#### cron_parser.rs
Cron 表达式解析器，核心功能：
- 解析 6 字段 Cron 表达式
- 计算时间窗口内的触发时间
- 错误分类和映射

### 执行器模块 (executor/)

#### task_queue/task_queue.rs
任务队列实现，核心功能：
- 连接 RabbitMQ
- 声明持久化队列
- 发布任务消息

#### retry/retry_logic.rs
重试管理器，核心功能：
- 判断是否需要重试
- 计算重试延迟时间
- 执行任务重试
- 批量重试失败任务
- 支持三种重试策略

### 协调器模块 (coord/)

#### etcd.rs
etcd 管理器，核心功能：
- 服务注册
- 服务发现
- 心跳保持
- 服务注销
- Lease 管理

### 存储层模块 (storage/)

#### mongo.rs
MongoDB 数据源，核心功能：
- 连接 MongoDB
- 管理 4 个集合
- 提供完整的 CRUD 操作
- 支持批量查询和分页

### API 层模块 (api/)

#### models/api_state.rs
API 状态定义，包含：
- `ApiState`: API 状态结构
  - 数据库连接
  - etcd 管理器
  - 任务队列
  - 认证配置

#### routes/routes.rs
路由定义，包含：
- 任务管理路由
- 集群信息路由
- 执行日志路由
- 分发日志路由
- 认证路由

#### handlers/
请求处理器：
- `tasks.rs`: 任务管理处理器
- `clusters.rs`: 集群信息处理器
- `execution.rs`: 执行日志处理器
- `dispatch.rs`: 分发日志处理器
- `auth.rs`: 认证处理器

### 可执行程序 (bin/)

#### simple-executor.rs
简单执行器，核心功能：
- 连接 MongoDB
- 连接 etcd 并注册服务
- 连接 RabbitMQ
- 监听任务队列
- 执行任务（Command/HTTP）
- 更新任务实例状态
- 记录执行日志
- 提供系统监控接口
- 保持心跳

## 配置文件

### config.toml
应用配置文件，包含：
- 服务器配置
- 日志配置
- 数据库配置
- RabbitMQ 配置
- etcd 配置
- 分发器配置
- 重试配置
- 服务配置
- 认证配置

## 依赖说明

### 核心依赖
- `tokio`: 异步运行时
- `axum`: Web 框架
- `mongodb`: MongoDB 客户端
- `lapin`: RabbitMQ 客户端
- `etcd-client`: etcd 客户端
- `cron`: Cron 表达式解析

### 工具依赖
- `serde`: 序列化/反序列化
- `tracing`: 日志框架
- `anyhow`: 错误处理
- `thiserror`: 错误派生
- `chrono`: 时间处理
- `sysinfo`: 系统信息
- `reqwest`: HTTP 客户端

## 构建和运行

### 构建项目
```bash
cargo build --release
```

### 运行调度器
```bash
cargo run --bin rapidcron
```

### 运行执行器
```bash
cargo run --bin simple-executor -- --port 8081
```

### 运行测试
```bash
cargo test
```
