# RapidCron

基于 Rust 的分布式定时任务调度系统，支持高可用、可扩展的任务调度和执行。

## 特性

- **分布式架构**: 支持多节点部署，自动负载均衡
- **高可用**: 基于 etcd 的服务注册与发现，RabbitMQ 持久化队列
- **任务类型**: 支持 Command 和 HTTP 两种任务类型
- **灵活调度**: 支持标准 6 字段 Cron 表达式
- **重试机制**: 支持固定延迟、指数退避、线性退避三种重试策略
- **可观测性**: 详细的执行日志、分发日志和集群监控
- **REST API**: 完整的 HTTP API，支持任务管理和监控

## 快速开始

### 环境要求

- Rust 1.75+
- MongoDB 4.4+
- RabbitMQ 3.8+
- etcd 3.4+

### 安装依赖

使用 Docker Compose 快速启动依赖服务：

```bash
docker-compose up -d
```

### 配置

编辑 `config.toml` 文件，配置数据库、消息队列和服务发现等参数。

### 运行

#### 启动调度器

```bash
cargo run --bin rapidcron
```

#### 启动执行器

```bash
cargo run --bin simple-executor -- --port 8081
```

### 创建任务

```bash
curl -X POST http://localhost:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-task",
    "description": "测试任务",
    "schedule": "0/5 * * * * *",
    "task_type": "command",
    "command": "echo \"Hello World\"",
    "enabled": true,
    "timeout_seconds": 30,
    "max_retries": 3
  }'
```

## 配置说明

### 服务器配置

```toml
[server]
name = "rapidcron"
host = "127.0.0.1"
http_port = 8080
grpc_port = 50051
```

### 数据库配置

```toml
[database]
uri = "mongodb://localhost:27017"
database_name = "rapidcron"
username = "mongo"
password = "mongo@12"
```

### 消息队列配置

```toml
[rabbitmq]
host = "localhost"
port = 5672
username = "guest"
password = "guest"
queue_name = "rapidcron-tasks"
```

### etcd 配置

```toml
[etcd]
host = "localhost"
port = 2379
service_prefix = "rapidcron/services"
heartbeat_interval_secs = 10
offline_threshold_secs = 30
dead_threshold_secs = 60
```

### 分发器配置

```toml
[dispatcher]
scan_interval_secs = 30
max_concurrent_tasks = 10
log_retention_days = 30
```

### 重试配置

```toml
[retry]
scan_interval_secs = 60
batch_size = 100
default_max_retries = 3
default_strategy = "exponential"
exponential_base_delay = 5
exponential_max_delay = 300
```

## API 文档

完整的 API 文档请参考 [api-reference.md](docs/api-reference.md)

### 主要接口

- `GET /api/tasks/stats` - 获取统计信息
- `GET /api/tasks` - 获取任务列表
- `POST /api/tasks` - 创建任务
- `GET /api/tasks/{id}` - 获取任务详情
- `PUT /api/tasks/{id}` - 更新任务
- `DELETE /api/tasks/{id}` - 删除任务
- `POST /api/tasks/{id}/enable` - 启用任务
- `POST /api/tasks/{id}/disable` - 禁用任务
- `POST /api/tasks/{id}/trigger` - 手动触发任务
- `GET /api/tasks/instances` - 获取任务实例列表
- `GET /api/clusters/info` - 获取集群信息
- `GET /api/execution/logs` - 获取执行日志
- `GET /api/dispatch/logs` - 获取分发日志

## 文档

- [架构文档](docs/architecture.md) - 系统架构设计
- [项目结构](docs/project-structure.md) - 项目目录结构说明
- [数据库设计](docs/database-schema.md) - 数据库表结构和索引
- [API 参考](docs/api-reference.md) - 完整的 API 文档
- [测试文档](docs/testing.md) - 测试指南

## 核心代码

### 服务注册

