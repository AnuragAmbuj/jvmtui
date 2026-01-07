# Connector Trait

This document describes the `JvmConnector` trait that abstracts JVM communication.

## Design Goals

1. **Abstraction**: Hide implementation details (jcmd vs Jolokia)
2. **Async**: All operations are async for non-blocking I/O
3. **Extensible**: Easy to add new connector types
4. **Testable**: Enable mocking for unit tests

## Trait Definition

```rust
// src/jvm/connector.rs

use async_trait::async_trait;
use crate::jvm::types::*;
use crate::error::Result;

/// Abstraction for JVM communication
/// 
/// Implementations:
/// - `JdkToolsConnector`: Uses jcmd/jstat subprocess (default, agentless)
/// - `JolokiaConnector`: Uses HTTP/JSON to Jolokia agent (optional)
#[async_trait]
pub trait JvmConnector: Send + Sync {
    /// Get the PID of the connected JVM
    fn pid(&self) -> u32;
    
    /// Check if the JVM is still running and accessible
    async fn is_alive(&self) -> bool;
    
    // ─────────────────────────────────────────────────────────────────
    // Static Information (cached after first call)
    // ─────────────────────────────────────────────────────────────────
    
    /// Get JVM version information
    async fn get_vm_version(&self) -> Result<VmVersion>;
    
    /// Get JVM flags (-XX options)
    async fn get_vm_flags(&self) -> Result<VmFlags>;
    
    /// Get system properties
    async fn get_system_properties(&self) -> Result<SystemProperties>;
    
    // ─────────────────────────────────────────────────────────────────
    // Dynamic Metrics (polled frequently)
    // ─────────────────────────────────────────────────────────────────
    
    /// Get JVM uptime in seconds
    async fn get_uptime(&self) -> Result<f64>;
    
    /// Get heap memory information
    async fn get_heap_info(&self) -> Result<HeapInfo>;
    
    /// Get GC statistics
    async fn get_gc_stats(&self) -> Result<GcStats>;
    
    /// Get thread summary (counts, states)
    async fn get_thread_summary(&self) -> Result<ThreadSummary>;
    
    /// Get class loading statistics
    async fn get_class_stats(&self) -> Result<ClassStats>;
    
    // ─────────────────────────────────────────────────────────────────
    // On-Demand Operations (user-triggered)
    // ─────────────────────────────────────────────────────────────────
    
    /// Get full thread dump with stack traces
    async fn get_thread_dump(&self) -> Result<ThreadDump>;
    
    /// Get class histogram (top memory consumers)
    async fn get_class_histogram(&self) -> Result<ClassHistogram>;
    
    /// Get comprehensive VM info
    async fn get_vm_info(&self) -> Result<VmInfo>;
    
    // ─────────────────────────────────────────────────────────────────
    // Actions
    // ─────────────────────────────────────────────────────────────────
    
    /// Trigger garbage collection
    async fn trigger_gc(&self) -> Result<()>;
}
```

## Type Definitions

