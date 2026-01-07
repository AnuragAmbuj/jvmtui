use crate::jvm::types::{
    ClassInfo, HeapInfo, MemoryPool, PoolType, StackFrame, ThreadInfo, ThreadState,
};
use once_cell::sync::Lazy;
use regex::Regex;

static HEAP_TOTAL_USED: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"total\s+(\d+)K,\s+used\s+(\d+)K").unwrap());

static METASPACE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Metaspace\s+used\s+(\d+)K,\s+committed\s+(\d+)K,\s+reserved\s+(\d+)K").unwrap()
});

static CLASS_SPACE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"class space\s+used\s+(\d+)K,\s+committed\s+(\d+)K,\s+reserved\s+(\d+)K").unwrap()
});

static UPTIME: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+\.\d+)\s+s").unwrap());

static THREAD_HEADER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""([^"]+)"\s+#(\d+).*tid=0x[0-9a-f]+\s+nid=\d+\s+(.*)\s+\["#).unwrap()
});

static THREAD_STATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"java\.lang\.Thread\.State:\s+(\w+)").unwrap());

static STACK_FRAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s+at\s+([a-zA-Z0-9_.$<>]+)\.([a-zA-Z0-9_<>]+)\((?:([^:)]+):(\d+)|([^)]+))\)")
        .unwrap()
});

static CLASS_HISTOGRAM_LINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(\d+):\s+(\d+)\s+(\d+)\s+(.+?)\s*(?:\(.*\))?$").unwrap());

pub fn parse_heap_info(output: &str) -> Result<HeapInfo, String> {
    let mut used_bytes = 0u64;
    let mut max_bytes = 0u64;
    let mut committed_bytes = 0u64;
    let mut pools = Vec::new();

    for line in output.lines() {
        if let Some(caps) = HEAP_TOTAL_USED.captures(line) {
            max_bytes = caps[1].parse::<u64>().unwrap() * 1024;
            used_bytes = caps[2].parse::<u64>().unwrap() * 1024;
            committed_bytes = max_bytes;
        }

        if let Some(caps) = METASPACE.captures(line) {
            let used = caps[1].parse::<u64>().unwrap() * 1024;
            let committed = caps[2].parse::<u64>().unwrap() * 1024;
            let reserved = caps[3].parse::<u64>().unwrap() * 1024;

            pools.push(MemoryPool {
                name: "Metaspace".to_string(),
                pool_type: PoolType::Metaspace,
                used_bytes: used,
                max_bytes: reserved,
                committed_bytes: committed,
            });
        }

        if let Some(caps) = CLASS_SPACE.captures(line) {
            let used = caps[1].parse::<u64>().unwrap() * 1024;
            let committed = caps[2].parse::<u64>().unwrap() * 1024;
            let reserved = caps[3].parse::<u64>().unwrap() * 1024;

            pools.push(MemoryPool {
                name: "Class Space".to_string(),
                pool_type: PoolType::Metaspace,
                used_bytes: used,
                max_bytes: reserved,
                committed_bytes: committed,
            });
        }
    }

    if used_bytes == 0 && max_bytes == 0 {
        return Err("Failed to parse heap info".to_string());
    }

    Ok(HeapInfo {
        used_bytes,
        max_bytes,
        committed_bytes,
        pools,
    })
}

pub fn parse_jvm_version(output: &str) -> Result<String, String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("JDK ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return Ok(parts[1].to_string());
            }
        }
    }
    Err("Failed to parse JVM version".to_string())
}

pub fn parse_vm_uptime(output: &str) -> Result<u64, String> {
    for line in output.lines() {
        if let Some(caps) = UPTIME.captures(line) {
            let seconds = caps[1]
                .parse::<f64>()
                .map_err(|e| format!("Failed to parse uptime: {}", e))?;
            return Ok(seconds as u64);
        }
    }
    Err("Failed to parse VM uptime".to_string())
}

pub fn parse_vm_flags(output: &str) -> Result<Vec<String>, String> {
    let mut flags = Vec::new();

    for line in output.lines() {
        if line.contains("-XX:") || line.contains("-Xms") || line.contains("-Xmx") {
            for token in line.split_whitespace() {
                if token.starts_with('-') {
                    flags.push(token.to_string());
                }
            }
        }
    }

    if flags.is_empty() {
        return Err("No VM flags found".to_string());
    }

    Ok(flags)
}

