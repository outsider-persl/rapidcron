use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CronExpression {
    Once(DateTime<Utc>),
    Interval { seconds: u64 },
    Cron(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: CronExpression,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub timeout: Option<u64>,
    pub max_retries: u32,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub retry_count: u32,
}

impl Task {
    pub fn new(
        name: String,
        cron_expression: CronExpression,
        command: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            cron_expression,
            command,
            args: Vec::new(),
            env: HashMap::new(),
            timeout: None,
            max_retries: 3,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            last_run: None,
            next_run: None,
            retry_count: 0,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: String,
    pub task_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error: Option<String>,
}

impl TaskExecution {
    pub fn new(task_id: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_id,
            started_at: Utc::now(),
            finished_at: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: NodeStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub task_capacity: u32,
    pub current_tasks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Inactive,
    Maintenance,
}

impl NodeInfo {
    pub fn new(name: String, address: String, task_capacity: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            address,
            status: NodeStatus::Active,
            last_heartbeat: Utc::now(),
            task_capacity,
            current_tasks: 0,
        }
    }
}