use clap::{Parser, ValueEnum};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub enable_tls: bool,
    pub shutdown_grace_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub stdout: bool,
    pub file_path: String,
    pub json: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub bind_addr: String,
    pub path: String,
    // 指标收集间隔（秒）
    pub collection_interval: u64,
    // 最大重启次数，0表示无限重启
    pub max_restarts: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchedulerConfig {
    pub timezone: String,
    pub max_concurrency: u32,
    pub default_shard_count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutorRetryConfig {
    pub max_retries: i32,
    pub backoff_seconds: i32,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutorIdempotentConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExecutorConfig {
    pub retry: ExecutorRetryConfig,
    pub idempotent: ExecutorIdempotentConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageMongodbConfig {
    pub uri: String,
    pub database: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageEtcdConfig {
    pub endpoints: Vec<String>,
    pub namespace: String,
    pub lease_ttl_seconds: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub mongodb: StorageMongodbConfig,
    pub etcd: StorageEtcdConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessagingRabbitmqConfig {
    pub uri: String,
    pub queue: String,
    pub prefetch: u16,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessagingConfig {
    pub rabbitmq: MessagingRabbitmqConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub token_required: bool,
    pub allowed_alg: String,
    pub public_key_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcConfig {
    pub enable_compression: bool,
    pub max_decoding_message_size: usize,
    pub max_encoding_message_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeaturesConfig {
    pub enable_task_log_export: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
    pub scheduler: SchedulerConfig,
    pub executor: ExecutorConfig,
    pub storage: StorageConfig,
    pub messaging: MessagingConfig,
    pub security: SecurityConfig,
    pub grpc: GrpcConfig,
    pub features: FeaturesConfig,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// dev env
    #[arg(long, conflicts_with = "dev")]
    prod: bool,

    /// prod env
    #[arg(long, conflicts_with = "prod")]
    dev: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Env {
    /// 开发环境
    Dev,
    /// 生产环境
    Prod,
}

impl From<Env> for String {
    fn from(env: Env) -> Self {
        match env {
            Env::Dev => "development".to_string(),
            Env::Prod => "production".to_string(),
        }
    }
}

/// 加载应用配置
/// 优先从命令行参数获取环境类型，其次从环境变量RAPIDCRON_ENV，默认使用development环境
pub fn load_config() -> Result<AppConfig, anyhow::Error> {
    let cli = Cli::parse();
    let env_from_cli = if cli.prod {
        "production".to_string()
    } else {
        // default
        "development".to_string()
    };

    let env = env::var("RAPIDCRON_ENV").unwrap_or(env_from_cli);
    let config_path = format!("src/config/{}", env);

    let builder = Config::builder()
        .add_source(File::with_name(&config_path).required(true))
        .add_source(Environment::with_prefix("RAPIDCRON").separator("__"));

    let cfg = builder.build()?;
    let app: AppConfig = cfg.try_deserialize()?;
    Ok(app)
}
