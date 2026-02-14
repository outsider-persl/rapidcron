#![allow(dead_code)]
use serde::Deserialize;
use std::fs;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub rabbitmq: RabbitMQConfig,
    pub etcd: EtcdConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub http_port: u16,
    pub grpc_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub log_file: String,
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtcdConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
}

pub fn load(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}