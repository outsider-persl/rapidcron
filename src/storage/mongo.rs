#![allow(dead_code)]
use crate::config::DatabaseConfig;
use crate::types::*;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::{
    Collection, Database,
    bson::{Document, doc, oid::ObjectId},
    options::FindOptions,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MongoDataSource {
    database: Arc<Database>,
}

impl MongoDataSource {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let mut options = mongodb::options::ClientOptions::parse(&config.uri).await?;

        // 设置认证
        if !config.username.is_empty() && !config.password.is_empty() {
            options.credential = Some(
                mongodb::options::Credential::builder()
                    .username(config.username.clone())
                    .password(config.password.clone())
                    .build(),
            );
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

    fn dispatch_logs(&self) -> Collection<DispatchLog> {
        self.database.collection("dispatch_logs")
    }
}
impl MongoDataSource {
    pub async fn create_task(&self, task: &Task) -> Result<ObjectId> {
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

    pub async fn find_tasks(
        &self,
        filter: Option<Document>,
        _options: Option<FindOptions>,
    ) -> Result<Vec<Task>> {
        let collection = self.tasks();
        let mut cursor = collection.find(filter.unwrap_or_default()).await?;
        let mut tasks = Vec::new();
        while let Some(task) = cursor.try_next().await? {
            tasks.push(task);
        }
        Ok(tasks)
    }

    pub async fn create_task_instance(&self, instance: &TaskInstance) -> Result<ObjectId> {
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

    pub async fn find_task_instances(
        &self,
        filter: Option<Document>,
        _options: Option<FindOptions>,
    ) -> Result<Vec<TaskInstance>> {
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

    pub async fn find_execution_logs(
        &self,
        filter: Option<Document>,
        _options: Option<FindOptions>,
    ) -> Result<Vec<ExecutionLog>> {
        let collection = self.execution_logs();
        let mut cursor = collection.find(filter.unwrap_or_default()).await?;
        let mut logs = Vec::new();
        while let Some(log) = cursor.try_next().await? {
            logs.push(log);
        }
        Ok(logs)
    }

    pub async fn create_dispatch_log(&self, log: &DispatchLog) -> Result<ObjectId> {
        let collection = self.dispatch_logs();
        let result = collection.insert_one(log).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_dispatch_logs(
        &self,
        filter: Option<Document>,
        _options: Option<FindOptions>,
    ) -> Result<Vec<DispatchLog>> {
        let collection = self.dispatch_logs();
        let mut cursor = collection.find(filter.unwrap_or_default()).await?;
        let mut logs = Vec::new();
        while let Some(log) = cursor.try_next().await? {
            logs.push(log);
        }
        Ok(logs)
    }

    pub async fn get_last_dispatch_log(&self) -> Result<Option<DispatchLog>> {
        let collection = self.dispatch_logs();
        let mut cursor = collection.find(doc! {}).await?;
        Ok(cursor.try_next().await?)
    }

    pub async fn delete_dispatch_logs_before(
        &self,
        cutoff_time: &chrono::DateTime<chrono::Utc>,
    ) -> Result<u64> {
        let collection = self.dispatch_logs();
        let result = collection
            .delete_many(doc! {
                "scan_time": { "$lt": cutoff_time }
            })
            .await?;
        Ok(result.deleted_count)
    }

    pub async fn clear_all_data(&self) -> Result<()> {
        self.tasks().delete_many(doc! {}).await?;
        self.task_instances().delete_many(doc! {}).await?;
        self.execution_logs().delete_many(doc! {}).await?;
        Ok(())
    }
}