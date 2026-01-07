# Testing Strategy

This document describes the testing approach for JVM-TUI.

## Testing Pyramid

```
                    ┌─────────────────┐
                    │    E2E Tests    │  Few, slow, high confidence
                    │   (manual/CI)   │
                    └────────┬────────┘
                             │
               ┌─────────────┴─────────────┐
               │    Integration Tests      │  Some, medium speed
               │  (real JVM interaction)   │
               └─────────────┬─────────────┘
                             │
        ┌────────────────────┴────────────────────┐
        │              Unit Tests                  │  Many, fast
        │  (parsers, data structures, logic)      │
        └─────────────────────────────────────────┘
```

## Test Categories

### 1. Unit Tests

Fast, isolated tests for pure functions.

**Location:** Same file as implementation (`#[cfg(test)]` module)

**Coverage:**
- All parsers (jcmd, jstat, jps)
- Ring buffer operations
- Data type conversions
- Display formatting

### 2. Integration Tests

Tests that interact with real JVMs or simulate subprocess execution.

**Location:** `tests/` directory

**Coverage:**
- JVM discovery
- Connector operations
- End-to-end data flow

### 3. Snapshot Tests

For complex output parsing, use snapshot testing.

**Tool:** `insta` crate

**Coverage:**
- Parser output structures
- UI rendering (future)

## Test Fixtures

### Directory Structure

```
assets/
└── sample_outputs/
    ├── jcmd/
    │   ├── heap_info_g1.txt
    │   ├── heap_info_zgc.txt
    │   ├── heap_info_parallel.txt
    │   ├── vm_version_openjdk21.txt
    │   ├── vm_version_openjdk17.txt
    │   ├── vm_flags.txt
    │   ├── vm_uptime.txt
    │   ├── thread_print.txt
    │   └── class_histogram.txt
    ├── jstat/
    │   ├── gcutil.txt
    │   ├── gcutil_zgc.txt
    │   ├── gc.txt
    │   └── class.txt
    └── jps/
        └── output.txt
```

### Creating Fixtures

Capture real output from running JVMs:

```bash
# Capture heap info
jcmd $(pgrep -f YourApp) GC.heap_info > assets/sample_outputs/jcmd/heap_info_g1.txt

# Capture GC stats
jstat -gcutil $(pgrep -f YourApp) > assets/sample_outputs/jstat/gcutil.txt
```

## Parser Unit Tests

### Example: jstat Parser

```rust
// src/jvm/jdk_tools/parsers/jstat.rs

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn test_parse_gcutil_normal() {
        let output = include_str!("../../../../assets/sample_outputs/jstat/gcutil.txt");
        let result = parse_gcutil(output).unwrap();
        
        assert_eq!(result.eden_pct, 1.52);
        assert_eq!(result.old_pct, 69.85);
        assert_eq!(result.young_gc_count, 695);
        assert!((result.young_gc_time_sec - 7.803).abs() < 0.001);
    }
    
    #[test]
    fn test_parse_gcutil_with_dashes() {
        // Survivor spaces show "-" when unused
        let output = "  S0     S1     E      O      M     CCS    YGC     YGCT    FGC    FGCT     CGC    CGCT       GCT\n   -      -   1.52  69.85  98.62  95.69    695    7.803     1    0.236   436    4.121    12.160";
        let result = parse_gcutil(output).unwrap();
        
        assert_eq!(result.survivor0_pct, 0.0);
        assert_eq!(result.survivor1_pct, 0.0);
    }
    
    #[test]
    fn test_parse_gcutil_malformed() {
        let output = "invalid data";
        let result = parse_gcutil(output);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_gcutil_empty() {
        let output = "";
        let result = parse_gcutil(output);
        
        assert!(result.is_err());
    }
}
```

### Example: jcmd Parser

