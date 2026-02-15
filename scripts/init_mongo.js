// init_mongo.js

db = db.getSiblingDB('rapidcron'); // 切换到目标数据库

// ======================
// 1. tasks 集合
// ======================

// 创建集合（MongoDB 通常自动创建，但显式调用更清晰）
db.createCollection("tasks");

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

// 插入测试数据
db.tasks.insertOne({
  _id: ObjectId("670000000000000000000001"),
  name: "Test HTTP Task",
  description: "A simple HTTP task for testing",
  dependency_ids: [],
  type: "http",
  schedule: "0 0 * * * *", // 每天午夜执行
  enabled: true,
  payload: { url: "https://example.com/health", method: "GET" },
  timeout_seconds: 30,
  max_retries: 3,
  created_at: new Date("2026-02-15T00:00:00"),
  updated_at: new Date("2026-02-15T00:00:00"),
  deleted_at: null
});

// ======================
// 2. task_instances 集合
// ======================

db.createCollection("task_instances");

// 创建索引
db.task_instances.createIndex({ task_id: 1, scheduled_time: -1 });
db.task_instances.createIndex({ status: 1 });
db.task_instances.createIndex({ scheduled_time: 1 });
db.task_instances.createIndex({ end_time: 1 });

// 插入测试数据
db.task_instances.insertOne({
  _id: ObjectId("670000000000000000000101"),
  task_id: ObjectId("670000000000000000000001"),
  scheduled_time: new Date("2026-02-16T00:00:00"),
  status: "pending",
  executor_id: null,
  start_time: null,
  end_time: null,
  retry_count: 0,
  result: null,
  created_at: new Date("2026-02-15T06:00:00")
});

// ======================
// 3. execution_logs 集合
// ======================

db.createCollection("execution_logs");

// 创建索引
db.execution_logs.createIndex({ task_id: 1, end_time: -1 });
db.execution_logs.createIndex({ scheduled_time: 1 });
db.execution_logs.createIndex({ status: 1, end_time: -1 });
db.execution_logs.createIndex({ triggered_by: 1, end_time: -1 });

// 插入测试数据（模拟一次成功执行）
db.execution_logs.insertOne({
  _id: ObjectId("670000000000000000000201"),
  task_id: ObjectId("670000000000000000000001"),
  task_name: "Test HTTP Task",
  instance_id: ObjectId("670000000000000000000101"),
  scheduled_time: new Date("2026-02-16T00:00:00"),
  start_time: new Date("2026-02-16T00:00:05"),
  end_time: new Date("2026-02-16T00:00:10"),
  status: "success",
  duration_ms: 5000,
  output_summary: "HTTP 200 OK",
  error_message: null,
  triggered_by: "scheduler"
});

print("✅ Database initialized with collections, indexes, and sample data.");