```rust
// src/jvm/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────
// Static Information Types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmVersion {
    pub vm_name: String,
    pub vm_version: String,
    pub jdk_version: String,
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmFlags {
    pub flags: Vec<String>,
    pub gc_type: GcType,
    pub max_heap_mb: u64,
    pub initial_heap_mb: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GcType {
    G1,
    ZGC,
    Shenandoah,
    Parallel,
    Serial,
    CMS,  // deprecated but still exists
    Unknown,
}

impl GcType {
    pub fn from_flags(flags: &[String]) -> Self {
        for flag in flags {
            if flag.contains("UseG1GC") {
                return Self::G1;
            }
            if flag.contains("UseZGC") {
                return Self::ZGC;
            }
            if flag.contains("UseShenandoahGC") {
                return Self::Shenandoah;
            }
            if flag.contains("UseParallelGC") {
                return Self::Parallel;
            }
            if flag.contains("UseSerialGC") {
                return Self::Serial;
            }
        }
        Self::Unknown
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::G1 => "G1GC",
            Self::ZGC => "ZGC",
            Self::Shenandoah => "Shenandoah",
            Self::Parallel => "Parallel GC",
            Self::Serial => "Serial GC",
            Self::CMS => "CMS (deprecated)",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemProperties {
    pub properties: HashMap<String, String>,
}

// ─────────────────────────────────────────────────────────────────
// Dynamic Metrics Types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeapInfo {
    pub gc_type: String,
    pub total_kb: u64,
    pub used_kb: u64,
    pub committed_kb: Option<u64>,
    pub max_kb: Option<u64>,
    pub region_size_kb: Option<u64>,
    pub young_regions: Option<u32>,
    pub survivor_regions: Option<u32>,
    pub metaspace_used_kb: u64,
    pub metaspace_committed_kb: u64,
    pub metaspace_reserved_kb: u64,
    pub class_space_used_kb: u64,
    pub class_space_committed_kb: u64,
}

impl HeapInfo {
    pub fn heap_used_mb(&self) -> f64 {
        self.used_kb as f64 / 1024.0
    }
    
    pub fn heap_total_mb(&self) -> f64 {
        self.total_kb as f64 / 1024.0
    }
    
    pub fn heap_used_pct(&self) -> f64 {
        if self.total_kb == 0 {
            0.0
        } else {
            (self.used_kb as f64 / self.total_kb as f64) * 100.0
        }
    }
    
    pub fn metaspace_used_mb(&self) -> f64 {
        self.metaspace_used_kb as f64 / 1024.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GcStats {
    // Percentages
    pub eden_pct: f64,
    pub survivor0_pct: f64,
    pub survivor1_pct: f64,
    pub old_pct: f64,
    pub metaspace_pct: f64,
    
    // Counts
    pub young_gc_count: u64,
    pub full_gc_count: u64,
    pub concurrent_gc_count: u64,
    
    // Times
    pub young_gc_time_sec: f64,
    pub full_gc_time_sec: f64,
    pub concurrent_gc_time_sec: f64,
    pub total_gc_time_sec: f64,
}

impl GcStats {
    pub fn young_gc_avg_ms(&self) -> f64 {
        if self.young_gc_count == 0 {
            0.0
        } else {
            (self.young_gc_time_sec / self.young_gc_count as f64) * 1000.0
        }
    }
    
    pub fn full_gc_avg_ms(&self) -> f64 {
        if self.full_gc_count == 0 {
            0.0
        } else {
            (self.full_gc_time_sec / self.full_gc_count as f64) * 1000.0
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadSummary {
    pub total_count: u32,
    pub daemon_count: u32,
    pub peak_count: u32,
    pub by_state: HashMap<ThreadState, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreadState {
    New,
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassStats {
    pub loaded_count: u64,
    pub unloaded_count: u64,
    pub total_loaded_count: u64,
}

// ─────────────────────────────────────────────────────────────────
// On-Demand Types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDump {
    pub timestamp: String,
    pub threads: Vec<ThreadInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub name: String,
    pub id: u64,
    pub daemon: bool,
    pub priority: u8,
    pub state: ThreadState,
    pub state_detail: Option<String>,
    pub cpu_time_ms: Option<f64>,
    pub stack_trace: Vec<String>,
}

impl ThreadInfo {
    pub fn stack_preview(&self, max_lines: usize) -> String {
        self.stack_trace
            .iter()
            .take(max_lines)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassHistogram {
    pub entries: Vec<ClassHistogramEntry>,
    pub total_instances: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassHistogramEntry {
    pub rank: u32,
    pub instances: u64,
    pub bytes: u64,
    pub class_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub raw_output: String,
    // Parsed sections can be added as needed
}
```

## JdkToolsConnector Implementation

