# Parsers

This document describes the parsers for JDK tool output in JVM-TUI.

## Parser Design Principles

1. **Defensive parsing**: Handle malformed output gracefully
2. **Regex caching**: Compile patterns once with `once_cell`
3. **Structured output**: Return strongly-typed structs
4. **Testable**: Use fixtures for comprehensive testing

## jcmd Parsers

### GC.heap_info Parser

**Input example:**
```
76660:
 garbage-first heap   total 2097152K, used 2034889K [0x000000052a800000, 0x00000005aa800000)
  region size 1024K, 436 young (446464K), 4 survivors (4096K)
 Metaspace       used 422035K, committed 427968K, reserved 1441792K
  class space    used 56631K, committed 59200K, reserved 1048576K
```

**Output struct:**
```rust
#[derive(Debug, Clone, Default)]
pub struct HeapInfo {
    pub gc_type: String,           // "garbage-first heap"
    pub total_kb: u64,             // 2097152
    pub used_kb: u64,              // 2034889
    pub region_size_kb: Option<u64>,   // 1024
    pub young_regions: Option<u32>,    // 436
    pub survivor_regions: Option<u32>, // 4
    pub metaspace_used_kb: u64,    // 422035
    pub metaspace_committed_kb: u64,// 427968
    pub metaspace_reserved_kb: u64, // 1441792
    pub class_space_used_kb: u64,   // 56631
    pub class_space_committed_kb: u64, // 59200
}
```

**Implementation:**
```rust
// src/jvm/jdk_tools/parsers/jcmd.rs

use regex::Regex;
use once_cell::sync::Lazy;

static HEAP_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\S+(?:\s+\S+)*)\s+total\s+(\d+)K,\s+used\s+(\d+)K").unwrap()
});

static REGION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"region size (\d+)K,\s+(\d+)\s+young.*?,\s+(\d+)\s+survivors").unwrap()
});

static METASPACE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Metaspace\s+used\s+(\d+)K,\s+committed\s+(\d+)K,\s+reserved\s+(\d+)K").unwrap()
});

static CLASS_SPACE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"class space\s+used\s+(\d+)K,\s+committed\s+(\d+)K").unwrap()
});

pub fn parse_gc_heap_info(output: &str) -> Result<HeapInfo, ParseError> {
    let mut info = HeapInfo::default();
    
    // Parse main heap line
    if let Some(caps) = HEAP_PATTERN.captures(output) {
        info.gc_type = caps[1].trim().to_string();
        info.total_kb = caps[2].parse()?;
        info.used_kb = caps[3].parse()?;
    } else {
        return Err(ParseError::MissingField("heap info"));
    }
    
    // Parse region info (G1 specific)
    if let Some(caps) = REGION_PATTERN.captures(output) {
        info.region_size_kb = Some(caps[1].parse()?);
        info.young_regions = Some(caps[2].parse()?);
        info.survivor_regions = Some(caps[3].parse()?);
    }
    
    // Parse metaspace
    if let Some(caps) = METASPACE_PATTERN.captures(output) {
        info.metaspace_used_kb = caps[1].parse()?;
        info.metaspace_committed_kb = caps[2].parse()?;
        info.metaspace_reserved_kb = caps[3].parse()?;
    }
    
    // Parse class space
    if let Some(caps) = CLASS_SPACE_PATTERN.captures(output) {
        info.class_space_used_kb = caps[1].parse()?;
        info.class_space_committed_kb = caps[2].parse()?;
    }
    
    Ok(info)
}
```

### VM.version Parser

**Input:**
```
76660:
OpenJDK 64-Bit Server VM version 21.0.9+10-b1163.86
JDK 21.0.9
```

**Output:**
```rust
#[derive(Debug, Clone)]
pub struct VmVersion {
    pub vm_name: String,    // "OpenJDK 64-Bit Server VM"
    pub vm_version: String, // "21.0.9+10-b1163.86"
    pub jdk_version: String, // "21.0.9"
}
```

