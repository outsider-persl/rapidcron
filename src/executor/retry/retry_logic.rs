use crate::error::{Error, Result};
use crate::executor::TaskQueue;
use crate::storage::mongo::MongoDataSource;
use crate::types::{ExecutionResult, Task, TaskInstance, TaskStatus};
use chrono::{Duration, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 重试策略
#[derive(Debug, Clone, Copy)]
pub enum RetryStrategy {
    /// 固定延迟重试
    Fixed { delay_seconds: i64 },
    /// 指数退避重试
    Exponential {
        base_delay_seconds: i64,
        max_delay_seconds: i64,
    },
    /// 线性退避重试
    Linear {
        initial_delay_seconds: i64,
        increment_seconds: i64,
    },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::Exponential {
            base_delay_seconds: 5,
            max_delay_seconds: 300,
        }
    }
}

/// 重试配置
#[derive(Debug, Clone, Default)]
pub struct RetryConfig {
    /// 重试策略
    pub strategy: RetryStrategy,
}

/// 重试管理器
pub struct RetryManager {
    db: Arc<MongoDataSource>,
    task_queue: Arc<TaskQueue>,
    config: crate::config::RetryConfig,
}

impl RetryManager {
    /// 创建新的重试管理器
    pub fn new(
        db: Arc<MongoDataSource>,
        task_queue: Arc<TaskQueue>,
        config: crate::config::RetryConfig,
    ) -> Self {
        Self {
            db,
            task_queue,
            config,
        }
    }

    /// 判断是否应该重试
    pub fn should_retry(
        &self,
        task: &Task,
        instance: &TaskInstance,
        result: &ExecutionResult,
    ) -> bool {
        let max_retries = task.max_retries.unwrap_or(self.config.default_max_retries);

        if max_retries <= 0 {
            debug!("[RetryManager] 任务 {} 配置不重试", task.name);
            return false;
        }

        if instance.retry_count >= max_retries {
            debug!(
                "[RetryManager] 任务 {} 已达到最大重试次数: {}",
                task.name, instance.retry_count
            );
            return false;
        }

        if result.error.is_none() {
            debug!("[RetryManager] 任务 {} 执行成功，无需重试", task.name);
            return false;
        }

        true
    }

    /// 计算重试延迟时间
    pub fn calculate_retry_delay(
        &self,
        _task: &Task,
        instance: &TaskInstance,
        config: &RetryConfig,
    ) -> i64 {
        let retry_count = instance.retry_count;

        match config.strategy {
            RetryStrategy::Fixed { delay_seconds } => delay_seconds,
            RetryStrategy::Exponential {
                base_delay_seconds,
                max_delay_seconds,
            } => {
                let delay = base_delay_seconds * 2_i64.pow(retry_count as u32);
                delay.min(max_delay_seconds)
            }
            RetryStrategy::Linear {
                initial_delay_seconds,
                increment_seconds,
            } => initial_delay_seconds + increment_seconds * retry_count as i64,
        }
    }

    /// 执行重试
    pub async fn retry_task(
        &self,
        instance_id: ObjectId,
        config: Option<RetryConfig>,
    ) -> Result<bool> {
        let instance = self
            .db
            .get_task_instance(instance_id)
            .await
            .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?
            .ok_or_else(|| Error::Execution("任务实例不存在".to_string()))?;

        if instance.status != TaskStatus::Failed {
            return Err(Error::Execution("只能重试失败的任务".to_string()));
        }

        let task = self
            .db
            .get_task(instance.task_id)
            .await
            .map_err(|e| Error::Database(format!("查询任务失败: {}", e)))?
            .ok_or_else(|| Error::Execution("任务不存在".to_string()))?;

        let result = instance
            .result
            .as_ref()
            .ok_or_else(|| Error::Execution("任务执行结果不存在".to_string()))?;

        let retry_count = instance.retry_count;
        let task_name = task.name.clone();

        let retry_config = config.unwrap_or_else(|| {
            let strategy = match self.config.default_strategy.as_str() {
                "fixed" => RetryStrategy::Fixed { delay_seconds: 5 },
                "linear" => RetryStrategy::Linear {
                    initial_delay_seconds: 5,
                    increment_seconds: 5,
                },
                _ => RetryStrategy::Exponential {
                    base_delay_seconds: self.config.exponential_base_delay,
                    max_delay_seconds: self.config.exponential_max_delay,
                },
            };
            RetryConfig { strategy }
        });

        if !self.should_retry(&task, &instance, result) {
            debug!("[RetryManager] 任务 {} 不满足重试条件", task_name);
            return Ok(false);
        }

        let delay_seconds = self.calculate_retry_delay(&task, &instance, &retry_config);
        let retry_time = Utc::now() + Duration::seconds(delay_seconds);

        let update = doc! {
            "$set": {
                "status": "pending",
                "retry_count": retry_count + 1,
                "scheduled_time": retry_time,
                "executor_id": mongodb::bson::Bson::Null,
                "start_time": mongodb::bson::Bson::Null,
                "end_time": mongodb::bson::Bson::Null,
            }
        };

        self.db
            .update_task_instance(instance_id, update)
            .await
            .map_err(|e| Error::Execution(format!("更新任务实例失败: {}", e)))?;

        // 发布任务到队列
        let task_msg = crate::executor::TaskMessage {
            instance_id,
            task_id: task.id.unwrap(),
            task_name: task.name.clone(),
            scheduled_time: retry_time.timestamp(),
            retry_count: retry_count + 1,
            triggered_by: instance.triggered_by,
        };

        self.task_queue
            .publish_task(task_msg)
            .await
            .map_err(|e| Error::Execution(format!("发布任务到队列失败: {}", e)))?;

        info!(
            "[RetryManager] 任务 {} 将在 {} 秒后重试（第 {} 次重试）",
            task_name,
            delay_seconds,
            retry_count + 1
        );

        Ok(true)
    }

    /// 批量重试失败的任务
    pub async fn retry_failed_tasks(
        &self,
        task_id: Option<ObjectId>,
        limit: usize,
    ) -> Result<usize> {
        let filter = if let Some(tid) = task_id {
            doc! {
                "status": "failed",
                "task_id": tid
            }
        } else {
            doc! {
                "status": "failed"
            }
        };

        let failed_instances = self
            .db
            .find_task_instances(Some(filter), None)
            .await
            .map_err(|e| Error::Database(format!("查询失败任务失败: {}", e)))?;

        let mut retry_count = 0;

        for instance in failed_instances.iter().take(limit) {
            if let Some(instance_id) = instance.id {
                match self.retry_task(instance_id, None).await {
                    Ok(true) => retry_count += 1,
                    Ok(false) => debug!("[RetryManager] 任务实例 {} 不需要重试", instance_id),
                    Err(e) => warn!("[RetryManager] 重试任务实例 {} 失败: {}", instance_id, e),
                }
            }
        }

        if retry_count > 0 {
            info!("[RetryManager] 成功安排 {} 个失败任务重试", retry_count);
        } else {
            debug!("[RetryManager] 没有失败任务需要重试");
        }

        Ok(retry_count)
    }
}
