mod api;
mod config;
mod coord;
mod error;
mod executor;
mod logging;
mod scheduler;
mod storage;
mod types;

use anyhow::Result;
use axum::Router;
use coord::ServiceInfo;
use executor::TaskQueue;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load("config.toml")?;
    logging::init(&cfg.logging)?;
    info!("[Main] configuration loaded");

    let db = Arc::new(storage::mongo::MongoDataSource::new(&cfg.database).await?);
    info!("[Main] mongodb connection established");

    let etcd_endpoints = vec![format!("{}:{}", cfg.etcd.host, cfg.etcd.port)];
    let etcd_manager =
        coord::EtcdManager::new_with_prefix(etcd_endpoints, cfg.etcd.service_prefix.clone())
            .await?;
    info!("[Main] etcd connection established");

    let etcd_manager = Arc::new(etcd_manager);

    let now = chrono::Utc::now().timestamp();

    let service_info = ServiceInfo {
        service_name: cfg.service.service_name.clone(),
        service_id: uuid::Uuid::new_v4().to_string(),
        host: cfg.server.host.clone(),
        port: cfg.server.http_port,
        metadata: Some(cfg.service.metadata.clone()),
        started_at: now,
        last_heartbeat: now,
    };

    let lease_ttl_secs = cfg.etcd.dead_threshold_secs as i64;

    etcd_manager
        .registry()
        .await
        .register(service_info.clone(), lease_ttl_secs)
        .await?;

    let amqp_url = format!(
        "amqp://{}:{}@{}:{}",
        cfg.rabbitmq.username, cfg.rabbitmq.password, cfg.rabbitmq.host, cfg.rabbitmq.port
    );

    let task_queue = Arc::new(TaskQueue::new(&amqp_url, cfg.rabbitmq.queue_name.clone()).await?);
    info!("[Main] rabbitmq task queue initialized");

    let dispatcher = scheduler::dispatcher::Dispatcher::new(
        Arc::clone(&db),
        Arc::clone(&task_queue),
        cfg.dispatcher.scan_interval_secs,
        cfg.dispatcher.log_retention_days,
        cfg.dispatcher.scheduling.clone(),
    );
    dispatcher.start().await?;
    info!("[Main] task dispatcher started");

    let retry_config = cfg.retry.clone();
    let retry_manager =
        executor::RetryManager::new(Arc::clone(&db), Arc::clone(&task_queue), retry_config);
    info!("[Main] retry manager initialized");

    tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(cfg.retry.scan_interval_secs));
        loop {
            timer.tick().await;
            match retry_manager
                .retry_failed_tasks(None, cfg.retry.batch_size)
                .await
            {
                Ok(count) => {
                    if count > 0 {
                        info!("[RetryScheduler] 安排了 {} 个失败任务重试", count);
                    }
                }
                Err(e) => {
                    error!("[RetryScheduler] 重试失败任务时出错: {}", e);
                }
            }
        }
    });
    info!("[Main] retry scheduler started");

    let api_router = api::create_router_with_etcd(
        (*db).clone(),
        etcd_manager.clone(),
        task_queue.clone(),
        cfg.auth,
    );

    let app = Router::new()
        .nest("/api", api_router)
        .layer(CorsLayer::permissive());

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", cfg.server.host, cfg.server.http_port))
            .await?;
    info!(
        "[Main] API server listening on http://{}:{}",
        cfg.server.host, cfg.server.http_port
    );

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    info!(
        "[Main] RapidCron server is running on http://{}:{}",
        cfg.server.host, cfg.server.http_port
    );

    tokio::signal::ctrl_c().await?;
    info!("[Main] RapidCron server is shutting down...");

    dispatcher.stop().await?;
    info!("[Main] dispatcher stopped");

    etcd_manager
        .registry()
        .await
        .deregister(&service_info.service_name)
        .await?;
    info!("[Main] service deregistered");

    info!("[Main] RapidCron server shutdown complete");

    Ok(())
}
