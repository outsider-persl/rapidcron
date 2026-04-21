use axum::{
    Json,
    extract::{Path, Query, State},
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;

use crate::{
    error::Error,
    types::{
        ApiResponse, CreateTaskRequest, PaginatedResponse, StatsResponse, Task, TaskInstance,
        TaskPayload, TaskStatus, TaskType, TriggerTaskRequest, TriggeredBy, UpdateTaskRequest,
        parse_object_id, parse_object_ids,
    },
};

use super::super::models::api_state::ApiState;

/// 任务列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct TaskListQuery {
    pub enabled: Option<bool>,
    pub name: Option<String>,
    pub task_type: Option<String>,
    pub page: Option<String>,
    pub page_size: Option<String>,
}

/// 任务实例列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct InstanceListQuery {
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<String>,
    pub page_size: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateTestTasksResponse {
    pub created: Vec<Task>,
    pub existed: Vec<Task>,
}

/// 获取任务列表
pub async fn list_tasks(
    State(state): State<ApiState>,
    Query(query): Query<TaskListQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<Task>>>, Error> {
    let mut filter = doc! { "deleted_at": null };

    if let Some(enabled) = query.enabled {
        filter.insert("enabled", enabled);
    }

    if let Some(name) = query.name {
        filter.insert("name", doc! { "$regex": name, "$options": "i" });
    }

    if let Some(task_type) = query.task_type {
        filter.insert("type", task_type);
    }

    let page = query
        .page
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    let page_size = query
        .page_size
        .and_then(|ps| ps.parse::<usize>().ok())
        .unwrap_or(20);

    let tasks = state.db.find_tasks(Some(filter), None).await?;

    Ok(Json(ApiResponse::success(PaginatedResponse::from_items(
        tasks, page, page_size,
    ))))
}

/// 获取任务详情
pub async fn get_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let task = state
        .db
        .get_task(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

    Ok(Json(ApiResponse::success(task)))
}

/// 创建任务
pub async fn create_task(
    State(state): State<ApiState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, Error> {
    let task = req.to_task().map_err(Error::Validation)?;

    let task_id = state.db.create_task(&task).await?;

    let created_task = state
        .db
        .get_task(task_id)
        .await?
        .ok_or_else(|| Error::Execution("任务创建失败".to_string()))?;

    Ok(Json(ApiResponse::success(created_task)))
}

/// 更新任务
pub async fn update_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let mut update = doc! { "$set": { "updated_at": chrono::Utc::now() } };

    if let Some(name) = req.name {
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("name", name);
    }
    if let Some(description) = req.description {
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("description", description);
    }
    if let Some(schedule) = req.schedule {
        // 验证Cron表达式
        if let Err(e) = cron::Schedule::from_str(&schedule) {
            return Err(Error::Validation(format!("无效的Cron表达式: {}", e)));
        }
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("schedule", schedule);
    }
    if let Some(enabled) = req.enabled {
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("enabled", enabled);
    }
    if let Some(timeout_seconds) = req.timeout_seconds {
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("timeout_seconds", timeout_seconds);
    }
    if let Some(max_retries) = req.max_retries {
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("max_retries", max_retries);
    }
    if let Some(dependency_ids) = req.dependency_ids {
        let ids = parse_object_ids(&dependency_ids);
        update
            .get_mut("$set")
            .unwrap()
            .as_document_mut()
            .unwrap()
            .insert("dependency_ids", ids);
    }

    state.db.update_task(object_id, update).await?;

    let updated_task = state
        .db
        .get_task(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

    Ok(Json(ApiResponse::success(updated_task)))
}

/// 删除任务
pub async fn delete_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let update = doc! {
        "$set": {
            "deleted_at": chrono::Utc::now(),
            "enabled": false
        }
    };

    state.db.update_task(object_id, update).await?;

    Ok(Json(ApiResponse::success("任务已删除".to_string())))
}

/// 启用任务
pub async fn enable_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let update = doc! {
        "$set": {
            "enabled": true,
            "updated_at": chrono::Utc::now()
        }
    };

    state.db.update_task(object_id, update).await?;

    let task = state
        .db
        .get_task(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

    Ok(Json(ApiResponse::success(task)))
}

/// 禁用任务
pub async fn disable_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let update = doc! {
        "$set": {
            "enabled": false,
            "updated_at": chrono::Utc::now()
        }
    };

    state.db.update_task(object_id, update).await?;

    let task = state
        .db
        .get_task(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

    Ok(Json(ApiResponse::success(task)))
}

/// 手动触发任务
pub async fn trigger_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<TriggerTaskRequest>,
) -> Result<Json<ApiResponse<TaskInstance>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let task = state
        .db
        .get_task(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

    let now = chrono::Utc::now();
    let scheduled_time = req
        .scheduled_time
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or(now))
        .unwrap_or(now);

    let instance = TaskInstance {
        id: None,
        task_id: object_id,
        scheduled_time,
        status: TaskStatus::Pending,
        executor_id: None,
        start_time: None,
        end_time: None,
        retry_count: 0,
        result: None,
        triggered_by: TriggeredBy::Manual,
        created_at: now,
    };

    let instance_id = state.db.create_task_instance(&instance).await?;

    // 将任务发布到队列中
    if let Some(task_queue) = &state.task_queue {
        let task_msg = crate::executor::TaskMessage {
            instance_id,
            task_id: object_id,
            task_name: task.name.clone(),
            scheduled_time: scheduled_time.timestamp(),
            retry_count: 0,
            triggered_by: TriggeredBy::Manual,
        };

        task_queue
            .publish_task(task_msg)
            .await
            .map_err(|e| Error::Execution(format!("发布任务到队列失败: {}", e)))?;
    }

    let created_instance = state
        .db
        .get_task_instance(instance_id)
        .await?
        .ok_or_else(|| Error::Execution("任务实例创建失败".to_string()))?;

    Ok(Json(ApiResponse::success(created_instance)))
}

