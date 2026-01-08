# Phase 3 Implementation Plan

## Overview

Phase 3 focuses on **production-ready features** for remote monitoring and configuration management. JFR integration has been moved to Phase 4.

**Scope**: Items 3.1-3.4 from original Phase 3 plan
**Duration**: Estimated 2-3 weeks
**Goal**: Enable remote JVM monitoring with persistent configuration

---

## Implementation Order

We'll implement features in this order for logical dependency management:

```
1. Configuration File System (Foundation)
   ↓
2. Enhanced Export Features (Extend existing export.rs)
   ↓
3. Jolokia Connector (Remote monitoring capability)
   ↓
4. SSH Tunnel Support (Secure remote access)
```

---

## 3.1 Configuration File System

**Priority**: HIGH (foundation for other features)
**Complexity**: LOW
**Estimated Time**: 1-2 days

### Goals
- Persist user preferences between sessions
- Save JVM connection profiles
- Configure default behavior

### Tasks Breakdown

#### Task 3.1.1: Define Configuration Schema
- [ ] Create config.toml example file
- [ ] Define TOML structure:
  ```toml
  [preferences]
  default_interval = "1s"
  max_history_samples = 300
  
  [[connections]]
  name = "Production API"
  type = "jolokia"
  url = "http://prod-server:8080/jolokia"
  
  [[connections]]
  name = "Local Dev"
  type = "local"
  pid = 12345
  ```

**Files to create:**
- `config.example.toml` (example configuration)
- `docs/configuration.md` (configuration guide)

#### Task 3.1.2: Implement Config Loading
- [ ] Add `serde` and `toml` dependencies to Cargo.toml
- [ ] Create `src/config.rs` with Config struct
- [ ] Implement config file lookup (XDG dirs, home dir, current dir)
- [ ] Add config validation
- [ ] Add default config if missing

**Files to create:**
- `src/config.rs` (~150 lines)

**Code structure:**
```rust
pub struct Config {
    pub preferences: Preferences,
    pub connections: Vec<ConnectionProfile>,
}

pub struct Preferences {
    pub default_interval: Duration,
    pub max_history_samples: usize,
    pub theme: Option<String>, // for future use
}

pub enum ConnectionProfile {
    Local { name: String, pid: Option<u32> },
    Jolokia { name: String, url: String },
}
```

#### Task 3.1.3: Integrate Config into App
- [ ] Update `main.rs` to load config on startup
- [ ] Use config values for defaults
- [ ] CLI args override config values
- [ ] Add `--config <path>` CLI option

**Files to modify:**
- `src/main.rs`
- `src/cli.rs`

#### Task 3.1.4: Add Connection Picker
- [ ] Extend JVM picker to show saved connections
- [ ] Display connection type (local vs remote)
- [ ] Allow selecting saved connections
- [ ] Add "Discover local" option

**Files to modify:**
- `src/tui/screens/jvm_picker.rs`

**Acceptance Criteria:**
- [x] Config file loads from standard locations
- [x] Config values respected throughout app
- [x] CLI args override config
- [x] Missing config doesn't crash app
- [x] Saved connections appear in picker

---

## 3.2 Enhanced Export Features

**Priority**: MEDIUM (builds on existing export.rs)
**Complexity**: LOW
**Estimated Time**: 1-2 days

### Goals
- Add Prometheus format export
- Add CSV format for metrics
- Add configurable export paths
- Improve export UX

### Tasks Breakdown

#### Task 3.2.1: Prometheus Format Export
- [ ] Implement Prometheus text format
- [ ] Export current metrics snapshot
- [ ] Include help text and type info
- [ ] Add timestamp support

**Prometheus format example:**
```
# HELP jvm_memory_heap_used Heap memory used in bytes
# TYPE jvm_memory_heap_used gauge
jvm_memory_heap_used{pool="eden"} 12345678
jvm_memory_heap_used{pool="old"} 98765432
```

**Files to modify:**
- `src/export.rs` (add `export_prometheus()`)

