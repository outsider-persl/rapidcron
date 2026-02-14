#![allow(dead_code)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RapidCronError {
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
}

pub type Result<T> = std::result::Result<T, RapidCronError>;