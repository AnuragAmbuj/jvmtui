use crate::error::Result;
use crate::jvm::types::{ClassInfo, GcStats, HeapInfo, JvmInfo, ThreadInfo};
use async_trait::async_trait;

#[async_trait]
pub trait JvmConnector: Send + Sync {
    async fn connect(&mut self, pid: u32) -> Result<()>;

    async fn disconnect(&mut self) -> Result<()>;

    async fn is_connected(&self) -> bool;

    async fn reconnect(&mut self) -> Result<()>;

    async fn get_jvm_info(&self) -> Result<JvmInfo>;

    async fn get_heap_info(&self) -> Result<HeapInfo>;

    async fn get_gc_stats(&self) -> Result<GcStats>;

    async fn get_thread_info(&self) -> Result<Vec<ThreadInfo>>;

    async fn get_class_histogram(&self) -> Result<Vec<ClassInfo>>;

    async fn trigger_gc(&self) -> Result<()>;
}