#### Task 3.2.2: CSV Export for Metrics
- [ ] Export historical metrics to CSV
- [ ] Include headers
- [ ] Support heap, GC, and thread metrics
- [ ] Configurable time range

**CSV format example:**
```csv
timestamp,heap_used_mb,heap_max_mb,gc_young_count,gc_old_count
2024-01-08T10:00:00Z,714,817,125500,37
2024-01-08T10:00:01Z,715,817,125500,37
```

**Files to modify:**
- `src/export.rs` (add `export_metrics_csv()`)

#### Task 3.2.3: Configurable Export Paths
- [ ] Read export directory from config
- [ ] Support environment variables
- [ ] Support timestamp in filenames
- [ ] Show export location in UI

**Files to modify:**
- `src/config.rs` (add export_directory field)
- `src/export.rs` (use config directory)

#### Task 3.2.4: Enhanced Export UI
- [ ] Add export format selection dialog
- [ ] Show export progress for large datasets
- [ ] Add "e" submenu (e,j = JSON, e,p = Prometheus, e,c = CSV)
- [ ] Update help overlay

**Files to modify:**
- `src/tui/widgets/confirmation_dialog.rs` (add format selection)
- `src/tui/widgets/help_overlay.rs`
- `src/main.rs` (handle export submenu)

**Acceptance Criteria:**
- [x] Prometheus format exports correctly
- [x] CSV exports with proper timestamps
- [x] Export paths configurable
- [x] Export format selectable in UI
- [x] Help overlay updated

---

## 3.3 Jolokia Connector (Remote JVMs)

**Priority**: HIGH (core Phase 3 feature)
**Complexity**: MEDIUM-HIGH
**Estimated Time**: 3-5 days

### Goals
- Connect to remote JVMs via Jolokia HTTP/JSON protocol
- Implement JolokiaConnector trait
- Support basic authentication
- Parse Jolokia JSON responses

### Background: What is Jolokia?

Jolokia is an HTTP/JSON bridge for JMX. It provides a REST API to access JMX beans.

**Setup on target JVM:**
```bash
# Add Jolokia agent to JVM
java -javaagent:/path/to/jolokia-jvm-agent.jar=port=8778 MyApp
```

**Jolokia endpoints:**
- `GET /jolokia/read/java.lang:type=Memory/HeapMemoryUsage` - Read MBean attribute
- `POST /jolokia/exec/java.lang:type=Memory/gc` - Execute MBean operation
- `POST /jolokia` - Bulk requests

### Tasks Breakdown

#### Task 3.3.1: Add HTTP Dependencies
- [ ] Add `reqwest` with `json` feature to Cargo.toml
- [ ] Add `serde_json` for JSON parsing
- [ ] Add async HTTP client support

**Dependencies:**
```toml
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
```

#### Task 3.3.2: Define Jolokia Request/Response Types
- [ ] Create Jolokia request structures
- [ ] Create Jolokia response structures
- [ ] Add serde derives
- [ ] Handle error responses

**Files to create:**
- `src/jvm/jolokia/mod.rs`
- `src/jvm/jolokia/types.rs` (~200 lines)

**Example types:**
```rust
#[derive(Serialize)]
pub struct JolokiaRequest {
    #[serde(rename = "type")]
    pub request_type: String, // "read", "exec", "search"
    pub mbean: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
}

#[derive(Deserialize)]
pub struct JolokiaResponse<T> {
    pub status: u32,
    pub value: T,
    pub timestamp: u64,
}
```

#### Task 3.3.3: Implement JolokiaConnector
- [ ] Implement `JvmConnector` trait for `JolokiaConnector`
- [ ] Map JMX MBean paths to metrics
- [ ] Handle HTTP timeouts
- [ ] Add basic auth support
- [ ] Add connection pooling

**Files to create:**
- `src/jvm/jolokia/connector.rs` (~400 lines)

**MBean mapping:**
```rust
// Memory
"java.lang:type=Memory" -> HeapMemoryUsage, NonHeapMemoryUsage

// GC
"java.lang:type=GarbageCollector,name=*" -> Collection counts/times

// Threads
"java.lang:type=Threading" -> ThreadCount, ThreadInfo
```