```rust
/// 注册服务
pub async fn register(&self, service: ServiceInfo, lease_ttl_secs: i64) -> Result<i64> {
    let key = format!("{}/{}", self.service_prefix, service.service_name);

    let value = serde_json::to_string(&service).map_err(Error::Serialization)?;

    let lease = self
        .client
        .lock()
        .await
        .lease_grant(lease_ttl_secs, None)
        .await
        .map_err(|e| Error::Etcd(format!("创建 Lease 失败: {}", e)))?;

    let lease_id = lease.id();

    let options = PutOptions::new().with_lease(lease_id);

    self.client
        .lock()
        .await
        .put(key.clone(), value, Some(options))
        .await
        .map_err(|e| Error::Etcd(format!("注册服务失败: {}", e)))?;

    let mut leases = self.service_leases.write().await;
    leases.insert(service.service_name.clone(), lease_id);
    drop(leases);

    info!(
        "[KeepAlive] 服务注册成功: {} ({}) - Lease: {}",
        service.service_name, service.service_id, lease_id
    );

    self.start_keepalive(service.service_name.clone(), lease_id, lease_ttl_secs)
        .await;

    Ok(lease_id)
}
```

### 服务发现

```rust
/// 从 etcd 获取所有服务
pub async fn discover_all_services(&self) -> Result<Vec<ServiceInfo>> {
    let service_prefix = "rapidcron/services".to_string();

    let options = Some(GetOptions::new().with_prefix());

    let mut client = self.client.lock().await;
    let response = client
        .get(service_prefix, options)
        .await
        .map_err(|e| Error::Etcd(format!("获取所有服务失败: {}", e)))?;

    if response.kvs().is_empty() {
        return Ok(Vec::new());
    }

    let mut services = Vec::new();
    for kv in response.kvs() {
        if let Ok(service) = serde_json::from_slice::<ServiceInfo>(kv.value()) {
            services.push(service);
        }
    }

    debug!("[Etcd] 发现所有服务: {} 个实例", services.len());

    Ok(services)
}
```

### 心跳保持

```rust
/// 启动 KeepAlive 任务
async fn start_keepalive(&self, service_name: String, lease_id: i64, ttl_secs: i64) {
    let client = Arc::clone(&self.client);
    let keepalive_interval = std::time::Duration::from_secs((ttl_secs / 3).max(1) as u64);
    let service_name_clone = service_name.clone();
    let service_prefix = self.service_prefix.clone();

    let task = tokio::spawn(async move {
        let (mut keeper, mut stream) = {
            let mut client = client.lock().await;
            match client.lease_keep_alive(lease_id).await {
                Ok(result) => result,
                Err(e) => {
                    error!(
                        "[KeepAlive] 启动 KeepAlive 失败 (服务: {}, Lease: {}): {}",
                        service_name_clone, lease_id, e
                    );
                    return;
                }
            }
        };

        let mut ticker = tokio::time::interval(keepalive_interval);

        loop {
            ticker.tick().await;

            if let Err(e) = keeper.keep_alive().await {
                error!(
                    "[KeepAlive] KeepAlive 失败 (服务: {}, Lease: {}): {}",
                    service_name_clone, lease_id, e
                );
                break;
            }

            match stream.message().await {
                Ok(Some(resp)) => {
                    debug!(
                        "[KeepAlive] 心跳发送成功 (服务: {}, Lease: {}), TTL: {}s",
                        service_name_clone,
                        lease_id,
                        resp.ttl()
                    );

                    let key = format!("{}/{}", service_prefix, service_name_clone);
                    let mut client = client.lock().await;

                    let get_result = client.get(key.clone(), None).await;
                    if let Ok(get_resp) = get_result
                        && let Some(kv) = get_resp.kvs().first()
                        && let Ok(mut service_info) =
                            serde_json::from_slice::<ServiceInfo>(kv.value())
                    {
                        service_info.last_heartbeat = chrono::Utc::now().timestamp();
                        if let Ok(value) = serde_json::to_string(&service_info) {
                            let options = PutOptions::new().with_lease(lease_id);
                            let _ = client.put(key, value, Some(options)).await;
                        }
                    }
                }
                Ok(None) => {
                    error!(
                        "[KeepAlive] KeepAlive 流已关闭 (服务: {}, Lease: {}), 服务可能被剔除",
                        service_name_clone, lease_id
                    );
                    break;
                }
                Err(e) => {
                    error!(
                        "[KeepAlive] KeepAlive 响应读取失败 (服务: {}, Lease: {}): {}",
                        service_name_clone, lease_id, e
                    );
                    break;
                }
            }
        }
    });

    let mut tasks = self.keepalive_tasks.write().await;
    tasks.insert(service_name, task);
}
```

### Cron解析

