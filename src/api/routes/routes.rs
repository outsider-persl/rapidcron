use axum::Router;
use std::sync::Arc;

use crate::api::{
    ApiState,
    handlers::{auth, clusters, tasks},
};
use crate::config::AuthConfig;
use crate::coord::EtcdManager;
use crate::executor::TaskQueue;
use crate::storage::mongo::MongoDataSource;

pub fn create_router_with_etcd(
    db: MongoDataSource,
    etcd_manager: Arc<EtcdManager>,
    task_queue: Arc<TaskQueue>,
    auth_config: AuthConfig,
) -> Router {
    let api_state = ApiState::new(db)
        .with_etcd(etcd_manager.clone())
        .with_task_queue(task_queue);
    let cluster_api_state = clusters::ClusterApiState::new(api_state.clone(), etcd_manager);
    let auth_state = auth::AuthState::new(auth_config);

    Router::new()
        .nest("/tasks", task_routes(api_state))
        .nest("/clusters", cluster_routes_with_etcd(cluster_api_state))
        .nest("/auth", auth_routes(auth_state))
}

fn auth_routes(state: auth::AuthState) -> Router {
    Router::new()
        .route("/login", axum::routing::post(auth::login))
        .with_state(state)
}

fn task_routes(state: ApiState) -> Router {
    Router::new()
        .route("/", axum::routing::get(tasks::list_tasks))
        .route("/", axum::routing::post(tasks::create_task))
        .route("/:id", axum::routing::get(tasks::get_task))
        .route("/:id", axum::routing::put(tasks::update_task))
        .route("/:id", axum::routing::delete(tasks::delete_task))
        .route("/:id/enable", axum::routing::post(tasks::enable_task))
        .route("/:id/disable", axum::routing::post(tasks::disable_task))
        .route("/:id/trigger", axum::routing::post(tasks::trigger_task))
        .route("/instances", axum::routing::get(tasks::list_instances))
        .route("/instances/:id", axum::routing::get(tasks::get_instance))
        .route("/logs", axum::routing::get(tasks::list_execution_logs))
        .route("/logs/:id", axum::routing::get(tasks::get_execution_log))
        .route("/stats", axum::routing::get(tasks::get_stats))
        .with_state(state)
}

fn cluster_routes_with_etcd(state: clusters::ClusterApiState) -> Router {
    Router::new()
        .route("/info", axum::routing::get(clusters::get_cluster_info))
        .with_state(state)
}
