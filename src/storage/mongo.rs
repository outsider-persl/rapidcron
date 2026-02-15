#![allow(dead_code)]
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::{bson::{doc, Document, oid::ObjectId}, options::FindOptions, Collection, Database};
use std::sync::Arc;
use crate::config::DatabaseConfig;
use crate::types::*;

pub struct MongoDataSource {
    database: Arc<Database>,
}

impl MongoDataSource {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let mut options = mongodb::options::ClientOptions::parse(&config.uri).await?;
        
        // 设置认证
        if !config.username.is_empty() && !config.password.is_empty() {
            options.credential = Some(mongodb::options::Credential::builder()
                .username(config.username.clone())
                .password(config.password.clone())
                .build());
        }
        
        let client = mongodb::Client::with_options(options)?;
        let database = client.database(&config.database_name);
        
        Ok(Self {
            database: Arc::new(database),
        })
    }

    fn tasks(&self) -> Collection<Task> {
        self.database.collection("tasks")
    }

    fn task_instances(&self) -> Collection<TaskInstance> {
        self.database.collection("task_instances")
    }

    fn execution_logs(&self) -> Collection<ExecutionLog> {
        self.database.collection("execution_logs")
    }
}

impl MongoDataSource {
    pub async fn create_task(&self, task: Task) -> Result<ObjectId> {
        let collection = self.tasks();
        let result = collection.insert_one(task).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn get_task(&self, id: ObjectId) -> Result<Option<Task>> {
        let collection = self.tasks();
        let task = collection.find_one(doc! { "_id": id }).await?;
        Ok(task)
    }

    pub async fn update_task(&self, id: ObjectId, update: Document) -> Result<bool> {
        let collection = self.tasks();
        let result = collection.update_one(doc! { "_id": id }, update).await?;
        Ok(result.modified_count > 0)
    }

    pub async fn delete_task(&self, id: ObjectId) -> Result<bool> {
        let collection = self.tasks();
        let result = collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn find_tasks(&self, filter: Option<Document>, _options: Option<FindOptions>) -> Result<Vec<Task>> {
        let collection = self.tasks();
        let mut cursor = collection.find(filter.unwrap_or_default()).await?;
        let mut tasks = Vec::new();
        while let Some(task) = cursor.try_next().await? {
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub async fn create_task_instance(&self, instance: TaskInstance) -> Result<ObjectId> {
        let collection = self.task_instances();
        let result = collection.insert_one(instance).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn get_task_instance(&self, id: ObjectId) -> Result<Option<TaskInstance>> {
        let collection = self.task_instances();
        let instance = collection.find_one(doc! { "_id": id }).await?;
        Ok(instance)
    }

    pub async fn update_task_instance(&self, id: ObjectId, update: Document) -> Result<bool> {
        let collection = self.task_instances();
        let result = collection.update_one(doc! { "_id": id }, update).await?;
        Ok(result.modified_count > 0)
    }

    pub async fn delete_task_instance(&self, id: ObjectId) -> Result<bool> {
        let collection = self.task_instances();
        let result = collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn find_task_instances(&self, filter: Option<Document>, _options: Option<FindOptions>) -> Result<Vec<TaskInstance>> {
        let collection = self.task_instances();
        let mut cursor = collection.find(filter.unwrap_or_default()).await?;
        let mut instances = Vec::new();
        while let Some(instance) = cursor.try_next().await? {
            instances.push(instance);
        }
        Ok(instances)
    }

    pub async fn create_execution_log(&self, log: ExecutionLog) -> Result<ObjectId> {
        let collection = self.execution_logs();
        let result = collection.insert_one(log).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn get_execution_log(&self, id: ObjectId) -> Result<Option<ExecutionLog>> {
        let collection = self.execution_logs();
        let log = collection.find_one(doc! { "_id": id }).await?;
        Ok(log)
    }

    pub async fn update_execution_log(&self, id: ObjectId, update: Document) -> Result<bool> {
        let collection = self.execution_logs();
        let result = collection.update_one(doc! { "_id": id }, update).await?;
        Ok(result.modified_count > 0)
    }

    pub async fn delete_execution_log(&self, id: ObjectId) -> Result<bool> {
        let collection = self.execution_logs();
        let result = collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn find_execution_logs(&self, filter: Option<Document>, _options: Option<FindOptions>) -> Result<Vec<ExecutionLog>> {
        let collection = self.execution_logs();
        let mut cursor = collection.find(filter.unwrap_or_default()).await?;
        let mut logs = Vec::new();
        while let Some(log) = cursor.try_next().await? {
            logs.push(log);
        }
        Ok(logs)
    }

    pub async fn clear_all_data(&self) -> Result<()> {
        self.tasks().delete_many(doc! {}).await?;
        self.task_instances().delete_many(doc! {}).await?;
        self.execution_logs().delete_many(doc! {}).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::bson::{doc, oid::ObjectId};

    async fn get_test_db() -> MongoDataSource {
        let config = crate::config::load("config.toml")
            .expect("Failed to load config")
            .database;
        MongoDataSource::new(&config)
            .await
            .expect("Failed to create MongoDataSource")
    }

    // ========== 公共测试数据 ==========
    fn create_test_task() -> Task {
        Task {
            id: None,
            name: "unit-test-task".to_string(),
            description: Some("单元测试任务".to_string()),
            dependency_ids: vec![],
            task_type: TaskType::Command,
            schedule: "0 2 * * * *".to_string(),
            enabled: true,
            payload: TaskPayload::Command {
                command: "echo test".to_string(),
                timeout_seconds: Some(30),
            },
            timeout_seconds: Some(30),
            max_retries: Some(3),
            created_at: chrono::Local::now(),
            updated_at: chrono::Local::now(),
            deleted_at: None,
        }
    }

    fn create_test_instance() -> TaskInstance {
        TaskInstance {
            id: None,
            task_id: ObjectId::new(),
            scheduled_time: chrono::Local::now(),
            status: TaskStatus::Pending,
            executor_id: None,
            start_time: None,
            end_time: None,
            retry_count: 0,
            result: None,
            created_at: chrono::Local::now(),
        }
    }

    fn create_test_log() -> ExecutionLog {
        ExecutionLog {
            id: None,
            task_id: ObjectId::new(),
            task_name: "unit-test-task".to_string(),
            instance_id: ObjectId::new(),
            scheduled_time: chrono::Local::now(),
            start_time: Some(chrono::Local::now()),
            end_time: chrono::Local::now(),
            status: TaskStatus::Success,
            duration_ms: 1000,
            output_summary: Some("单元测试执行成功".to_string()),
            error_message: None,
            triggered_by: TriggeredBy::Scheduler,
        }
    }

    // ========== 公共测试方法 ==========
    async fn test_insert_and_verify_task(db: &MongoDataSource) -> ObjectId {
        let task = create_test_task();
        
        let existing_tasks = db.find_tasks(
            Some(doc! { "name": "unit-test-task" }),
            None
        ).await.unwrap();
        assert_eq!(existing_tasks.len(), 0);
        
        let task_id = db.create_task(task).await.unwrap();
        assert!(!task_id.to_hex().is_empty());
        task_id
    }

    async fn test_update_and_verify_task(db: &MongoDataSource, task_id: ObjectId) {
        let update = doc! { "$set": { "enabled": false, "description": "已修改的单元测试任务" } };
        let updated = db.update_task(task_id, update).await.unwrap();
        assert!(updated);
        
        let task_data = db.get_task(task_id).await.unwrap().unwrap();
        assert_eq!(task_data.enabled, false);
        assert_eq!(task_data.description.unwrap(), "已修改的单元测试任务");
    }

    async fn test_delete_and_verify_task(db: &MongoDataSource, task_id: ObjectId) {
        let deleted = db.delete_task(task_id).await.unwrap();
        assert!(deleted);
        
        let deleted_task = db.get_task(task_id).await.unwrap();
        assert!(deleted_task.is_none());
    }

    async fn test_insert_and_verify_instance(db: &MongoDataSource) -> ObjectId {
        let instance = create_test_instance();
        let task_id = instance.task_id;
        
        let existing_instances = db.find_task_instances(
            Some(doc! { "task_id": task_id }),
            None
        ).await.unwrap();
        let initial_count = existing_instances.len();
        
        let instance_id = db.create_task_instance(instance).await.unwrap();
        assert!(!instance_id.to_hex().is_empty());
        
        let new_instances = db.find_task_instances(
            Some(doc! { "task_id": task_id }),
            None
        ).await.unwrap();
        assert_eq!(new_instances.len(), initial_count + 1);
        instance_id
    }

    async fn test_update_and_verify_instance(db: &MongoDataSource, instance_id: ObjectId) {
        let update = doc! { "$set": { "status": "running", "retry_count": 1 } };
        let updated = db.update_task_instance(instance_id, update).await.unwrap();
        assert!(updated);
        
        let instance_data = db.get_task_instance(instance_id).await.unwrap().unwrap();
        assert_eq!(instance_data.status, TaskStatus::Running);
        assert_eq!(instance_data.retry_count, 1);
    }

    async fn test_delete_and_verify_instance(db: &MongoDataSource, instance_id: ObjectId) {
        let deleted = db.delete_task_instance(instance_id).await.unwrap();
        assert!(deleted);
        
        let deleted_instance = db.get_task_instance(instance_id).await.unwrap();
        assert!(deleted_instance.is_none());
    }

    async fn test_insert_and_verify_log(db: &MongoDataSource) -> ObjectId {
        let log = create_test_log();
        let task_id = log.task_id;
        
        let existing_logs = db.find_execution_logs(
            Some(doc! { "task_id": task_id }),
            None
        ).await.unwrap();
        let initial_count = existing_logs.len();
        
        let log_id = db.create_execution_log(log).await.unwrap();
        assert!(!log_id.to_hex().is_empty());
        
        let new_logs = db.find_execution_logs(
            Some(doc! { "task_id": task_id }),
            None
        ).await.unwrap();
        assert_eq!(new_logs.len(), initial_count + 1);
        log_id
    }

    async fn test_update_and_verify_log(db: &MongoDataSource, log_id: ObjectId) {
        let update = doc! { "$set": { "duration_ms": 2000, "output_summary": "修改后的执行结果" } };
        let updated = db.update_execution_log(log_id, update).await.unwrap();
        assert!(updated);
        
        let log_data = db.get_execution_log(log_id).await.unwrap().unwrap();
        assert_eq!(log_data.duration_ms, 2000);
        assert_eq!(log_data.output_summary.unwrap(), "修改后的执行结果");
    }

    async fn test_delete_and_verify_log(db: &MongoDataSource, log_id: ObjectId) {
        let deleted = db.delete_execution_log(log_id).await.unwrap();
        assert!(deleted);
        
        let deleted_log = db.get_execution_log(log_id).await.unwrap();
        assert!(deleted_log.is_none());
    }

    // ========== 任务CRUD测试 ==========
    
    // 聚合测试 - 完整CRUD流程
    #[tokio::test]
    async fn test_task_crud_aggregated() {
        let db = get_test_db().await;
        
        let task_id = test_insert_and_verify_task(&db).await;
        test_update_and_verify_task(&db, task_id).await;
        test_delete_and_verify_task(&db, task_id).await;
    }

    // 拆分测试 - 手动验证
    #[tokio::test]
    async fn test_task_insert() {
        let db = get_test_db().await;
        test_insert_and_verify_task(&db).await;
    }

    // #[tokio::test]
    async fn test_task_update() {
        let db = get_test_db().await;
        let task_id = db.find_tasks(Some(doc! { "name": "unit-test-task" }), None)
            .await.unwrap()[0].id.unwrap();
        test_update_and_verify_task(&db, task_id).await;
    }

    // #[tokio::test]
    async fn test_task_search() {
        let db = get_test_db().await;
        let task_id = db.find_tasks(Some(doc! { "name": "unit-test-task" }), None)
            .await.unwrap()[0].id.unwrap();
        test_update_and_verify_task(&db, task_id).await;
    }

    // #[tokio::test]
    async fn test_task_delete() {
        let db = get_test_db().await;
        let task_id = db.find_tasks(Some(doc! { "name": "unit-test-task" }), None)
            .await.unwrap()[0].id.unwrap();
        test_delete_and_verify_task(&db, task_id).await;
    }

// ========== 任务实例CRUD测试 ==========

    // 聚合测试 - 完整CRUD流程
    #[tokio::test]
    async fn test_task_instance_crud_aggregated() {
        let db = get_test_db().await;
        
        let instance_id = test_insert_and_verify_instance(&db).await;
        test_update_and_verify_instance(&db, instance_id).await;
        test_delete_and_verify_instance(&db, instance_id).await;
    }

    // 拆分测试 - 手动验证
    // #![tokio::test]
    async fn test_task_instance_insert() {
        let db = get_test_db().await;
        test_insert_and_verify_instance(&db).await;
    }

    // #![tokio::test]
    async fn test_task_instance_update() {
        let db = get_test_db().await;
        let instance_id = db.find_task_instances(Some(doc! {}), None)
            .await.unwrap()[0].id.unwrap();
        test_update_and_verify_instance(&db, instance_id).await;
    }

    // #![tokio::test]
    async fn test_task_instance_search() {
        let db = get_test_db().await;
        let instance_id = db.find_task_instances(Some(doc! {}), None)
            .await.unwrap()[0].id.unwrap();
        test_update_and_verify_instance(&db, instance_id).await;
    }

    // #![tokio::test]
    async fn test_task_instance_delete() {
        let db = get_test_db().await;
        let instance_id = db.find_task_instances(Some(doc! {}), None)
            .await.unwrap()[0].id.unwrap();
        test_delete_and_verify_instance(&db, instance_id).await;
    }

// ========== 执行日志CRUD测试 ==========

    // 聚合测试 - 完整CRUD流程
    #[tokio::test]
    async fn test_execution_log_crud_aggregated() {
        let db = get_test_db().await;
        
        let log_id = test_insert_and_verify_log(&db).await;
        test_update_and_verify_log(&db, log_id).await;
        test_delete_and_verify_log(&db, log_id).await;
    }

    // 拆分测试 - 手动验证
    // #![tokio::test]
    async fn test_execution_log_insert() {
        let db = get_test_db().await;
        test_insert_and_verify_log(&db).await;
    }

    // #![tokio::test]
    async fn test_execution_log_update() {
        let db = get_test_db().await;
        let log_id = db.find_execution_logs(Some(doc! {}), None)
            .await.unwrap()[0].id.unwrap();
        test_update_and_verify_log(&db, log_id).await;
    }

    // #![tokio::test]
    async fn test_execution_log_search() {
        let db = get_test_db().await;
        let log_id = db.find_execution_logs(Some(doc! {}), None)
            .await.unwrap()[0].id.unwrap();
        test_update_and_verify_log(&db, log_id).await;
    }

    // #![tokio::test]
    async fn test_execution_log_delete() {
        let db = get_test_db().await;
        let log_id = db.find_execution_logs(Some(doc! {}), None)
            .await.unwrap()[0].id.unwrap();
        test_delete_and_verify_log(&db, log_id).await;
    }
}