```rust
// src/jvm/jdk_tools/parsers/jcmd.rs

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    
    #[test]
    fn test_parse_heap_info_g1() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd/heap_info_g1.txt");
        let result = parse_gc_heap_info(output).unwrap();
        
        assert_eq!(result.gc_type, "garbage-first heap");
        assert!(result.total_kb > 0);
        assert!(result.used_kb > 0);
        assert!(result.used_kb <= result.total_kb);
        assert!(result.region_size_kb.is_some());
    }
    
    #[test]
    fn test_parse_heap_info_snapshot() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd/heap_info_g1.txt");
        let result = parse_gc_heap_info(output).unwrap();
        
        // Snapshot test - will fail if output structure changes
        assert_debug_snapshot!(result);
    }
    
    #[test]
    fn test_parse_vm_version() {
        let output = "76660:\nOpenJDK 64-Bit Server VM version 21.0.9+10-b1163.86\nJDK 21.0.9";
        let result = parse_vm_version(output).unwrap();
        
        assert_eq!(result.vm_name, "OpenJDK 64-Bit Server VM");
        assert_eq!(result.vm_version, "21.0.9+10-b1163.86");
        assert_eq!(result.jdk_version, "21.0.9");
    }
    
    #[test]
    fn test_parse_vm_uptime() {
        let output = "76660:\n3029.822 s";
        let result = parse_vm_uptime(output).unwrap();
        
        assert!((result - 3029.822).abs() < 0.001);
    }
}
```

## Integration Tests

### JVM Discovery Test

```rust
// tests/integration/discovery_tests.rs

use jvm_tui::jvm::discovery::discover_local_jvms;

/// This test requires at least one JVM running on the system
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_discover_finds_running_jvms() {
    let jvms = discover_local_jvms().await.unwrap();
    
    // At least jcmd itself should be visible briefly
    // In CI, we'd start a test JVM first
    println!("Found {} JVMs", jvms.len());
    
    for jvm in &jvms {
        println!("  PID: {}, Main: {}", jvm.pid, jvm.main_class);
        assert!(jvm.pid > 0);
        assert!(!jvm.main_class.is_empty());
    }
}
```

### Connector Test

```rust
// tests/integration/connector_tests.rs

use jvm_tui::jvm::connector::*;
use jvm_tui::jvm::jdk_tools::*;

/// Test connecting to a real JVM
#[tokio::test]
#[ignore]
async fn test_connector_reads_heap_info() {
    // Find a JVM
    let jvms = discover_local_jvms().await.unwrap();
    let jvm = jvms.first().expect("No JVMs running");
    
    // Create connector
    let status = JdkToolsStatus::detect().await;
    let executor = JdkExecutor::new(&status).unwrap();
    let connector = JdkToolsConnector::new(jvm.pid, executor);
    
    // Test heap info
    let heap = connector.get_heap_info().await.unwrap();
    
    assert!(heap.total_kb > 0);
    assert!(heap.used_kb > 0);
    assert!(!heap.gc_type.is_empty());
}
```

## Ring Buffer Tests

```rust
// src/metrics/ring_buffer.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_push_within_capacity() {
        let mut buf = RingBuffer::new(5);
        
        buf.push(1);
        buf.push(2);
        buf.push(3);
        
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);
    }
    
    #[test]
    fn test_push_exceeds_capacity() {
        let mut buf = RingBuffer::new(3);
        
        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4);  // Should evict 1
        
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_vec(), vec![2, 3, 4]);
    }
    
    #[test]
    fn test_latest() {
        let mut buf = RingBuffer::new(5);
        
        buf.push(1);
        buf.push(2);
        buf.push(3);
        
        assert_eq!(buf.latest(), Some(&3));
    }
    
    #[test]
    fn test_empty() {
        let buf: RingBuffer<u64> = RingBuffer::new(5);
        
        assert!(buf.is_empty());
        assert_eq!(buf.latest(), None);
    }
}
```

## Mock Testing

### Mock Connector

