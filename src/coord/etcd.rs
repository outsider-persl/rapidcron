use crate::error::{Error, Result};
use etcd_client::{Client, GetOptions, PutOptions};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// 服务注册信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceInfo {
    pub service_name: String,
    pub service_id: String,
    pub host: String,
    pub port: u16,
    pub metadata: Option<String>,
    pub started_at: i64,
    pub last_heartbeat: i64,
}

/// 服务注册和发现
pub struct ServiceRegistry {
    client: Arc<Mutex<Client>>,
    service_prefix: String,
    service_leases: RwLock<HashMap<String, i64>>,
    keepalive_tasks: RwLock<HashMap<String, JoinHandle<()>>>,
}

impl ServiceRegistry {
    /// 创建新的服务注册器
    pub fn new(client: Client, service_prefix: String) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            service_prefix,
            service_leases: RwLock::new(HashMap::new()),
            keepalive_tasks: RwLock::new(HashMap::new()),
        }
    }

    /// 注册服务
    pub async fn register(&self, service: ServiceInfo, lease_ttl_secs: i64) -> Result<i64> {
        let key = format!("{}/{}", self.service_prefix, service.service_name);

        let value = serde_json::to_string(&service).map_err(Error::Serialization)?;

        let lease = self
            .client
            .lock()
            .await
            .lease_grant(lease_ttl_secs, None)
            .await
            .map_err(|e| Error::Etcd(format!("创建 Lease 失败: {}", e)))?;

        let lease_id = lease.id();

        let options = PutOptions::new().with_lease(lease_id);

        self.client
            .lock()
            .await
            .put(key.clone(), value, Some(options))
            .await
            .map_err(|e| Error::Etcd(format!("注册服务失败: {}", e)))?;

        let mut leases = self.service_leases.write().await;
        leases.insert(service.service_name.clone(), lease_id);
        drop(leases);

        info!(
            "[KeepAlive] 服务注册成功: {} ({}) - Lease: {}",
            service.service_name, service.service_id, lease_id
        );

        self.start_keepalive(service.service_name.clone(), lease_id, lease_ttl_secs)
            .await;

        Ok(lease_id)
    }

    /// 启动 KeepAlive 任务
    async fn start_keepalive(&self, service_name: String, lease_id: i64, ttl_secs: i64) {
        let client = Arc::clone(&self.client);
        let keepalive_interval = std::time::Duration::from_secs((ttl_secs / 3).max(1) as u64);
        let service_name_clone = service_name.clone();
        let service_prefix = self.service_prefix.clone();

        let task = tokio::spawn(async move {
            let (mut keeper, mut stream) = {
                let mut client = client.lock().await;
                match client.lease_keep_alive(lease_id).await {
                    Ok(result) => result,
                    Err(e) => {
                        error!(
                            "[KeepAlive] 启动 KeepAlive 失败 (服务: {}, Lease: {}): {}",
                            service_name_clone, lease_id, e
                        );
                        return;
                    }
                }
            };

            let mut ticker = tokio::time::interval(keepalive_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = keeper.keep_alive().await {
                    error!(
                        "[KeepAlive] KeepAlive 失败 (服务: {}, Lease: {}): {}",
                        service_name_clone, lease_id, e
                    );
                    break;
                }

                match stream.message().await {
                    Ok(Some(resp)) => {
                        debug!(
                            "[KeepAlive] 心跳发送成功 (服务: {}, Lease: {}), TTL: {}s",
                            service_name_clone,
                            lease_id,
                            resp.ttl()
                        );

                        let key = format!("{}/{}", service_prefix, service_name_clone);
                        let mut client = client.lock().await;

                        let get_result = client.get(key.clone(), None).await;
                        if let Ok(get_resp) = get_result
                            && let Some(kv) = get_resp.kvs().first()
                            && let Ok(mut service_info) =
                                serde_json::from_slice::<ServiceInfo>(kv.value())
                        {
                            service_info.last_heartbeat = chrono::Utc::now().timestamp();
                            if let Ok(value) = serde_json::to_string(&service_info) {
                                let options = PutOptions::new().with_lease(lease_id);
                                let _ = client.put(key, value, Some(options)).await;
                            }
                        }
                    }
                    Ok(None) => {
                        error!(
                            "[KeepAlive] KeepAlive 流已关闭 (服务: {}, Lease: {}), 服务可能被剔除",
                            service_name_clone, lease_id
                        );
                        break;
                    }
                    Err(e) => {
                        error!(
                            "[KeepAlive] KeepAlive 响应读取失败 (服务: {}, Lease: {}): {}",
                            service_name_clone, lease_id, e
                        );
                        break;
                    }
                }
            }
        });

        let mut tasks = self.keepalive_tasks.write().await;
        tasks.insert(service_name, task);
    }

    /// 注销服务
    pub async fn deregister(&self, service_name: &str) -> Result<()> {
        let key = format!("{}/{}", self.service_prefix, service_name);

        let leases = self.service_leases.read().await;
        let lease_id = leases.get(service_name).copied();
        drop(leases);

        if let Some(lease_id) = lease_id {
            self.client
                .lock()
                .await
                .lease_revoke(lease_id)
                .await
                .map_err(|e| Error::Etcd(format!("撤销 Lease 失败: {}", e)))?;

            info!(
                "[KeepAlive] Lease 已撤销: {} (Lease: {})",
                service_name, lease_id
            );
        }

        self.client
            .lock()
            .await
            .delete(key.clone(), None)
            .await
            .map_err(|e| Error::Etcd(format!("注销服务失败: {}", e)))?;

        let mut leases = self.service_leases.write().await;
        leases.remove(service_name);
        drop(leases);

        let mut tasks = self.keepalive_tasks.write().await;
        if let Some(task) = tasks.remove(service_name) {
            task.abort();
            info!("[KeepAlive] KeepAlive 任务已停止: {}", service_name);
        }
        drop(tasks);

        info!("[KeepAlive] 服务注销成功: {}", service_name);

        Ok(())
    }
}

