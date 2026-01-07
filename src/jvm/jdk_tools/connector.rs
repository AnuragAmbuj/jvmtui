use crate::error::Result;
use crate::jvm::connector::JvmConnector;
use crate::jvm::jdk_tools::detector::{JdkToolsStatus, ToolStatus};
use crate::jvm::jdk_tools::executor::execute_command;
use crate::jvm::jdk_tools::parsers::{jcmd, jstat};
use crate::jvm::types::{GcStats, HeapInfo, JvmInfo, ThreadInfo};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct JdkToolsConnector {
    pid: Option<u32>,
    tools: JdkToolsStatus,
    jcmd_path: Option<PathBuf>,
    jstat_path: Option<PathBuf>,
    cache: Arc<RwLock<ConnectorCache>>,
}

struct ConnectorCache {
    jvm_info: Option<JvmInfo>,
    vm_flags: Option<Vec<String>>,
}

impl JdkToolsConnector {
    pub fn new() -> Self {
        let tools = JdkToolsStatus::detect();
        let jcmd_path = if let ToolStatus::Available { path, .. } = &tools.jcmd {
            Some(path.clone())
        } else {
            None
        };
        let jstat_path = if let ToolStatus::Available { path, .. } = &tools.jstat {
            Some(path.clone())
        } else {
            None
        };

        Self {
            pid: None,
            tools,
            jcmd_path,
            jstat_path,
            cache: Arc::new(RwLock::new(ConnectorCache {
                jvm_info: None,
                vm_flags: None,
            })),
        }
    }

    async fn execute_jcmd(&self, command: &str) -> Result<String> {
        let pid = self
            .pid
            .ok_or_else(|| crate::error::AppError::Connection("Not connected".to_string()))?;

        let jcmd_path = self
            .jcmd_path
            .as_ref()
            .ok_or_else(|| crate::error::AppError::Connection("jcmd not available".to_string()))?;

        let output = execute_command(
            jcmd_path.to_str().unwrap(),
            &[&pid.to_string(), command],
            None,
        )
        .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn execute_jstat(&self, option: &str) -> Result<String> {
        let pid = self
            .pid
            .ok_or_else(|| crate::error::AppError::Connection("Not connected".to_string()))?;

        let jstat_path = self
            .jstat_path
            .as_ref()
            .ok_or_else(|| crate::error::AppError::Connection("jstat not available".to_string()))?;

        let output = execute_command(
            jstat_path.to_str().unwrap(),
            &[option, &pid.to_string()],
            None,
        )
        .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for JdkToolsConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JvmConnector for JdkToolsConnector {
    async fn connect(&mut self, pid: u32) -> Result<()> {
        self.tools.validate()?;
        self.pid = Some(pid);

        let version_output = self.execute_jcmd("VM.version").await?;
        let version = jcmd::parse_jvm_version(&version_output)
            .map_err(|e| crate::error::AppError::Parse(e))?;

        let uptime_output = self.execute_jcmd("VM.uptime").await?;
        let uptime_seconds =
            jcmd::parse_vm_uptime(&uptime_output).map_err(|e| crate::error::AppError::Parse(e))?;

        let flags_output = self.execute_jcmd("VM.flags").await?;
        let vm_flags =
            jcmd::parse_vm_flags(&flags_output).map_err(|e| crate::error::AppError::Parse(e))?;

        let jvm_info = JvmInfo {
            pid,
            main_class: format!("PID {}", pid),
            version,
            uptime_seconds,
            vm_flags: vm_flags.clone(),
        };

        let mut cache = self.cache.write().await;
        cache.jvm_info = Some(jvm_info);
        cache.vm_flags = Some(vm_flags);

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.pid = None;
        let mut cache = self.cache.write().await;
        cache.jvm_info = None;
        cache.vm_flags = None;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.pid.is_some()
    }

    async fn get_jvm_info(&self) -> Result<JvmInfo> {
        let cache = self.cache.read().await;
        cache
            .jvm_info
            .clone()
            .ok_or_else(|| crate::error::AppError::Connection("Not connected".to_string()))
    }

    async fn get_heap_info(&self) -> Result<HeapInfo> {
        let output = self.execute_jcmd("GC.heap_info").await?;
        jcmd::parse_heap_info(&output).map_err(|e| crate::error::AppError::Parse(e))
    }

    async fn get_gc_stats(&self) -> Result<GcStats> {
        let output = self.execute_jstat("-gcutil").await?;
        jstat::parse_gc_stats(&output).map_err(|e| crate::error::AppError::Parse(e))
    }

    async fn get_thread_info(&self) -> Result<Vec<ThreadInfo>> {
        Ok(vec![])
    }

    async fn trigger_gc(&self) -> Result<()> {
        self.execute_jcmd("GC.run").await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let connector = JdkToolsConnector::new();
        assert!(!connector.is_connected().await);
    }

    #[tokio::test]
    async fn test_connect_to_real_jvm() {
        let mut connector = JdkToolsConnector::new();

        let jvms = crate::jvm::discovery::discover_local_jvms().await.unwrap();
        if jvms.is_empty() {
            println!("No JVMs found, skipping test");
            return;
        }

        let test_pid = jvms[0].pid;
        let result = connector.connect(test_pid).await;
        assert!(result.is_ok(), "Failed to connect: {:?}", result.err());
        assert!(connector.is_connected().await);

        let jvm_info = connector.get_jvm_info().await.unwrap();
        println!("JVM Info: {:#?}", jvm_info);
        assert_eq!(jvm_info.pid, test_pid);

        let heap_info = connector.get_heap_info().await.unwrap();
        println!("Heap Info: {:#?}", heap_info);
        assert!(heap_info.used_bytes > 0);
        assert!(heap_info.max_bytes > 0);

        let gc_stats = connector.get_gc_stats().await.unwrap();
        println!("GC Stats: {:#?}", gc_stats);

        connector.disconnect().await.unwrap();
        assert!(!connector.is_connected().await);
    }
}
