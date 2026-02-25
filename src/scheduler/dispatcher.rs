use crate::error::{Error, Result};
use crate::executor::{TaskMessage, TaskQueue};
use crate::scheduler::cron_parser::CronParser;
use crate::scheduler::sorter::TaskSorter;
use crate::storage::mongo::MongoDataSource;
use crate::types::{Task, TaskInstance, TaskStatus};
use chrono::{DateTime, Local};
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 任务分发器
pub struct Dispatcher {
    db: Arc<MongoDataSource>,
    task_queue: Arc<TaskQueue>,
    running: Arc<RwLock<bool>>,
    scan_interval: Duration,
}

impl Dispatcher {
    /// 创建新的分发器
    pub fn new(
        db: Arc<MongoDataSource>,
        task_queue: Arc<TaskQueue>,
        scan_interval_secs: u64,
    ) -> Self {
        Self {
            db,
            task_queue,
            running: Arc::new(RwLock::new(false)),
            scan_interval: Duration::from_secs(scan_interval_secs),
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

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);

            while *running_flag.read().await {
                timer.tick().await;

                match Self::scan_and_dispatch(&db, &task_queue).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("成功分发 {} 个任务", count);
                        }
                    }
                    Err(e) => {
                        error!("任务分发失败: {}", e);
                    }
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
    ) -> Result<usize> {
        let now = Local::now();

        debug!("开始扫描待执行任务，当前时间: {}", now);

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

        if enabled_tasks.is_empty() {
            debug!("没有启用的任务");
            return Ok(0);
        }

        let mut tasks_to_dispatch: Vec<Task> = Vec::new();

        for task in enabled_tasks {
            if let Some(_task_id) = task.id {
                match Self::should_dispatch_task(db, &task, &now).await {
                    Ok(true) => {
                        debug!("任务 {} 需要分发", task.name);
                        tasks_to_dispatch.push(task);
                    }
                    Ok(false) => {
                        debug!("任务 {} 暂时不需要分发", task.name);
                    }
                    Err(e) => {
                        warn!("检查任务 {} 是否需要分发时出错: {}", task.name, e);
                    }
                }
            }
        }

        if tasks_to_dispatch.is_empty() {
            debug!("没有需要分发的任务");
            return Ok(0);
        }

        let sorted_tasks = TaskSorter::sort_tasks(&tasks_to_dispatch)
            .map_err(|e| Error::Scheduling(format!("任务排序失败: {}", e)))?;

        let grouped_tasks = TaskSorter::group_tasks_by_dependency(&sorted_tasks)
            .map_err(|e| Error::Scheduling(format!("任务分组失败: {}", e)))?;

        let mut dispatched_count = 0;

        for (group_index, group) in grouped_tasks.iter().enumerate() {
            info!(
                "开始分发第 {} 组任务，共 {} 个任务",
                group_index + 1,
                group.len()
            );

            for task in group {
                match Self::dispatch_single_task(db, task_queue, task, &now).await {
                    Ok(_) => {
                        dispatched_count += 1;
                        debug!("成功分发任务: {}", task.name);
                    }
                    Err(e) => {
                        error!("分发任务 {} 失败: {}", task.name, e);
                    }
                }
            }
        }

        Ok(dispatched_count)
    }

    /// 检查任务是否需要分发
    async fn should_dispatch_task(
        db: &Arc<MongoDataSource>,
        task: &Task,
        now: &DateTime<Local>,
    ) -> Result<bool> {
        let task_id = task
            .id
            .ok_or_else(|| Error::Validation("任务 ID 不能为空".to_string()))?;

        let cron_parser = CronParser::new(&task.schedule)
            .map_err(|e| Error::Scheduling(format!("解析 Cron 表达式失败: {}", e)))?;

        let next_trigger = cron_parser
            .next_trigger()
            .ok_or_else(|| Error::Scheduling("无法获取下一次触发时间".to_string()))?;

        let next_trigger_local = next_trigger.with_timezone(&Local);

        if next_trigger_local > *now {
            return Ok(false);
        }

        let recent_instances = db
            .find_task_instances(
                Some(doc! {
                    "task_id": task_id,
                    "status": { "$ne": "cancelled" }
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?;

        for instance in recent_instances {
            if instance.scheduled_time >= next_trigger_local {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 分发单个任务
    async fn dispatch_single_task(
        db: &Arc<MongoDataSource>,
        task_queue: &Arc<TaskQueue>,
        task: &Task,
        now: &DateTime<Local>,
    ) -> Result<ObjectId> {
        let task_id = task
            .id
            .ok_or_else(|| Error::Validation("任务 ID 不能为空".to_string()))?;

        let cron_parser = CronParser::new(&task.schedule)
            .map_err(|e| Error::Scheduling(format!("解析 Cron 表达式失败: {}", e)))?;

        let scheduled_time = cron_parser
            .next_trigger()
            .ok_or_else(|| Error::Scheduling("无法获取触发时间".to_string()))?
            .with_timezone(&Local);

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
            .create_task_instance(instance)
            .await
            .map_err(|e| Error::Database(format!("创建任务实例失败: {}", e)))?;

        let task_msg = crate::executor::TaskMessage {
            instance_id,
            task_id,
            task_name: task.name.clone(),
            scheduled_time: scheduled_time.timestamp(),
            retry_count: 0,
        };

        task_queue
            .publish_task(task_msg)
            .await
            .map_err(|e| Error::MessageQueue(format!("发布任务到队列失败: {}", e)))?;

        info!(
            "已为任务 {} 创建实例 {}，计划执行时间: {}，已发布到队列",
            task.name, instance_id, scheduled_time
        );

        Ok(instance_id)
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

        let now = Local::now();

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
            .create_task_instance(instance)
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
