use crate::error::{Error, Result};
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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
    connection: Arc<Connection>,
    channel: Arc<Channel>,
    queue_name: String,
    executor_id: String,
    max_concurrent_tasks: usize,
    running_tasks: Arc<RwLock<Vec<ObjectId>>>,
}

impl TaskQueue {
    /// 创建新的任务队列
    pub async fn new(
        amqp_url: &str,
        queue_name: String,
        executor_id: String,
        max_concurrent_tasks: usize,
    ) -> Result<Self> {
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

        channel
            .basic_qos(max_concurrent_tasks as u16, BasicQosOptions::default())
            .await
            .map_err(|e| Error::MessageQueue(format!("设置 QoS 失败: {}", e)))?;

        info!("成功连接到 RabbitMQ: {}", queue_name);

        Ok(Self {
            connection: Arc::new(connection),
            channel: Arc::new(channel),
            queue_name,
            executor_id,
            max_concurrent_tasks,
            running_tasks: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// 发布任务到队列
    pub async fn publish_task(&self, task_msg: TaskMessage) -> Result<()> {
        let payload = serde_json::to_vec(&task_msg).map_err(|e| Error::Serialization(e))?;

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

    /// 消费任务
    pub async fn consume_tasks(&self) -> Result<Consumer> {
        let consumer = self
            .channel
            .basic_consume(
                &self.queue_name,
                &format!("{}_consumer", self.executor_id),
                BasicConsumeOptions {
                    no_ack: false,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| Error::MessageQueue(format!("消费任务失败: {}", e)))?;

        info!("开始消费任务队列: {}", self.queue_name);

        Ok(consumer)
    }

    /// 确认任务完成
    pub async fn ack_task(&self, delivery_tag: u64) -> Result<()> {
        self.channel
            .basic_ack(delivery_tag, BasicAckOptions::default())
            .await
            .map_err(|e| Error::MessageQueue(format!("确认任务失败: {}", e)))?;

        debug!("确认任务完成: {}", delivery_tag);

        Ok(())
    }

    /// 拒绝任务
    pub async fn reject_task(&self, delivery_tag: u64, requeue: bool) -> Result<()> {
        use lapin::options::BasicNackOptions;

        let options = BasicNackOptions {
            multiple: false,
            requeue,
        };

        self.channel
            .basic_nack(delivery_tag, options)
            .await
            .map_err(|e| Error::MessageQueue(format!("拒绝任务失败: {}", e)))?;

        warn!("拒绝任务: {}, requeue: {}", delivery_tag, requeue);

        Ok(())
    }

    /// 获取当前运行中的任务数
    pub async fn running_count(&self) -> usize {
        self.running_tasks.read().await.len()
    }

    /// 添加运行中的任务
    pub async fn add_running_task(&self, instance_id: ObjectId) {
        let mut tasks = self.running_tasks.write().await;
        tasks.push(instance_id);
    }

    /// 移除运行中的任务
    pub async fn remove_running_task(&self, instance_id: ObjectId) {
        let mut tasks = self.running_tasks.write().await;
        tasks.retain(|id| id != &instance_id);
    }

    /// 队列统计信息
    pub async fn get_queue_stats(&self) -> Result<TaskQueueStats> {
        let queue_stats = self
            .channel
            .queue_declare(
                &self.queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| Error::MessageQueue(format!("获取队列统计失败: {}", e)))?;

        let running_count = self.running_count().await;

        Ok(TaskQueueStats {
            queue_name: self.queue_name.clone(),
            message_count: queue_stats.message_count(),
            consumer_count: queue_stats.consumer_count(),
            running_count,
            max_concurrent_tasks: self.max_concurrent_tasks,
        })
    }
}

/// 任务队列统计信息
#[derive(Debug, Clone)]
pub struct TaskQueueStats {
    pub queue_name: String,
    pub message_count: u32,
    pub consumer_count: u32,
    pub running_count: usize,
    pub max_concurrent_tasks: usize,
}

/// 任务队列管理器（用于管理多个队列）
pub struct TaskQueueManager {
    queues: Arc<RwLock<Vec<Arc<TaskQueue>>>>,
}

impl TaskQueueManager {
    /// 创建新的任务队列管理器
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加队列
    pub async fn add_queue(&self, queue: Arc<TaskQueue>) {
        let mut queues = self.queues.write().await;
        queues.push(queue);
    }

    /// 获取所有队列的统计信息
    pub async fn get_all_stats(&self) -> Vec<TaskQueueStats> {
        let queues = self.queues.read().await;
        let mut stats = Vec::new();

        for queue in queues.iter() {
            if let Ok(stat) = queue.get_queue_stats().await {
                stats.push(stat);
            }
        }

        stats
    }
}

impl Default for TaskQueueManager {
    fn default() -> Self {
        Self::new()
    }
}
