use anyhow::Result;
use axum::{Router, extract::State, response::Json, routing::get};
use chrono::{Local, TimeZone, Utc};
use futures::StreamExt;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;
use tracing::{error, info};

use rapidcron::config;
use rapidcron::coord::{EtcdManager, ServiceInfo};
use rapidcron::storage::mongo::MongoDataSource;
use rapidcron::types::{ExecutionLog, ExecutionResult, TaskStatus, TriggeredBy};

/// 任务消息
#[derive(Debug, Deserialize)]
struct TaskMessage {
    instance_id: mongodb::bson::oid::ObjectId,
    task_id: mongodb::bson::oid::ObjectId,
    task_name: String,
    scheduled_time: i64,
    triggered_by: TriggeredBy,
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

    info!("Simple Executor 启动中...");

    let cfg = config::load("config.toml")?;

    let args: Vec<String> = std::env::args().collect();

    let executor_port = args
        .iter()
        .position(|arg| arg == "--port")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8081);

    let executor_id = format!("worker-{}", uuid::Uuid::new_v4());

    info!("执行器 ID: {}", executor_id);
    info!("监听端口: {}", executor_port);

    // 连接数据库
    let db = Arc::new(MongoDataSource::new(&cfg.database).await?);
    info!("已连接到 MongoDB");

    let etcd_endpoints = vec!["localhost:2379".to_string()];
    let etcd_manager =
        EtcdManager::new_with_prefix(etcd_endpoints, "rapidcron/services/".to_string()).await?;
    info!("已连接到 etcd");

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

    etcd_manager
        .registry()
        .await
        .register(service_info.clone(), lease_ttl_secs)
        .await?;
    info!("服务已注册: {} (端口: {})", executor_id, executor_port);

    let amqp_url = "amqp://guest:guest@localhost:5672";
    let connection = Connection::connect(amqp_url, ConnectionProperties::default())
        .await
        .map_err(|e| anyhow::anyhow!("连接 RabbitMQ 失败: {}", e))?;
    info!("已连接到 RabbitMQ");

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
    info!("已声明队列: {}", queue_name);

    let consumer = channel
        .basic_consume(
            queue_name,
            "",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("创建消费者失败: {}", e))?;
    info!("已创建消费者，任务执行超时时间: 300秒");

    let executor_state = Arc::new(ExecutorState {
        executor_id: executor_id.clone(),
        executor_port,
        system_info: Arc::new(Mutex::new(System::new_all())),
        db: Arc::clone(&db),
    });

    tokio::spawn({
        let state = Arc::clone(&executor_state);
        async move {
            info!("开始监听任务队列...");
            let mut consumer = consumer;
            while let Some(delivery_result) = consumer.next().await {
                match delivery_result {
                    Ok(delivery) => {
                        let data = delivery.data.clone();
                        let _delivery_tag = delivery.delivery_tag;

                        // 使用超时处理任务执行
                        let state_clone = Arc::clone(&state);
                        let result = tokio::time::timeout(
                            tokio::time::Duration::from_secs(300),
                            async move {
                                match serde_json::from_slice::<TaskMessage>(&data) {
                                    Ok(task_msg) => {
                                        let scheduled_time_local = Utc
                                            .timestamp_opt(task_msg.scheduled_time, 0)
                                            .single()
                                            .unwrap()
                                            .with_timezone(&Local);

                                        info!(
                                            "收到任务: {} (实例ID: {})",
                                            task_msg.task_name, task_msg.instance_id
                                        );
                                        info!(
                                            "将会在 {} 执行 (调度时间)",
                                            scheduled_time_local.format("%Y-%m-%d %H:%M:%S")
                                        );

                                        // 获取任务信息
                                        let task = state_clone.db.get_task(task_msg.task_id).await;
                                        let task_name = task_msg.task_name.clone();
                                        let instance_id = task_msg.instance_id;
                                        let executor_id = state_clone.executor_id.clone();

                                        // 更新任务实例状态为运行中
                                        let start_time = Utc::now();
                                        let update_running = bson::doc! {
                                            "$set": {
                                                "status": "running",
                                                "executor_id": executor_id.clone(),
                                                "start_time": start_time
                                            }
                                        };
                                        if let Err(e) = state_clone
                                            .db
                                            .update_task_instance(instance_id, update_running)
                                            .await
                                        {
                                            error!("更新任务实例状态为运行中失败: {}", e);
                                        }

                                        // 实际执行任务
                                        let end_time = Utc::now();
                                        let duration_ms =
                                            (end_time - start_time).num_milliseconds();

                                        let (
                                            execution_result,
                                            task_status,
                                            output_summary,
                                            error_message,
                                        ) = match task {
                                            Ok(Some(task)) => {
                                                // 实际发送 HTTP 请求
                                                match &task.payload {
                                                    rapidcron::types::TaskPayload::Http {
                                                        url,
                                                        method,
                                                        headers,
                                                        body,
                                                        ..
                                                    } => {
                                                        let client = reqwest::Client::new();
                                                        let method =
                                                            method.as_deref().unwrap_or("GET");

                                                        info!("执行 HTTP 任务: {} {}", method, url);

                                                        let request_builder =
                                                            match method.to_uppercase().as_str() {
                                                                "GET" => client.get(url),
                                                                "POST" => client.post(url),
                                                                "PUT" => client.put(url),
                                                                "DELETE" => client.delete(url),
                                                                _ => client.get(url),
                                                            };

                                                        let request_builder = if let Some(headers) =
                                                            headers
                                                        {
                                                            let mut request_builder =
                                                                request_builder;
                                                            if let Some(obj) = headers.as_object() {
                                                                for (key, value) in obj {
                                                                    if let Some(value_str) =
                                                                        value.as_str()
                                                                    {
                                                                        request_builder =
                                                                            request_builder.header(
                                                                                key, value_str,
                                                                            );
                                                                    }
                                                                }
                                                            }
                                                            request_builder
                                                        } else {
                                                            request_builder
                                                        };

                                                        let request_builder =
                                                            if let Some(body) = body {
                                                                request_builder.body(body.clone())
                                                            } else {
                                                                request_builder
                                                            };

                                                        match request_builder.send().await {
                                                            Ok(response) => {
                                                                let status = response.status();
                                                                let output =
                                                                    match response.text().await {
                                                                        Ok(text) => Some(text),
                                                                        Err(e) => Some(format!(
                                                                            "读取响应失败: {}",
                                                                            e
                                                                        )),
                                                                    };

                                                                if status.is_success() {
                                                                    // 成功情况
                                                                    (
                                                                        ExecutionResult {
                                                                            output,
                                                                            error: None,
                                                                            exit_code: Some(0),
                                                                        },
                                                                        TaskStatus::Success,
                                                                        Some(format!(
                                                                            "HTTP {} 成功",
                                                                            status
                                                                        )),
                                                                        None,
                                                                    )
                                                                } else {
                                                                    // 失败情况
                                                                    (
                                                                        ExecutionResult {
                                                                            output,
                                                                            error: Some(format!(
                                                                                "HTTP 错误: {}",
                                                                                status
                                                                            )),
                                                                            exit_code: Some(
                                                                                status.as_u16()
                                                                                    as i32,
                                                                            ),
                                                                        },
                                                                        TaskStatus::Failed,
                                                                        Some(format!(
                                                                            "HTTP {} 失败",
                                                                            status
                                                                        )),
                                                                        Some(format!(
                                                                            "HTTP 错误: {}",
                                                                            status
                                                                        )),
                                                                    )
                                                                }
                                                            }
                                                            Err(e) => {
                                                                // 请求失败
                                                                (
                                                                    ExecutionResult {
                                                                        output: Some(
                                                                            "请求失败".to_string(),
                                                                        ),
                                                                        error: Some(format!(
                                                                            "HTTP 请求失败: {}",
                                                                            e
                                                                        )),
                                                                        exit_code: Some(1),
                                                                    },
                                                                    TaskStatus::Failed,
                                                                    Some(
                                                                        "HTTP 请求失败".to_string(),
                                                                    ),
                                                                    Some(format!(
                                                                        "HTTP 请求失败: {}",
                                                                        e
                                                                    )),
                                                                )
                                                            }
                                                        }
                                                    }
                                                    rapidcron::types::TaskPayload::Command {
                                                        command,
                                                        ..
                                                    } => {
                                                        // 模拟命令执行
                                                        info!("执行命令任务: {}", command);
                                                        tokio::time::sleep(
                                                            tokio::time::Duration::from_secs(1),
                                                        )
                                                        .await;

                                                        // 模拟命令执行结果
                                                        (
                                                            ExecutionResult {
                                                                output: Some(format!(
                                                                    "执行命令: {}",
                                                                    command
                                                                )),
                                                                error: None,
                                                                exit_code: Some(0),
                                                            },
                                                            TaskStatus::Success,
                                                            Some("命令执行成功".to_string()),
                                                            None,
                                                        )
                                                    }
                                                }
                                            }
                                            _ => {
                                                // 任务不存在或查询失败
                                                (
                                                    ExecutionResult {
                                                        output: None,
                                                        error: Some(
                                                            "任务不存在或查询失败".to_string(),
                                                        ),
                                                        exit_code: Some(1),
                                                    },
                                                    TaskStatus::Failed,
                                                    Some("任务不存在".to_string()),
                                                    Some("任务不存在或查询失败".to_string()),
                                                )
                                            }
                                        };

                                        // 保存状态用于后续日志
                                        let status_for_log = task_status.clone();

                                        // 将ExecutionResult转换为Bson
                                        let result_bson =
                                            match serde_json::to_value(execution_result) {
                                                Ok(value) => match bson::Bson::try_from(value) {
                                                    Ok(bson) => bson,
                                                    Err(e) => {
                                                        error!("转换执行结果为Bson失败: {}", e);
                                                        bson::Bson::Null
                                                    }
                                                },
                                                Err(e) => {
                                                    error!("序列化执行结果失败: {}", e);
                                                    bson::Bson::Null
                                                }
                                            };

                                        // 更新任务实例状态
                                        let status_str = match task_status {
                                            TaskStatus::Pending => "pending",
                                            TaskStatus::Running => "running",
                                            TaskStatus::Success => "success",
                                            TaskStatus::Failed => "failed",
                                            TaskStatus::Cancelled => "cancelled",
                                        };
                                        let update_status = bson::doc! {
                                            "$set": {
                                                "status": status_str,
                                                "end_time": end_time,
                                                "result": result_bson
                                            }
                                        };
                                        if let Err(e) = state_clone
                                            .db
                                            .update_task_instance(instance_id, update_status)
                                            .await
                                        {
                                            error!("更新任务实例状态失败: {}", e);
                                        }

                                        // 创建执行日志
                                        let execution_log = ExecutionLog {
                                            id: None,
                                            task_id: task_msg.task_id,
                                            task_name: task_name.clone(),
                                            instance_id,
                                            scheduled_time: start_time,
                                            start_time: Some(start_time),
                                            end_time,
                                            status: task_status,
                                            duration_ms,
                                            output_summary,
                                            error_message,
                                            triggered_by: task_msg.triggered_by,
                                        };
                                        if let Err(e) =
                                            state_clone.db.create_execution_log(execution_log).await
                                        {
                                            error!("创建执行日志失败: {}", e);
                                        }

                                        info!(
                                            "任务 {} 执行{:?}，实例ID: {}",
                                            task_name, status_for_log, instance_id
                                        );
                                    }
                                    Err(e) => {
                                        let task_info = String::from_utf8_lossy(&data);
                                        info!("收到任务: {}", task_info);
                                        error!("解析任务消息失败: {}", e);
                                    }
                                }
                            },
                        )
                        .await;

                        match result {
                            Ok(_) => {
                                // 任务执行成功，确认消息
                                if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                    error!("确认消息失败: {}", e);
                                }
                            }
                            Err(_) => {
                                // 任务执行超时，拒绝消息并重新入队
                                error!("任务执行超时，将消息重新入队");
                                if let Err(e) = delivery
                                    .nack(BasicNackOptions {
                                        multiple: false,
                                        requeue: true,
                                    })
                                    .await
                                {
                                    error!("拒绝消息失败: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("接收消息失败: {}", e);
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/info", get(executor_info))
        .route("/execute", get(execute_task))
        .route("/error", get(error_task))
        .route("/node", get(node_info))
        .with_state(executor_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", executor_port)).await?;
    info!("HTTP 服务监听端口: {}", executor_port);

    info!("Simple Executor 启动完成");
    info!("可以通过以下方式访问:");
    info!("  - 健康检查: http://localhost:{}/health", executor_port);
    info!("  - 执行器信息: http://localhost:{}/info", executor_port);
    info!("  - 执行任务: http://localhost:{}/execute", executor_port);
    info!("  - 节点信息: http://localhost:{}/node", executor_port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 执行器状态
#[derive(Clone)]
struct ExecutorState {
    executor_id: String,
    executor_port: u16,
    system_info: Arc<Mutex<System>>,
    db: Arc<MongoDataSource>,
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

/// 错误任务接口
async fn error_task(State(_state): State<Arc<ExecutorState>>) -> axum::http::StatusCode {
    let now = Local::now();
    println!("[{}] 执行错误任务接口", now.format("%Y-%m-%d %H:%M:%S"));
    // 返回 500 错误，模拟接口失败
    axum::http::StatusCode::INTERNAL_SERVER_ERROR
}

/// 节点信息接口
async fn node_info(State(state): State<Arc<ExecutorState>>) -> Json<NodeInfoResponse> {
    let mut system = state.system_info.lock().await;
    system.refresh_all();

    let cpu_usage = system.global_cpu_usage() as f64;
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();

    drop(system);

    let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
    let memory_total_gb = total_memory as f64 / 1024.0 / 1024.0 / 1024.0;

    Json(NodeInfoResponse {
        node_name: state.executor_id.clone(),
        node_id: state.executor_id.clone(),
        host: "localhost".to_string(),
        port: state.executor_port,
        status: "active".to_string(),
        cpu_usage,
        memory_usage: memory_usage_percent,
        memory_total: memory_total_gb as u64,
        active_tasks: 0,
        metadata: Some("executor".to_string()),
        timestamp: chrono::Local::now().timestamp(),
    })
}
