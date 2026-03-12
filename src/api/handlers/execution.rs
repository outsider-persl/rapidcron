use axum::{
    Json,
    extract::{Path, Query, State},
};
use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    error::Error,
    types::{ApiResponse, ExecutionLog, PaginatedResponse, parse_object_id},
};

use super::super::models::api_state::ApiState;

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
