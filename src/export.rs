use crate::error::Result;
use crate::jvm::types::ThreadInfo;
use crate::metrics::store::MetricsStore;
use chrono::Local;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn export_thread_dump(threads: &[ThreadInfo], base_dir: Option<PathBuf>) -> Result<PathBuf> {
    let dir = base_dir.unwrap_or_else(|| {
        directories::ProjectDirs::from("com", "jvmtui", "JVM-TUI")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    });

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

pub fn export_metrics_json(store: &MetricsStore, base_dir: Option<PathBuf>) -> Result<PathBuf> {
    let dir = base_dir.unwrap_or_else(|| {
        directories::ProjectDirs::from("com", "jvmtui", "JVM-TUI")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    });

    std::fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("metrics_{}.json", timestamp);
    let filepath = dir.join(&filename);

    let json = serde_json::to_string_pretty(&store)?;
    std::fs::write(&filepath, json)?;

    Ok(filepath)
}