**Implementation:**
```rust
static VM_VERSION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(.+)\s+version\s+(\S+)").unwrap()
});

static JDK_VERSION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"JDK\s+(\S+)").unwrap()
});

pub fn parse_vm_version(output: &str) -> Result<VmVersion, ParseError> {
    let vm_caps = VM_VERSION_PATTERN.captures(output)
        .ok_or(ParseError::MissingField("VM version"))?;
    
    let jdk_version = JDK_VERSION_PATTERN.captures(output)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| vm_caps[2].to_string());
    
    Ok(VmVersion {
        vm_name: vm_caps[1].to_string(),
        vm_version: vm_caps[2].to_string(),
        jdk_version,
    })
}
```

### VM.uptime Parser

**Input:**
```
76660:
3029.822 s
```

**Output:** `f64` (seconds)

```rust
pub fn parse_vm_uptime(output: &str) -> Result<f64, ParseError> {
    // Skip PID line, find the number
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_suffix(" s") {
            return stripped.parse().map_err(|_| ParseError::InvalidNumber);
        }
        // Also handle without space: "3029.822s"
        if let Some(stripped) = trimmed.strip_suffix("s") {
            return stripped.parse().map_err(|_| ParseError::InvalidNumber);
        }
    }
    Err(ParseError::MissingField("uptime"))
}
```

### Thread.print Parser

**Input (truncated):**
```
76660:
2026-01-06 23:56:54
Full thread dump OpenJDK 64-Bit Server VM (21.0.9+10-b1163.86 mixed mode):

"main" #1 prio=5 os_prio=31 cpu=1272.89ms elapsed=3020.36s tid=0x... nid=7427 waiting on condition
   java.lang.Thread.State: TIMED_WAITING (parking)
	at jdk.internal.misc.Unsafe.park(Native Method)
	at java.util.concurrent.locks.LockSupport.parkNanos(LockSupport.java:269)
	at kotlinx.coroutines.BlockingCoroutine.joinBlocking(Builders.kt:121)

"Common-Cleaner" #9 daemon prio=8 os_prio=31 cpu=15.61ms elapsed=3020.34s tid=0x... nid=23043 waiting on condition
   java.lang.Thread.State: TIMED_WAITING (parking)
	at jdk.internal.misc.Unsafe.park(Native Method)
```

**Output:**
```rust
#[derive(Debug, Clone)]
pub struct ThreadDump {
    pub timestamp: String,
    pub vm_info: String,
    pub threads: Vec<ThreadInfo>,
}

#[derive(Debug, Clone)]
pub struct ThreadInfo {
    pub name: String,
    pub id: u64,
    pub daemon: bool,
    pub priority: u8,
    pub state: ThreadState,
    pub state_detail: Option<String>,
    pub cpu_time_ms: Option<f64>,
    pub elapsed_sec: Option<f64>,
    pub stack_trace: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    New,
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
}
```

**Implementation:**
```rust
static THREAD_HEADER_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#""([^"]+)"\s+#(\d+)(?:\s+daemon)?\s+prio=(\d+).*?cpu=([0-9.]+)ms.*?elapsed=([0-9.]+)s"#
    ).unwrap()
});

static THREAD_STATE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"java\.lang\.Thread\.State:\s+(\S+)(?:\s+\(([^)]+)\))?").unwrap()
});

pub fn parse_thread_dump(output: &str) -> Result<ThreadDump, ParseError> {
    let lines: Vec<&str> = output.lines().collect();
    let mut threads = Vec::new();
    let mut current_thread: Option<ThreadInfo> = None;
    let mut timestamp = String::new();
    let mut vm_info = String::new();
    
    for line in &lines {
        // Parse timestamp (first line after PID)
        if line.contains("Full thread dump") {
            vm_info = line.to_string();
            continue;
        }
        
        // Check for thread header
        if line.starts_with('"') {
            // Save previous thread
            if let Some(thread) = current_thread.take() {
                threads.push(thread);
            }
            
            // Parse new thread
            if let Some(caps) = THREAD_HEADER_PATTERN.captures(line) {
                current_thread = Some(ThreadInfo {
                    name: caps[1].to_string(),
                    id: caps[2].parse().unwrap_or(0),
                    daemon: line.contains(" daemon "),
                    priority: caps[3].parse().unwrap_or(5),
                    cpu_time_ms: caps[4].parse().ok(),
                    elapsed_sec: caps[5].parse().ok(),
                    state: ThreadState::Runnable,
                    state_detail: None,
                    stack_trace: Vec::new(),
                });
            }
        }
        // Parse thread state
        else if line.contains("java.lang.Thread.State:") {
            if let (Some(thread), Some(caps)) = (&mut current_thread, THREAD_STATE_PATTERN.captures(line)) {
                thread.state = parse_thread_state(&caps[1]);
                thread.state_detail = caps.get(2).map(|m| m.as_str().to_string());
            }
        }
        // Parse stack trace line
        else if line.starts_with("\tat ") || line.starts_with("	at ") {
            if let Some(thread) = &mut current_thread {
                thread.stack_trace.push(line.trim().to_string());
            }
        }
    }
    
    // Don't forget last thread
    if let Some(thread) = current_thread {
        threads.push(thread);
    }
    
    Ok(ThreadDump {
        timestamp,
        vm_info,
        threads,
    })
}

fn parse_thread_state(state: &str) -> ThreadState {
    match state {
        "NEW" => ThreadState::New,
        "RUNNABLE" => ThreadState::Runnable,
        "BLOCKED" => ThreadState::Blocked,
        "WAITING" => ThreadState::Waiting,
        "TIMED_WAITING" => ThreadState::TimedWaiting,
        "TERMINATED" => ThreadState::Terminated,
        _ => ThreadState::Runnable,
    }
}
```

