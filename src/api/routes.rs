use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::{
    api::{
        clusters::{ClusterApiState, get_cluster_info},
        tasks::{
            ApiState, create_task, delete_task, disable_task, enable_task, get_instance, get_stats,
            get_task, list_instances, list_tasks, trigger_task, update_task,
        },
    },
    coord::EtcdManager,
    storage::mongo::MongoDataSource,
};

pub fn create_router(db: MongoDataSource) -> Router {
    let state = ApiState::new(db);

    Router::new()
        .route("/stats", get(get_stats))
        .route("/tasks", get(list_tasks))
        .route("/tasks", post(create_task))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        .route("/tasks/:id/enable", post(enable_task))
        .route("/tasks/:id/disable", post(disable_task))
        .route("/tasks/:id/trigger", post(trigger_task))
        .route("/instances", get(list_instances))
        .route("/instances/:id", get(get_instance))
        .with_state(state)
}

pub fn create_router_with_etcd(db: MongoDataSource, etcd_manager: Arc<EtcdManager>) -> Router {
    let api_state = ApiState::new(db).with_etcd(etcd_manager.clone());
    let cluster_state = ClusterApiState::new(api_state.clone(), etcd_manager);

    let task_router = Router::new()
        .route("/stats", get(get_stats))
        .route("/tasks", get(list_tasks))
        .route("/tasks", post(create_task))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        .route("/tasks/:id/enable", post(enable_task))
        .route("/tasks/:id/disable", post(disable_task))
        .route("/tasks/:id/trigger", post(trigger_task))
        .route("/instances", get(list_instances))
        .route("/instances/:id", get(get_instance))
        .with_state(api_state);

    let cluster_router = Router::new()
        .route("/clusters", get(get_cluster_info))
        .with_state(cluster_state);

    task_router.merge(cluster_router)
}
