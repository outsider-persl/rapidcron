use axum::{
    Json,
    extract::{Path, Query, State},
};
use mongodb::bson::{Bson, doc};

use crate::{
    error::Error,
    types::{ApiResponse, DispatchLog, PaginatedResponse, parse_object_id},
};

use super::super::models::api_state::ApiState;

/// 分发日志列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct DispatchLogListQuery {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub has_error: Option<bool>,
    pub page: Option<String>,
    pub page_size: Option<String>,
}

/// 获取分发日志列表
pub async fn list_dispatch_logs(
    State(state): State<ApiState>,
    Query(query): Query<DispatchLogListQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<DispatchLog>>>, Error> {
    let mut filter = doc! {};

    if let Some(start_time_str) = query.start_time {
        if let Ok(start_time) = chrono::DateTime::parse_from_rfc3339(&start_time_str) {
            filter.insert(
                "scan_time",
                doc! { "$gte": start_time.with_timezone(&chrono::Utc) },
            );
        } else {
            return Err(Error::Validation("无效的开始时间格式".to_string()));
        }
    }

    if let Some(end_time_str) = query.end_time {
        if let Ok(end_time) = chrono::DateTime::parse_from_rfc3339(&end_time_str) {
            filter.insert(
                "scan_time",
                doc! { "$lte": end_time.with_timezone(&chrono::Utc) },
            );
        } else {
            return Err(Error::Validation("无效的结束时间格式".to_string()));
        }
    }

    if let Some(has_error) = query.has_error {
        if has_error {
            filter.insert("error_message", doc! { "$ne": Bson::Null });
        } else {
            filter.insert("error_message", Bson::Null);
        }
    }

    let page = query
        .page
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(1);
    let page_size = query
        .page_size
        .and_then(|ps| ps.parse::<usize>().ok())
        .unwrap_or(20);

    let logs = state.db.find_dispatch_logs(Some(filter), None).await?;

    Ok(Json(ApiResponse::success(PaginatedResponse::from_items(
        logs, page, page_size,
    ))))
}

/// 获取分发日志详情
pub async fn get_dispatch_log(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<DispatchLog>>, Error> {
    let object_id = parse_object_id(&id).map_err(Error::Validation)?;

    let log = state
        .db
        .get_dispatch_log(object_id)
        .await?
        .ok_or_else(|| Error::Execution("分发日志不存在".to_string()))?;

    Ok(Json(ApiResponse::success(log)))
}
