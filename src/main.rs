// src/main.rs
mod config;
mod error;
mod logging;
mod storage;
mod types;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load("config.toml")?;
    let _log_guard = logging::init(&cfg.logging)?;
    info!("configuration loaded");
    
    let _db = storage::mongo::MongoDataSource::new(&cfg.database).await?;
    info!("mongodb connection established");
    
    info!("RapidCron server is running on {}:{}", cfg.server.host, cfg.server.http_port);
    
    tokio::signal::ctrl_c().await?;
    info!("RapidCron server is shutting down...");
    
    Ok(())
}
