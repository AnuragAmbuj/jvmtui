# JDK Tools Integration

This document describes how JVM-TUI integrates with JDK command-line tools for agentless JVM monitoring.

## Overview

JVM-TUI uses three JDK tools for agentless monitoring:

| Tool | Purpose | Data Source |
|------|---------|-------------|
| `jcmd` | Diagnostic commands | Attach API |
| `jstat` | GC statistics | hsperfdata files |
| `jps` | JVM discovery (fallback) | hsperfdata files |

## Tool Detection

### Detection Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     Tool Detection Flow                         │
└─────────────────────────────────────────────────────────────────┘

1. Check $JAVA_HOME/bin/jcmd
   │
   ├─ Found & Executable ────► Mark jcmd available
   │
   └─ Not found ─────────────► Check $PATH for jcmd
                                │
                                ├─ Found ────► Mark jcmd available
                                │
                                └─ Not found ─► Mark jcmd unavailable

2. Repeat for jstat, jps

3. Determine capabilities based on available tools
```

### Implementation

```rust
// src/jvm/jdk_tools/detector.rs

use std::process::Command;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct JdkToolsStatus {
    pub jcmd: ToolStatus,
    pub jstat: ToolStatus,
    pub jps: ToolStatus,
    pub java_home: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum ToolStatus {
    Available { path: PathBuf, version: String },
    NotFound,
    NotExecutable { path: PathBuf },
}

impl JdkToolsStatus {
    pub async fn detect() -> Self {
        let java_home = std::env::var("JAVA_HOME")
            .ok()
            .map(PathBuf::from);
        
        Self {
            jcmd: detect_tool("jcmd", &java_home).await,
            jstat: detect_tool("jstat", &java_home).await,
            jps: detect_tool("jps", &java_home).await,
            java_home,
        }
    }
    
    /// Check if we have minimum tools for operation
    pub fn is_usable(&self) -> bool {
        matches!(self.jcmd, ToolStatus::Available { .. }) ||
        (matches!(self.jps, ToolStatus::Available { .. }) && 
         matches!(self.jstat, ToolStatus::Available { .. }))
    }
    
    /// Get detailed capabilities
    pub fn capabilities(&self) -> Capabilities {
        Capabilities {
            can_discover: self.jcmd.is_available() || self.jps.is_available(),
            can_heap_info: self.jcmd.is_available(),
            can_gc_stats: self.jstat.is_available() || self.jcmd.is_available(),
            can_thread_dump: self.jcmd.is_available(),
            can_class_histogram: self.jcmd.is_available(),
            can_trigger_gc: self.jcmd.is_available(),
        }
    }
}

async fn detect_tool(name: &str, java_home: &Option<PathBuf>) -> ToolStatus {
    // Priority: JAVA_HOME/bin > PATH
    let candidates: Vec<PathBuf> = java_home
        .iter()
        .map(|h| h.join("bin").join(name))
        .chain(std::iter::once(PathBuf::from(name)))
        .collect();
    
    for path in candidates {
        match try_execute(&path).await {
            Ok(version) => {
                return ToolStatus::Available { 
                    path, 
                    version 
                };
            }
            Err(TryExecuteError::NotExecutable) => {
                return ToolStatus::NotExecutable { path };
            }
            Err(TryExecuteError::NotFound) => continue,
        }
    }
    
    ToolStatus::NotFound
}

async fn try_execute(path: &PathBuf) -> Result<String, TryExecuteError> {
    use tokio::process::Command;
    
    let output = Command::new(path)
        .arg("-version")
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TryExecuteError::NotFound
            } else {
                TryExecuteError::NotExecutable
            }
        })?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(TryExecuteError::NotExecutable)
    }
}
```

## Installation Guidance

When tools are missing, show platform-specific installation instructions:

```rust
// src/jvm/jdk_tools/detector.rs

