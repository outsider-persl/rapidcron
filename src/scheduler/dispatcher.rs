use crate::error::{Error, Result};
use crate::executor::TaskQueue;
use crate::scheduler::cron_parser::CronParser;
use crate::storage::mongo::MongoDataSource;
use crate::types::{DispatchLog, Task, TaskInstance, TaskStatus};
use chrono::{DateTime, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// 任务分发器
pub struct Dispatcher {
    db: Arc<MongoDataSource>,
    task_queue: Arc<TaskQueue>,
    running: Arc<RwLock<bool>>,
    scan_interval: Duration,
    log_retention_days: u32,
}

impl Dispatcher {
    pub fn new(
        db: Arc<MongoDataSource>,
        task_queue: Arc<TaskQueue>,
        scan_interval_secs: u64,
        log_retention_days: u32,
    ) -> Self {
        Self {
            db,
            task_queue,
            running: Arc::new(RwLock::new(false)),
            scan_interval: Duration::from_secs(scan_interval_secs),
            log_retention_days,
        }
    }

    /// 启动分发器
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(Error::Scheduling("分发器已经在运行中".to_string()));
        }
        *running = true;
        drop(running);

        info!("任务分发器已启动，扫描间隔: {:?}", self.scan_interval);

        let db = Arc::clone(&self.db);
        let task_queue = Arc::clone(&self.task_queue);
        let running_flag = Arc::clone(&self.running);
        let interval = self.scan_interval;
        let scan_interval_secs = interval.as_secs();
        let log_retention_days = self.log_retention_days;

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);

            while *running_flag.read().await {
                timer.tick().await;

                match Self::scan_and_dispatch(&db, &task_queue, scan_interval_secs).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("成功分发 {} 个任务", count);
                        }
                    }
                    Err(e) => {
                        error!("任务分发失败: {}", e);
                    }
                }

                if let Err(e) = Self::cleanup_old_logs(&db, log_retention_days).await {
                    error!("清理旧日志失败: {}", e);
                }
            }

            info!("任务分发器已停止");
        });

        Ok(())
    }

    /// 停止分发器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("正在停止任务分发器...");
        Ok(())
    }

    /// 扫描并分发任务
    async fn scan_and_dispatch(
        db: &Arc<MongoDataSource>,
        task_queue: &Arc<TaskQueue>,
        scan_interval_secs: u64,
    ) -> Result<usize> {
        let now = Utc::now();
        let scan_window = chrono::Duration::seconds(scan_interval_secs as i64);
        let scan_window_start = now;
        let scan_window_end = now + scan_window;

        debug!("开始扫描待执行任务，当前时间: {}", now);
        debug!("扫描窗口: {} 到 {}", scan_window_start, scan_window_end);

        let enabled_tasks = db
            .find_tasks(
                Some(doc! {
                    "enabled": true,
                    "deleted_at": null
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询任务失败: {}", e)))?;

        let total_tasks = enabled_tasks.len() as i32;

        if enabled_tasks.is_empty() {
            debug!("没有启用的任务");

            let dispatch_log = DispatchLog {
                id: None,
                scan_time: now,
                scan_window_start,
                scan_window_end,
                total_tasks: 0,
                enabled_tasks: 0,
                dispatched_instances: 0,
                error_message: None,
            };

            if let Err(e) = db.create_dispatch_log(&dispatch_log).await {
                error!("创建调度日志失败: {}", e);
            }

            return Ok(0);
        }

        let mut dispatched_count = 0;

        for task in enabled_tasks {
            if let Some(_task_id) = task.id {
                match Self::dispatch_task_instances(db, task_queue, &task, &now, &scan_window_end)
                    .await
                {
                    Ok(count) => {
                        if count > 0 {
                            info!("为任务 {} 创建并分发了 {} 个实例", task.name, count);
                            dispatched_count += count;
                        }
                    }
                    Err(e) => {
                        error!("分发任务 {} 失败: {}", task.name, e);
                    }
                }
            }
        }

        let dispatch_log = DispatchLog {
            id: None,
            scan_time: now,
            scan_window_start,
            scan_window_end,
            total_tasks,
            enabled_tasks: total_tasks,
            dispatched_instances: dispatched_count as i32,
            error_message: None,
        };

        if let Err(e) = db.create_dispatch_log(&dispatch_log).await {
            error!("创建调度日志失败: {}", e);
        }

        info!(
            "调度扫描完成: 总任务数 {}, 启用任务数 {}, 分发实例数 {}",
            total_tasks, total_tasks, dispatched_count
        );

        Ok(dispatched_count)
    }

    /// 为任务创建并分发实例
    async fn dispatch_task_instances(
        db: &Arc<MongoDataSource>,
        task_queue: &Arc<TaskQueue>,
        task: &Task,
        now: &DateTime<Utc>,
        scan_window_end: &DateTime<Utc>,
    ) -> Result<usize> {
        let task_id = task
            .id
            .ok_or_else(|| Error::Validation("任务 ID 不能为空".to_string()))?;

        let cron_parser = CronParser::new(&task.schedule)
            .map_err(|e| Error::Scheduling(format!("解析 Cron 表达式失败: {}", e)))?;

        let next_triggers = cron_parser.next_triggers_in_window(*now, *scan_window_end);

        if next_triggers.is_empty() {
            return Ok(0);
        }

        let existing_instances = db
            .find_task_instances(
                Some(doc! {
                    "task_id": task_id,
                    "scheduled_time": { "$gte": now, "$lte": scan_window_end }
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?;

        let existing_scheduled_times: std::collections::HashSet<i64> = existing_instances
            .iter()
            .map(|inst| inst.scheduled_time.timestamp())
            .collect();

        let mut dispatched_count = 0;

        for scheduled_time in next_triggers {
            let scheduled_timestamp = scheduled_time.timestamp();

            if existing_scheduled_times.contains(&scheduled_timestamp) {
                debug!(
                    "任务 {} 在 {} 的实例已存在，跳过",
                    task.name, scheduled_time
                );
                continue;
            }

            let instance = TaskInstance {
                id: None,
                task_id,
                scheduled_time,
                status: TaskStatus::Pending,
                executor_id: None,
                start_time: None,
                end_time: None,
                retry_count: 0,
                result: None,
                created_at: *now,
            };

            let instance_id = db
                .create_task_instance(&instance)
                .await
                .map_err(|e| Error::Database(format!("创建任务实例失败: {}", e)))?;

            let task_msg = crate::executor::TaskMessage {
                instance_id,
                task_id,
                task_name: task.name.clone(),
                scheduled_time: scheduled_timestamp,
                retry_count: 0,
            };

            task_queue
                .publish_task(task_msg)
                .await
                .map_err(|e| Error::MessageQueue(format!("发布任务到队列失败: {}", e)))?;

            debug!(
                "为任务 {} 创建实例 {}，计划执行时间: {}",
                task.name, instance_id, scheduled_time
            );

            dispatched_count += 1;
        }

        Ok(dispatched_count)
    }

    /// 清理旧日志
    async fn cleanup_old_logs(db: &Arc<MongoDataSource>, retention_days: u32) -> Result<()> {
        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days as i64);

        debug!("开始清理 {} 天前的调度日志", retention_days);

        let deleted_dispatch_logs = db
            .delete_dispatch_logs_before(&cutoff_time)
            .await
            .map_err(|e| Error::Database(format!("清理调度日志失败: {}", e)))?;

        if deleted_dispatch_logs > 0 {
            info!("已清理 {} 条调度日志", deleted_dispatch_logs);
        }

        Ok(())
    }

    /// 手动触发任务执行
    pub async fn trigger_task_manually(
        &self,
        task_id: mongodb::bson::oid::ObjectId,
    ) -> Result<ObjectId> {
        let task = self
            .db
            .get_task(task_id)
            .await
            .map_err(|e| Error::Database(format!("查询任务失败: {}", e)))?
            .ok_or_else(|| Error::Validation("任务不存在".to_string()))?;

        let now = Utc::now();

        let instance = TaskInstance {
            id: None,
            task_id,
            scheduled_time: now,
            status: TaskStatus::Pending,
            executor_id: None,
            start_time: None,
            end_time: None,
            retry_count: 0,
            result: None,
            created_at: now,
        };

        let instance_id = self
            .db
            .create_task_instance(&instance)
            .await
            .map_err(|e| Error::Database(format!("创建任务实例失败: {}", e)))?;

        info!("手动触发任务 {}，创建实例 {}", task.name, instance_id);

        Ok(instance_id)
    }

    /// 获取待执行的任务实例
    pub async fn get_pending_instances(&self, limit: usize) -> Result<Vec<TaskInstance>> {
        self.db
            .find_task_instances(
                Some(doc! {
                    "status": "pending"
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询待执行实例失败: {}", e)))
            .map(|mut instances| {
                instances.sort_by(|a, b| a.scheduled_time.cmp(&b.scheduled_time));
                instances.truncate(limit);
                instances
            })
    }

    /// 获取分发器统计信息
    pub async fn get_stats(&self) -> Result<DispatcherStats> {
        let pending_count = self
            .db
            .find_task_instances(Some(doc! { "status": "pending" }), None)
            .await
            .map_err(|e| Error::Database(format!("查询待执行实例失败: {}", e)))?
            .len();

        let running_count = self
            .db
            .find_task_instances(Some(doc! { "status": "running" }), None)
            .await
            .map_err(|e| Error::Database(format!("查询执行中实例失败: {}", e)))?
            .len();

        let enabled_tasks_count = self
            .db
            .find_tasks(
                Some(doc! {
                    "enabled": true,
                    "deleted_at": null
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询启用任务失败: {}", e)))?
            .len();

        let is_running = *self.running.read().await;

        Ok(DispatcherStats {
            is_running,
            pending_count,
            running_count,
            enabled_tasks_count,
        })
    }
}

/// 分发器统计信息
#[derive(Debug, Clone)]
pub struct DispatcherStats {
    pub is_running: bool,
    pub pending_count: usize,
    pub running_count: usize,
    pub enabled_tasks_count: usize,
}
