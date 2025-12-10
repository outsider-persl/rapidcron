use tracing_subscriber::fmt::format::FmtSpan;

use crate::config::loader::load_config;
use crate::grpc::server::GrpcServer;
use crate::storage::mongodb::MongoDB;
use crate::common::logger::init_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化日志
    init_logger();
    // 2. 加载配置
    let config = load_config()?;
    // 3. 初始化MongoDB连接
    let mongodb = MongoDB::new(&config.mongodb.uri, &config.mongodb.db_name).await?;
    // 4. 启动gRPC服务
    let grpc_server = GrpcServer::new(&config.grpc, mongodb);
    grpc_server.start().await?;

    Ok(())
}