use axum::{Json, extract::State};
use reqwest::Client;
use std::sync::Arc;

use crate::{
    api::tasks::ApiState,
    coord::{EtcdManager, ServiceInfo},
    error::Error,
    types::{ApiResponse, ClusterNode, ClusterResponse, TaskInstance, TaskStatus},
};

/// 节点信息响应（从executor获取）
#[derive(Debug, serde::Deserialize)]
struct ExecutorNodeInfo {
    node_name: String,
    node_id: String,
    host: String,
    port: u16,
    status: String,
    cpu_usage: f64,
    memory_usage: f64,
    memory_total: u64,
    active_tasks: u64,
    metadata: Option<String>,
    timestamp: i64,
}

/// API 状态（扩展以支持集群信息）
#[derive(Clone)]
pub struct ClusterApiState {
    pub api_state: ApiState,
    pub etcd_manager: Arc<EtcdManager>,
}

impl ClusterApiState {
    pub fn new(api_state: ApiState, etcd_manager: Arc<EtcdManager>) -> Self {
        Self {
            api_state,
            etcd_manager,
        }
    }
}

/// 获取所有集群信息
pub async fn get_cluster_info(
    State(state): State<ClusterApiState>,
) -> Result<Json<ApiResponse<ClusterResponse>>, Error> {
    let all_instances: Vec<TaskInstance> =
        state.api_state.db.find_task_instances(None, None).await?;

    let mut nodes: Vec<ClusterNode> = Vec::new();

    let client = Client::new();

    let services = state.etcd_manager.discover_all_services().await?;

    let now = chrono::Utc::now().timestamp();
    let offline_threshold = 30; // 30秒没有心跳标记为离线
    let dead_threshold = 60; // 60秒没有心跳标记为永久下线

    for service in services {
        let heartbeat_age = now - service.last_heartbeat;

        let status = if heartbeat_age > dead_threshold {
            "dead"
        } else if heartbeat_age > offline_threshold {
            "offline"
        } else {
            "active"
        };

        let running_instances = all_instances
            .iter()
            .filter(|i| {
                i.executor_id.as_ref() == Some(&service.service_id)
                    && i.status == TaskStatus::Running
            })
            .count() as u64;

        let mut node = ClusterNode {
            node_name: service.service_name.clone(),
            node_id: service.service_id.clone(),
            host: service.host.clone(),
            port: service.port,
            status: status.to_string(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_total: 0,
            active_tasks: running_instances,
            metadata: service.metadata.clone(),
        };

        let executor_url = format!("http://{}:{}/node", service.host, service.port);
        if let Ok(response) = client.get(&executor_url).send().await {
            if let Ok(node_info) = response.json::<ExecutorNodeInfo>().await {
                node.cpu_usage = node_info.cpu_usage;
                node.memory_usage = node_info.memory_usage;
                node.memory_total = node_info.memory_total;
            }
        }

        nodes.push(node);
    }

    let total_nodes = nodes.len() as u64;
    let active_nodes = nodes.iter().filter(|n| n.status == "active").count() as u64;

    let response = ClusterResponse {
        nodes,
        total_nodes,
        active_nodes,
    };

    Ok(Json(ApiResponse::success(response)))
}
