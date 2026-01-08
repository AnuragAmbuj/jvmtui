use crate::error::{AppError, Result};
use crate::jvm::connector::JvmConnector;
use crate::jvm::jolokia::types::{JolokiaRequest, JolokiaResponse};
use crate::jvm::types::{
    ClassInfo, GcStats, HeapInfo, JvmInfo, MemoryPool, PoolType, ThreadInfo, ThreadState,
};
use async_trait::async_trait;
use chrono::Local;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct JolokiaConnector {
    url: String,
    client: Client,
    connected: bool,
    username: Option<String>,
    password: Option<String>,
}

impl JolokiaConnector {
    pub fn new(url: String, username: Option<String>, password: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            url,
            client,
            connected: false,
            username,
            password,
        }
    }

    async fn execute_request(&self, request: JolokiaRequest) -> Result<JolokiaResponse> {
        let mut req_builder = self.client.post(&self.url).json(&request);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            req_builder = req_builder.basic_auth(username, Some(password));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| AppError::Connection(format!("Jolokia HTTP error: {}", e)))?;

        let jolokia_resp: JolokiaResponse = response
            .json()
            .await
            .map_err(|e| AppError::Parse(format!("Failed to parse Jolokia response: {}", e)))?;

        if jolokia_resp.status != 200 {
            return Err(AppError::Connection(format!(
                "Jolokia error: {}",
                jolokia_resp
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        Ok(jolokia_resp)
    }

    async fn read_attribute(&self, mbean: &str, attribute: &str) -> Result<Value> {
        let request = JolokiaRequest::read(mbean, attribute);
        let response = self.execute_request(request).await?;
        Ok(response.value)
    }

    async fn exec_operation(
        &self,
        mbean: &str,
        operation: &str,
        arguments: Vec<Value>,
    ) -> Result<Value> {
        let request = JolokiaRequest::exec(mbean, operation, arguments);
        let response = self.execute_request(request).await?;
        Ok(response.value)
    }
}

#[async_trait]
impl JvmConnector for JolokiaConnector {
    async fn connect(&mut self, _pid: u32) -> Result<()> {
        let request = JolokiaRequest::read("java.lang:type=Runtime", "Name");
        self.execute_request(request).await?;
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn reconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.connect(0).await
    }

    async fn get_jvm_info(&self) -> Result<JvmInfo> {
        let runtime_name = self
            .read_attribute("java.lang:type=Runtime", "Name")
            .await?;
        let vm_version = self
            .read_attribute("java.lang:type=Runtime", "VmVersion")
            .await?;
        let uptime_ms = self
            .read_attribute("java.lang:type=Runtime", "Uptime")
            .await?;

        let runtime_str = runtime_name.as_str().unwrap_or("");
        let pid = runtime_str
            .split('@')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        Ok(JvmInfo {
            pid,
            main_class: "Remote JVM".to_string(),
            version: vm_version.as_str().unwrap_or("Unknown").to_string(),
            uptime_seconds: uptime_ms.as_u64().unwrap_or(0) / 1000,
            vm_flags: vec![],
        })
    }

    async fn get_heap_info(&self) -> Result<HeapInfo> {
        let heap_mem = self
            .read_attribute("java.lang:type=Memory", "HeapMemoryUsage")
            .await?;

        let used = heap_mem["used"].as_u64().unwrap_or(0);
        let max = heap_mem["max"].as_u64().unwrap_or(0);
        let committed = heap_mem["committed"].as_u64().unwrap_or(0);

        let pools = vec![MemoryPool {
            name: "Remote Heap".to_string(),
            pool_type: PoolType::Old,
            used_bytes: used,
            max_bytes: max,
            committed_bytes: committed,
        }];

        Ok(HeapInfo {
            used_bytes: used,
            max_bytes: max,
            committed_bytes: committed,
            pools,
        })
    }

    async fn get_gc_stats(&self) -> Result<GcStats> {
        let young_count = self
            .read_attribute("java.lang:type=GarbageCollector,name=*", "CollectionCount")
            .await
            .unwrap_or(Value::from(0))
            .as_u64()
            .unwrap_or(0);

        let young_time = self
            .read_attribute("java.lang:type=GarbageCollector,name=*", "CollectionTime")
            .await
            .unwrap_or(Value::from(0))
            .as_u64()
            .unwrap_or(0);

        Ok(GcStats {
            young_gc_count: young_count,
            young_gc_time_ms: young_time,
            old_gc_count: 0,
            old_gc_time_ms: 0,
            timestamp: Local::now(),
        })
    }

    async fn get_thread_info(&self) -> Result<Vec<ThreadInfo>> {
        let thread_count = self
            .read_attribute("java.lang:type=Threading", "ThreadCount")
            .await?
            .as_u64()
            .unwrap_or(0);

        let threads = (0..thread_count.min(50))
            .map(|i| ThreadInfo {
                id: i,
                name: format!("Thread-{}", i),
                state: ThreadState::Runnable,
                stack_trace: vec![],
            })
            .collect();

        Ok(threads)
    }

    async fn get_class_histogram(&self) -> Result<Vec<ClassInfo>> {
        let loaded_classes = self
            .read_attribute("java.lang:type=ClassLoading", "LoadedClassCount")
            .await?
            .as_u64()
            .unwrap_or(0);

        Ok(vec![ClassInfo {
            rank: 1,
            instances: loaded_classes,
            bytes: 0,
            name: "Classes (remote)".to_string(),
        }])
    }

    async fn trigger_gc(&self) -> Result<()> {
        self.exec_operation("java.lang:type=Memory", "gc", vec![])
            .await?;
        Ok(())
    }
}
