use crate::jvm::types::{HeapInfo, MemoryPool, PoolType};
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
}