impl JdkToolsStatus {
    pub fn installation_guide(&self) -> String {
        let mut guide = String::new();
        
        guide.push_str("JDK tools not found. Please install a JDK (not just JRE).\n\n");
        guide.push_str("Installation options:\n\n");
        
        #[cfg(target_os = "macos")]
        {
            guide.push_str("  macOS (Homebrew):\n");
            guide.push_str("    brew install openjdk@21\n\n");
            guide.push_str("  macOS (SDKMAN):\n");
            guide.push_str("    sdk install java 21-tem\n\n");
        }
        
        #[cfg(target_os = "linux")]
        {
            guide.push_str("  Ubuntu/Debian:\n");
            guide.push_str("    sudo apt install openjdk-21-jdk\n\n");
            guide.push_str("  Fedora/RHEL:\n");
            guide.push_str("    sudo dnf install java-21-openjdk-devel\n\n");
            guide.push_str("  Arch Linux:\n");
            guide.push_str("    sudo pacman -S jdk21-openjdk\n\n");
        }
        
        #[cfg(target_os = "windows")]
        {
            guide.push_str("  Windows (winget):\n");
            guide.push_str("    winget install Microsoft.OpenJDK.21\n\n");
            guide.push_str("  Windows (Scoop):\n");
            guide.push_str("    scoop install openjdk21\n\n");
        }
        
        guide.push_str("  Manual download:\n");
        guide.push_str("    https://adoptium.net/\n\n");
        guide.push_str("After installation, ensure:\n");
        guide.push_str("  1. JAVA_HOME is set to your JDK directory\n");
        guide.push_str("  2. $JAVA_HOME/bin is in your PATH\n");
        
        guide
    }
}
```

## Command Execution

### Subprocess Executor

```rust
// src/jvm/jdk_tools/executor.rs

use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct JdkExecutor {
    jcmd_path: PathBuf,
    jstat_path: PathBuf,
    timeout: Duration,
}

impl JdkExecutor {
    pub fn new(status: &JdkToolsStatus) -> Result<Self, JdkToolsError> {
        let jcmd_path = status.jcmd.path()
            .ok_or(JdkToolsError::JcmdNotFound)?;
        let jstat_path = status.jstat.path()
            .ok_or(JdkToolsError::JstatNotFound)?;
        
        Ok(Self {
            jcmd_path,
            jstat_path,
            timeout: DEFAULT_TIMEOUT,
        })
    }
    
    /// Execute jcmd command
    pub async fn jcmd(&self, pid: u32, command: &str) -> Result<String, JdkToolsError> {
        self.execute(&self.jcmd_path, &[&pid.to_string(), command]).await
    }
    
    /// Execute jstat command
    pub async fn jstat(&self, option: &str, pid: u32) -> Result<String, JdkToolsError> {
        self.execute(&self.jstat_path, &[option, &pid.to_string()]).await
    }
    
    async fn execute(&self, path: &PathBuf, args: &[&str]) -> Result<String, JdkToolsError> {
        let result = timeout(self.timeout, async {
            Command::new(path)
                .args(args)
                .output()
                .await
        }).await;
        
        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(JdkToolsError::CommandFailed {
                        command: format!("{} {}", path.display(), args.join(" ")),
                        stderr: stderr.to_string(),
                    })
                }
            }
            Ok(Err(e)) => Err(JdkToolsError::ExecutionFailed {
                command: path.display().to_string(),
                source: e,
            }),
            Err(_) => Err(JdkToolsError::Timeout {
                command: format!("{} {}", path.display(), args.join(" ")),
                timeout: self.timeout,
            }),
        }
    }
}
```

## jcmd Commands

### Available Commands

```bash
# List all jcmd commands for a JVM
$ jcmd <pid> help
```

### Commands We Use

| Command | Purpose | Output Format |
|---------|---------|---------------|
| `jcmd -l` | List JVMs | `<pid> <main_class>` per line |
| `VM.version` | JVM version | Multi-line text |
| `VM.uptime` | Process uptime | `<seconds> s` |
| `VM.flags` | JVM flags | Space-separated flags |
| `GC.heap_info` | Heap usage | Structured text |
| `Thread.print` | Thread dump | Stack traces |
| `GC.class_histogram` | Class stats | Table format |
| `GC.run` | Trigger GC | Empty on success |

### Example Outputs

#### VM.version
```
76660:
OpenJDK 64-Bit Server VM version 21.0.9+10-b1163.86
JDK 21.0.9
```

#### VM.uptime
```
76660:
3029.822 s
```

#### GC.heap_info
```
76660:
 garbage-first heap   total 2097152K, used 2034889K [0x..., 0x...)
  region size 1024K, 436 young (446464K), 4 survivors (4096K)
 Metaspace       used 422035K, committed 427968K, reserved 1441792K
  class space    used 56631K, committed 59200K, reserved 1048576K
```

#### Thread.print (truncated)
```
76660:
2026-01-06 23:56:54
Full thread dump OpenJDK 64-Bit Server VM (21.0.9+10-b1163.86 mixed mode):