```rust
// src/jvm/connector.rs

#[cfg(test)]
pub mod mock {
    use super::*;
    use async_trait::async_trait;
    
    pub struct MockConnector {
        pub heap_info: HeapInfo,
        pub gc_stats: GcStats,
        pub fail_after: Option<u32>,
        call_count: std::sync::atomic::AtomicU32,
    }
    
    impl MockConnector {
        pub fn new() -> Self {
            Self {
                heap_info: HeapInfo {
                    gc_type: "mock heap".into(),
                    total_kb: 1024 * 1024,
                    used_kb: 512 * 1024,
                    ..Default::default()
                },
                gc_stats: GcStats::default(),
                fail_after: None,
                call_count: std::sync::atomic::AtomicU32::new(0),
            }
        }
        
        pub fn failing_after(mut self, calls: u32) -> Self {
            self.fail_after = Some(calls);
            self
        }
    }
    
    #[async_trait]
    impl JvmConnector for MockConnector {
        fn pid(&self) -> u32 { 12345 }
        
        async fn is_alive(&self) -> bool { true }
        
        async fn get_heap_info(&self) -> Result<HeapInfo> {
            let count = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            
            if let Some(fail_after) = self.fail_after {
                if count >= fail_after {
                    return Err(eyre::eyre!("Mock failure"));
                }
            }
            
            Ok(self.heap_info.clone())
        }
        
        async fn get_gc_stats(&self) -> Result<GcStats> {
            Ok(self.gc_stats.clone())
        }
        
        // ... other methods return defaults
    }
}
```

### Using Mocks

```rust
#[tokio::test]
async fn test_collector_handles_disconnection() {
    let connector = Arc::new(mock::MockConnector::new().failing_after(3));
    let store = Arc::new(RwLock::new(MetricsStore::new(100)));
    
    let config = PollingConfig {
        interval: Duration::from_millis(10),
        ..Default::default()
    };
    
    let (handle, mut rx) = spawn_collector(connector, store.clone(), config);
    
    // Collect events
    let mut events = Vec::new();
    while let Ok(evt) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        if let Some(e) = evt {
            events.push(e);
        }
    }
    
    // Should have some successes then errors
    assert!(events.iter().any(|e| matches!(e, MetricsEvent::Updated)));
    assert!(events.iter().any(|e| matches!(e, MetricsEvent::Error(_))));
}
```

## CI Configuration

### GitHub Actions

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, 1.91.0]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          toolchain: ${{ matrix.rust }}
          components: clippy, rustfmt
      
      - name: Install JDK (for integration tests)
        uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '21'
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Clippy
        run: cargo clippy -- -D warnings
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test '*' -- --ignored
        continue-on-error: true  # May fail without running JVM

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Test Commands

```bash
# Run all unit tests
cargo test --lib

# Run specific test module
cargo test jstat

# Run integration tests (requires JVM)
cargo test --test '*' -- --ignored

# Run with output
cargo test -- --nocapture

# Run snapshot tests and update
cargo insta test
cargo insta review

# Generate coverage report
cargo tarpaulin --out Html
open tarpaulin-report.html
```

## Coverage Goals

| Module | Target Coverage |
|--------|-----------------|
| Parsers | 90%+ |
| Ring buffer | 100% |
| Types | 80%+ |
| Connector | 70%+ |
| TUI | 50%+ (harder to test) |

## Test Data Management

Keep test fixtures updated:

```bash
#!/bin/bash
# scripts/update_fixtures.sh

# Find a JVM to capture from
PID=$(jps -l | grep -v Jps | head -1 | cut -d' ' -f1)

if [ -z "$PID" ]; then
    echo "No JVM found. Start one first."
    exit 1
fi

echo "Capturing from PID $PID..."

jcmd $PID GC.heap_info > assets/sample_outputs/jcmd/heap_info_g1.txt
jcmd $PID VM.version > assets/sample_outputs/jcmd/vm_version.txt
jcmd $PID VM.uptime > assets/sample_outputs/jcmd/vm_uptime.txt
jcmd $PID VM.flags > assets/sample_outputs/jcmd/vm_flags.txt
jcmd $PID Thread.print > assets/sample_outputs/jcmd/thread_print.txt
jstat -gcutil $PID > assets/sample_outputs/jstat/gcutil.txt
jstat -gc $PID > assets/sample_outputs/jstat/gc.txt
jps -l > assets/sample_outputs/jps/output.txt

echo "Done!"
```
