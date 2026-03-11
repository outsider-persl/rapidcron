use crate::error::{Error, Result};
use crate::storage::mongo::MongoDataSource;
use crate::types::{ExecutionResult, Task, TaskInstance, TaskStatus};
use chrono::{Duration, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 重试策略
    pub strategy: RetryStrategy,
    /// 最大重试次数（0 表示不重试）
    pub max_retries: i32,
    /// 是否在特定错误时重试
    pub retry_on_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            strategy: RetryStrategy::default(),
            max_retries: 3,
            retry_on_errors: Vec::new(),
        }
    }
}

/// 重试管理器
pub struct RetryManager {
    db: Arc<MongoDataSource>,
}

impl RetryManager {
    /// 创建新的重试管理器
    pub fn new(db: Arc<MongoDataSource>) -> Self {
        Self { db }
    }

    /// 判断是否应该重试
    pub fn should_retry(
        &self,
        task: &Task,
        instance: &TaskInstance,
        result: &ExecutionResult,
    ) -> bool {
        let max_retries = task.max_retries.unwrap_or(3);

        if max_retries <= 0 {
            debug!("任务 {} 配置不重试", task.name);
            return false;
        }

        if instance.retry_count >= max_retries {
            debug!(
                "任务 {} 已达到最大重试次数: {}",
                task.name, instance.retry_count
            );
            return false;
        }

        if result.error.is_none() {
            debug!("任务 {} 执行成功，无需重试", task.name);
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

        let retry_config = config.unwrap_or_else(|| RetryConfig {
            strategy: RetryStrategy::default(),
            max_retries: task.max_retries.unwrap_or(3),
            retry_on_errors: Vec::new(),
        });

        if !self.should_retry(&task, &instance, result) {
            debug!("任务 {} 不满足重试条件", task_name);
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

        info!(
            "任务 {} 将在 {} 秒后重试（第 {} 次重试）",
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
                    Ok(false) => debug!("任务实例 {} 不需要重试", instance_id),
                    Err(e) => warn!("重试任务实例 {} 失败: {}", instance_id, e),
                }
            }
        }

        info!("成功安排 {} 个失败任务重试", retry_count);

        Ok(retry_count)
    }

    /// 获取重试统计信息
    pub async fn get_retry_stats(&self, task_id: Option<ObjectId>) -> Result<RetryStats> {
        let filter = if let Some(tid) = task_id {
            doc! { "task_id": tid }
        } else {
            doc! {}
        };

        let all_instances = self
            .db
            .find_task_instances(Some(filter), None)
            .await
            .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?;

        let mut total_retries = 0;
        let mut max_retry_count = 0;
        let mut failed_instances = 0;
        let mut success_after_retry = 0;

        for instance in &all_instances {
            total_retries += instance.retry_count;
            max_retry_count = max_retry_count.max(instance.retry_count);

            if instance.status == TaskStatus::Failed {
                failed_instances += 1;
            } else if instance.retry_count > 0 && instance.status == TaskStatus::Success {
                success_after_retry += 1;
            }
        }

        let avg_retry_count = if all_instances.is_empty() {
            0.0
        } else {
            total_retries as f64 / all_instances.len() as f64
        };

        Ok(RetryStats {
            total_instances: all_instances.len(),
            total_retries,
            avg_retry_count,
            max_retry_count,
            failed_instances,
            success_after_retry,
        })
    }

    /// 清理过期的失败任务实例
    pub async fn cleanup_old_failed_instances(&self, days_old: i64) -> Result<usize> {
        let cutoff_time = Utc::now() - Duration::days(days_old);

        let filter = doc! {
            "status": "failed",
            "end_time": { "$lt": cutoff_time }
        };

        let old_instances = self
            .db
            .find_task_instances(Some(filter), None)
            .await
            .map_err(|e| Error::Database(format!("查询旧任务实例失败: {}", e)))?;

        let mut deleted_count = 0;

        for instance in old_instances {
            if let Some(instance_id) = instance.id {
                match self.db.delete_task_instance(instance_id).await {
                    Ok(true) => {
                        deleted_count += 1;
                        debug!("删除过期失败任务实例 {}", instance_id);
                    }
                    Ok(false) => warn!("删除任务实例 {} 失败", instance_id),
                    Err(e) => error!("删除任务实例 {} 时出错: {}", instance_id, e),
                }
            }
        }

        info!("清理了 {} 个过期失败任务实例", deleted_count);

        Ok(deleted_count)
    }
}

/// 重试统计信息
#[derive(Debug, Clone)]
pub struct RetryStats {
    pub total_instances: usize,
    pub total_retries: i32,
    pub avg_retry_count: f64,
    pub max_retry_count: i32,
    pub failed_instances: usize,
    pub success_after_retry: usize,
}
