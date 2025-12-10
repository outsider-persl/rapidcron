rapidcron/
├── src/
│   ├── main.rs                 # 程序入口（启动gRPC服务、初始化组件）
│   ├── lib.rs                  # 核心库导出（供测试/其他模块调用）
│   ├── config/                 # 配置模块
│   │   ├── mod.rs              # 配置入口（聚合子模块）
│   │   ├── loader.rs           # 从文件/环境变量加载配置
│   │   └── types.rs            # 配置结构体定义（数据库、MQ、服务端口等）
│   ├── scheduler/              # 任务调度核心
│   │   ├── mod.rs
│   │   ├── cron.rs             # Cron表达式解析与定时触发
│   │   ├── dependency.rs       # 任务依赖拓扑排序与执行链生成
│   │   └── dispatcher.rs       # 任务分发（向执行器/消息队列推送任务）
│   ├── executor/               # 任务执行模块
│   │   ├── mod.rs
│   │   ├── sharder.rs          # 任务分片逻辑（哈希/范围分片）
│   │   ├── retry.rs            # 失败重试（指数退避策略）
│   │   └── idempotent.rs       # 幂等性校验（基于操作ID）
│   ├── storage/                # 数据存储模块
│   │   ├── mod.rs
│   │   ├── mongodb.rs          # MongoDB交互（任务日志、状态）
│   │   └── etcd.rs             # etcd交互（分布式锁、服务注册）
│   ├── messaging/              # 消息队列模块
│   │   ├── mod.rs
│   │   └── rabbitmq.rs         # RabbitMQ生产者/消费者（任务队列）
│   ├── grpc/                   # gRPC通信模块
│   │   ├── mod.rs
│   │   ├── server.rs           # gRPC服务实现（注册/执行/查询接口）
│   │   └── proto/              # protobuf定义
│   │       └── task_scheduler.proto  # 跨语言接口定义
│   └── common/                 # 通用工具
│       ├── mod.rs
│       ├── error.rs            # 全局错误类型
│       ├── logger.rs           # 日志工具
│       └── metrics.rs          # 性能指标采集（供Prometheus）
└── tests/                      # 集成测试
    ├── scheduler_test.rs       # 调度逻辑测试
    └── executor_test.rs        # 执行器功能测试