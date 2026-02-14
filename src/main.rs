// src/main.rs
mod config;
mod error;
mod logging;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load("config.toml")?;
    let _log_guard = logging::init(&cfg.logging)?;
    info!("configuration loaded: {:?}", cfg.server);
    
    // 保持程序运行一段时间以确保日志写入
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    Ok(())
}
