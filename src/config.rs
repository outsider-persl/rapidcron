use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub rabbitmq: RabbitMQConfig,
    pub etcd: EtcdConfig,
    pub dispatcher: DispatcherConfig,
    pub retry: RetryConfig,
    pub logging: LoggingConfig,
    pub service: ServiceConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub http_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub database_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMQConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub queue_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtcdConfig {
    pub host: String,
    pub port: u16,
    pub service_prefix: String,
    pub dead_threshold_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DispatcherConfig {
    pub scan_interval_secs: u64,
    pub log_retention_days: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    pub scan_interval_secs: u64,
    pub batch_size: usize,
    pub default_max_retries: i32,
    pub default_strategy: String,
    pub exponential_base_delay: i64,
    pub exponential_max_delay: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub log_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub service_name: String,
    pub metadata: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
    pub role: String,
}

pub fn load(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