## jstat Parsers

### -gcutil Parser

**Input:**
```
  S0     S1     E      O      M     CCS    YGC     YGCT    FGC    FGCT     CGC    CGCT       GCT   
   -      -   1.52  69.85  98.62  95.69    695    7.803     1    0.236   436    4.121    12.160
```

**Output:**
```rust
#[derive(Debug, Clone, Default)]
pub struct GcUtilStats {
    pub survivor0_pct: f64,
    pub survivor1_pct: f64,
    pub eden_pct: f64,
    pub old_pct: f64,
    pub metaspace_pct: f64,
    pub compressed_class_pct: f64,
    pub young_gc_count: u64,
    pub young_gc_time_sec: f64,
    pub full_gc_count: u64,
    pub full_gc_time_sec: f64,
    pub concurrent_gc_count: u64,
    pub concurrent_gc_time_sec: f64,
    pub total_gc_time_sec: f64,
}
```

**Implementation:**
```rust
// src/jvm/jdk_tools/parsers/jstat.rs

pub fn parse_gcutil(output: &str) -> Result<GcUtilStats, ParseError> {
    let lines: Vec<&str> = output.lines().collect();
    
    if lines.len() < 2 {
        return Err(ParseError::InsufficientData);
    }
    
    // Skip header, parse data line
    let values: Vec<&str> = lines[1].split_whitespace().collect();
    
    if values.len() < 13 {
        return Err(ParseError::InsufficientColumns { 
            expected: 13, 
            got: values.len() 
        });
    }
    
    Ok(GcUtilStats {
        survivor0_pct: parse_pct(values[0]),
        survivor1_pct: parse_pct(values[1]),
        eden_pct: parse_pct(values[2]),
        old_pct: parse_pct(values[3]),
        metaspace_pct: parse_pct(values[4]),
        compressed_class_pct: parse_pct(values[5]),
        young_gc_count: values[6].parse().unwrap_or(0),
        young_gc_time_sec: values[7].parse().unwrap_or(0.0),
        full_gc_count: values[8].parse().unwrap_or(0),
        full_gc_time_sec: values[9].parse().unwrap_or(0.0),
        concurrent_gc_count: values[10].parse().unwrap_or(0),
        concurrent_gc_time_sec: values[11].parse().unwrap_or(0.0),
        total_gc_time_sec: values[12].parse().unwrap_or(0.0),
    })
}

/// Parse percentage value, handling "-" as 0
fn parse_pct(s: &str) -> f64 {
    if s == "-" {
        0.0
    } else {
        s.parse().unwrap_or(0.0)
    }
}
```

### -gc Parser

**Input:**
```
    S0C         S1C         S0U         S1U          EC           EU           OC           OU          MC         MU       CCSC      CCSU     YGC     YGCT     FGC    FGCT     CGC    CGCT       GCT   
        0.0         0.0         0.0         0.0    1009664.0      15360.0    1087488.0     759556.7   428160.0   422242.0   59200.0   56647.0    695     7.803     1     0.236   436     4.121    12.160
```

