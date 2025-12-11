use rapidcron::common::loader::load_config;
use rapidcron::common::logger::init_logger;
use rapidcron::common::metrics::{init_metrics, start_metrics_server_with_restart};
use rapidcron::grpc::server::create_server;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let cfg = load_config()?;

    // 初始化日志
    init_logger(&cfg.logging);

    // 初始化指标
    init_metrics();

    // 启动带自动重启的Metrics服务
    if cfg.metrics.enabled {
        let bind = cfg.metrics.bind_addr.clone();
        let max_restarts = cfg.metrics.max_restarts;

        tokio::spawn(async move {
            info!(
                "Starting metrics server with auto-restart (max: {})",
                max_restarts
            );
            start_metrics_server_with_restart(bind, max_restarts).await;
        });
    }

    // 启动gRPC服务
    let addr = cfg.server.bind_addr.parse()?;
    info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(create_server())
        .serve(addr)
        .await?;

    Ok(())
}
