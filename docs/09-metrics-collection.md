# Metrics Collection

This document describes the async metrics collection system in JVM-TUI.

## Overview

The metrics collection system:
1. Polls JVM metrics at configurable intervals
2. Stores historical data in ring buffers
3. Notifies the UI when new data is available
4. Handles disconnection gracefully

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Metrics Collection Flow                      │
└─────────────────────────────────────────────────────────────────┘

  ┌─────────────┐                              ┌─────────────────┐
  │   Ticker    │──── tick ────────────────────▶│    Collector   │
  │ (interval)  │                              │     Task       │
  └─────────────┘                              └────────┬────────┘
                                                        │
                                                        │ collect
                                                        ▼
                                               ┌─────────────────┐
                                               │   Connector     │
                                               │ (jcmd/jstat)    │
                                               └────────┬────────┘
                                                        │
                                                        │ metrics
                                                        ▼
  ┌─────────────┐                              ┌─────────────────┐
  │     TUI     │◀──── MetricsEvent ───────────│  MetricsStore  │
  │   (render)  │                              │ (ring buffers) │
  └─────────────┘                              └─────────────────┘
```

## Configuration

```rust
// src/config.rs

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PollingConfig {
    /// Interval between metric collections
    /// Default: 1 second
    /// Range: 250ms - 10s
    pub interval: Duration,
    
    /// Number of historical samples to keep
    /// Default: 300 (5 minutes at 1s interval)
    pub history_size: usize,
    
    /// Timeout for individual commands
    /// Default: 5 seconds
    pub command_timeout: Duration,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),
            history_size: 300,
            command_timeout: Duration::from_secs(5),
        }
    }
}

impl PollingConfig {
    pub fn with_interval(mut self, interval: Duration) -> Self {
        // Clamp to valid range
        self.interval = interval.clamp(
            Duration::from_millis(250),
            Duration::from_secs(10),
        );
        self
    }
}
```

## Ring Buffer

```rust
// src/metrics/ring_buffer.rs

use std::collections::VecDeque;

/// Fixed-size circular buffer for time-series data
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Push a new value, removing oldest if at capacity
    pub fn push(&mut self, value: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }
    
    /// Get all values as a slice (oldest to newest)
    pub fn as_slice(&self) -> Vec<&T> {
        self.data.iter().collect()
    }
    
    /// Get the most recent value
    pub fn latest(&self) -> Option<&T> {
        self.data.back()
    }
    
    /// Get values as owned Vec (for sparkline data)
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.data.iter().cloned().collect()
    }
    
    /// Number of samples currently stored
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

// Specialized for u64 values (sparkline data)
impl RingBuffer<u64> {
    /// Get data suitable for Sparkline widget
    pub fn sparkline_data(&self) -> Vec<u64> {
        self.to_vec()
    }
}
```

## Metrics Store

```rust
// src/metrics/store.rs

use crate::jvm::types::*;
use crate::metrics::ring_buffer::RingBuffer;
use std::time::Instant;

/// Timestamped metric sample
#[derive(Debug, Clone)]
pub struct Sample<T> {
    pub value: T,
    pub timestamp: Instant,
}

impl<T> Sample<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }
}

/// Central storage for all JVM metrics
#[derive(Debug)]
pub struct MetricsStore {
    // Configuration
    history_size: usize,
    
    // Static info (fetched once)
    pub vm_version: Option<VmVersion>,
    pub vm_flags: Option<VmFlags>,
    
    // Time-series data
    pub heap_used_kb: RingBuffer<u64>,
    pub heap_total_kb: RingBuffer<u64>,
    pub metaspace_used_kb: RingBuffer<u64>,
    pub gc_time_sec: RingBuffer<f64>,
    
    // Latest samples
    pub heap_info: Option<Sample<HeapInfo>>,
    pub gc_stats: Option<Sample<GcStats>>,
    pub thread_summary: Option<Sample<ThreadSummary>>,
    pub class_stats: Option<Sample<ClassStats>>,
    
    // On-demand data
    pub thread_dump: Option<Sample<ThreadDump>>,
    pub class_histogram: Option<Sample<ClassHistogram>>,
    
    // Connection state
    pub uptime_sec: f64,
    pub last_update: Option<Instant>,
    pub consecutive_errors: u32,
}

impl MetricsStore {
    pub fn new(history_size: usize) -> Self {
        Self {
            history_size,
            vm_version: None,
            vm_flags: None,
            heap_used_kb: RingBuffer::new(history_size),
            heap_total_kb: RingBuffer::new(history_size),
            metaspace_used_kb: RingBuffer::new(history_size),
            gc_time_sec: RingBuffer::new(history_size),
            heap_info: None,
            gc_stats: None,
            thread_summary: None,
            class_stats: None,
            thread_dump: None,
            class_histogram: None,
            uptime_sec: 0.0,
            last_update: None,
            consecutive_errors: 0,
        }
    }
    