#### Task 3.3.4: Add Jolokia-Specific Parsers
- [ ] Parse Memory MBean JSON
- [ ] Parse GC MBean JSON
- [ ] Parse Threading MBean JSON
- [ ] Handle different GC types (G1, ZGC, etc.)

**Files to create:**
- `src/jvm/jolokia/parsers.rs` (~300 lines)

#### Task 3.3.5: Update Connection UI
- [ ] Add "Remote (Jolokia)" connection type
- [ ] Prompt for Jolokia URL
- [ ] Optional auth credentials
- [ ] Test connection before connecting
- [ ] Show connection status

**Files to modify:**
- `src/tui/screens/jvm_picker.rs` (add remote connection flow)

#### Task 3.3.6: Error Handling
- [ ] Handle network errors gracefully
- [ ] Handle auth failures
- [ ] Handle timeout errors
- [ ] Show meaningful error messages
- [ ] Add retry logic

**Files to modify:**
- `src/error.rs` (add Jolokia error types)
- `src/tui/widgets/error_screen.rs`

**Acceptance Criteria:**
- [x] Jolokia connector implements JvmConnector trait
- [x] Can connect to remote JVM via HTTP
- [x] Memory, GC, and Thread metrics work
- [x] Basic auth supported
- [x] Network errors handled gracefully
- [x] Connection pooling works

---

## 3.4 SSH Tunnel Support

**Priority**: MEDIUM (enhances Jolokia for secure access)
**Complexity**: HIGH
**Estimated Time**: 3-4 days

### Goals
- Establish SSH tunnels automatically
- Support key-based authentication
- Tunnel Jolokia port to localhost
- Manage tunnel lifecycle

### Background: SSH Tunneling

SSH tunneling forwards a remote port to localhost:
```bash
ssh -L 8778:localhost:8778 user@remote-server
# Now localhost:8778 -> remote-server:8778
```

We'll automate this and manage the SSH connection lifecycle.

### Tasks Breakdown

#### Task 3.4.1: Add SSH Dependencies
- [ ] Add `async-ssh2-tokio` to Cargo.toml
- [ ] Add `ssh2` for SSH protocol
- [ ] Research async SSH libraries

**Dependencies to evaluate:**
```toml
# Option 1: async-ssh2-tokio
async-ssh2-tokio = "0.8"

# Option 2: thrussh (pure Rust, async)
thrussh = "0.35"
```

#### Task 3.4.2: Implement SSH Tunnel Manager
- [ ] Create SSH tunnel struct
- [ ] Establish SSH connection
- [ ] Set up port forwarding
- [ ] Keep connection alive
- [ ] Tear down on disconnect

**Files to create:**
- `src/jvm/ssh/mod.rs`
- `src/jvm/ssh/tunnel.rs` (~300 lines)

**API design:**
```rust
pub struct SshTunnel {
    session: Session,
    local_port: u16,
    remote_port: u16,
}

impl SshTunnel {
    pub async fn establish(
        host: &str,
        user: &str,
        auth: SshAuth,
        remote_port: u16,
    ) -> Result<Self>;
    
    pub fn local_port(&self) -> u16;
    pub async fn close(self) -> Result<()>;
}
```

#### Task 3.4.3: SSH Authentication
- [ ] Support password auth
- [ ] Support key-based auth (SSH agent)
- [ ] Support key file paths
- [ ] Handle passphrase-protected keys
- [ ] Store auth in config (securely)

**Files to modify:**
- `src/config.rs` (add SSH connection type)

**Config example:**
```toml
[[connections]]
name = "Production via SSH"
type = "ssh-jolokia"
ssh_host = "prod-server.example.com"
ssh_user = "jvmuser"
ssh_key = "~/.ssh/id_rsa"
jolokia_port = 8778
```

