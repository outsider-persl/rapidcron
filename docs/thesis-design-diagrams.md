# 4. 系统设计图表

```text
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    表现层       │     │    API层        │     │  业务逻辑层      │     │  数据访问层      │     │  基础设施层      │
│ ┌─────────────┐ │     │ ┌─────────────┐ │     │ ┌─────────────┐ │     │ ┌─────────────┐ │     │ ┌─────────────┐ │
│ │ Web管理界面  │─┼────>│ │ RESTful API │─┼────>│ │  调度器     │─┼────>│ │  MongoDB    │─┼────>│ │ Tokio运行时 │ │
│ └─────────────┘ │     │ └─────────────┘ │     │ └─────────────┘ │     │ └─────────────┘ │     │ └─────────────┘ │
│ ┌─────────────┐ │     │                 │     │ ┌─────────────┐ │     │ ┌─────────────┐ │     │ ┌─────────────┐ │
│ │  API文档    │─┼────>│                 │     │ │  执行器     │─┼────>│ │  RabbitMQ   │─┼────>│ │ 网络通信    │ │
│ └─────────────┘ │     │                 │     │ └─────────────┘ │     │ └─────────────┘ │     │ └─────────────┘ │
└─────────────────┘     └─────────────────┘     │ ┌─────────────┐ │     │ ┌─────────────┐ │     │ ┌─────────────┐ │
                                                │ │  协调器     │─┼────>│ │   etcd      │─┼────>│ │ 日志系统    │ │
                                                │ └─────────────┘ │     │ └─────────────┘ │     │ └─────────────┘ │
                                                │ ┌─────────────┐ │     └─────────────────┘     └─────────────────┘
                                                │ │ 重试管理器   │ │
                                                │ └─────────────┘ │
                                                └─────────────────┘
```

```text
┌──────────────────────────────────────────────────────┐
│                      表现层                          │
│        [ Web管理界面 ]   [ API文档 ]                 │
└──────────────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────┐
│                       API层                          │
│                   [ RESTful API ]                    │
└──────────────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────┐
│                   业务逻辑层                         │
│ [ 调度器 ] [ 执行器 ] [ 协调器 ] [ 重试管理器 ]      │
└──────────────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────┐
│                   数据访问层                         │
│     [ MongoDB ]   [ RabbitMQ ]   [ etcd ]            │
└──────────────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────┐
│                   基础设施层                         │
│ [ Tokio运行时 ] [ 网络通信 ] [ 日志系统 ]            │
└──────────────────────────────────────────────────────┘
```

## 4.1 调度器工作流程图

```mermaid
flowchart TD
    Start[加载配置] --> Init[初始化数据库和消息队列]
    Init --> StartScanner[启动任务扫描器]
    StartScanner --> ScanTasks[扫描启用的任务]
    ScanTasks --> ParseCron[解析Cron并计算触发时间]
    ParseCron --> CheckExisting[检查任务实例是否存在]
    CheckExisting -->|不存在| CreateInstance[创建任务实例并发布到队列]
    CheckExisting -->|存在| SkipCreation[跳过创建]
    SkipCreation --> Sleep[等待下一次扫描]
    CreateInstance --> Sleep
    Sleep --> ScanTasks
```

## 4.2 执行器工作流程图

```mermaid
flowchart TD
    Start[开始] --> Load[注册、链接MQ、启动监听]
    Load --> WaitForTask[等待任务消息]
    WaitForTask --> ReceiveTask[接收任务消息]
    ReceiveTask --> UpdateStatus[更新statu为running、执行]
    UpdateStatus -->|成功| UpdateSuccess[更新任务状态为成功]
    UpdateStatus -->|失败| UpdateFailed[更新任务状态为失败]
    UpdateSuccess --> CreateLog[创建执行日志]
    UpdateFailed --> CreateLog
    CreateLog --> CheckRetry[检查是否需要重试]
    CheckRetry -->|不需要重试| WaitForTask
    CheckRetry -->|需要重试| RequeueTask[计算延迟、重新发布到队列]
    RequeueTask --> WaitForTask
```

## 4.3 重试策略流程图

```mermaid
flowchart TD
    Start[任务执行失败] --> CheckRetries[检查重试次数]
    CheckRetries -->|达到最大重试次数| MarkFailed[标记任务失败]
    CheckRetries -->|未达到最大重试次数| SelectStrategy[选择策略:固定/指数/线性]
    SelectStrategy --> Calculate[计算退避延迟]
    Calculate --> ScheduleRetry[安排重试]
    ScheduleRetry --> UpdateInstance[更新任务实例]
    UpdateInstance --> PublishToQueue[发布到队列]
    MarkFailed --> CreateLog[创建执行日志]
    PublishToQueue --> End[结束]
    CreateLog --> End
```

## 4.4 服务注册与发现流程图

```mermaid
sequenceDiagram
    participant Service as 服务实例
    participant Etcd as etcd集群
    participant Client as 客户端

    Service->>Etcd: 创建Lease
    Etcd-->>Service: Lease ID
    Service->>Etcd: 注册服务信息
    Etcd-->>Service: 注册成功
    Service->>Etcd: 定期发送心跳
    Etcd-->>Service: 心跳确认
    Client->>Etcd: 发现服务
    Etcd-->>Client: 返回服务列表
    Service->>Etcd: 服务关闭，注销
    Etcd-->>Service: 注销成功
```
