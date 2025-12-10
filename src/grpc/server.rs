use tonic::transport::Server;
use tracing::info;

use crate::grpc::interceptor::auth::AuthInterceptor;
use crate::grpc::proto::task_scheduler::*;
use crate::grpc::service::{task_service::TaskServiceImpl, node_service::NodeServiceImpl};
use crate::storage::mongodb::MongoDB;
use crate::config::types::GrpcConfig;

// gRPC服务启动器
pub struct GrpcServer {
    addr: String,
    task_service: TaskServiceImpl,
    node_service: NodeServiceImpl,
    auth_interceptor: AuthInterceptor,
}

impl GrpcServer {
    // 初始化gRPC服务
    pub fn new(config: &GrpcConfig, mongodb: MongoDB) -> Self {
        Self {
            addr: format!("{}:{}", config.host, config.port),
            task_service: TaskServiceImpl::new(mongodb.clone()),
            node_service: NodeServiceImpl::new(),
            auth_interceptor: AuthInterceptor::new(),
        }
    }

    // 启动gRPC服务
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.addr.parse()?;
        info!("gRPC服务启动，监听地址: {}", addr);

        // 注册服务并启动
        Server::builder()
            .add_service(
                task_service_server::TaskServiceServer::with_interceptor(
                    self.task_service,
                    self.auth_interceptor.clone(),
                )
            )
            .add_service(
                node_service_server::NodeServiceServer::with_interceptor(
                    self.node_service,
                    self.auth_interceptor,
                )
            )
            .serve(addr)
            .await?;

        Ok(())
    }
}