    /// Update with new heap info
    pub fn push_heap_info(&mut self, info: HeapInfo) {
        self.heap_used_kb.push(info.used_kb);
        self.heap_total_kb.push(info.total_kb);
        self.metaspace_used_kb.push(info.metaspace_used_kb);
        self.heap_info = Some(Sample::new(info));
        self.mark_updated();
    }
    
    /// Update with new GC stats
    pub fn push_gc_stats(&mut self, stats: GcStats) {
        self.gc_time_sec.push(stats.total_gc_time_sec);
        self.gc_stats = Some(Sample::new(stats));
        self.mark_updated();
    }
    
    /// Update thread summary
    pub fn update_thread_summary(&mut self, summary: ThreadSummary) {
        self.thread_summary = Some(Sample::new(summary));
        self.mark_updated();
    }
    
    /// Update class stats
    pub fn update_class_stats(&mut self, stats: ClassStats) {
        self.class_stats = Some(Sample::new(stats));
        self.mark_updated();
    }
    
    /// Update uptime
    pub fn update_uptime(&mut self, uptime: f64) {
        self.uptime_sec = uptime;
        self.mark_updated();
    }
    
    /// Store thread dump (on-demand)
    pub fn store_thread_dump(&mut self, dump: ThreadDump) {
        self.thread_dump = Some(Sample::new(dump));
    }
    
    /// Store class histogram (on-demand)
    pub fn store_class_histogram(&mut self, histogram: ClassHistogram) {
        self.class_histogram = Some(Sample::new(histogram));
    }
    
    /// Record successful update
    fn mark_updated(&mut self) {
        self.last_update = Some(Instant::now());
        self.consecutive_errors = 0;
    }
    
    /// Record error
    pub fn record_error(&mut self) {
        self.consecutive_errors += 1;
    }
    
    /// Check if data is stale
    pub fn is_stale(&self, threshold: std::time::Duration) -> bool {
        self.last_update
            .map(|t| t.elapsed() > threshold)
            .unwrap_or(true)
    }
    
    /// Get heap usage percentage history for sparkline
    pub fn heap_usage_pct_history(&self) -> Vec<u64> {
        let used = self.heap_used_kb.to_vec();
        let total = self.heap_total_kb.to_vec();
        
        used.iter()
            .zip(total.iter())
            .map(|(u, t)| {
                if *t == 0 { 0 } else { (u * 100 / t) }
            })
            .collect()
    }
    
    /// Format uptime as human-readable string
    pub fn uptime_display(&self) -> String {
        let secs = self.uptime_sec as u64;
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }
}
```

## Metrics Collector

```rust
// src/metrics/collector.rs

use crate::jvm::connector::JvmConnector;
use crate::metrics::store::MetricsStore;
use crate::config::PollingConfig;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration, Instant};

/// Events sent from collector to UI
#[derive(Debug, Clone)]
pub enum MetricsEvent {
    /// New metrics are available
    Updated,
    /// Error during collection
    Error(String),
    /// JVM disconnected
    Disconnected,
}

/// Async metrics collector task
pub struct MetricsCollector {
    connector: Arc<dyn JvmConnector>,
    store: Arc<RwLock<MetricsStore>>,
    config: PollingConfig,
    event_tx: mpsc::Sender<MetricsEvent>,
}

impl MetricsCollector {
    pub fn new(
        connector: Arc<dyn JvmConnector>,
        store: Arc<RwLock<MetricsStore>>,
        config: PollingConfig,
    ) -> (Self, mpsc::Receiver<MetricsEvent>) {
        let (event_tx, event_rx) = mpsc::channel(32);
        
        let collector = Self {
            connector,
            store,
            config,
            event_tx,
        };
        
        (collector, event_rx)
    }
    
    /// Run the collection loop
    pub async fn run(self) {
        let mut ticker = interval(self.config.interval);
        let mut consecutive_errors = 0u32;
        const MAX_ERRORS: u32 = 5;
        
        // Fetch static info first
        self.fetch_static_info().await;
        
        loop {
            ticker.tick().await;
            
            match self.collect_metrics().await {
                Ok(()) => {
                    consecutive_errors = 0;
                    let _ = self.event_tx.send(MetricsEvent::Updated).await;
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let _ = self.event_tx.send(MetricsEvent::Error(e.to_string())).await;
                    
                    // Update store error count
                    {
                        let mut store = self.store.write().await;
                        store.record_error();
                    }
                    
                    // Check if JVM is gone
                    if consecutive_errors >= MAX_ERRORS {
                        if !self.connector.is_alive().await {
                            let _ = self.event_tx.send(MetricsEvent::Disconnected).await;
                            break;
                        }
                    }
                }
            }
        }
    }
    
