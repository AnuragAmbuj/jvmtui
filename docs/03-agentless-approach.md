# Agentless JVM Monitoring

This document explains how JVM-TUI monitors JVMs without requiring any agent deployment.

## Why Agentless?

| Approach | Pros | Cons |
|----------|------|------|
| **Agentless (jcmd/jstat)** | No deployment, no JVM restart, works everywhere | Requires JDK tools in PATH |
| **Jolokia Agent** | Rich HTTP/JSON API, remote-friendly | Requires agent deployment in target JVM |
| **JMX RMI** | Standard Java | Complex protocol, firewall issues |
| **JNI** | Full access | Crash risk, complex build |

**Our choice: Agentless-first** because:
- Production JVMs often can't be restarted to add agents
- Security policies may prohibit agent deployment
- Zero friction for local development
- JDK tools are usually already available

## How JDK Tools Work

### The Attach API

When you run `jcmd <pid> <command>`, here's what happens:

```
┌─────────────┐         ┌─────────────────────────────┐
│    jcmd     │         │        Target JVM           │
│  (process)  │         │                             │
└──────┬──────┘         │  ┌─────────────────────┐   │
       │                │  │   Attach Listener   │   │
       │  1. Connect    │  │   (internal thread) │   │
       ├───────────────▶│  └──────────┬──────────┘   │
       │                │             │              │
       │  2. Send cmd   │             │              │
       ├───────────────▶│             ▼              │
       │                │  ┌─────────────────────┐   │
       │                │  │  Command Executor   │   │
       │                │  └──────────┬──────────┘   │
       │  3. Response   │             │              │
       │◀───────────────┤◀────────────┘              │
       │                │                             │
└──────┴──────┘         └─────────────────────────────┘
```

**Communication channel:**
- **Linux/macOS**: Unix domain socket at `/tmp/.java_pidXXXX`
- **Windows**: Named pipe
- **macOS (newer)**: Mach ports

### jps / jcmd -l: JVM Discovery

```bash
$ jcmd -l
76660 com.intellij.idea.Main
48127 /path/to/app.jar
```

**How it works:**
1. Scans `/tmp/hsperfdata_<user>/` directory
2. Each file is named by PID
3. Contains performance counters (memory-mapped)
4. Or uses Attach API to query running JVMs

### jstat: GC Statistics

```bash
$ jstat -gcutil 76660
  S0     S1     E      O      M     CCS    YGC     YGCT    FGC    FGCT     CGC    CGCT       GCT
   -      -   1.52  69.85  98.62  95.69    695    7.803     1    0.236   436    4.121    12.160
```

**Data source:** Reads from hsperfdata memory-mapped files (no JVM interaction needed).

### jcmd: Diagnostic Commands

```bash
$ jcmd 76660 GC.heap_info
garbage-first heap   total 2097152K, used 2034889K [0x..., 0x...)
  region size 1024K, 436 young (446464K), 4 survivors (4096K)
 Metaspace       used 422035K, committed 427968K, reserved 1441792K
  class space    used 56631K, committed 59200K, reserved 1048576K
```

**Data source:** Attach API → JVM internal command execution.

## Available Data via Agentless Tools

### jcmd Commands We Use

| Command | Data Retrieved | Use Case |
|---------|----------------|----------|
| `jcmd -l` | PID + main class | JVM discovery |
| `jcmd <pid> VM.version` | JVM version, vendor | Overview |
| `jcmd <pid> VM.uptime` | Uptime in seconds | Overview |
| `jcmd <pid> VM.flags` | All JVM flags | Overview, GC type |
| `jcmd <pid> GC.heap_info` | Heap usage, regions | Memory view |
| `jcmd <pid> Thread.print` | Full thread dump | Thread view |
| `jcmd <pid> GC.class_histogram` | Top memory consumers | Classes view |
| `jcmd <pid> GC.run` | Trigger GC | Action |
| `jcmd <pid> VM.info` | Comprehensive VM info | Detailed view |

### jstat Options We Use

| Option | Columns | Update Frequency |
|--------|---------|------------------|
| `-gcutil` | S0, S1, E, O, M, CCS, YGC, YGCT, FGC, FGCT, CGC, CGCT, GCT | Every poll |
| `-gc` | Sizes in KB for each region | Every poll |
| `-gccapacity` | Capacity info | On demand |
| `-class` | Class loading stats | Every poll |

### Data Mapping

