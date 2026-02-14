use crate::config::LoggingConfig;
use anyhow::Result;
use std::fs;
use time::OffsetDateTime;
use tracing_appender;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// 初始化日志系统
/// 根据配置选择日志输出方式（文件、控制台或两者）和格式（JSON或纯文本）
/// 日志文件按日期命名，保存在指定目录下
pub fn init(config: &LoggingConfig) -> Result<()> {
    if config.output == "file" || config.output == "both" {
        fs::create_dir_all(&config.log_file)?;
    }

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("rapidcron={}", config.level)));

    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let date_str = format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    );

    match config.output.as_str() {
        "stdout" => init_stdout(config, env_filter),
        "file" => init_file(config, env_filter, &date_str),
        "both" => init_both(config, env_filter, &date_str),
        _ => Err(anyhow::anyhow!("Unsupported log output: {}", config.output)),
    }
}

fn init_stdout(config: &LoggingConfig, env_filter: EnvFilter) -> Result<()> {
    match config.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        }
        "plain" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339()),
                )
                .init();
        }
        _ => return Err(anyhow::anyhow!("Unsupported log format: {}", config.format)),
    }
    Ok(())
}

fn init_file(config: &LoggingConfig, env_filter: EnvFilter, date_str: &str) -> Result<()> {
    let writer = tracing_appender::rolling::never(&config.log_file, &format!("{}.log", date_str));

    match config.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json().with_writer(writer).with_ansi(false))
                .init();
        }
        "plain" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .with_writer(writer)
                        .with_ansi(false)
                        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339()),
                )
                .init();
        }
        _ => return Err(anyhow::anyhow!("Unsupported log format: {}", config.format)),
    }
    Ok(())
}

fn init_both(config: &LoggingConfig, env_filter: EnvFilter, date_str: &str) -> Result<()> {
    let writer = tracing_appender::rolling::never(&config.log_file, &format!("{}.log", date_str));

    match config.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .with(fmt::layer().json().with_writer(writer).with_ansi(false))
                .init();
        }
        "plain" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339()),
                )
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .with_writer(writer)
                        .with_ansi(false)
                        .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339()),
                )
                .init();
        }
        _ => return Err(anyhow::anyhow!("Unsupported log format: {}", config.format)),
    }
    Ok(())
}