**Output:**
```rust
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    pub survivor0_capacity_kb: f64,
    pub survivor1_capacity_kb: f64,
    pub survivor0_used_kb: f64,
    pub survivor1_used_kb: f64,
    pub eden_capacity_kb: f64,
    pub eden_used_kb: f64,
    pub old_capacity_kb: f64,
    pub old_used_kb: f64,
    pub metaspace_capacity_kb: f64,
    pub metaspace_used_kb: f64,
    pub compressed_class_capacity_kb: f64,
    pub compressed_class_used_kb: f64,
    pub young_gc_count: u64,
    pub young_gc_time_sec: f64,
    pub full_gc_count: u64,
    pub full_gc_time_sec: f64,
}
```

## jps Parser

**Input:**
```
76660 com.intellij.idea.Main
46168 com.intellij.idea.Main
48127 /path/to/sonarlint-ls.jar
78096 jdk.jcmd/sun.tools.jps.Jps
```

**Output:**
```rust
#[derive(Debug, Clone)]
pub struct DiscoveredJvm {
    pub pid: u32,
    pub main_class: String,
    pub display_name: String,
}
```

**Implementation:**
```rust
// src/jvm/jdk_tools/parsers/jps.rs

pub fn parse_jps_output(output: &str) -> Vec<DiscoveredJvm> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() < 2 {
                return None;
            }
            
            let pid: u32 = parts[0].parse().ok()?;
            let main_class = parts[1].to_string();
            
            // Skip jps/jcmd themselves
            if main_class.contains("sun.tools.jps") || 
               main_class.contains("sun.tools.jcmd") {
                return None;
            }
            
            let display_name = extract_display_name(&main_class);
            
            Some(DiscoveredJvm {
                pid,
                main_class,
                display_name,
            })
        })
        .collect()
}

fn extract_display_name(main_class: &str) -> String {
    // Handle JAR paths: /path/to/app.jar -> app.jar
    if main_class.ends_with(".jar") {
        return main_class
            .rsplit('/')
            .next()
            .unwrap_or(main_class)
            .to_string();
    }
    
    // Handle class names: com.example.Main -> Main
    main_class
        .rsplit('.')
        .next()
        .unwrap_or(main_class)
        .to_string()
}
```

## Error Types

```rust
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    
    #[error("Invalid number format")]
    InvalidNumber,
    
    #[error("Insufficient data in output")]
    InsufficientData,
    
    #[error("Expected {expected} columns, got {got}")]
    InsufficientColumns { expected: usize, got: usize },
    
    #[error("Unexpected output format: {0}")]
    UnexpectedFormat(String),
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    
    #[test]
    fn test_parse_heap_info_g1() {
        let output = include_str!("../../../assets/sample_outputs/jcmd_heap_info_g1.txt");
        let info = parse_gc_heap_info(output).unwrap();
        
        assert_eq!(info.gc_type, "garbage-first heap");
        assert_eq!(info.total_kb, 2097152);
        assert!(info.used_kb > 0);
        assert!(info.region_size_kb.is_some());
    }
    
    #[test]
    fn test_parse_gcutil() {
        let output = "  S0     S1     E      O      M     CCS    YGC     YGCT    FGC    FGCT     CGC    CGCT       GCT\n   -      -   1.52  69.85  98.62  95.69    695    7.803     1    0.236   436    4.121    12.160";
        let stats = parse_gcutil(output).unwrap();
        
        assert_eq!(stats.survivor0_pct, 0.0);  // "-" becomes 0
        assert_eq!(stats.eden_pct, 1.52);
        assert_eq!(stats.old_pct, 69.85);
        assert_eq!(stats.young_gc_count, 695);
    }
    
    #[test]
    fn test_parse_jps_filters_tools() {
        let output = "12345 com.example.App\n67890 jdk.jcmd/sun.tools.jps.Jps";
        let jvms = parse_jps_output(output);
        
        assert_eq!(jvms.len(), 1);
        assert_eq!(jvms[0].pid, 12345);
    }
}
```