"main" #1 prio=5 os_prio=31 cpu=1272.89ms elapsed=3020.36s tid=0x... nid=7427 waiting on condition
   java.lang.Thread.State: TIMED_WAITING (parking)
	at jdk.internal.misc.Unsafe.park(Native Method)
	- parking to wait for  <0x...> (a kotlinx.coroutines.BlockingCoroutine)
	at java.util.concurrent.locks.LockSupport.parkNanos(LockSupport.java:269)
	...
```

## jstat Options

### Available Options

```bash
$ jstat -options
-class
-compiler
-gc
-gccapacity
-gccause
-gcmetacapacity
-gcnew
-gcnewcapacity
-gcold
-gcoldcapacity
-gcutil
-printcompilation
```

### Options We Use

| Option | Purpose | Key Columns |
|--------|---------|-------------|
| `-gcutil` | GC percentages | S0, S1, E, O, M, CCS, YGC, FGC, GCT |
| `-gc` | GC sizes (KB) | S0C, S1C, EC, OC, MC, YGC, FGC |
| `-class` | Class loading | Loaded, Unloaded, Bytes |

### Example Outputs

#### -gcutil
```
  S0     S1     E      O      M     CCS    YGC     YGCT    FGC    FGCT     CGC    CGCT       GCT   
   -      -   1.52  69.85  98.62  95.69    695    7.803     1    0.236   436    4.121    12.160
```

#### -gc
```
    S0C         S1C         S0U         S1U          EC           EU           OC           OU          MC         MU       CCSC      CCSU     YGC     YGCT     FGC    FGCT     CGC    CGCT       GCT   
        0.0         0.0         0.0         0.0    1009664.0      15360.0    1087488.0     759556.7   428160.0   422242.0   59200.0   56647.0    695     7.803     1     0.236   436     4.121    12.160
```

## Graceful Degradation

When some tools are missing, degrade gracefully:

```rust
pub struct Capabilities {
    pub can_discover: bool,        // jcmd -l OR jps
    pub can_heap_info: bool,       // jcmd GC.heap_info
    pub can_gc_stats: bool,        // jstat -gcutil OR jcmd
    pub can_thread_dump: bool,     // jcmd Thread.print
    pub can_class_histogram: bool, // jcmd GC.class_histogram
    pub can_trigger_gc: bool,      // jcmd GC.run
}

impl Capabilities {
    pub fn degradation_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if !self.can_heap_info {
            warnings.push("Heap breakdown unavailable (jcmd missing)".into());
        }
        if !self.can_thread_dump {
            warnings.push("Thread dumps unavailable (jcmd missing)".into());
        }
        if !self.can_class_histogram {
            warnings.push("Class histogram unavailable (jcmd missing)".into());
        }
        
        warnings
    }
}
```

## Error Handling

```rust
#[derive(Error, Debug)]
pub enum JdkToolsError {
    #[error("jcmd not found. Install a JDK (not JRE) and ensure JAVA_HOME/bin is in PATH.")]
    JcmdNotFound,
    
    #[error("jstat not found. Install a JDK (not JRE) and ensure JAVA_HOME/bin is in PATH.")]
    JstatNotFound,
    
    #[error("Command failed: {command}\nError: {stderr}")]
    CommandFailed { command: String, stderr: String },
    
    #[error("Command timed out after {timeout:?}: {command}")]
    Timeout { command: String, timeout: Duration },
    
    #[error("Failed to execute {command}: {source}")]
    ExecutionFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Cannot attach to JVM {pid}: {reason}")]
    AttachFailed { pid: u32, reason: String },
    
    #[error("JVM {pid} is not running or not accessible")]
    JvmNotFound { pid: u32 },
}
```

## Security Considerations

1. **PID Validation**: Always validate PID is numeric
   ```rust
   fn validate_pid(pid: u32) -> bool {
       // PID must be positive and reasonable
       pid > 0 && pid < 4_194_304  // Max PID on Linux
   }
   ```

2. **No Shell Execution**: Never use shell interpolation
   ```rust
   // GOOD: Direct execution
   Command::new("jcmd").arg(pid.to_string()).arg("VM.version")
   
   // BAD: Shell interpolation (vulnerable to injection)
   Command::new("sh").arg("-c").arg(format!("jcmd {} VM.version", pid))
   ```

3. **Timeout Protection**: Always set execution timeouts
   ```rust
   timeout(Duration::from_secs(5), command.output()).await
   ```
