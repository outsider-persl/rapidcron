#![allow(dead_code)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task scheduling error: {0}")]
    Scheduling(String),

    #[error("Task execution error: {0}")]
    Execution(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Cron field count error: {0}")]
    CronFieldCount(String),

    #[error("Cron syntax error: {0}")]
    CronSyntax(String),

    #[error("Cron time range error: {0}")]
    CronTimeRange(String),

    #[error("Cron internal error: {0}")]
    CronInternal(String),

    #[error("Etcd error: {0}")]
    Etcd(String),

    #[error("Message queue error: {0}")]
    MessageQueue(String),

    #[error("Service registration error: {0}")]
    ServiceRegistration(String),

    #[error("Distributed lock error: {0}")]
    DistributedLock(String),
}

pub type Result<T> = std::result::Result<T, Error>;