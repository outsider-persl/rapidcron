rapidcron/
├── Cargo.toml
├── LICENSE
├── README.md
├── build.rs                  # 构建脚本，用于生成 gRPC Rust 代码
├── docs/
│   ├── architecture-design.md
│   ├── architecture-design.png
│   ├── devflow.md
│   ├── directory structure.md
│   ├── grpc_api.md
│   └── srs.md
├── examples/                 # 示例程序
├── proto/                    # 根目录下 protobuf 文件
│   └── task_scheduler.proto  # 跨语言接口定义
├── src/
│   ├── main.rs               # 程序入口（启动 gRPC 服务、初始化组件）
│   ├── lib.rs                # 核心库导出（供测试/其他模块调用）
│   ├── common/               # 通用工具（原 utils 模块整合至 common）
│   │   ├── error.rs          # 全局错误类型
│   │   ├── loader.rs         # 通用加载器（配置/资源加载）
│   │   ├── logger.rs         # 日志工具
│   │   ├── metrics.rs        # 性能指标采集（供 Prometheus）
│   │   ├── mod.rs            # 通用工具入口（聚合子模块）
│   │   └── time.rs           # 时间处理工具
│   ├── config/               # 配置文件目录
│   │   └── default.toml      # 默认配置文件（数据库、MQ、服务端口等）
│   ├── executor/             # 任务执行模块（待完善具体逻辑文件）
│   ├── grpc/                 # gRPC 通信模块
│   │   ├── mod.rs
│   │   └── server.rs         # gRPC 服务实现（注册/执行/查询接口）
│   ├── messaging/            # 消息队列模块（待完善具体逻辑文件）
│   ├── scheduler/            # 任务调度核心
│   │   ├── mod.rs
│   │   ├── cron.rs           # Cron 表达式解析与定时触发
│   │   ├── dependency.rs     # 任务依赖拓扑排序与执行链生成
│   │   └── dispatcher.rs     # 任务分发（向执行器/消息队列推送任务）
│   ├── storage/              # 数据存储模块（待完善具体逻辑文件）
│   └── utils/                # 兼容保留的工具目录（核心逻辑已迁移至 common）
│       └── mod.rs
└── tests/                    # 集成测试
    └── tz_test.rs            # 时区相关测试（原调度/执行器测试待补充）