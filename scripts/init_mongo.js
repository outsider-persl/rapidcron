// init_mongo.js

db = db.getSiblingDB('rapidcron'); // 切换到目标数据库

// 清理旧数据
print("清理旧数据...");
db.tasks.deleteMany({});
db.task_instances.deleteMany({});
db.execution_logs.deleteMany({});
db.dispatch_logs.deleteMany({});
print("✓ 旧数据已清理");

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
var taskData = {
  _id: ObjectId("670000000000000000000001"),
  name: "Test Simple Executor",
  description: "每10秒调用一次 simple-executor 的测试接口",
  dependency_ids: [],
  type: "http",
  schedule: "*/10 * * * * *",
  enabled: true,
  payload: {
    url: "http://127.0.0.1:3000/execute",
    method: "GET"
  },
  timeout_seconds: NumberInt(30),
  max_retries: NumberInt(3),
  created_at: new Date(),
  updated_at: new Date(),
  deleted_at: null
};

print("插入测试数据...");
db.tasks.insertOne(taskData);
print("✓ 测试数据已插入");

// ======================
// 2. task_instances 集合
// ======================

db.createCollection("task_instances");

// 创建索引
db.task_instances.createIndex({ task_id: 1, scheduled_time: -1 });
db.task_instances.createIndex({ status: 1 });
db.task_instances.createIndex({ scheduled_time: 1 });
db.task_instances.createIndex({ end_time: 1 });

// ======================
// 3. execution_logs 集合
// ======================

db.createCollection("execution_logs");

// 创建索引
db.execution_logs.createIndex({ task_id: 1, end_time: -1 });
db.execution_logs.createIndex({ scheduled_time: 1 });
db.execution_logs.createIndex({ status: 1, end_time: -1 });
db.execution_logs.createIndex({ triggered_by: 1, end_time: -1 });

// ======================
// 4. dispatch_logs 集合
// ======================

db.createCollection("dispatch_logs");

// 创建索引
db.dispatch_logs.createIndex({ scan_time: -1 });
db.dispatch_logs.createIndex({ task_id: 1, scan_time: -1 });
db.dispatch_logs.createIndex({ executor_id: 1, scan_time: -1 });

print("✅ Database initialized with collections, indexes, and sample data.");