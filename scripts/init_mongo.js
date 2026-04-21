// init_mongo.js

db = db.getSiblingDB('rapidcron'); // 切换到目标数据库

// 清理旧数据
print("清理旧数据...");
db.tasks.deleteMany({});
db.task_instances.deleteMany({});
db.execution_logs.deleteMany({});
db.dispatch_logs.deleteMany({});
print("✓ 旧数据已清理");

function ensureCollection(name) {
  if (!db.getCollectionNames().includes(name)) {
    db.createCollection(name);
  }
}

// ======================
// 1. tasks 集合
// ======================

ensureCollection("tasks");

// 创建索引
db.tasks.createIndex(
  { name: 1 },
  {
    unique: true,
    partialFilterExpression: { deleted_at: null }
  }
);
db.tasks.createIndex({ "dependency_ids": 1 });
db.tasks.createIndex(
  { enabled: 1 },
  {
    partialFilterExpression: { enabled: true, deleted_at: null }
  }
);
db.tasks.createIndex({ deleted_at: 1 });

// 插入演示任务数据（覆盖成功、失败、节点状态、日志运维、手动触发等场景）
var now = new Date();
var taskDocs = [
  {
    _id: ObjectId("670000000000000000000001"),
    name: "demo-http-success-fast",
    description: "每20秒调用执行成功接口，用于展示成功曲线",
    dependency_ids: [],
    type: "http",
    schedule: "*/20 * * * * *",
    enabled: true,
    payload: { url: "http://127.0.0.1:8081/execute", method: "GET" },
    timeout_seconds: NumberInt(20),
    max_retries: NumberInt(1),
    created_at: now,
    updated_at: now,
    deleted_at: null
  },
  {
    _id: ObjectId("670000000000000000000002"),
    name: "demo-http-error-retry",
    description: "每45秒调用失败接口，用于展示失败与重试",
    dependency_ids: [],
    type: "http",
    schedule: "*/45 * * * * *",
    enabled: true,
    payload: { url: "http://127.0.0.1:8081/error", method: "GET" },
    timeout_seconds: NumberInt(20),
    max_retries: NumberInt(3),
    created_at: now,
    updated_at: now,
    deleted_at: null
  },
  {
    _id: ObjectId("670000000000000000000003"),
    name: "demo-http-health-check",
    description: "每30秒检查执行器健康状态",
    dependency_ids: [],
    type: "http",
    schedule: "*/30 * * * * *",
    enabled: true,
    payload: { url: "http://127.0.0.1:8081/health", method: "GET" },
    timeout_seconds: NumberInt(15),
    max_retries: NumberInt(1),
    created_at: now,
    updated_at: now,
    deleted_at: null
  },
  {
    _id: ObjectId("670000000000000000000004"),
    name: "demo-http-node-metrics",
    description: "每40秒采集节点资源信息，用于节点监控演示",
    dependency_ids: [],
    type: "http",
    schedule: "*/40 * * * * *",
    enabled: true,
    payload: { url: "http://127.0.0.1:8081/node", method: "GET" },
    timeout_seconds: NumberInt(15),
    max_retries: NumberInt(2),
    created_at: now,
    updated_at: now,
    deleted_at: null
  },
  {
    _id: ObjectId("670000000000000000000005"),
    name: "demo-cleanup-scheduler-logs",
    description: "每6小时清理 logs 目录30天前日志",
    dependency_ids: [],
    type: "command",
    schedule: "0 0 */6 * * *",
    enabled: true,
    payload: {
      command: "bash -lc 'mkdir -p logs && find logs -type f -name \"*.log\" -mtime +30 -delete'",
      timeout_seconds: NumberInt(60)
    },
    timeout_seconds: NumberInt(60),
    max_retries: NumberInt(2),
    created_at: now,
    updated_at: now,
    deleted_at: null
  },
  {
    _id: ObjectId("670000000000000000000006"),
    name: "demo-export-dispatch-stats-hourly",
    description: "每小时整点统计分发结果并导出到 logs",
    dependency_ids: [],
    type: "command",
    schedule: "0 0 * * * *",
    enabled: true,
    payload: {
      command: "bash -lc 'mkdir -p logs && NOW=\"$(date \"+%Y-%m-%d %H:%M:%S\")\" && TS=\"$(date \"+%Y-%m-%d-%H:%M:%S\")\" && OUT=\"logs/dispatch-stats-${TS}.log\" && TOTAL=\"$(ls -1 logs/*.log 2>/dev/null | wc -l | tr -d \" \")\" && RECENT=\"$(find logs -type f -name \"*.log\" -mtime -1 | wc -l | tr -d \" \")\" && OLD=\"$(find logs -type f -name \"*.log\" -mtime +30 | wc -l | tr -d \" \")\" && { echo \"dispatch_stats_time=${NOW}\"; echo \"total_log_files=${TOTAL}\"; echo \"recent_24h_log_files=${RECENT}\"; echo \"older_than_30d_log_files=${OLD}\"; } > \"${OUT}\"'",
      timeout_seconds: NumberInt(60)
    },
    timeout_seconds: NumberInt(60),
    max_retries: NumberInt(2),
    created_at: now,
    updated_at: now,
    deleted_at: null
  },
  {
    _id: ObjectId("670000000000000000000007"),
    name: "demo-manual-only-task",
    description: "默认禁用，供前端手动触发演示",
    dependency_ids: [],
    type: "http",
    schedule: "0 */10 * * * *",
    enabled: false,
    payload: { url: "http://127.0.0.1:8081/execute", method: "GET" },
    timeout_seconds: NumberInt(20),
    max_retries: NumberInt(0),
    created_at: now,
    updated_at: now,
    deleted_at: null
  }
];

print("插入测试数据...");
db.tasks.insertMany(taskDocs);
print("✓ 测试数据已插入");
print("  - demo-http-success-fast (成功链路)");
print("  - demo-http-error-retry (失败/重试链路)");
print("  - demo-http-health-check (健康检查链路)");
print("  - demo-http-node-metrics (节点监控链路)");
print("  - demo-cleanup-scheduler-logs (日志清理链路)");
print("  - demo-export-dispatch-stats-hourly (分发统计导出链路)");
print("  - demo-manual-only-task (手动触发链路)");

// ======================
// 2. task_instances 集合
// ======================

ensureCollection("task_instances");

// 创建索引
db.task_instances.createIndex({ task_id: 1, scheduled_time: -1 });
db.task_instances.createIndex({ status: 1 });
db.task_instances.createIndex({ scheduled_time: 1 });
db.task_instances.createIndex({ end_time: 1 });
db.task_instances.createIndex({ triggered_by: 1, created_at: -1 });

// ======================
// 3. execution_logs 集合
// ======================

ensureCollection("execution_logs");

// 创建索引
db.execution_logs.createIndex({ task_id: 1, end_time: -1 });
db.execution_logs.createIndex({ scheduled_time: 1 });
db.execution_logs.createIndex({ status: 1, end_time: -1 });
db.execution_logs.createIndex({ triggered_by: 1, end_time: -1 });

// ======================
// 4. dispatch_logs 集合
// ======================

ensureCollection("dispatch_logs");

// 创建索引
db.dispatch_logs.createIndex({ scan_time: -1 });
db.dispatch_logs.createIndex({ scan_window_start: -1, scan_window_end: -1 });
db.dispatch_logs.createIndex({ error_message: 1, scan_time: -1 });

print("✅ Database initialized with collections, indexes, and sample data.");