    /// Fetch static info (run once on connect)
    async fn fetch_static_info(&self) {
        let mut store = self.store.write().await;
        
        // VM Version
        if let Ok(version) = self.connector.get_vm_version().await {
            store.vm_version = Some(version);
        }
        
        // VM Flags
        if let Ok(flags) = self.connector.get_vm_flags().await {
            store.vm_flags = Some(flags);
        }
    }
    
    /// Collect all dynamic metrics
    async fn collect_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Collect metrics in parallel
        let (heap_result, gc_result, thread_result, uptime_result) = tokio::join!(
            self.connector.get_heap_info(),
            self.connector.get_gc_stats(),
            self.connector.get_thread_summary(),
            self.connector.get_uptime(),
        );
        
        // Update store
        let mut store = self.store.write().await;
        
        if let Ok(heap) = heap_result {
            store.push_heap_info(heap);
        }
        
        if let Ok(gc) = gc_result {
            store.push_gc_stats(gc);
        }
        
        if let Ok(threads) = thread_result {
            store.update_thread_summary(threads);
        }
        
        if let Ok(uptime) = uptime_result {
            store.update_uptime(uptime);
        }
        
        Ok(())
    }
}

/// Handle for controlling the collector
pub struct CollectorHandle {
    cancel_tx: mpsc::Sender<()>,
}

impl CollectorHandle {
    /// Stop the collector
    pub async fn stop(&self) {
        let _ = self.cancel_tx.send(()).await;
    }
}

/// Spawn the collector as a background task
pub fn spawn_collector(
    connector: Arc<dyn JvmConnector>,
    store: Arc<RwLock<MetricsStore>>,
    config: PollingConfig,
) -> (CollectorHandle, mpsc::Receiver<MetricsEvent>) {
    let (collector, event_rx) = MetricsCollector::new(connector, store, config);
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
    
    tokio::spawn(async move {
        tokio::select! {
            _ = collector.run() => {}
            _ = cancel_rx.recv() => {}
        }
    });
    
    (CollectorHandle { cancel_tx }, event_rx)
}
```

## Integration with TUI Event Loop

```rust
// src/tui/event.rs

use crate::metrics::collector::MetricsEvent;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    /// Key press
    Key(KeyEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Time to refresh display
    Tick,
    /// Metrics updated
    Metrics(MetricsEvent),
}

pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration, metrics_rx: mpsc::Receiver<MetricsEvent>) -> Self {
        let (tx, rx) = mpsc::channel(100);
        
        // Spawn terminal event handler
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        let event = match evt {
                            CrosstermEvent::Key(key) => Event::Key(key),
                            CrosstermEvent::Resize(w, h) => Event::Resize(w, h),
                            _ => continue,
                        };
                        if tx_clone.send(event).await.is_err() {
                            break;
                        }
                    }
                } else {
                    if tx_clone.send(Event::Tick).await.is_err() {
                        break;
                    }
                }
            }
        });
        
        // Spawn metrics event forwarder
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut metrics_rx = metrics_rx;
            while let Some(evt) = metrics_rx.recv().await {
                if tx_clone.send(Event::Metrics(evt)).await.is_err() {
                    break;
                }
            }
        });
        
        Self { rx }
    }
    
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
```

## Usage Example

```rust
// In app.rs

async fn run_monitoring(pid: u32) -> Result<()> {
    // Create connector
    let connector = create_connector(ConnectorConfig::Local { pid }).await?;
    let connector = Arc::new(connector);
    
    // Create metrics store
    let config = PollingConfig::default();
    let store = Arc::new(RwLock::new(MetricsStore::new(config.history_size)));
    
    // Spawn collector
    let (handle, metrics_rx) = spawn_collector(
        Arc::clone(&connector),
        Arc::clone(&store),
        config.clone(),
    );
    
    // Create event handler
    let mut events = EventHandler::new(
        Duration::from_millis(100),
        metrics_rx,
    );
    
    // Main loop
    loop {
        // Render UI
        terminal.draw(|f| {
            let store = store.blocking_read();
            render_ui(f, &store);
        })?;
        
        // Handle events
        match events.next().await {
            Some(Event::Key(key)) => {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
            Some(Event::Metrics(MetricsEvent::Disconnected)) => {
                // Return to JVM picker
                break;
            }
            Some(Event::Tick) | Some(Event::Metrics(MetricsEvent::Updated)) => {
                // Trigger re-render (handled by loop)
            }
            _ => {}
        }
    }
    
    handle.stop().await;
    Ok(())
}
```