/// etcd 客户端管理器
pub struct EtcdManager {
    client: Arc<Mutex<Client>>,
    registry: RwLock<ServiceRegistry>,
}

impl EtcdManager {
    /// 尝试连接 etcd，带有重试机制
    async fn connect_with_retry(
        endpoints: Vec<String>,
        max_retries: usize,
        retry_delay: Duration,
    ) -> Result<Client> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match Client::connect(endpoints.clone(), None).await {
                Ok(client) => {
                    info!("[Etcd] etcd 连接成功 (尝试 {} / {})", attempt, max_retries);
                    return Ok(client);
                }
                Err(e) => {
                    error!(
                        "[Etcd] etcd 连接失败 (尝试 {} / {}): {}",
                        attempt, max_retries, e
                    );
                    last_error = Some(e);

                    if attempt < max_retries {
                        warn!("[Etcd] 将在 {:?} 后重试...", retry_delay);
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }

        Err(Error::Etcd(format!(
            "多次尝试后连接 etcd 失败: {:?}",
            last_error
        )))
    }

    /// 创建新的 etcd 管理器（指定前缀）
    pub async fn new_with_prefix(endpoints: Vec<String>, service_prefix: String) -> Result<Self> {
        let client = Self::connect_with_retry(endpoints, 5, Duration::from_secs(2)).await?;

        let registry = ServiceRegistry::new(client.clone(), service_prefix);

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            registry: RwLock::new(registry),
        })
    }

    /// 获取服务注册器
    pub async fn registry(&self) -> tokio::sync::RwLockWriteGuard<'_, ServiceRegistry> {
        self.registry.write().await
    }

    /// 从 etcd 获取所有服务
    pub async fn discover_all_services(&self) -> Result<Vec<ServiceInfo>> {
        let service_prefix = "rapidcron/services".to_string();

        let options = Some(GetOptions::new().with_prefix());

        let mut client = self.client.lock().await;
        let response = client
            .get(service_prefix, options)
            .await
            .map_err(|e| Error::Etcd(format!("获取所有服务失败: {}", e)))?;

        if response.kvs().is_empty() {
            return Ok(Vec::new());
        }

        let mut services = Vec::new();
        for kv in response.kvs() {
            if let Ok(service) = serde_json::from_slice::<ServiceInfo>(kv.value()) {
                services.push(service);
            }
        }

        debug!("[Etcd] 发现所有服务: {} 个实例", services.len());

        Ok(services)
    }
}
