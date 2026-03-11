use anyhow::Result;
use axum::{Router, extract::State, response::Json, routing::get};
use chrono::{Local, TimeZone, Utc};
use futures::StreamExt;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::Mutex;
use tokio::time::interval;

use rapidcron::config;
use rapidcron::coord::{EtcdManager, ServiceInfo};

/// 任务消息
#[derive(Debug, Deserialize)]
struct TaskMessage {
    instance_id: mongodb::bson::oid::ObjectId,
    task_id: mongodb::bson::oid::ObjectId,
    task_name: String,
    scheduled_time: i64,
    retry_count: i32,
}

/// Simple Executor - 简单的任务执行器
///
/// 功能：
/// 1. 注册服务到 etcd
/// 2. 监听 RabbitMQ 队列
/// 3. 提供 HTTP 执行接口
/// 4. 保持心跳
///
/// 使用方式：
/// ```bash
/// cargo run --bin simple-executor --port 8081
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!(
        "[{}] Simple Executor 启动中...",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let cfg = config::load("config.toml")?;

    let args: Vec<String> = std::env::args().collect();

    let executor_port = args
        .iter()
        .position(|arg| arg == "--port")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8081);

    let executor_id = format!("worker-{}", uuid::Uuid::new_v4());

    println!(
        "[{}] 执行器 ID: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_id
    );
    println!(
        "[{}] 监听端口: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_port
    );

    let etcd_endpoints = vec!["localhost:2379".to_string()];
    let etcd_manager =
        EtcdManager::new_with_prefix(etcd_endpoints, "rapidcron/services/".to_string()).await?;
    println!(
        "[{}] 已连接到 etcd",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let etcd_manager = Arc::new(etcd_manager);

    let now = chrono::Utc::now().timestamp();

    let service_info = ServiceInfo {
        service_name: format!("executor-{}", executor_port),
        service_id: executor_id.clone(),
        host: "localhost".to_string(),
        port: executor_port,
        metadata: Some("executor".to_string()),
        started_at: now,
        last_heartbeat: now,
    };

    let lease_ttl_secs = cfg.etcd.dead_threshold_secs as i64;

    let lease_id = etcd_manager
        .registry()
        .await
        .register(service_info.clone(), lease_ttl_secs)
        .await?;
    println!(
        "[{}] 服务已注册: {} (端口: {})",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_id,
        executor_port
    );

    let amqp_url = "amqp://guest:guest@localhost:5672";
    let connection = Connection::connect(amqp_url, ConnectionProperties::default())
        .await
        .map_err(|e| anyhow::anyhow!("连接 RabbitMQ 失败: {}", e))?;
    println!(
        "[{}] 已连接到 RabbitMQ",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let channel = connection
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("创建 Channel 失败: {}", e))?;

    let queue_name = "rapidcron-tasks";
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("声明队列失败: {}", e))?;
    println!(
        "[{}] 已声明队列: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        queue_name
    );

    let consumer = channel
        .basic_consume(
            queue_name,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("创建消费者失败: {}", e))?;
    println!(
        "[{}] 已创建消费者",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let executor_state = Arc::new(ExecutorState {
        executor_id: executor_id.clone(),
        executor_port,
        system_info: Arc::new(Mutex::new(System::new_all())),
    });

    tokio::spawn({
        let state = Arc::clone(&executor_state);
        async move {
            println!(
                "[{}] 开始监听任务队列...",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            let mut consumer = consumer;
            while let Some(delivery_result) = consumer.next().await {
                match delivery_result {
                    Ok(delivery) => {
                        let data = delivery.data.clone();

                        match serde_json::from_slice::<TaskMessage>(&data) {
                            Ok(task_msg) => {
                                let scheduled_time_local = Utc
                                    .timestamp_opt(task_msg.scheduled_time, 0)
                                    .single()
                                    .unwrap()
                                    .with_timezone(&Local);

                                println!(
                                    "[{}] 收到任务: {} (实例ID: {})",
                                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    task_msg.task_name,
                                    task_msg.instance_id
                                );
                                println!(
                                    "[{}] 将会在 {} 执行 (调度时间)",
                                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    scheduled_time_local.format("%Y-%m-%d %H:%M:%S")
                                );
                            }
                            Err(e) => {
                                let task_info = String::from_utf8_lossy(&data);
                                println!(
                                    "[{}] 收到任务: {}",
                                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    task_info
                                );
                                println!(
                                    "[{}] 解析任务消息失败: {}",
                                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                                    e
                                );
                            }
                        }

                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            println!(
                                "[{}] 确认消息失败: {}",
                                Local::now().format("%Y-%m-%d %H:%M:%S"),
                                e
                            );
                        }
                    }
                    Err(e) => {
                        println!(
                            "[{}] 接收消息失败: {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S"),
                            e
                        );
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/info", get(executor_info))
        .route("/execute", get(execute_task))
        .route("/node", get(node_info))
        .with_state(executor_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", executor_port)).await?;
    println!(
        "[{}] HTTP 服务监听端口: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_port
    );

    println!(
        "[{}] Simple Executor 启动完成",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "[{}] 可以通过以下方式访问:",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "[{}]   - 健康检查: http://localhost:{}/health",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_port
    );
    println!(
        "[{}]   - 执行器信息: http://localhost:{}/info",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_port
    );
    println!(
        "[{}]   - 执行任务: http://localhost:{}/execute",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_port
    );
    println!(
        "[{}]   - 节点信息: http://localhost:{}/node",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        executor_port
    );

    axum::serve(listener, app).await?;

    Ok(())
}

/// 执行器状态
#[derive(Clone)]
struct ExecutorState {
    executor_id: String,
    executor_port: u16,
    system_info: Arc<Mutex<System>>,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    executor_id: String,
    timestamp: i64,
}

/// 执行器信息响应
#[derive(Debug, Serialize)]
struct ExecutorInfo {
    executor_id: String,
    service_name: String,
    version: String,
}

/// 执行任务响应
#[derive(Debug, Serialize)]
struct ExecuteResponse {
    status: String,
    executor_id: String,
    message: String,
    timestamp: i64,
}

/// 节点信息响应
#[derive(Debug, Serialize)]
struct NodeInfoResponse {
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

/// 健康检查接口
async fn health_check(State(state): State<Arc<ExecutorState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        executor_id: state.executor_id.clone(),
        timestamp: chrono::Local::now().timestamp(),
    })
}

/// 执行器信息接口
async fn executor_info(State(state): State<Arc<ExecutorState>>) -> Json<ExecutorInfo> {
    Json(ExecutorInfo {
        executor_id: state.executor_id.clone(),
        service_name: "simple-executor".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// 执行任务接口
async fn execute_task(State(state): State<Arc<ExecutorState>>) -> Json<ExecuteResponse> {
    let now = Local::now();
    println!(
        "[{}] 将会在 {} 执行 (调度时间)",
        now.format("%Y-%m-%d %H:%M:%S"),
        now.format("%Y-%m-%d %H:%M:%S")
    );
    Json(ExecuteResponse {
        status: "success".to_string(),
        executor_id: state.executor_id.clone(),
        message: "任务执行成功".to_string(),
        timestamp: chrono::Local::now().timestamp(),
    })
}

/// 节点信息接口
async fn node_info(State(state): State<Arc<ExecutorState>>) -> Json<NodeInfoResponse> {
    let mut system = state.system_info.lock().await;
    system.refresh_all();

    let cpu_usage = system.global_cpu_usage() as f64;
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();

    drop(system);

    let memory_usage_gb = used_memory as f64 / 1024.0 / 1024.0 / 1024.0;
    let memory_total_gb = total_memory as f64 / 1024.0 / 1024.0 / 1024.0;

    Json(NodeInfoResponse {
        node_name: state.executor_id.clone(),
        node_id: state.executor_id.clone(),
        host: "localhost".to_string(),
        port: state.executor_port,
        status: "active".to_string(),
        cpu_usage,
        memory_usage: memory_usage_gb,
        memory_total: memory_total_gb as u64,
        active_tasks: 0,
        metadata: Some("executor".to_string()),
        timestamp: chrono::Local::now().timestamp(),
    })
}
