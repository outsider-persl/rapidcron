# rapidcron: 基于Rust的分布式任务调度系统
`rapidcron` 是轻量、高性能的分布式任务调度系统，基于Rust+Tokio实现高并发与低资源消耗，支持Cron定时、任务依赖编排、分片执行、失败重试，提供跨语言gRPC接口（Java/Vue3/Python等），适配定时、批量、依赖任务等调度场景。

## 技术栈
| 维度         | 技术选型                                                                 |
|--------------|--------------------------------------------------------------------------|
| 核心语言     | Rust（Tokio异步运行时）                                                 |
| 通信协议     | gRPC（Protobuf定义跨语言接口）                                          |
| 存储组件     | MongoDB（任务/日志）、etcd（分布式锁/服务注册）                         |
| 消息队列     | RabbitMQ（任务异步队列）                                                 |
| 监控指标     | Prometheus（axum暴露HTTP指标端点）                                      |
| 配置管理     | 多环境配置文件+命令行参数+环境变量                                      |

## 项目结构

- [项目结构](./docs/directory%20structure.md)
## 快速启动
### 1. 依赖环境
提前安装并启动：MongoDB(≥6.0)、etcd(≥3.5)、RabbitMQ(≥3.10)（可选：Prometheus用于监控）。

### 2. 启动命令
#### 开发环境
```bash
# 编译+启动（默认dev，也可显式指定--dev）
cargo run -- --dev

# 仅编译
cargo build
```

#### 生产环境
```bash
# 1. 编译生产版本
cargo build --release

# 2. 执行可执行文件（指定prod环境）
./target/release/rapidcron --prod

# 或通过环境变量指定
RAPIDCRON_ENV=production ./target/release/rapidcron
```

#### 命令行帮助
```bash
cargo run -- --help
./target/release/rapidcron --help
```

### 3. 验证启动
- gRPC服务：`0.0.0.0:50051`（可通过gRPC客户端调用）
- 监控指标：`http://localhost:9000/metrics`（Prometheus可抓取）

## 核心能力示例（gRPC接口）
### 1. Protobuf接口定义
核心接口定义在`proto/task_scheduler.proto`，包含任务注册、触发、状态查询等核心能力，支持Cron表达式、任务依赖、重试配置等参数。

### 2. Java客户端调用示例
```java
import com.rapidcron.grpc.TaskSchedulerProto;
import com.rapidcron.grpc.TaskSchedulerGrpc;
import io.grpc.ManagedChannel;
import io.grpc.ManagedChannelBuilder;

public class RapidCronClient {
    public static void main(String[] args) {
        // 连接gRPC服务
        ManagedChannel channel = ManagedChannelBuilder.forAddress("localhost", 50051)
                .usePlaintext() // 生产环境启用TLS
                .build();

        // 注册任务
        TaskSchedulerProto.RegisterTaskRequest request = TaskSchedulerProto.RegisterTaskRequest.newBuilder()
                .setTaskId("task_001")
                .setCronExpr("0/5 * * * *")
                .addDependencies("task_000")
                .setTaskContent("{\"cmd\": \"echo 'hello rapidcron'\"}")
                .setMaxRetry(3)
                .build();

        TaskSchedulerProto.RegisterTaskResponse response = TaskSchedulerGrpc.newBlockingStub(channel)
                .registerTask(request);
        System.out.println("注册结果：" + response.getSuccess());

        channel.shutdown();
    }
}
```

## 文档参考
- 架构设计：`docs/architecture-design.md`（含组件交互与核心流程）
- 开发流程：`docs/devflow.md`（开发规范与提交流程）
- 目录说明：`docs/directory structure.md`（目录功能详细解析）
- gRPC接口：`docs/grpc_api.md`（接口参数、返回值、错误码）
- 需求规格：`docs/srs.md`（功能与非功能需求说明）

## 项目定位
`rapidcron` 是分布式任务调度的轻量级解决方案，主打高性能与低资源消耗，通过跨语言接口降低接入成本，适用于中小规模分布式系统的任务调度需求。