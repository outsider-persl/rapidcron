# RapidCron 系统架构

## 概述

RapidCron 是一个基于 Rust 的分布式定时任务调度系统，支持高可用、可扩展的任务调度和执行。

## 核心组件

### 1. 调度器

负责定时扫描任务并创建任务实例。

#### Dispatcher (任务分发器)
- 定期扫描启用的任务
- 根据 Cron 表达式计算任务触发时间
- 在扫描时间窗口内创建任务实例
- 将任务实例发布到消息队列
- 支持任务实例去重，避免重复调度
- 支持调度日志记录和清理

#### CronParser (Cron 解析器)
- 解析 6 字段 Cron 表达式（秒 分 时 日 月 周）
- 计算指定时间窗口内的所有触发时间
- 提供详细的错误分类（语法错误、字段范围错误等）

### 2. 执行器

负责任务的执行和重试。

#### TaskQueue (任务队列)
- 基于 RabbitMQ 的消息队列
- 支持任务消息的发布
- 持久化队列，确保任务不丢失

#### RetryManager (重试管理器)
- 支持三种重试策略：
  - **固定延迟重试**: 每次重试间隔固定时间
  - **指数退避重试**: 重试间隔按指数增长，可设置最大延迟
  - **线性退避重试**: 重试间隔按线性增长
- 自动判断是否需要重试
- 批量重试失败任务

### 3. 协调器

负责服务注册与发现。

#### EtcdManager (etcd 管理器)
- 基于 etcd 的服务注册与发现
- 支持服务自动注册和注销
- 提供服务查询接口

#### ServiceRegistry (服务注册器)
- 服务注册，使用 Lease 机制
- 自动心跳保持，定期更新服务状态
- 服务注销，自动清理资源

### 4. 存储层

负责数据持久化。

#### MongoDataSource (MongoDB 数据源)
- 管理 4 个核心集合：
  - `tasks`: 任务定义
  - `task_instances`: 任务实例
  - `execution_logs`: 执行日志
  - `dispatch_logs`: 分发日志
- 提供完整的 CRUD 操作
- 支持批量查询和分页

### 5. API 层

提供 HTTP REST API。

#### API Router (路由器)
- 基于 Axum 框架
- 支持 CORS 跨域
- 统一的响应格式

#### Handlers (处理器)
- **tasks**: 任务管理（创建、查询、更新、删除、启用/禁用、手动触发）
- **clusters**: 集群信息查询
- **execution**: 执行日志查询
- **dispatch**: 分发日志查询
- **auth**: 认证接口

### 6. 执行节点

负责任务的实际执行。

#### Simple Executor
- 监听 RabbitMQ 队列
- 支持两种任务类型：
  - **Command 任务**: 执行系统命令
  - **HTTP 任务**: 发送 HTTP 请求
- 更新任务实例状态
- 记录执行日志
- 提供系统监控接口（CPU、内存使用率）
- 注册到 etcd，支持服务发现

## 数据流

### 任务调度流程

```
1. Dispatcher 定期扫描启用的任务
   ↓
2. CronParser 解析 Cron 表达式，计算触发时间
   ↓
3. 创建 TaskInstance（状态：pending）
   ↓
4. 发布 TaskMessage 到 RabbitMQ
   ↓
5. Simple Executor 从队列消费任务
   ↓
6. 更新 TaskInstance 状态为 running
   ↓
7. 执行任务（Command 或 HTTP）
   ↓
8. 更新 TaskInstance 状态和结果（success/failed）
   ↓
9. 创建 ExecutionLog
   ↓
10. 如果失败且满足重试条件，RetryManager 安排重试
```

### 服务注册流程

```
1. 服务启动
   ↓
2. 连接 etcd
   ↓
3. 创建 Lease
   ↓
4. 注册服务信息（ServiceInfo）
   ↓
5. 启动 KeepAlive 任务，定期更新心跳
   ↓
6. 服务关闭时，注销服务并撤销 Lease
```

## 关键特性

### 高可用
- 基于 etcd 的服务注册与发现
- RabbitMQ 持久化队列，确保任务不丢失
- 支持多执行节点，自动负载均衡

### 可扩展
- 模块化设计，各组件职责清晰
- 支持水平扩展执行节点
- 支持自定义重试策略

### 可靠性
- 任务实例去重，避免重复执行
- 完善的错误处理和日志记录
- 支持任务重试，提高执行成功率

### 可观测性
- 详细的执行日志
- 分发日志记录调度过程
- 集群节点监控（CPU、内存、活跃任务数）

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
