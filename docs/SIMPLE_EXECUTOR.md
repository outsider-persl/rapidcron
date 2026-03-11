# Simple Executor 使用说明

Simple Executor 是 RapidCron 的简单任务执行器实现，用于测试和演示分布式任务调度功能。

## 功能特性

1. **服务注册** - 自动注册到 etcd 服务发现
2. **健康检查** - 提供 HTTP 健康检查接口
3. **任务执行** - 每 5 秒检查并执行待处理任务
4. **心跳保持** - 每 10 秒发送心跳保持服务在线

## 快速开始

### 1. 启动依赖服务

```bash
docker-compose up -d mongodb etcd rabbitmq
```

### 2. 启动调度器

```bash
cargo run --bin rapidcron
```

### 3. 启动执行器（可启动多个实例）

```bash
# 启动第一个执行器
cargo run --bin simple-executor executor-1

# 启动第二个执行器（在另一个终端）
cargo run --bin simple-executor executor-2
```

### 4. 查看执行器状态

```bash
# 健康检查
curl http://localhost:8081/health

# 执行器信息
curl http://localhost:8081/info
```

## API 接口

### 健康检查

```bash
GET http://localhost:8081/health
```

响应示例：
```json
{
  "status": "ok",
  "executor_id": "executor-1",
  "timestamp": 1704067200
}
```

### 执行器信息

```bash
GET http://localhost:8081/info
```

响应示例：
```json
{
  "executor_id": "executor-1",
  "service_name": "simple-executor",
  "version": "0.1.0"
}
```

## 测试任务

Simple Executor 启动后会自动插入两个测试任务：

1. **test-hello** - 每 5 秒执行一次，打印 Hello
2. **test-date** - 每 10 秒执行一次，显示当前日期

## Cron 表达式格式

RapidCron 使用标准的 6 字段 Cron 表达式：

```
秒 分 时 日 月 周
*  *  *  *  *  *
```

示例：
- `0/5 * * * * *` - 每 5 秒执行一次
- `0 */10 * * * *` - 每 10 分钟执行一次
- `0 0 * * * *` - 每小时执行一次
- `0 0 0 * * *` - 每天执行一次

## 架构说明

### 执行流程

1. **服务注册**
   - 执行器启动时注册到 etcd
   - 包含执行器 ID、主机、端口等信息

2. **任务调度**
   - RapidCron 调度器扫描待执行任务
   - 创建任务实例并写入数据库
   - 通过 RabbitMQ 分发任务（可选）

3. **任务执行**
   - 执行器每 5 秒检查数据库
   - 获取状态为 `pending` 的任务实例
   - 标记为 `running` 并执行
   - 执行完成后更新状态为 `success` 或 `failed`

4. **心跳保持**
   - 每 10 秒发送心跳
   - 保持服务在 etcd 中在线

## 开发 SDK 参考

Simple Executor 的代码结构可以作为开发 RapidCron SDK 的参考：

### 核心组件

1. **服务注册** - 参考 `EtcdManager::registry().register()`
2. **心跳保持** - 参考 `EtcdManager::registry().heartbeat()`
3. **任务获取** - 参考 `execute_next_task()` 函数
4. **状态更新** - 参考 MongoDB 更新操作
5. **健康检查** - 参考 HTTP 接口实现

### 关键数据结构

```rust
use rapidcron::{
    coord::{EtcdManager, ServiceInfo},
    types::{Task, TaskInstance, TaskStatus, TaskType},
};
```

## 故障排查

### 执行器无法启动

1. 检查 etcd 是否运行：`docker ps | grep etcd`
2. 检查 MongoDB 是否运行：`docker ps | grep mongodb`
3. 检查端口是否被占用：`lsof -i :8081`

### 任务未执行

1. 检查任务是否启用：`enabled: true`
2. 检查 Cron 表达式是否正确（6 个字段）
3. 查看执行器日志：`tail -f logs/simple-executor.log`

### 服务未注册

1. 检查 etcd 连接：`etcdctl get rapidcron/services/ --prefix`
2. 查看执行器日志中的错误信息

## 扩展开发

基于 Simple Executor 可以开发更复杂的执行器：

1. **添加任务类型支持** - HTTP 请求、Shell 脚本等
2. **添加超时控制** - 防止任务长时间运行
3. **添加资源限制** - CPU、内存使用限制
4. **添加日志收集** - 收集任务执行日志
5. **添加监控指标** - Prometheus 指标导出

## 许可证

MIT License