```rust
// src/jvm/jdk_tools/connector.rs

use async_trait::async_trait;
use crate::jvm::connector::JvmConnector;
use crate::jvm::jdk_tools::executor::JdkExecutor;
use crate::jvm::jdk_tools::parsers::{jcmd, jstat};
use crate::jvm::types::*;
use crate::error::Result;
use tokio::sync::RwLock;

pub struct JdkToolsConnector {
    pid: u32,
    executor: JdkExecutor,
    // Cached static info
    vm_version: RwLock<Option<VmVersion>>,
    vm_flags: RwLock<Option<VmFlags>>,
}

impl JdkToolsConnector {
    pub fn new(pid: u32, executor: JdkExecutor) -> Self {
        Self {
            pid,
            executor,
            vm_version: RwLock::new(None),
            vm_flags: RwLock::new(None),
        }
    }
}

#[async_trait]
impl JvmConnector for JdkToolsConnector {
    fn pid(&self) -> u32 {
        self.pid
    }
    
    async fn is_alive(&self) -> bool {
        // Quick check: try to get uptime
        self.get_uptime().await.is_ok()
    }
    
    async fn get_vm_version(&self) -> Result<VmVersion> {
        // Return cached if available
        {
            let cached = self.vm_version.read().await;
            if let Some(ref version) = *cached {
                return Ok(version.clone());
            }
        }
        
        // Fetch and cache
        let output = self.executor.jcmd(self.pid, "VM.version").await?;
        let version = jcmd::parse_vm_version(&output)?;
        
        {
            let mut cached = self.vm_version.write().await;
            *cached = Some(version.clone());
        }
        
        Ok(version)
    }
    
    async fn get_vm_flags(&self) -> Result<VmFlags> {
        // Return cached if available
        {
            let cached = self.vm_flags.read().await;
            if let Some(ref flags) = *cached {
                return Ok(flags.clone());
            }
        }
        
        let output = self.executor.jcmd(self.pid, "VM.flags").await?;
        let flags = jcmd::parse_vm_flags(&output)?;
        
        {
            let mut cached = self.vm_flags.write().await;
            *cached = Some(flags.clone());
        }
        
        Ok(flags)
    }
    
    async fn get_system_properties(&self) -> Result<SystemProperties> {
        let output = self.executor.jcmd(self.pid, "VM.system_properties").await?;
        jcmd::parse_system_properties(&output)
    }
    
    async fn get_uptime(&self) -> Result<f64> {
        let output = self.executor.jcmd(self.pid, "VM.uptime").await?;
        jcmd::parse_vm_uptime(&output)
    }
    
    async fn get_heap_info(&self) -> Result<HeapInfo> {
        let output = self.executor.jcmd(self.pid, "GC.heap_info").await?;
        jcmd::parse_gc_heap_info(&output)
    }
    
    async fn get_gc_stats(&self) -> Result<GcStats> {
        let output = self.executor.jstat("-gcutil", self.pid).await?;
        let util = jstat::parse_gcutil(&output)?;
        
        Ok(GcStats {
            eden_pct: util.eden_pct,
            survivor0_pct: util.survivor0_pct,
            survivor1_pct: util.survivor1_pct,
            old_pct: util.old_pct,
            metaspace_pct: util.metaspace_pct,
            young_gc_count: util.young_gc_count,
            full_gc_count: util.full_gc_count,
            concurrent_gc_count: util.concurrent_gc_count,
            young_gc_time_sec: util.young_gc_time_sec,
            full_gc_time_sec: util.full_gc_time_sec,
            concurrent_gc_time_sec: util.concurrent_gc_time_sec,
            total_gc_time_sec: util.total_gc_time_sec,
        })
    }
    
    async fn get_thread_summary(&self) -> Result<ThreadSummary> {
        // Parse thread dump but only extract summary
        let dump = self.get_thread_dump().await?;
        
        let mut by_state = std::collections::HashMap::new();
        let mut daemon_count = 0;
        
        for thread in &dump.threads {
            *by_state.entry(thread.state).or_insert(0) += 1;
            if thread.daemon {
                daemon_count += 1;
            }
        }
        
        Ok(ThreadSummary {
            total_count: dump.threads.len() as u32,
            daemon_count,
            peak_count: dump.threads.len() as u32, // Not available via jcmd
            by_state,
        })
    }
    
    async fn get_class_stats(&self) -> Result<ClassStats> {
        let output = self.executor.jstat("-class", self.pid).await?;
        jstat::parse_class(&output)
    }
    
    async fn get_thread_dump(&self) -> Result<ThreadDump> {
        let output = self.executor.jcmd(self.pid, "Thread.print").await?;
        jcmd::parse_thread_dump(&output)
    }
    
    async fn get_class_histogram(&self) -> Result<ClassHistogram> {
        let output = self.executor.jcmd(self.pid, "GC.class_histogram").await?;
        jcmd::parse_class_histogram(&output)
    }
    
    async fn get_vm_info(&self) -> Result<VmInfo> {
        let output = self.executor.jcmd(self.pid, "VM.info").await?;
        Ok(VmInfo { raw_output: output })
    }
    
    async fn trigger_gc(&self) -> Result<()> {
        self.executor.jcmd(self.pid, "GC.run").await?;
        Ok(())
    }
}
```

## Connector Factory

```rust
// src/jvm/connector.rs

use crate::jvm::jdk_tools::{JdkToolsConnector, JdkToolsStatus, JdkExecutor};

#[cfg(feature = "jolokia")]
use crate::jvm::jolokia::JolokiaConnector;

pub enum ConnectorConfig {
    /// Connect to local JVM via jcmd/jstat
    Local { pid: u32 },
    
    /// Connect via Jolokia HTTP
    #[cfg(feature = "jolokia")]
    Jolokia { url: String },
}

pub async fn create_connector(config: ConnectorConfig) -> Result<Box<dyn JvmConnector>> {
    match config {
        ConnectorConfig::Local { pid } => {
            let status = JdkToolsStatus::detect().await;
            let executor = JdkExecutor::new(&status)?;
            Ok(Box::new(JdkToolsConnector::new(pid, executor)))
        }
        
        #[cfg(feature = "jolokia")]
        ConnectorConfig::Jolokia { url } => {
            Ok(Box::new(JolokiaConnector::new(url)))
        }
    }
}
```

## Testing with Mocks

```rust
// src/jvm/connector.rs

#[cfg(test)]
pub mod mock {
    use super::*;
    
    pub struct MockConnector {
        pub pid: u32,
        pub heap_info: HeapInfo,
        pub gc_stats: GcStats,
        pub alive: bool,
    }
    
    #[async_trait]
    impl JvmConnector for MockConnector {
        fn pid(&self) -> u32 {
            self.pid
        }
        
        async fn is_alive(&self) -> bool {
            self.alive
        }
        
        async fn get_heap_info(&self) -> Result<HeapInfo> {
            Ok(self.heap_info.clone())
        }
        
        async fn get_gc_stats(&self) -> Result<GcStats> {
            Ok(self.gc_stats.clone())
        }
        
        // ... other methods return defaults
    }
}
```
