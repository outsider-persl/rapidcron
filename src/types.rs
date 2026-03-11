use chrono::{DateTime, Utc};
use mongodb::bson::{Bson, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Command,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TriggeredBy {
    Scheduler,
    Manual,
}

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
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstance {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub task_id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub scheduled_time: DateTime<Utc>,
    pub status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub end_time: Option<DateTime<Utc>>,
    pub retry_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ExecutionResult>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub task_id: ObjectId,
    pub task_name: String,
    pub instance_id: ObjectId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub scheduled_time: DateTime<Utc>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub end_time: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchLog {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(
        rename = "scan_time",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime"
    )]
    pub scan_time: DateTime<Utc>,
    #[serde(
        rename = "scan_window_start",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime"
    )]
    pub scan_window_start: DateTime<Utc>,
    #[serde(
        rename = "scan_window_end",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime"
    )]
    pub scan_window_end: DateTime<Utc>,
    #[serde(rename = "total_tasks")]
    pub total_tasks: i32,
    #[serde(rename = "enabled_tasks")]
    pub enabled_tasks: i32,
    #[serde(rename = "dispatched_instances")]
    pub dispatched_instances: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependency_ids: Vec<String>,
    #[serde(rename = "type")]
    pub task_type: Option<String>,
    pub schedule: String,
    #[serde(default)]
    pub enabled: bool,
    pub command: Option<String>,
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<i32>,
}

impl CreateTaskRequest {
    pub fn to_task(&self) -> Result<Task, String> {
        let dependency_ids: Vec<ObjectId> = self
            .dependency_ids
            .iter()
            .filter_map(|id| ObjectId::parse_str(id).ok())
            .collect();

        let task_type = match self.task_type.as_deref() {
            Some("http") => TaskType::Http,
            _ => TaskType::Command,
        };

        let payload = if task_type == TaskType::Http {
            TaskPayload::Http {
                url: self.url.clone().unwrap_or_default(),
                method: None,
                headers: None,
                body: None,
                timeout_seconds: self.timeout_seconds,
            }
        } else {
            TaskPayload::Command {
                command: self.command.clone().unwrap_or_default(),
                timeout_seconds: self.timeout_seconds,
            }
        };

        Ok(Task {
            id: None,
            name: self.name.clone(),
            description: self.description.clone(),
            dependency_ids,
            task_type,
            schedule: self.schedule.clone(),
            enabled: self.enabled,
            payload,
            timeout_seconds: self.timeout_seconds,
            max_retries: self.max_retries,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn from_items(items: Vec<T>, page: usize, page_size: usize) -> Self {
        let total = items.len() as u64;
        let total_pages = if total == 0 {
            0
        } else {
            ((total as f64 - 1.0) / page_size as f64).floor() as usize + 1
        };

        let start = (page - 1) * page_size;
        let end = (start + page_size).min(items.len());

        Self {
            items: items.into_iter().skip(start).take(end - start).collect(),
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_tasks: u64,
    pub enabled_tasks: u64,
    pub total_instances: u64,
    pub pending_instances: u64,
    pub running_instances: u64,
    pub success_instances: u64,
    pub failed_instances: u64,
}

pub fn parse_object_id(id: &str) -> Result<ObjectId, String> {
    ObjectId::parse_str(id).map_err(|_| "无效的 ID 格式".to_string())
}

pub fn parse_object_ids(ids: &[String]) -> Vec<ObjectId> {
    ids.iter()
        .filter_map(|id| ObjectId::parse_str(id).ok())
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInstanceFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub start_after: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub start_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLogFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered_by: Option<TriggeredBy>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub end_after: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime_optional"
    )]
    pub end_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub node_name: String,
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub status: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub active_tasks: u64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResponse {
    pub nodes: Vec<ClusterNode>,
    pub total_nodes: u64,
    pub active_nodes: u64,
}
