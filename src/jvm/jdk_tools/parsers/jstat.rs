use crate::jvm::types::GcStats;
use chrono::Local;

pub fn parse_gc_stats(output: &str) -> Result<GcStats, String> {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 {
        return Err("Invalid jstat output format".to_string());
    }

    let data_line = lines[1].trim();
    let values: Vec<&str> = data_line.split_whitespace().collect();

    if values.len() < 13 {
        return Err(format!("Expected at least 13 values, got {}", values.len()));
    }

    let young_gc_count = values[6]
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse YGC: {}", e))?;

    let young_gc_time = values[7]
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse YGCT: {}", e))?;

    let full_gc_count = values[8]
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse FGC: {}", e))?;

    let full_gc_time = values[9]
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse FGCT: {}", e))?;

    Ok(GcStats {
        young_gc_count,
        young_gc_time_ms: (young_gc_time * 1000.0) as u64,
        old_gc_count: full_gc_count,
        old_gc_time_ms: (full_gc_time * 1000.0) as u64,
        timestamp: Local::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gc_stats() {
        let output = include_str!("../../../../assets/sample_outputs/jstat_gcutil.txt");
        let stats = parse_gc_stats(output).unwrap();

        assert_eq!(stats.young_gc_count, 125387);
        assert_eq!(stats.young_gc_time_ms, 497699);
        assert_eq!(stats.old_gc_count, 37);
        assert_eq!(stats.old_gc_time_ms, 9222);
    }

    #[test]
    fn test_parse_invalid_format() {
        let output = "invalid output";
        let result = parse_gc_stats(output);
        assert!(result.is_err());
    }
}
