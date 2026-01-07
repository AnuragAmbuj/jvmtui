#[cfg(feature = "jolokia")]
use crate::error::Result;
#[cfg(feature = "jolokia")]
use crate::jvm::connector::JvmConnector;
#[cfg(feature = "jolokia")]
use crate::jvm::types::{GcStats, HeapInfo, JvmInfo, ThreadInfo};
#[cfg(feature = "jolokia")]
use async_trait::async_trait;

#[cfg(feature = "jolokia")]
pub struct JolokiaConnector {
    url: String,
}

#[cfg(feature = "jolokia")]
impl JolokiaConnector {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[cfg(feature = "jolokia")]
#[async_trait]
impl JvmConnector for JolokiaConnector {
    async fn connect(&mut self, _pid: u32) -> Result<()> {
        todo!()
    }

    async fn disconnect(&mut self) -> Result<()> {
        todo!()
    }

    async fn is_connected(&self) -> bool {
        todo!()
    }

    async fn get_jvm_info(&self) -> Result<JvmInfo> {
        todo!()
    }

    async fn get_heap_info(&self) -> Result<HeapInfo> {
        todo!()
    }

    async fn get_gc_stats(&self) -> Result<GcStats> {
        todo!()
    }

    async fn get_thread_info(&self) -> Result<Vec<ThreadInfo>> {
        todo!()
    }

    async fn trigger_gc(&self) -> Result<()> {
        todo!()
    }
}
