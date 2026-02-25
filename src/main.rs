mod config;
mod coord;
mod error;
mod executor;
mod logging;
mod scheduler;
mod storage;
mod types;

use anyhow::Result;
use coord::ServiceInfo;
use executor::TaskQueue;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load("config.toml")?;
    let _log_guard = logging::init(&cfg.logging)?;
    info!("configuration loaded");

    let db = Arc::new(storage::mongo::MongoDataSource::new(&cfg.database).await?);
    info!("mongodb connection established");

    let etcd_endpoints = vec![format!("{}:{}", cfg.etcd.host, cfg.etcd.port)];
    let mut etcd_manager = coord::EtcdManager::new(etcd_endpoints).await?;
    info!("etcd connection established");

    let service_info = ServiceInfo {
        service_name: "rapidcron-dispatcher".to_string(),
        service_id: uuid::Uuid::new_v4().to_string(),
        host: cfg.server.host.clone(),
        port: cfg.server.http_port,
        metadata: Some("dispatcher".to_string()),
    };

    etcd_manager
        .registry()
        .register(service_info.clone())
        .await?;
    info!("service registered");

    let amqp_url = format!(
        "amqp://{}:{}@{}:{}",
        cfg.rabbitmq.username, cfg.rabbitmq.password, cfg.rabbitmq.host, cfg.rabbitmq.port
    );

    let task_queue = Arc::new(
        TaskQueue::new(
            &amqp_url,
            "rapidcron-tasks".to_string(),
            "rapidcron-dispatcher".to_string(),
            10,
        )
        .await?,
    );
    info!("rabbitmq task queue initialized");

    let dispatcher =
        scheduler::dispatcher::Dispatcher::new(Arc::clone(&db), Arc::clone(&task_queue), 60);
    dispatcher.start().await?;
    info!("task dispatcher started");

    let retry_manager = executor::retry_logic::RetryManager::new(Arc::clone(&db));
    info!("retry manager initialized");

    tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(60));
        loop {
            timer.tick().await;
            match retry_manager.retry_failed_tasks(None, 100).await {
                Ok(count) => {
                    if count > 0 {
                        info!("安排了 {} 个失败任务重试", count);
                    }
                }
                Err(e) => {
                    error!("重试失败任务时出错: {}", e);
                }
            }
        }
    });
    info!("retry scheduler started");

    info!(
        "RapidCron server is running on {}:{}",
        cfg.server.host, cfg.server.http_port
    );

    tokio::signal::ctrl_c().await?;
    info!("RapidCron server is shutting down...");

    dispatcher.stop().await?;
    info!("dispatcher stopped");

    etcd_manager
        .registry()
        .deregister(&service_info.service_name)
        .await?;
    info!("service deregistered");

    info!("RapidCron server shutdown complete");

    Ok(())
}
