#![allow(dead_code)]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
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

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Error::Execution(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::Serialization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            Error::Io(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            Error::Scheduling(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::Network(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::CronFieldCount(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Error::CronSyntax(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Error::CronTimeRange(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Error::CronInternal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::Etcd(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::MessageQueue(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::ServiceRegistration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Error::DistributedLock(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(ErrorResponse {
            success: false,
            message,
        });

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Database(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
