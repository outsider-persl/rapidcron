use axum::{
    Json,
    extract::{Path, Query, State},
};
use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;

use crate::{
    error::Error,
    types::{
        ApiResponse, CreateTaskRequest, ExecutionLog, PaginatedResponse, StatsResponse, Task,
        TaskInstance, TaskStatus, TriggerTaskRequest, UpdateTaskRequest, parse_object_id,
        parse_object_ids,
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

/// 执行日志列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct ExecutionLogListQuery {
    pub task_id: Option<String>,
    pub instance_id: Option<String>,
    pub status: Option<String>,
    pub triggered_by: Option<String>,
    pub page: Option<String>,
    pub page_size: Option<String>,
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

/// 获取执行日志列表
pub async fn list_execution_logs(
    State(state): State<ApiState>,
    Query(query): Query<ExecutionLogListQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<ExecutionLog>>>, Error> {
    let mut filter = doc! {};

    if let Some(task_id) = query.task_id
        && let Ok(object_id) = ObjectId::parse_str(&task_id)
    {
        filter.insert("task_id", object_id);
    }

    if let Some(instance_id) = query.instance_id
        && let Ok(object_id) = ObjectId::parse_str(&instance_id)
    {
        filter.insert("instance_id", object_id);
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

    if let Some(triggered_by) = query.triggered_by {
        let triggered_by_value = match triggered_by.as_str() {
            "scheduler" => "scheduler",
            "manual" => "manual",
            _ => return Err(Error::Validation("无效的触发方式".to_string())),
        };
        filter.insert("triggered_by", triggered_by_value);
    }

    let page = query
        .page
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    let page_size = query
        .page_size
        .and_then(|ps| ps.parse::<usize>().ok())
        .unwrap_or(20);

    let logs = state.db.find_execution_logs(Some(filter), None).await?;

    Ok(Json(ApiResponse::success(PaginatedResponse::from_items(
        logs, page, page_size,
    ))))
}

/// 获取执行日志详情
pub async fn get_execution_log(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<ExecutionLog>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let log = state
        .db
        .get_execution_log(object_id)
        .await?
        .ok_or_else(|| Error::Execution("执行日志不存在".to_string()))?;

    Ok(Json(ApiResponse::success(log)))
}