#### Task 3.4.4: Integrate SSH Tunnel with Jolokia
- [ ] Create TunneledJolokiaConnector
- [ ] Establish tunnel before connecting
- [ ] Use tunnel's local port for Jolokia
- [ ] Close tunnel on disconnect
- [ ] Handle tunnel failures

**Files to create:**
- `src/jvm/jolokia/tunneled_connector.rs` (~200 lines)

#### Task 3.4.5: Update Connection UI
- [ ] Add "Remote via SSH" connection type
- [ ] Prompt for SSH credentials
- [ ] Show tunnel status
- [ ] Test SSH connection before connecting

**Files to modify:**
- `src/tui/screens/jvm_picker.rs` (add SSH flow)

#### Task 3.4.6: Tunnel Lifecycle Management
- [ ] Spawn tunnel in background task
- [ ] Monitor tunnel health
- [ ] Reconnect on tunnel failure
- [ ] Clean up on app exit

**Files to modify:**
- `src/main.rs` (tunnel lifecycle)

**Acceptance Criteria:**
- [x] SSH tunnel establishes successfully
- [x] Port forwarding works
- [x] Key-based auth supported
- [x] Jolokia works over tunnel
- [x] Tunnel closes cleanly
- [x] Tunnel failures handled gracefully

---

## Phase 3 Testing Strategy

### Unit Tests
- [ ] Config loading/parsing tests
- [ ] Export format tests (Prometheus, CSV)
- [ ] Jolokia request/response serialization tests
- [ ] SSH tunnel mock tests

### Integration Tests
- [ ] End-to-end Jolokia connection test (requires Docker)
- [ ] SSH tunnel test (requires SSH server)
- [ ] Config file loading from different locations

### Manual Testing Checklist
- [ ] Connect to local JVM (existing)
- [ ] Connect to remote JVM via Jolokia
- [ ] Connect to remote JVM via SSH tunnel
- [ ] Save/load connections from config
- [ ] Export in all formats (JSON, Prometheus, CSV)
- [ ] Handle network failures gracefully

---

## Dependencies to Add

```toml
# Configuration
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dirs = "5.0"  # XDG directory lookup

# HTTP Client (for Jolokia)
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"

# SSH (evaluate options)
async-ssh2-tokio = "0.8"  # or thrussh = "0.35"

# Export formats
csv = "1.3"  # for CSV export
```

---

## Implementation Phases

### Week 1: Foundation
- Day 1-2: Configuration File System (3.1)
- Day 3-4: Enhanced Export Features (3.2)
- Day 5: Testing and documentation

### Week 2: Remote Monitoring
- Day 1-3: Jolokia Connector (3.3.1 - 3.3.4)
- Day 4-5: Jolokia UI Integration (3.3.5 - 3.3.6)

### Week 3: Secure Access
- Day 1-2: SSH Tunnel Manager (3.4.1 - 3.4.3)
- Day 3-4: SSH Integration (3.4.4 - 3.4.6)
- Day 5: Final testing, documentation, polish

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Config load time | < 10ms |
| Jolokia request latency | < 200ms |
| SSH tunnel establishment | < 5s |
| Export speed (1000 samples) | < 500ms |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Jolokia not available on target JVM | Provide setup instructions, fallback to local mode |
| SSH library compatibility issues | Test multiple libraries, choose most stable |
| Config file corruption | Validation, backup config on save |
| Network timeouts | Configurable timeouts, retry logic |

---

## Post-Phase 3 Deliverable

A production-ready JVM monitoring tool with:
- ✅ Local JVM monitoring (Phase 1 & 2)
- ✅ Persistent configuration
- ✅ Multiple export formats (JSON, Prometheus, CSV)
- ✅ Remote JVM monitoring via Jolokia
- ✅ Secure remote access via SSH tunnels
- ✅ Saved connection profiles
- ⬜ JFR integration (moved to Phase 4)

---

## Phase 4 Preview (JFR Integration)

Moved to Phase 4 to keep Phase 3 focused on remote monitoring:
- Start/stop JFR recordings
- Download JFR files
- Basic event viewer
- Recording management
- Integration with existing views
