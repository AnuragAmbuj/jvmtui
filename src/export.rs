use crate::error::Result;
use crate::jvm::types::ThreadInfo;
use crate::metrics::store::MetricsStore;
use chrono::Local;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub enum ExportFormat {
    Json,
    Prometheus,
    Csv,
}

pub fn export_thread_dump(threads: &[ThreadInfo], base_dir: Option<&str>) -> Result<PathBuf> {
    let dir = if let Some(custom_dir) = base_dir {
        PathBuf::from(shellexpand::tilde(custom_dir).to_string())
    } else {
        directories::ProjectDirs::from("com", "jvmtui", "JVM-TUI")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    std::fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("thread_dump_{}.txt", timestamp);
    let filepath = dir.join(&filename);

    let mut file = File::create(&filepath)?;

    writeln!(file, "JVM-TUI Thread Dump")?;
    writeln!(file, "Generated: {}", Local::now())?;
    writeln!(file, "Total Threads: {}\n", threads.len())?;
    writeln!(file, "{}", "=".repeat(80))?;
    writeln!(file)?;

    for thread in threads {
        writeln!(file, "Thread #{}: \"{}\"", thread.id, thread.name)?;
        writeln!(file, "  State: {:?}", thread.state)?;
        writeln!(file, "  Stack Trace ({} frames):", thread.stack_trace.len())?;

        for (i, frame) in thread.stack_trace.iter().enumerate() {
            let location = if let (Some(file), Some(line)) = (&frame.file_name, frame.line_number) {
                format!("({}:{})", file, line)
            } else if let Some(file) = &frame.file_name {
                format!("({})", file)
            } else {
                String::from("(Unknown Source)")
            };

            writeln!(
                file,
                "    #{}: {}.{} {}",
                i, frame.class_name, frame.method_name, location
            )?;
        }

        writeln!(file)?;
    }

    writeln!(file, "{}", "=".repeat(80))?;
    writeln!(file, "End of thread dump")?;

    Ok(filepath)
}

pub fn export_metrics_json(store: &MetricsStore, base_dir: Option<&str>) -> Result<PathBuf> {
    let dir = if let Some(custom_dir) = base_dir {
        PathBuf::from(shellexpand::tilde(custom_dir).to_string())
    } else {
        directories::ProjectDirs::from("com", "jvmtui", "JVM-TUI")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    std::fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("metrics_{}.json", timestamp);
    let filepath = dir.join(&filename);

    let json = serde_json::to_string_pretty(&store)?;
    std::fs::write(&filepath, json)?;

    Ok(filepath)
}

pub fn export_metrics_prometheus(store: &MetricsStore, base_dir: Option<&str>) -> Result<PathBuf> {
    let dir = if let Some(custom_dir) = base_dir {
        PathBuf::from(shellexpand::tilde(custom_dir).to_string())
    } else {
        directories::ProjectDirs::from("com", "jvmtui", "JVM-TUI")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    std::fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("metrics_{}.prom", timestamp);
    let filepath = dir.join(&filename);

    let mut file = File::create(&filepath)?;

    writeln!(file, "# JVM-TUI Metrics Export")?;
    writeln!(file, "# Generated: {}", Local::now())?;
    writeln!(file)?;

    if let Some(heap) = store.heap_history.iter().last() {
        writeln!(
            file,
            "# HELP jvm_memory_heap_used_bytes Heap memory used in bytes"
        )?;
        writeln!(file, "# TYPE jvm_memory_heap_used_bytes gauge")?;
        writeln!(file, "jvm_memory_heap_used_bytes {}", heap.used_bytes)?;
        writeln!(file)?;

        writeln!(
            file,
            "# HELP jvm_memory_heap_max_bytes Heap memory max in bytes"
        )?;
        writeln!(file, "# TYPE jvm_memory_heap_max_bytes gauge")?;
        writeln!(file, "jvm_memory_heap_max_bytes {}", heap.max_bytes)?;
        writeln!(file)?;

        writeln!(
            file,
            "# HELP jvm_memory_heap_committed_bytes Heap memory committed in bytes"
        )?;
        writeln!(file, "# TYPE jvm_memory_heap_committed_bytes gauge")?;
        writeln!(
            file,
            "jvm_memory_heap_committed_bytes {}",
            heap.committed_bytes
        )?;
        writeln!(file)?;
    }

    if let Some(gc) = store.gc_history.iter().last() {
        writeln!(
            file,
            "# HELP jvm_gc_collections_total Total number of GC collections"
        )?;
        writeln!(file, "# TYPE jvm_gc_collections_total counter")?;
        writeln!(
            file,
            "jvm_gc_collections_total{{gc=\"young\"}} {}",
            gc.young_gc_count
        )?;
        writeln!(
            file,
            "jvm_gc_collections_total{{gc=\"old\"}} {}",
            gc.old_gc_count
        )?;
        writeln!(file)?;

        writeln!(
            file,
            "# HELP jvm_gc_time_seconds_total Total time spent in GC in seconds"
        )?;
        writeln!(file, "# TYPE jvm_gc_time_seconds_total counter")?;
        writeln!(
            file,
            "jvm_gc_time_seconds_total{{gc=\"young\"}} {:.3}",
            gc.young_gc_time_ms as f64 / 1000.0
        )?;
        writeln!(
            file,
            "jvm_gc_time_seconds_total{{gc=\"old\"}} {:.3}",
            gc.old_gc_time_ms as f64 / 1000.0
        )?;
        writeln!(file)?;
    }

    if let Some(heap) = store.heap_history.iter().last() {
        for pool in &heap.pools {
            writeln!(
                file,
                "# HELP jvm_memory_pool_used_bytes Memory pool used in bytes"
            )?;
            writeln!(file, "# TYPE jvm_memory_pool_used_bytes gauge")?;
            writeln!(
                file,
                "jvm_memory_pool_used_bytes{{pool=\"{}\"}} {}",
                pool.name, pool.used_bytes
            )?;
            writeln!(file)?;

            writeln!(
                file,
                "# HELP jvm_memory_pool_max_bytes Memory pool max in bytes"
            )?;
            writeln!(file, "# TYPE jvm_memory_pool_max_bytes gauge")?;
            writeln!(
                file,
                "jvm_memory_pool_max_bytes{{pool=\"{}\"}} {}",
                pool.name, pool.max_bytes
            )?;
            writeln!(file)?;

            writeln!(
                file,
                "# HELP jvm_memory_pool_committed_bytes Memory pool committed in bytes"
            )?;
            writeln!(file, "# TYPE jvm_memory_pool_committed_bytes gauge")?;
            writeln!(
                file,
                "jvm_memory_pool_committed_bytes{{pool=\"{}\"}} {}",
                pool.name, pool.committed_bytes
            )?;
            writeln!(file)?;
        }
    }

    let thread_counts: std::collections::HashMap<_, _> =
        store
            .thread_snapshot
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, thread| {
                *acc.entry(format!("{:?}", thread.state)).or_insert(0) += 1;
                acc
            });

    writeln!(
        file,
        "# HELP jvm_threads_total Total number of threads by state"
    )?;
    writeln!(file, "# TYPE jvm_threads_total gauge")?;
    for (state, count) in &thread_counts {
        writeln!(file, "jvm_threads_total{{state=\"{}\"}} {}", state, count)?;
    }
    writeln!(file)?;

    writeln!(
        file,
        "# HELP jvm_classes_loaded_total Total number of classes loaded"
    )?;
    writeln!(file, "# TYPE jvm_classes_loaded_total gauge")?;
    let total_classes: u64 = store.class_histogram.iter().map(|c| c.instances).sum();
    writeln!(file, "jvm_classes_loaded_total {}", total_classes)?;
    writeln!(file)?;

    Ok(filepath)
}

pub fn export_metrics_csv(store: &MetricsStore, base_dir: Option<&str>) -> Result<PathBuf> {
    let dir = if let Some(custom_dir) = base_dir {
        PathBuf::from(shellexpand::tilde(custom_dir).to_string())
    } else {
        directories::ProjectDirs::from("com", "jvmtui", "JVM-TUI")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    std::fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("metrics_{}.csv", timestamp);
    let filepath = dir.join(&filename);

    let mut file = File::create(&filepath)?;

    writeln!(file, "metric_name,value,unit,timestamp")?;

    let ts = Local::now().to_rfc3339();

    if let Some(heap) = store.heap_history.iter().last() {
        writeln!(file, "heap_used,{},bytes,{}", heap.used_bytes, ts)?;
        writeln!(file, "heap_max,{},bytes,{}", heap.max_bytes, ts)?;
        writeln!(file, "heap_committed,{},bytes,{}", heap.committed_bytes, ts)?;
        let usage_percent = (heap.used_bytes as f64 / heap.max_bytes as f64) * 100.0;
        writeln!(
            file,
            "heap_usage_percent,{:.2},percent,{}",
            usage_percent, ts
        )?;
    }

    if let Some(gc) = store.gc_history.iter().last() {
        writeln!(file, "young_gc_count,{},count,{}", gc.young_gc_count, ts)?;
        writeln!(file, "old_gc_count,{},count,{}", gc.old_gc_count, ts)?;
        writeln!(
            file,
            "young_gc_time_ms,{},milliseconds,{}",
            gc.young_gc_time_ms, ts
        )?;
        writeln!(
            file,
            "old_gc_time_ms,{},milliseconds,{}",
            gc.old_gc_time_ms, ts
        )?;
    }

    if let Some(heap) = store.heap_history.iter().last() {
        for pool in &heap.pools {
            let pool_name = pool.name.replace(',', "_");
            writeln!(
                file,
                "pool_{}_used,{},bytes,{}",
                pool_name, pool.used_bytes, ts
            )?;
            writeln!(
                file,
                "pool_{}_max,{},bytes,{}",
                pool_name, pool.max_bytes, ts
            )?;
            writeln!(
                file,
                "pool_{}_committed,{},bytes,{}",
                pool_name, pool.committed_bytes, ts
            )?;
        }
    }

    let thread_counts: std::collections::HashMap<_, _> =
        store
            .thread_snapshot
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, thread| {
                *acc.entry(format!("{:?}", thread.state)).or_insert(0) += 1;
                acc
            });

    for (state, count) in &thread_counts {
        writeln!(
            file,
            "threads_{},{},count,{}",
            state.to_lowercase(),
            count,
            ts
        )?;
    }

    let total_classes: u64 = store.class_histogram.iter().map(|c| c.instances).sum();
    writeln!(file, "classes_loaded,{},count,{}", total_classes, ts)?;

    Ok(filepath)
}