```rust
/// Cron 表达式解析器
pub struct CronParser {
    schedule: Schedule,
}

impl CronParser {
    pub fn new(expr: &str) -> Result<Self, Error> {
        let field_count = expr.split_whitespace().count();
        if field_count != 6 {
            return Err(Error::CronFieldCount(format!(
                "Cron 应包含6个字段，但收到: {} 字段 ({})",
                field_count, expr
            )));
        }

        let schedule = Schedule::from_str(expr).map_err(|e| map_cron_error(expr, e.to_string()))?;

        Ok(Self { schedule })
    }

    /// 获取在指定时间窗口内的所有触发时间
    pub fn next_triggers_in_window(
        &self,
        start: DateTime<chrono::Utc>,
        end: DateTime<chrono::Utc>,
    ) -> Vec<DateTime<chrono::Utc>> {
        let local_offset = *Local::now().offset();
        let mut current = start.with_timezone(&local_offset);
        let end_fixed = end.with_timezone(&local_offset);
        let mut triggers = Vec::new();

        while let Some(next) = self.schedule.after(&current).next() {
            if next > end_fixed {
                break;
            }

            let next_utc = next.with_timezone(&chrono::Utc);
            triggers.push(next_utc);
            current = next;
        }

        triggers
    }
}
```

### 扫描任务

```rust
/// 扫描并分发任务
async fn scan_and_dispatch(
    db: &Arc<MongoDataSource>,
    task_queue: &Arc<TaskQueue>,
    scan_interval_secs: u64,
    last_scan_end_time: &Arc<RwLock<DateTime<Utc>>>,
) -> Result<usize> {
    let now = Utc::now();

    let (scan_window_start, scan_window_end) =
        Self::calculate_scan_window(now, scan_interval_secs, last_scan_end_time).await;

    info!(
        "[Dispatcher] 开始扫描任务，窗口: {} 到 {}",
        scan_window_start.format("%H:%M:%S"),
        scan_window_end.format("%H:%M:%S")
    );

    let enabled_tasks = db
        .find_tasks(
            Some(doc! {
                "enabled": true,
                "deleted_at": null
            }),
            None,
        )
        .await
        .map_err(|e| Error::Database(format!("查询任务失败: {}", e)))?;
    let total_tasks = enabled_tasks.len() as i32;

    let task_ids: Vec<ObjectId> = enabled_tasks.iter().filter_map(|task| task.id).collect();

    let all_existing_instances = if !task_ids.is_empty() {
        db.find_task_instances(
            Some(doc! {
                "task_id": { "$in": task_ids },
                "scheduled_time": { "$gte": now, "$lte": scan_window_end }
            }),
            None,
        )
        .await
        .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?
    } else {
        Vec::new()
    };

    let mut existing_instances_map: TaskInstanceMap = TaskInstanceMap::new();
    for instance in all_existing_instances {
        existing_instances_map
            .entry(instance.task_id)
            .or_default()
            .insert(instance.scheduled_time.timestamp());
    }

    let mut dispatched_count = 0;

    for task in enabled_tasks {
        if let Some(task_id) = task.id {
            match Self::dispatch_task_instances(
                db,
                task_queue,
                &task,
                &now,
                &scan_window_end,
                existing_instances_map.get(&task_id),
            )
            .await
            {
                Ok(count) => {
                    if count > 0 {
                        info!(
                            "[Dispatcher] 任务 {} 创建并分发 {} 个实例",
                            task.name, count
                        );
                        dispatched_count += count;
                    }
                }
                Err(e) => {
                    error!("[Dispatcher] 分发任务 {} 失败: {}", task.name, e);
                }
            }
        }
    }

    Ok(dispatched_count)
}
```

### 分发任务