pub fn parse_thread_dump(output: &str) -> Result<Vec<ThreadInfo>, String> {
    let mut threads = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for thread header line
        if let Some(caps) = THREAD_HEADER.captures(line) {
            let name = caps[1].to_string();
            let id = caps[2]
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse thread id: {}", e))?;

            // Parse thread state from next few lines
            let mut state = ThreadState::Runnable;
            let mut stack_trace = Vec::new();

            // Look ahead for thread state and stack frames
            let mut j = i + 1;
            while j < lines.len() && j < i + 100 {
                // Limit lookahead
                let next_line = lines[j];

                // Check if we hit the next thread
                if THREAD_HEADER.is_match(next_line) {
                    break;
                }

                // Parse thread state
                if let Some(state_caps) = THREAD_STATE.captures(next_line) {
                    state = match state_caps[1].as_ref() {
                        "RUNNABLE" => ThreadState::Runnable,
                        "BLOCKED" => ThreadState::Blocked,
                        "WAITING" => ThreadState::Waiting,
                        "TIMED_WAITING" => ThreadState::TimedWaiting,
                        "TERMINATED" => ThreadState::Terminated,
                        "NEW" => ThreadState::New,
                        _ => ThreadState::Runnable,
                    };
                }

                // Parse stack frame
                if let Some(frame_caps) = STACK_FRAME.captures(next_line) {
                    let class_name = frame_caps[1].to_string();
                    let method_name = frame_caps[2].to_string();

                    let (file_name, line_number) = if frame_caps.get(3).is_some() {
                        (
                            Some(frame_caps[3].to_string()),
                            frame_caps[4].parse::<u32>().ok(),
                        )
                    } else {
                        (frame_caps.get(5).map(|m| m.as_str().to_string()), None)
                    };

                    stack_trace.push(StackFrame {
                        class_name,
                        method_name,
                        file_name,
                        line_number,
                    });
                }

                j += 1;
            }

            threads.push(ThreadInfo {
                id,
                name,
                state,
                stack_trace,
            });

            i = j;
        } else {
            i += 1;
        }
    }

    if threads.is_empty() {
        return Err("No threads found in dump".to_string());
    }

    Ok(threads)
}

pub fn parse_class_histogram(output: &str) -> Result<Vec<ClassInfo>, String> {
    let mut classes = Vec::new();

    for line in output.lines() {
        if let Some(caps) = CLASS_HISTOGRAM_LINE.captures(line) {
            let rank = caps[1]
                .parse::<u32>()
                .map_err(|e| format!("Failed to parse rank: {}", e))?;
            let instances = caps[2]
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse instances: {}", e))?;
            let bytes = caps[3]
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse bytes: {}", e))?;
            let name = caps[4].trim().to_string();

            classes.push(ClassInfo {
                rank,
                instances,
                bytes,
                name,
            });
        }
    }

    if classes.is_empty() {
        return Err("No classes found in histogram".to_string());
    }

    Ok(classes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heap_info() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd_heap_info.txt");
        let heap = parse_heap_info(output).unwrap();

        assert_eq!(heap.used_bytes, 618493 * 1024);
        assert_eq!(heap.max_bytes, 798720 * 1024);
        assert_eq!(heap.pools.len(), 2);

        let metaspace = &heap.pools[0];
        assert_eq!(metaspace.name, "Metaspace");
        assert_eq!(metaspace.used_bytes, 505590 * 1024);
    }

    #[test]
    fn test_parse_jvm_version() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd_vm_version.txt");
        let version = parse_jvm_version(output).unwrap();
        assert_eq!(version, "21.0.8");
    }

    #[test]
    fn test_parse_vm_uptime() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd_vm_uptime.txt");
        let uptime = parse_vm_uptime(output).unwrap();
        assert_eq!(uptime, 390327);
    }

    #[test]
    fn test_parse_vm_flags() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd_vm_flags.txt");
        let flags = parse_vm_flags(output).unwrap();

        assert!(flags.len() > 10);
        assert!(flags.contains(&"-XX:+UseG1GC".to_string()));
        assert!(flags.contains(&"-XX:+HeapDumpOnOutOfMemoryError".to_string()));
    }

    #[test]
    fn test_parse_thread_dump() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd_thread_print.txt");
        let threads = parse_thread_dump(output).unwrap();

        assert!(!threads.is_empty());
        assert!(threads.len() > 10);

        let main_thread = threads.iter().find(|t| t.name == "main");
        assert!(main_thread.is_some());

        let main = main_thread.unwrap();
        assert!(matches!(
            main.state,
            ThreadState::TimedWaiting | ThreadState::Waiting
        ));
        assert!(!main.stack_trace.is_empty());

        let first_frame = &main.stack_trace[0];
        assert!(
            first_frame.class_name.contains("Unsafe") || first_frame.class_name.contains("misc")
        );
    }

    #[test]
    fn test_parse_class_histogram() {
        let output = include_str!("../../../../assets/sample_outputs/jcmd_class_histogram.txt");
        let classes = parse_class_histogram(output).unwrap();

        assert!(!classes.is_empty());
        assert!(classes.len() > 100);

        let first_class = &classes[0];
        assert_eq!(first_class.rank, 1);
        assert!(first_class.instances > 0);
        assert!(first_class.bytes > 0);
        assert!(!first_class.name.is_empty());

        let byte_array = classes.iter().find(|c| c.name.contains("[B"));
        assert!(byte_array.is_some());
    }
}
