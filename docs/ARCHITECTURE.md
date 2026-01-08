# Architecture & Design Decisions

## Connection Types

### Why Multiple Connection Methods?

JVM-TUI supports three connection types to cover different deployment scenarios:

| Scenario | Connection Type | Reason |
|----------|----------------|--------|
| Local development | **Local JVM** | Zero config, auto-discovery |
| Remote server with SSH | **SSH+JDK** | No agent needed, uses SSH |
| Remote server with Jolokia | **Jolokia HTTP** | Firewall-friendly, HTTP/HTTPS |

---

## Why Not Native JMX?

### The JMX Dilemma

Native JMX (Java Management Extensions) is the standard Java monitoring API. However, implementing JMX in Rust poses significant challenges:

#### JMX Protocol Requirements
- **RMI (Remote Method Invocation)**: Java-specific remote procedure call protocol
- **Java Serialization**: Binary serialization format tied to Java runtime
- **JRMP (Java Remote Method Protocol)**: RMI wire protocol
- **Dynamic Class Loading**: JMX can send class definitions over the wire

#### Existing Rust Option: `jmx` Crate

The [`jmx` crate](https://crates.io/crates/jmx) exists but has critical limitations:

```toml
# jmx crate dependency chain
jmx = "0.2.1"
  └── j4rs = "0.11.2"  # "Java for Rust" - JNI wrapper
      └── Requires: Java Runtime Environment (JRE)
```

**How it works:**
```
Rust Application → JNI → Java JMX Client → Remote JVM
      (jmx crate)        (embedded Java)
```

**Problems with this approach:**

1. **JRE Dependency**
   - Requires Java Runtime on the monitoring machine
   - Defeats the purpose of a lightweight Rust binary
   - Binary size increases significantly (JRE is 100MB+)

2. **Maintenance Concerns**
   - Last updated: June 2020 (4.5 years stale)
   - Depends on `j4rs` which adds complexity
   - JNI overhead and error handling complexity

3. **Deployment Complexity**
   - Users must install JRE separately
   - JNI configuration and setup
   - Platform-specific JRE paths and versions

4. **Performance Overhead**
   - JNI boundary crossings
   - Java garbage collection affecting Rust app
   - Multiple serialization/deserialization steps

### Our Solution: SSH+JDK Connector

Instead of embedding Java runtime, we execute JDK tools remotely over SSH:

```
Rust Application → SSH → Remote Server → jcmd/jstat → Remote JVM
     (pure Rust)        (standard SSH)      (JDK tools)
```

**Advantages:**

✅ **Pure Rust**: No JRE dependency on monitoring machine
✅ **Standard Protocol**: Uses SSH (port 22), universally available
✅ **No Agent**: Remote server only needs JDK tools (already present)
✅ **Smaller Binary**: ~3MB vs 100MB+ with JRE
✅ **Easier Deployment**: Single binary, no Java installation
✅ **Better Maintained**: Uses actively maintained SSH libraries

**Trade-offs:**

⚠️ **Requires SSH Access**: Need SSH credentials to remote server
⚠️ **Latency**: SSH overhead vs direct JMX connection
✅ **Acceptable**: Polling interval (1s default) makes latency negligible

---

## Connection Architecture

### Local JVM Monitoring

```
┌─────────────┐
│  JVM-TUI    │
│  (Rust)     │
└──────┬──────┘
       │
       │ process::Command
       │
┌──────▼──────────────────┐
│  Local JDK Tools        │
│  - jcmd <pid> <command> │
│  - jstat -gc <pid>      │
└──────┬──────────────────┘
       │
       │ stdout parsing
       │
┌──────▼──────┐
│  JVM Process│
│  (same host)│
└─────────────┘
```

**Implementation:** `JdkToolsConnector`
- Spawns local processes (`jcmd`, `jstat`)
- Parses stdout into structured types
- No network overhead

---

### SSH+JDK Remote Monitoring

```
┌─────────────┐
│  JVM-TUI    │
│  (Rust)     │
└──────┬──────┘
       │
       │ async-ssh2-tokio
       │
┌──────▼──────────────────┐
│  SSH Connection         │
│  (authenticated)        │
└──────┬──────────────────┘
       │
       │ execute remote commands
       │
┌──────▼──────────────────┐
│  Remote Server          │
│  jcmd <pid> <command>   │
│  jstat -gc <pid>        │
└──────┬──────────────────┘
       │
       │ stdout over SSH
       │
┌──────▼──────┐
│  Remote JVM │
│  Process    │
└─────────────┘
```

**Implementation:** `SshJdkConnector`
- Uses `async-ssh2-tokio` for SSH client
- Authenticates via key file or password
- Executes same commands as local connector
- Reuses existing `jcmd`/`jstat` parsers
- No JRE required on monitoring machine

**Authentication Methods:**
1. SSH key file (recommended): `ssh_key = "~/.ssh/id_rsa"`
2. Password: `ssh_password = "secret"`
3. Default key fallback: `~/.ssh/id_rsa` if neither specified

---

### Jolokia HTTP Monitoring

```
┌─────────────┐
│  JVM-TUI    │
│  (Rust)     │
└──────┬──────┘
       │
       │ reqwest (HTTP client)
       │
┌──────▼──────────────────┐
│  HTTP Request           │
│  POST /jolokia          │
│  JSON-RPC payload       │
└──────┬──────────────────┘
       │
       │ HTTP/HTTPS
       │
┌──────▼──────────────────┐
│  Jolokia Agent          │
│  (JVM agent)            │
└──────┬──────────────────┘
       │
       │ JMX MBean access
       │
┌──────▼──────┐
│  Remote JVM │
│  MBeans     │
└─────────────┘
```

**Implementation:** `JolokiaConnector`
- Uses `reqwest` for HTTP client
- Sends JSON-RPC requests
- Deserializes JSON responses
- Supports HTTP Basic authentication

**Jolokia Request Example:**
```json
{
  "type": "read",
  "mbean": "java.lang:type=Memory",
  "attribute": "HeapMemoryUsage"
}
```

**Why Jolokia?**
- ✅ HTTP/HTTPS transport (firewall-friendly)
- ✅ Widely used in production (Docker, Kubernetes)
- ✅ No RMI port configuration needed
- ✅ Built-in authentication
- ⚠️ Requires agent installation (one-time setup)

---

## Export Architecture

### Multi-Format Export Design

```
┌─────────────────┐
│  MetricsStore   │
│  (in-memory)    │
└────────┬────────┘
         │
         │ clone snapshot
         │
┌────────▼────────────────────┐
│  Export Functions           │
│  - export_metrics_json()    │
│  - export_metrics_prometheus() │
│  - export_metrics_csv()     │
└────────┬────────────────────┘
         │
         │ serialize
         │
┌────────▼────────┐
│  File Output    │
│  metrics_*.ext  │
└─────────────────┘
```

**Format Selection:**
- **JSON**: Full snapshot for archival/debugging
- **Prometheus**: Time-series monitoring systems
- **CSV**: Spreadsheet analysis, data science

**Design Decisions:**

1. **In-memory snapshot before export**
   - Prevents inconsistent data during export
   - Minimal lock time on shared state

2. **Configurable export directory**
   - User-specified: `export_directory = "~/jvm-exports"`
   - Platform default: `~/.local/share/jvm-tui/` (Linux/macOS)
   - Current directory fallback

3. **Timestamped filenames**
   - Pattern: `metrics_YYYYMMDD_HHMMSS.{json,prom,csv}`
   - Prevents overwrites
   - Easy sorting by date

---

## Connector Trait Design

All connectors implement a unified `JvmConnector` trait:

```rust
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
```

**Benefits:**
- ✅ **Polymorphism**: `Arc<RwLock<dyn JvmConnector>>`
- ✅ **Extensibility**: New connectors without changing core code
- ✅ **Testability**: Mock connectors for testing
- ✅ **Type Safety**: Compile-time guarantees

**Runtime Dispatch:**
```rust
let connector_arc: Arc<RwLock<dyn JvmConnector>> = match connection_type {
    Local => Arc::new(RwLock::new(JdkToolsConnector::new())),
    SshJdk { .. } => Arc::new(RwLock::new(SshJdkConnector::new(...))),
    Jolokia { .. } => Arc::new(RwLock::new(JolokiaConnector::new(...))),
};
```

---

## Configuration System

### Design Philosophy

**Convention over Configuration** with escape hatches:

1. **Zero config for local monitoring**
   - Auto-discovers JVMs
   - Sane defaults (1s interval, 300 samples)

2. **Optional config for advanced use**
   - Save favorite connections
   - Customize intervals and export paths
   - Store credentials (with env var support)

3. **Multiple config locations**
   - CLI argument: `--config /path/to/config.toml`
   - Environment: `$JVM_TUI_CONFIG`
   - Current dir: `./config.toml`
   - XDG config: `~/.config/jvm-tui/config.toml`
   - Home dir: `~/.jvm-tui.toml`

### Environment Variable Expansion

Supports both `${VAR}` and `~` expansion:

```toml
export_directory = "~/jvm-exports"           # → /home/user/jvm-exports
ssh_key = "${HOME}/.ssh/production"          # → /home/user/.ssh/production
password = "${JOLOKIA_PASS}"                 # → value from env var
```

**Security Note:** Credentials in config should use env vars, not plain text.

---

## Performance Considerations

### Async Architecture

```
┌────────────────────┐
│  Main Event Loop   │
│  (TUI rendering)   │
└────────────────────┘
         │
         │ async
         │
┌────────▼─────────────┐
│  MetricsCollector    │
│  (background task)   │
└────────┬─────────────┘
         │
         │ polls every interval
         │
┌────────▼─────────────┐
│  JvmConnector        │
│  (local/SSH/HTTP)    │
└──────────────────────┘
```

**Benefits:**
- ✅ **Non-blocking UI**: TUI remains responsive during collection
- ✅ **Configurable interval**: Balance freshness vs overhead
- ✅ **Graceful errors**: Connection failures don't crash UI

### Memory Management

**Ring Buffer for History:**
- Fixed-size circular buffer (default: 300 samples)
- Constant memory usage regardless of uptime
- Automatic eviction of old data

**Clone on Export:**
- Minimal lock time on shared state
- Export doesn't block metric collection
- Snapshot ensures consistency

---

## Error Handling Strategy

### Connection Errors

**Automatic Reconnection:**
```rust
if connector.is_connected() == false {
    connector.reconnect().await?;
}
```

**User-visible Errors:**
- SSH auth failures → Show error, suggest checking credentials
- Network timeouts → Show error, suggest checking connectivity
- Parse failures → Log details, continue with partial data

### Graceful Degradation

**Principle:** Show what you can, hide what's broken.

Example: If thread dump fails, show:
- ✅ Heap info (if available)
- ✅ GC stats (if available)
- ❌ Threads view shows error message
- ✅ User can retry or switch tabs

---

## Future Considerations

### Why Not SSH+Jolokia Tunnel?

SSH+Jolokia connector is **configured but not implemented** because:

1. **SSH+JDK covers same use case**
   - Both require SSH access
   - SSH+JDK needs no agent installation
   - SSH+JDK is simpler (no HTTP layer)

2. **Jolokia HTTP works directly**
   - If Jolokia is installed, use direct HTTP
   - No need for SSH tunnel complexity

3. **Niche use case**
   - Scenario: Jolokia on server, but port 8080 blocked
   - Solution: Change Jolokia port or use SSH+JDK
   - Added complexity for rare scenario

**When to implement:** If users request it for strict firewall environments.

---

## Comparison to Other Tools

| Tool | Language | JRE Required | Remote Support | Agent Required |
|------|----------|--------------|----------------|----------------|
| **JVM-TUI** | Rust | ❌ No | ✅ SSH+JDK, Jolokia | ❌ No (SSH mode) |
| VisualVM | Java | ✅ Yes | ✅ JMX | ❌ No |
| JConsole | Java | ✅ Yes | ✅ JMX | ❌ No |
| Mission Control | Java | ✅ Yes | ✅ JMX, JFR | ❌ No |
| Arthas | Java | ✅ Yes | ✅ Agent attach | ✅ Yes |
| Async Profiler | C++ | ❌ No | ❌ Local only | ✅ Yes (attach) |

**JVM-TUI's Niche:**
- ✅ Pure Rust (no JRE)
- ✅ Remote monitoring without JRE on monitoring machine
- ✅ Terminal UI (SSH-friendly)
- ✅ Lightweight binary (~3MB)
- ✅ Agent-free option (SSH+JDK)

---

## Lessons Learned

### 1. Pure Rust is Worth It

Avoiding JNI/JRE dependency makes deployment significantly easier:
- Single binary distribution
- No Java version conflicts
- Smaller memory footprint
- Cleaner error messages

### 2. SSH is Ubiquitous

SSH is already configured in production environments:
- No new firewall rules
- Existing authentication infrastructure
- Standard port (22) usually open
- Audit trails already in place

### 3. Multiple Connectors > One-Size-Fits-All

Different environments have different constraints:
- Local dev: Want zero config
- Remote SSH: Want no agent installation
- Production HTTP: Want standard monitoring ports

Supporting all three covers 99% of use cases.

### 4. Configuration Should Be Optional

Best user experience:
1. Works perfectly with zero config (local monitoring)
2. One config line per saved connection (optional)
3. Advanced config for power users (intervals, exports)

**Anti-pattern:** Requiring config file before first use.

---

## References

- [JMX Specification](https://www.oracle.com/java/technologies/javase/javamanagement.html)
- [Jolokia Protocol](https://jolokia.org/reference/html/protocol.html)
- [`jmx` Rust crate](https://crates.io/crates/jmx)
- [`j4rs` - Java for Rust](https://crates.io/crates/j4rs)
- [async-ssh2-tokio](https://crates.io/crates/async-ssh2-tokio)