```rust
/// 为任务创建并分发实例
async fn dispatch_task_instances(
    db: &Arc<MongoDataSource>,
    task_queue: &Arc<TaskQueue>,
    task: &Task,
    now: &DateTime<Utc>,
    scan_window_end: &DateTime<Utc>,
    existing_instances: Option<&std::collections::HashSet<i64>>,
) -> Result<usize> {
    let task_id = task
        .id
        .ok_or_else(|| Error::Validation("任务 ID 不能为空".to_string()))?;
    let cron_parser = CronParser::new(&task.schedule)
        .map_err(|e| Error::Scheduling(format!("解析 Cron 表达式失败: {}", e)))?;

    let next_triggers = cron_parser.next_triggers_in_window(*now, *scan_window_end);

    if next_triggers.is_empty() {
        return Ok(0);
    }

    let empty_set = std::collections::HashSet::new();
    let existing_scheduled_times = existing_instances.unwrap_or(&empty_set);

    let mut dispatched_count = 0;

    for scheduled_time in next_triggers {
        let scheduled_timestamp = scheduled_time.timestamp();

        if existing_scheduled_times.contains(&scheduled_timestamp) {
            debug!(
                "任务 {} 在 {} 的实例已存在，跳过",
                task.name, scheduled_time
            );
            continue;
        }

        let instance = TaskInstance {
            id: None,
            task_id,
            scheduled_time,
            status: TaskStatus::Pending,
            executor_id: None,
            start_time: None,
            end_time: None,
            retry_count: 0,
            result: None,
            triggered_by: crate::types::TriggeredBy::Scheduler,
            created_at: *now,
        };

        let instance_id = db
            .create_task_instance(&instance)
            .await
            .map_err(|e| Error::Database(format!("创建任务实例失败: {}", e)))?;

        let task_msg = crate::executor::TaskMessage {
            instance_id,
            task_id,
            task_name: task.name.clone(),
            scheduled_time: scheduled_timestamp,
            retry_count: 0,
            triggered_by: crate::types::TriggeredBy::Scheduler,
        };

        task_queue
            .publish_task(task_msg)
            .await
            .map_err(|e| Error::MessageQueue(format!("发布任务到队列失败: {}", e)))?;

        debug!(
            "为任务 {} 创建实例 {}，计划执行时间: {}",
            task.name, instance_id, scheduled_time
        );

        dispatched_count += 1;
    }

    Ok(dispatched_count)
}
```

### 重试逻辑

```rust
/// 重试策略
#[derive(Debug, Clone, Copy)]
pub enum RetryStrategy {
    Fixed { delay_seconds: i64 },
    Exponential {
        base_delay_seconds: i64,
        max_delay_seconds: i64,
    },
    Linear {
        initial_delay_seconds: i64,
        increment_seconds: i64,
    },
}

/// 计算重试延迟时间
pub fn calculate_retry_delay(
    &self,
    _task: &Task,
    instance: &TaskInstance,
    config: &RetryConfig,
) -> i64 {
    let retry_count = instance.retry_count;

    match config.strategy {
        RetryStrategy::Fixed { delay_seconds } => delay_seconds,
        RetryStrategy::Exponential {
            base_delay_seconds,
            max_delay_seconds,
        } => {
            let delay = base_delay_seconds * 2_i64.pow(retry_count as u32);
            delay.min(max_delay_seconds)
        }
        RetryStrategy::Linear {
            initial_delay_seconds,
            increment_seconds,
        } => initial_delay_seconds + increment_seconds * retry_count as i64,
    }
}
```

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

## 测试统计

### 单元测试

| 模块            | 测试数量 | 通过   | 失败  | 忽略  |
| --------------- | -------- | ------ | ----- | ----- |
| cron_parser     | 12       | 12     | 0     | 0     |
| retry_logic     | 10       | 10     | 0     | 0     |
| task_execution  | 14       | 14     | 0     | 0     |
| task_management | 21       | 21     | 0     | 0     |
| **总计**        | **57**   | **57** | **0** | **0** |

### 集成测试

| 模块                    | 测试数量 | 通过   | 失败  | 忽略  |
| ----------------------- | -------- | ------ | ----- | ----- |
| cron_parser_integration | 10       | 10     | 0     | 0     |
| retry_logic             | 10       | 10     | 0     | 0     |
| task_execution          | 14       | 14     | 0     | 0     |
| task_management         | 21       | 21     | 0     | 0     |
| **总计**                | **55**   | **55** | **0** | **0** |

### 基准测试

| 模块                    | 测试数量 | 平均时间 |
| ----------------------- | -------- | -------- |
| cron_parser_bench       | 12       | 1.5 µs   |
| retry_calculation_bench | 9        | 315 ps   |
| task_creation_bench     | 4        | 1.5 µs   |
| task_query_bench        | 8        | 12.0 µs  |
| **总计**                | **33**   | -        |

> 测试时间：2026-03-12
> 测试命令：`cargo test`、`cargo test --tests`、`cargo bench`

## 许可证

MIT License
