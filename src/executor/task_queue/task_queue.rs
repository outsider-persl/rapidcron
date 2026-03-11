use crate::error::{Error, Result};
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties,
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;
use tracing::{debug, info};

/// 任务消息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskMessage {
    pub instance_id: ObjectId,
    pub task_id: ObjectId,
    pub task_name: String,
    pub scheduled_time: i64,
    pub retry_count: i32,
}

/// 任务队列
pub struct TaskQueue {
    _connection: Arc<Connection>,
    channel: Arc<Channel>,
    queue_name: String,
}

impl TaskQueue {
    /// 创建新的任务队列
    pub async fn new(amqp_url: &str, queue_name: String) -> Result<Self> {
        let connection = Connection::connect(amqp_url, ConnectionProperties::default())
            .await
            .map_err(|e| Error::MessageQueue(format!("连接 RabbitMQ 失败: {}", e)))?;

        let channel = connection
            .create_channel()
            .await
            .map_err(|e| Error::MessageQueue(format!("创建 channel 失败: {}", e)))?;

        channel
            .queue_declare(
                &queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| Error::MessageQueue(format!("声明队列失败: {}", e)))?;

        info!("成功连接到 RabbitMQ: {}", queue_name);

        Ok(Self {
            _connection: Arc::new(connection),
            channel: Arc::new(channel),
            queue_name,
        })
    }

    /// 发布任务到队列
    pub async fn publish_task(&self, task_msg: TaskMessage) -> Result<()> {
        let payload = serde_json::to_vec(&task_msg).map_err(Error::Serialization)?;

        self.channel
            .basic_publish(
                "",
                &self.queue_name,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default(),
            )
            .await
            .map_err(|e| Error::MessageQueue(format!("发布任务失败: {}", e)))?;

        debug!("发布任务到队列: {}", task_msg.task_name);

        Ok(())
    }
}
