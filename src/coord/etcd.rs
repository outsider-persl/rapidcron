use crate::error::{Error, Result};
use etcd_client::Client;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 服务注册信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceInfo {
    pub service_name: String,
    pub service_id: String,
    pub host: String,
    pub port: u16,
    pub metadata: Option<String>,
}

/// 服务注册和发现
pub struct ServiceRegistry {
    client: Client,
    service_prefix: String,
    registered_services: RwLock<Vec<ServiceInfo>>,
}

impl ServiceRegistry {
    /// 创建新的服务注册器
    pub fn new(client: Client, service_prefix: String) -> Self {
        Self {
            client,
            service_prefix,
            registered_services: RwLock::new(Vec::new()),
        }
    }

    /// 注册服务
    pub async fn register(&mut self, service: ServiceInfo) -> Result<()> {
        let key = format!("{}/{}", self.service_prefix, service.service_name);

        let value = serde_json::to_string(&service).map_err(|e| Error::Serialization(e))?;

        self.client
            .put(key.clone(), value, None)
            .await
            .map_err(|e| Error::Etcd(format!("注册服务失败: {}", e)))?;

        let mut services = self.registered_services.write().await;
        services.push(service.clone());
        drop(services);

        info!(
            "成功注册服务: {} ({})",
            service.service_name, service.service_id
        );

        Ok(())
    }

    /// 注销服务
    pub async fn deregister(&mut self, service_name: &str) -> Result<()> {
        let key = format!("{}/{}", self.service_prefix, service_name);

        self.client
            .delete(key.clone(), None)
            .await
            .map_err(|e| Error::Etcd(format!("注销服务失败: {}", e)))?;

        let mut services = self.registered_services.write().await;
        services.retain(|s| s.service_name != service_name);
        drop(services);

        info!("成功注销服务: {}", service_name);

        Ok(())
    }

    /// 发现服务
    pub async fn discover(&mut self, service_name: &str) -> Result<Vec<ServiceInfo>> {
        let key = format!("{}/{}", self.service_prefix, service_name);

        let response = self
            .client
            .get(key.clone(), None)
            .await
            .map_err(|e| Error::Etcd(format!("发现服务失败: {}", e)))?;

        if response.kvs().is_empty() {
            warn!("未找到服务: {}", service_name);
            return Ok(Vec::new());
        }

        let mut services = Vec::new();
        for kv in response.kvs() {
            if let Ok(service) = serde_json::from_slice::<ServiceInfo>(kv.value()) {
                services.push(service);
            }
        }

        debug!("发现服务 {}: {} 个实例", service_name, services.len());

        Ok(services)
    }

    /// 刷新服务注册（心跳）
    pub async fn refresh(&mut self, service: &ServiceInfo) -> Result<()> {
        let key = format!("{}/{}", self.service_prefix, service.service_name);

        let value = serde_json::to_string(service).map_err(|e| Error::Serialization(e))?;

        self.client
            .put(key.clone(), value, None)
            .await
            .map_err(|e| Error::Etcd(format!("刷新服务失败: {}", e)))?;

        debug!("刷新服务注册: {}", service.service_name);

        Ok(())
    }

    /// 获取所有已注册的服务
    pub async fn get_registered_services(&self) -> Vec<ServiceInfo> {
        self.registered_services.read().await.clone()
    }
}

/// etcd 客户端管理器
pub struct EtcdManager {
    client: Client,
    registry: ServiceRegistry,
}

impl EtcdManager {
    /// 创建新的 etcd 管理器
    pub async fn new(endpoints: Vec<String>) -> Result<Self> {
        let client = Client::connect(endpoints, None)
            .await
            .map_err(|e| Error::Etcd(format!("连接 etcd 失败: {}", e)))?;

        let registry = ServiceRegistry::new(client.clone(), "rapidcron/services".to_string());

        info!("成功连接到 etcd");

        Ok(Self { client, registry })
    }

    /// 获取服务注册器
    pub fn registry(&mut self) -> &mut ServiceRegistry {
        &mut self.registry
    }

    /// 健康检查
    pub async fn health_check(&mut self) -> Result<bool> {
        let _ = self
            .client
            .get("health".to_string(), None)
            .await
            .map_err(|e| Error::Etcd(format!("健康检查失败: {}", e)))?;
        Ok(true)
    }
}