```
┌────────────────────────────────────────────────────────────────┐
│                     jstat -gcutil Output                       │
├────────────────────────────────────────────────────────────────┤
│  S0     S1     E      O      M     CCS    YGC   YGCT   ...    │
│   -      -   1.52  69.85  98.62  95.69   695   7.803  ...    │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                     GcStats Struct                             │
├────────────────────────────────────────────────────────────────┤
│  eden_pct: 1.52                                                │
│  old_pct: 69.85                                                │
│  metaspace_pct: 98.62                                          │
│  young_gc_count: 695                                           │
│  young_gc_time_sec: 7.803                                      │
│  ...                                                           │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                     TUI Rendering                              │
├────────────────────────────────────────────────────────────────┤
│  ┌─ GC Statistics ──────────────────────────────────────────┐  │
│  │ Young GC: 695 collections (7.8s total)                   │  │
│  │ Old GC:   1 collection (0.2s total)                      │  │
│  │ Eden: ▓░░░░░░░░░ 1.5%   Old: ▓▓▓▓▓▓▓░░░ 69.8%           │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

## Polling Strategy

### What to Poll Frequently (every tick)

| Data | Command | Rationale |
|------|---------|-----------|
| GC stats | `jstat -gcutil` | Lightweight, memory-mapped |
| Heap info | `jcmd GC.heap_info` | Core metric, fast |
| Uptime | `jcmd VM.uptime` | Verify connection alive |

### What to Cache (once on connect)

| Data | Command | Rationale |
|------|---------|-----------|
| VM version | `jcmd VM.version` | Never changes |
| VM flags | `jcmd VM.flags` | Never changes |
| VM info | `jcmd VM.info` | Rarely changes |

### What to Fetch On-Demand

| Data | Command | Rationale |
|------|---------|-----------|
| Thread dump | `jcmd Thread.print` | Expensive, user-triggered |
| Class histogram | `jcmd GC.class_histogram` | Very expensive |
| Full VM info | `jcmd VM.info` | Large output |

## Limitations of Agentless Approach

| Limitation | Impact | Workaround |
|------------|--------|------------|
| JDK tools required | Won't work with JRE only | Detect and show install guidance |
| Same-user access | Can't monitor other users' JVMs | Run as root or same user |
| Subprocess overhead | ~50ms per command | Parallel execution, caching |
| No real-time events | Polling only | Configurable interval (250ms min) |
| Limited thread CPU | No per-thread CPU time | Use JFR in Phase 3 |

## Fallback Chain

```
┌─────────────────────────────────────────────────────────────────┐
│                     Tool Detection Flow                         │
└─────────────────────────────────────────────────────────────────┘

1. Check for jcmd in $JAVA_HOME/bin
   │
   ├─ Found → Use jcmd for everything
   │
   └─ Not found → Check PATH
                   │
                   ├─ Found → Use jcmd
                   │
                   └─ Not found → Check for jstat + jps
                                   │
                                   ├─ Found → Limited mode (no thread dumps)
                                   │
                                   └─ Not found → Show install guidance
                                                   │
                                                   └─ Offer Jolokia as alternative
```

## Example Session

```bash
# JVM-TUI auto-discovers local JVMs
$ jvm-tui

# Behind the scenes:
# 1. Run: jcmd -l
# 2. Parse output, present picker
# 3. User selects PID 76660
# 4. Run: jcmd 76660 VM.version (cache)
# 5. Run: jcmd 76660 VM.flags (cache)
# 6. Start polling loop:
#    - jstat -gcutil 76660
#    - jcmd 76660 GC.heap_info
#    - jcmd 76660 VM.uptime
# 7. Render TUI with collected data
```

## Security Notes

1. **No network exposure**: All communication is local (Unix sockets)
2. **Process isolation**: Can only attach to JVMs owned by same user
3. **Read-mostly**: Most operations are read-only
4. **Explicit actions**: GC trigger requires confirmation

## Comparison with Jolokia

| Feature | Agentless (jcmd/jstat) | Jolokia |
|---------|------------------------|---------|
| Setup required | None (if JDK installed) | Deploy agent |
| Remote access | No (local only) | Yes (HTTP) |
| Data richness | Good | Excellent (full MBeans) |
| Latency | ~50ms | ~10ms |
| Thread dump | Yes | Yes |
| GC trigger | Yes | Yes |
| MBean browser | No | Yes |
| JFR control | Yes (jcmd JFR.*) | Limited |

**Recommendation**: Use agentless for local, Jolokia for remote.
