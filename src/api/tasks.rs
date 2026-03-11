use axum::{
    Json,
    extract::{Path, Query, State},
};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;

use crate::{
    coord::EtcdManager,
    error::Error,
    storage::mongo::MongoDataSource,
    types::{
        ApiResponse, CreateTaskRequest, PaginatedResponse, StatsResponse, Task, TaskInstance,
        TaskStatus, TriggerTaskRequest, UpdateTaskRequest, parse_object_id, parse_object_ids,
    },
};

/// API 状态
#[derive(Clone)]
pub struct ApiState {
    pub db: MongoDataSource,
    pub etcd_manager: Option<Arc<EtcdManager>>,
}

impl ApiState {
    pub fn new(db: MongoDataSource) -> Self {
        Self {
            db,
            etcd_manager: None,
        }
    }

    pub fn with_etcd(mut self, etcd_manager: Arc<EtcdManager>) -> Self {
        self.etcd_manager = Some(etcd_manager);
        self
    }
}

/// 任务列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct TaskListQuery {
    pub enabled: Option<bool>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

/// 任务实例列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct InstanceListQuery {
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
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

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

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
    let object_id = parse_object_id(&id).map_err(|e| Error::Validation(e))?;

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
    let task = req.to_task().map_err(|e| Error::Validation(e))?;

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
    let object_id = parse_object_id(&id).map_err(|e| Error::Validation(e))?;

    let mut update = doc! { "updated_at": chrono::Utc::now() };

    if let Some(name) = req.name {
        update.insert("name", name);
    }
    if let Some(description) = req.description {
        update.insert("description", description);
    }
    if let Some(schedule) = req.schedule {
        update.insert("schedule", schedule);
    }
    if let Some(enabled) = req.enabled {
        update.insert("enabled", enabled);
    }
    if let Some(timeout_seconds) = req.timeout_seconds {
        update.insert("timeout_seconds", timeout_seconds);
    }
    if let Some(max_retries) = req.max_retries {
        update.insert("max_retries", max_retries);
    }
    if let Some(dependency_ids) = req.dependency_ids {
        let ids = parse_object_ids(&dependency_ids);
        update.insert("dependency_ids", ids);
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
    let object_id = parse_object_id(&id).map_err(|e| Error::Validation(e))?;

    let update = doc! {
        "deleted_at": chrono::Utc::now(),
        "enabled": false
    };

    state.db.update_task(object_id, update).await?;

    Ok(Json(ApiResponse::success("任务已删除".to_string())))
}

/// 启用任务
pub async fn enable_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, Error> {
    let object_id = parse_object_id(&id).map_err(|e| Error::Validation(e))?;

    let update = doc! {
        "enabled": true,
        "updated_at": chrono::Utc::now()
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
    let object_id = parse_object_id(&id).map_err(|e| Error::Validation(e))?;

    let update = doc! {
        "enabled": false,
        "updated_at": chrono::Utc::now()
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
    Json(_req): Json<TriggerTaskRequest>,
) -> Result<Json<ApiResponse<TaskInstance>>, Error> {
    let object_id = parse_object_id(&id).map_err(|e| Error::Validation(e))?;

    let _task = state
        .db
        .get_task(object_id)
        .await?
        .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

    let instance = TaskInstance {
        id: None,
        task_id: object_id,
        scheduled_time: chrono::Utc::now(),
        status: TaskStatus::Pending,
        executor_id: None,
        start_time: None,
        end_time: None,
        retry_count: 0,
        result: None,
        created_at: chrono::Utc::now(),
    };

    let instance_id = state.db.create_task_instance(&instance).await?;

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

    if let Some(task_id) = query.task_id {
        if let Ok(object_id) = ObjectId::parse_str(&task_id) {
            filter.insert("task_id", object_id);
        }
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

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

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
