use crate::error::{AppError, Result};
use crate::jvm::connector::JvmConnector;
use crate::jvm::jdk_tools::parsers::{jcmd, jstat};
use crate::jvm::types::{ClassInfo, GcStats, HeapInfo, JvmInfo, ThreadInfo};
use async_ssh2_tokio::{client::AuthMethod, Client, ServerCheckMethod};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct SshJdkConnector {
    host: String,
    port: u16,
    user: String,
    auth_method: AuthMethod,
    pid: u32,
    client: Option<Client>,
}

impl SshJdkConnector {
    pub fn new(
        host: String,
        port: u16,
        user: String,
        key_path: Option<String>,
        password: Option<String>,
        pid: u32,
    ) -> Self {
        let auth_method = if let Some(key) = key_path {
            if let Some(pwd) = password {
                AuthMethod::with_key_file(PathBuf::from(key), Some(&pwd))
            } else {
                AuthMethod::with_key_file(PathBuf::from(key), None)
            }
        } else if let Some(pwd) = password {
            AuthMethod::with_password(&pwd)
        } else {
            AuthMethod::with_key_file(
                PathBuf::from(format!(
                    "{}/.ssh/id_rsa",
                    std::env::var("HOME").unwrap_or_default()
                )),
                None,
            )
        };

        Self {
            host,
            port,
            user,
            auth_method,
            pid,
            client: None,
        }
    }

    async fn execute_command(&self, command: &str) -> Result<String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| AppError::Connection("Not connected".to_string()))?;

        let result = client
            .execute(command)
            .await
            .map_err(|e| AppError::Connection(format!("SSH command failed: {}", e)))?;

        Ok(result.stdout)
    }
}

#[async_trait]
impl JvmConnector for SshJdkConnector {
    async fn connect(&mut self, _pid: u32) -> Result<()> {
        let client = Client::connect(
            (self.host.clone(), self.port),
            &self.user,
            self.auth_method.clone(),
            ServerCheckMethod::NoCheck,
        )
        .await
        .map_err(|e| AppError::Connection(format!("SSH connection failed: {}", e)))?;

        self.client = Some(client);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.client = None;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    async fn reconnect(&mut self) -> Result<()> {
        self.disconnect().await?;
        self.connect(self.pid).await
    }

    async fn get_jvm_info(&self) -> Result<JvmInfo> {
        let vm_version_output = self
            .execute_command(&format!("jcmd {} VM.version", self.pid))
            .await?;
        let uptime_output = self
            .execute_command(&format!("jcmd {} VM.uptime", self.pid))
            .await?;
        let flags_output = self
            .execute_command(&format!("jcmd {} VM.flags", self.pid))
            .await?;

        let version = jcmd::parse_jvm_version(&vm_version_output)
            .map_err(|e| AppError::Parse(format!("Failed to parse VM version: {}", e)))?;
        let uptime_seconds = jcmd::parse_vm_uptime(&uptime_output)
            .map_err(|e| AppError::Parse(format!("Failed to parse uptime: {}", e)))?;
        let vm_flags = jcmd::parse_vm_flags(&flags_output)
            .map_err(|e| AppError::Parse(format!("Failed to parse VM flags: {}", e)))?;

        Ok(JvmInfo {
            pid: self.pid,
            main_class: format!("Remote JVM ({})", self.host),
            version,
            uptime_seconds,
            vm_flags,
        })
    }

    async fn get_heap_info(&self) -> Result<HeapInfo> {
        let output = self
            .execute_command(&format!("jcmd {} GC.heap_info", self.pid))
            .await?;

        jcmd::parse_heap_info(&output)
            .map_err(|e| AppError::Parse(format!("Failed to parse heap info: {}", e)))
    }

    async fn get_gc_stats(&self) -> Result<GcStats> {
        let output = self
            .execute_command(&format!("jstat -gc {}", self.pid))
            .await?;

        jstat::parse_gc_stats(&output)
            .map_err(|e| AppError::Parse(format!("Failed to parse GC stats: {}", e)))
    }

    async fn get_thread_info(&self) -> Result<Vec<ThreadInfo>> {
        let output = self
            .execute_command(&format!("jcmd {} Thread.print", self.pid))
            .await?;

        jcmd::parse_thread_dump(&output)
            .map_err(|e| AppError::Parse(format!("Failed to parse thread dump: {}", e)))
    }

    async fn get_class_histogram(&self) -> Result<Vec<ClassInfo>> {
        let output = self
            .execute_command(&format!("jcmd {} GC.class_histogram", self.pid))
            .await?;

        jcmd::parse_class_histogram(&output)
            .map_err(|e| AppError::Parse(format!("Failed to parse class histogram: {}", e)))
    }

    async fn trigger_gc(&self) -> Result<()> {
        self.execute_command(&format!("jcmd {} GC.run", self.pid))
            .await?;
        Ok(())
    }
}
