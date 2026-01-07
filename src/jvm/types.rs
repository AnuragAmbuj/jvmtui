use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmInfo {
    pub pid: u32,
    pub main_class: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub vm_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapInfo {
    pub used_bytes: u64,
    pub max_bytes: u64,
    pub committed_bytes: u64,
    pub pools: Vec<MemoryPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPool {
    pub name: String,
    pub pool_type: PoolType,
    pub used_bytes: u64,
    pub max_bytes: u64,
    pub committed_bytes: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoolType {
    Eden,
    Survivor,
    Old,
    Metaspace,
    CodeCache,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcStats {
    pub young_gc_count: u64,
    pub young_gc_time_ms: u64,
    pub old_gc_count: u64,
    pub old_gc_time_ms: u64,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub id: u64,
    pub name: String,
    pub state: ThreadState,
    pub stack_trace: Vec<StackFrame>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThreadState {
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub class_name: String,
    pub method_name: String,
    pub file_name: Option<String>,
    pub line_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub rank: u32,
    pub instances: u64,
    pub bytes: u64,
    pub name: String,
}
