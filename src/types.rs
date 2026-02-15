#![allow(dead_code)]
use chrono::{DateTime, Local};
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Command,
    Http,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

/// 触发方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TriggeredBy {
    Scheduler,
    Manual,
}

/// 任务载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskPayload {
    Command {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_seconds: Option<i32>,
    },
    Http {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        method: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_seconds: Option<i32>,
    },
}

impl TryFrom<TaskPayload> for Bson {
    type Error = serde_json::Error;
    
    fn try_from(payload: TaskPayload) -> Result<Self, Self::Error> {
        let value = serde_json::to_value(payload)?;
        match Bson::try_from(value) {
            Ok(bson) => Ok(bson),
            Err(_) => Ok(Bson::Null),
        }
    }
}

/// 任务集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependency_ids: Vec<ObjectId>,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub schedule: String,
    pub enabled: bool,
    pub payload: TaskPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Local>>,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

/// 任务实例集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstance {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub task_id: ObjectId,
    pub scheduled_time: DateTime<Local>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Local>>,
    pub retry_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ExecutionResult>,
    pub created_at: DateTime<Local>,
}

/// 执行日志集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub task_id: ObjectId,
    pub task_name: String,
    pub instance_id: ObjectId,
    pub scheduled_time: DateTime<Local>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Local>>,
    pub end_time: DateTime<Local>,
    pub status: TaskStatus,
    #[serde(rename = "duration_ms")]
    pub duration_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(rename = "triggered_by")]
    pub triggered_by: TriggeredBy,
}

/// 创建任务的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependency_ids: Vec<ObjectId>,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub schedule: String,
    #[serde(default)]
    pub enabled: bool,
    pub payload: TaskPayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<i32>,
}

/// 更新任务的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_ids: Option<Vec<ObjectId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TaskPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<i32>,
}

/// 任务查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

/// 任务实例查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstanceFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_after: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_before: Option<DateTime<Local>>,
}

/// 执行日志查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered_by: Option<TriggeredBy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_after: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_before: Option<DateTime<Local>>,
}

// 为请求结构体实现 Default trait
impl Default for CreateTaskRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            dependency_ids: Vec::new(),
            task_type: TaskType::Command,
            schedule: String::new(),
            enabled: false,
            payload: TaskPayload::Command {
                command: String::new(),
                timeout_seconds: None,
            },
            timeout_seconds: None,
            max_retries: None,
        }
    }
}

impl Default for UpdateTaskRequest {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            dependency_ids: None,
            schedule: None,
            enabled: None,
            payload: None,
            timeout_seconds: None,
            max_retries: None,
        }
    }
}

impl Default for TaskFilter {
    fn default() -> Self {
        Self {
            name: None,
            enabled: None,
            task_type: None,
            deleted: Some(false), // 默认只查找未删除的任务
        }
    }
}

impl Default for TaskInstanceFilter {
    fn default() -> Self {
        Self {
            task_id: None,
            status: None,
            executor_id: None,
            start_after: None,
            start_before: None,
        }
    }
}

impl Default for ExecutionLogFilter {
    fn default() -> Self {
        Self {
            task_id: None,
            status: None,
            triggered_by: None,
            end_after: None,
            end_before: None,
        }
    }
}