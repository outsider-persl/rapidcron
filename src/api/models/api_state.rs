use std::sync::Arc;

use crate::coord::EtcdManager;
use crate::executor::TaskQueue;
use crate::storage::mongo::MongoDataSource;

/// API 状态
#[derive(Clone)]
pub struct ApiState {
    pub db: MongoDataSource,
    pub etcd_manager: Option<Arc<EtcdManager>>,
    pub task_queue: Option<Arc<TaskQueue>>,
}

impl ApiState {
    pub fn new(db: MongoDataSource) -> Self {
        Self {
            db,
            etcd_manager: None,
            task_queue: None,
        }
    }

    pub fn with_etcd(mut self, etcd_manager: Arc<EtcdManager>) -> Self {
        self.etcd_manager = Some(etcd_manager);
        self
    }

    pub fn with_task_queue(mut self, task_queue: Arc<TaskQueue>) -> Self {
        self.task_queue = Some(task_queue);
        self
    }
}
