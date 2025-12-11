use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use once_cell::sync::Lazy;
use prometheus::{Counter, Encoder, Gauge, Opts, Registry, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

/// 全局指标注册器，统一管理所有业务指标
static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

/// 任务处理总数计数器
static TASK_PROCESSED_TOTAL: Lazy<Counter> = Lazy::new(|| {
    let opts = Opts::new("task_processed_total", "Total number of tasks processed");
    let c = Counter::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(c.clone())).unwrap();
    c
});

/// 任务队列长度仪表
static TASK_QUEUE_SIZE: Lazy<Gauge> = Lazy::new(|| {
    let opts = Opts::new("task_queue_size", "Current number of tasks in the queue");
    let g = Gauge::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(g.clone())).unwrap();
    g
});

/// 初始化指标模块，触发所有懒加载指标的注册
pub fn init_metrics() {
    let _ = &*TASK_PROCESSED_TOTAL;
    let _ = &*TASK_QUEUE_SIZE;
    info!("Metrics initialized: task_processed_total, task_queue_size");
}

/// 增加任务处理计数
pub fn inc_task_processed() {
    TASK_PROCESSED_TOTAL.inc();
}

/// 设置任务队列长度
pub fn set_task_queue_size(size: f64) {
    TASK_QUEUE_SIZE.set(size);
}

/// 增加任务队列长度
pub fn inc_task_queue_size() {
    TASK_QUEUE_SIZE.inc();
}

/// 减少任务队列长度
pub fn dec_task_queue_size() {
    TASK_QUEUE_SIZE.dec();
}

/// 处理 /metrics 请求的核心逻辑（axum 处理器）
async fn metrics_handler() -> impl IntoResponse {
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let content_type = encoder.format_type();

    // 编码指标为 Prometheus 支持的文本格式
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!("Failed to encode metrics: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to encode metrics".to_string(),
        )
            .into_response();
    }

    // 构建标准的 HTTP 响应
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, content_type)],
        buffer,
    )
        .into_response()
}

/// 启动带自动重启功能的指标服务
/// 当服务异常退出时自动重启，直至达到最大重启次数
pub async fn start_metrics_server_with_restart(bind_addr: String, max_restarts: u32) {
    let mut restart_count = 0;
    let addr: SocketAddr = bind_addr.parse().expect("Invalid metrics bind address");

    loop {
        // 检查重启次数限制
        if max_restarts > 0 && restart_count >= max_restarts {
            error!(
                "Metrics server reached max restarts ({}/{}), stopping",
                restart_count, max_restarts
            );
            break;
        }

        // 记录重启信息
        if restart_count > 0 {
            info!(
                "Restarting metrics server (attempt {}/{})",
                restart_count, max_restarts
            );
        }

        // 启动 axum HTTP 服务并等待退出
        match start_metrics_server(addr).await {
            Ok(_) => {
                info!("Metrics server stopped normally");
                break;
            }
            Err(e) => {
                error!("Metrics server exited with error: {}", e);
                restart_count += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}

/// 启动指标暴露服务核心逻辑（axum 实现）
async fn start_metrics_server(addr: SocketAddr) -> anyhow::Result<()> {
    // 构建 axum 路由：仅处理 /metrics GET 请求
    let app = Router::new().route("/metrics", get(metrics_handler));

    // 绑定地址并启动服务
    let listener = TcpListener::bind(addr).await?;
    info!("Metrics server running on {}", addr);

    // axum 启动服务（阻塞直到服务退出）
    axum::serve(listener, app).await?;

    Ok(())
}