/// 获取任务实例列表
pub async fn list_instances(
    State(state): State<ApiState>,
    Query(query): Query<InstanceListQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<TaskInstance>>>, Error> {
    let mut filter = doc! {};

    if let Some(task_id) = query.task_id
        && let Ok(object_id) = ObjectId::parse_str(&task_id)
    {
        filter.insert("task_id", object_id);
    }

    if let Some(status) = query.status {
        let task_status = match status.as_str() {
            "pending" => "pending",
            "running" => "running",
            "success" => "success",
            "failed" => "failed",
            "cancelled" => "cancelled",
            _ => return Err(Error::Validation("无效的任务状态".to_string())),
        };
        filter.insert("status", task_status);
    }

    let page = query
        .page
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    let page_size = query
        .page_size
        .and_then(|ps| ps.parse::<usize>().ok())
        .unwrap_or(20);

    let instances = state.db.find_task_instances(Some(filter), None).await?;

    Ok(Json(ApiResponse::success(PaginatedResponse::from_items(
        instances, page, page_size,
    ))))
}

/// 获取任务实例详情
pub async fn get_instance(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TaskInstance>>, Error> {
    let object_id =
        ObjectId::parse_str(&id).map_err(|_| Error::Validation("无效的实例 ID".to_string()))?;

    let instance = state
        .db
        .get_task_instance(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务实例不存在".to_string()))?;

    Ok(Json(ApiResponse::success(instance)))
}

/// 获取统计信息
pub async fn get_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<StatsResponse>>, Error> {
    let all_tasks = state
        .db
        .find_tasks(Some(doc! { "deleted_at": null }), None)
        .await?;

    let enabled_tasks = state
        .db
        .find_tasks(
            Some(doc! {
                "deleted_at": null,
                "enabled": true
            }),
            None,
        )
        .await?;

    let all_instances = state.db.find_task_instances(None, None).await?;

    let pending_instances = all_instances
        .iter()
        .filter(|i| i.status == TaskStatus::Pending)
        .count();

    let running_instances = all_instances
        .iter()
        .filter(|i| i.status == TaskStatus::Running)
        .count();

    let success_instances = all_instances
        .iter()
        .filter(|i| i.status == TaskStatus::Success)
        .count();

    let failed_instances = all_instances
        .iter()
        .filter(|i| i.status == TaskStatus::Failed)
        .count();

    let stats = StatsResponse {
        total_tasks: all_tasks.len() as u64,
        enabled_tasks: enabled_tasks.len() as u64,
        total_instances: all_instances.len() as u64,
        pending_instances: pending_instances as u64,
        running_instances: running_instances as u64,
        success_instances: success_instances as u64,
        failed_instances: failed_instances as u64,
    };

    Ok(Json(ApiResponse::success(stats)))
}

/// 创建测试数据任务（幂等）
pub async fn create_test_tasks(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<CreateTestTasksResponse>>, Error> {
    let now = chrono::Utc::now();
    let mut created = Vec::new();
    let mut existed = Vec::new();

    let task_specs = vec![
        (
            "demo-http-success-fast",
            "演示任务：快速成功链路",
            TaskType::Http,
            "*/20 * * * * *",
            Some("http://127.0.0.1:8081/execute"),
            None,
            true,
            20,
            1,
        ),
        (
            "demo-http-error-retry",
            "演示任务：失败与重试链路",
            TaskType::Http,
            "*/45 * * * * *",
            Some("http://127.0.0.1:8081/error"),
            None,
            true,
            20,
            3,
        ),
        (
            "demo-http-health-check",
            "演示任务：执行器健康检查",
            TaskType::Http,
            "*/30 * * * * *",
            Some("http://127.0.0.1:8081/health"),
            None,
            true,
            15,
            1,
        ),
        (
            "demo-http-node-metrics",
            "演示任务：节点资源采集",
            TaskType::Http,
            "*/40 * * * * *",
            Some("http://127.0.0.1:8081/node"),
            None,
            true,
            15,
            2,
        ),
        (
            "demo-cleanup-scheduler-logs",
            "演示任务：每6小时清理 logs 目录30天前日志",
            TaskType::Command,
            "0 0 */6 * * *",
            None,
            Some(
                "bash -lc 'mkdir -p logs && find logs -type f -name \"*.log\" -mtime +30 -delete'",
            ),
            true,
            60,
            2,
        ),
        (
            "demo-export-dispatch-stats-hourly",
            "演示任务：每小时整点导出分发统计到 logs",
            TaskType::Command,
            "0 0 * * * *",
            None,
            Some(
                "bash -lc 'mkdir -p logs && NOW=\"$(date \"+%Y-%m-%d %H:%M:%S\")\" && TS=\"$(date \"+%Y-%m-%d-%H:%M:%S\")\" && OUT=\"logs/dispatch-stats-${TS}.log\" && TOTAL=\"$(ls -1 logs/*.log 2>/dev/null | wc -l | tr -d \" \")\" && RECENT=\"$(find logs -type f -name \"*.log\" -mtime -1 | wc -l | tr -d \" \")\" && OLD=\"$(find logs -type f -name \"*.log\" -mtime +30 | wc -l | tr -d \" \")\" && { echo \"dispatch_stats_time=${NOW}\"; echo \"total_log_files=${TOTAL}\"; echo \"recent_24h_log_files=${RECENT}\"; echo \"older_than_30d_log_files=${OLD}\"; } > \"${OUT}\"'",
            ),
            true,
            60,
            2,
        ),
        (
            "demo-manual-only-task",
            "演示任务：默认禁用，仅用于手动触发",
            TaskType::Http,
            "0 */10 * * * *",
            Some("http://127.0.0.1:8081/execute"),
            None,
            false,
            20,
            0,
        ),
    ];

    for (
        name,
        description,
        task_type,
        schedule,
        url,
        command,
        enabled,
        timeout_seconds,
        max_retries,
    ) in task_specs
    {
        let existing = state
            .db
            .find_tasks(
                Some(doc! {
                    "name": name,
                    "deleted_at": null
                }),
                None,
            )
            .await?;

        if let Some(task) = existing.into_iter().next() {
            existed.push(task);
            continue;
        }

        let task = Task {
            id: None,
            name: name.to_string(),
            description: Some(description.to_string()),
            dependency_ids: Vec::new(),
            task_type: task_type.clone(),
            schedule: schedule.to_string(),
            enabled,
            payload: match task_type {
                TaskType::Http => TaskPayload::Http {
                    url: url.unwrap_or_default().to_string(),
                    method: Some("GET".to_string()),
                    headers: None,
                    body: None,
                    timeout_seconds: Some(timeout_seconds),
                },
                TaskType::Command => TaskPayload::Command {
                    command: command.unwrap_or_default().to_string(),
                    timeout_seconds: Some(timeout_seconds),
                },
            },
            timeout_seconds: Some(timeout_seconds),
            max_retries: Some(max_retries),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let task_id = state.db.create_task(&task).await?;
        let inserted = state
            .db
            .get_task(task_id)
            .await?
            .ok_or_else(|| Error::Execution("测试任务创建后读取失败".to_string()))?;
        created.push(inserted);
    }

    Ok(Json(ApiResponse::success(CreateTestTasksResponse {
        created,
        existed,
    })))
}
