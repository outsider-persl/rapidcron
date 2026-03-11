use crate::error::{Error, Result};
use crate::executor::TaskQueue;
use crate::scheduler::cron_parser::CronParser;
use crate::storage::mongo::MongoDataSource;
use crate::types::{DispatchLog, Task, TaskInstance, TaskStatus};
use chrono::{DateTime, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

type TaskInstanceMap = HashMap<ObjectId, HashSet<i64>>;

/// 任务分发器
pub struct Dispatcher {
    db: Arc<MongoDataSource>,
    task_queue: Arc<TaskQueue>,
    running: Arc<RwLock<bool>>,
    last_scan_end_time: Arc<RwLock<DateTime<Utc>>>,
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
            last_scan_end_time: Arc::new(RwLock::new(Utc::now())),
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

        info!(
            "[Dispatcher] 任务分发器已启动，扫描间隔: {:?}",
            self.scan_interval
        );

        let interval = self.scan_interval;
        let scan_interval_secs = interval.as_secs();

        // 从数据库读取上次扫描结束时间，避免重启时重复扫描
        if let Ok(Some(last_log)) = self.db.get_last_dispatch_log().await {
            let mut last_end = self.last_scan_end_time.write().await;
            *last_end = last_log.scan_window_end;
            drop(last_end);
            info!(
                "[Dispatcher] 从数据库恢复上次扫描结束时间: {}",
                last_log.scan_window_end.format("%H:%M:%S")
            );
        }

        // 启动时做一次全局去重，与上次扫描时间无关
        // 注意：即使恢复了 last_scan_end_time，启动去重是"全局 pending 去重"，与时间窗口无关
        // 这是为了清理所有状态为 pending 的重复实例，避免重启后出现重复执行
        if let Err(e) = Self::check_and_dedup_instances(&self.db).await {
            error!("[Dispatcher] 启动去重失败: {}", e);
        }

        let db = Arc::clone(&self.db);
        let task_queue = Arc::clone(&self.task_queue);
        let running_flag = Arc::clone(&self.running);
        let db_cleanup = Arc::clone(&self.db);
        let running_flag_cleanup = Arc::clone(&self.running);
        let log_retention_days = self.log_retention_days;
        let last_scan_end_time = Arc::clone(&self.last_scan_end_time);

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);

            while *running_flag.read().await {
                timer.tick().await;

                match Self::scan_and_dispatch(
                    &db,
                    &task_queue,
                    scan_interval_secs,
                    &last_scan_end_time,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        error!("[Dispatcher] 任务分发失败: {}", e);
                    }
                }
            }

            info!("[Dispatcher] 任务分发器已停止");
        });

        // 启动独立的清理任务
        tokio::spawn(async move {
            let mut cleanup_timer = tokio::time::interval(Duration::from_secs(86400)); // 24小时 = 86400秒

            while *running_flag_cleanup.read().await {
                cleanup_timer.tick().await;

                if let Err(e) = Self::cleanup_old_logs(&db_cleanup, log_retention_days).await {
                    error!("[Dispatcher] 清理旧日志失败: {}", e);
                }
            }
        });

        Ok(())
    }

    /// 停止分发器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("[Dispatcher] 正在停止任务分发器...");
        Ok(())
    }

    /// 计算扫描窗口并更新上次扫描结束时间
    async fn calculate_scan_window(
        now: DateTime<Utc>,
        scan_interval_secs: u64,
        last_scan_end_time: &Arc<RwLock<DateTime<Utc>>>,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        let scan_window = chrono::Duration::seconds(scan_interval_secs as i64);

        let mut last_end = last_scan_end_time.write().await;
        let scan_window_start = *last_end;
        let scan_window_end = now + scan_window;
        *last_end = scan_window_end;
        drop(last_end);

        (scan_window_start, scan_window_end)
    }

    /// 扫描并分发任务
    async fn scan_and_dispatch(
        db: &Arc<MongoDataSource>,
        task_queue: &Arc<TaskQueue>,
        scan_interval_secs: u64,
        last_scan_end_time: &Arc<RwLock<DateTime<Utc>>>,
    ) -> Result<usize> {
        let now = Utc::now();

        let (scan_window_start, scan_window_end) =
            Self::calculate_scan_window(now, scan_interval_secs, last_scan_end_time).await;

        info!(
            "[Dispatcher] 开始扫描任务，窗口: {} 到 {}",
            scan_window_start.format("%H:%M:%S"),
            scan_window_end.format("%H:%M:%S")
        );

        // 批量查询所有启用的任务
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
            debug!("[Dispatcher] 没有启用的任务");

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
                error!("[Dispatcher] 创建调度日志失败: {}", e);
            }

            return Ok(0);
        }

        // 收集所有任务ID
        let task_ids: Vec<ObjectId> = enabled_tasks.iter().filter_map(|task| task.id).collect();

        // 批量查询所有任务在扫描窗口内的实例
        let all_existing_instances = if !task_ids.is_empty() {
            db.find_task_instances(
                Some(doc! {
                    "task_id": { "$in": task_ids },
                    "scheduled_time": { "$gte": now, "$lte": scan_window_end }
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?
        } else {
            Vec::new()
        };

        // 按任务ID分组，存储已存在的调度时间
        let mut existing_instances_map: TaskInstanceMap = TaskInstanceMap::new();
        for instance in all_existing_instances {
            existing_instances_map
                .entry(instance.task_id)
                .or_default()
                .insert(instance.scheduled_time.timestamp());
        }

        let mut dispatched_count = 0;

        for task in enabled_tasks {
            if let Some(task_id) = task.id {
                match Self::dispatch_task_instances(
                    db,
                    task_queue,
                    &task,
                    &now,
                    &scan_window_end,
                    existing_instances_map.get(&task_id),
                )
                .await
                {
                    Ok(count) => {
                        if count > 0 {
                            info!(
                                "[Dispatcher] 任务 {} 创建并分发 {} 个实例",
                                task.name, count
                            );
                            dispatched_count += count;
                        }
                    }
                    Err(e) => {
                        error!("[Dispatcher] 分发任务 {} 失败: {}", task.name, e);
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
            error!("[Dispatcher] 创建调度日志失败: {}", e);
        }

        info!(
            "[Dispatcher] 扫描完成 - 总任务: {}, 启用: {}, 分发实例: {}",
            total_tasks, total_tasks, dispatched_count
        );

        Ok(dispatched_count)
    }

    /// 启动时对所有待执行实例做一次去重（无关上次扫描时间）
    async fn check_and_dedup_instances(db: &Arc<MongoDataSource>) -> Result<()> {
        let all_existing_instances = db
            .find_task_instances(
                Some(doc! {
                    "status": "pending"
                }),
                None,
            )
            .await
            .map_err(|e| Error::Database(format!("查询任务实例失败: {}", e)))?;

        let mut existing_instances_map: TaskInstanceMap = TaskInstanceMap::new();
        let mut removed_count = 0u64;

        for instance in all_existing_instances {
            let scheduled_ts = instance.scheduled_time.timestamp();
            let entry = existing_instances_map.entry(instance.task_id).or_default();

            if entry.contains(&scheduled_ts) {
                if let Some(id) = instance.id {
                    db.delete_task_instance(id)
                        .await
                        .map_err(|e| Error::Database(format!("删除重复任务实例失败: {}", e)))?;
                    removed_count += 1;
                }
            } else {
                entry.insert(scheduled_ts);
            }
        }

        if removed_count > 0 {
            if removed_count > 1000 {
                warn!(
                    "[Dispatcher] 启动去重完成，删除 {} 个重复的待执行任务实例",
                    removed_count
                );
            } else {
                info!(
                    "[Dispatcher] 启动去重完成，删除 {} 个重复的待执行任务实例",
                    removed_count
                );
            }
        } else {
            info!("[Dispatcher] 启动去重完成，未发现重复的待执行任务实例");
        }

        Ok(())
    }

    /// 为任务创建并分发实例
    async fn dispatch_task_instances(
        db: &Arc<MongoDataSource>,
        task_queue: &Arc<TaskQueue>,
        task: &Task,
        now: &DateTime<Utc>,
        scan_window_end: &DateTime<Utc>,
        existing_instances: Option<&std::collections::HashSet<i64>>,
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

        // 使用传入的已存在实例集合，避免重复查询数据库
        let empty_set = std::collections::HashSet::new();
        let existing_scheduled_times = existing_instances.unwrap_or(&empty_set);

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
}
