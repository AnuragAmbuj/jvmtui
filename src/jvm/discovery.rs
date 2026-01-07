use crate::error::Result;
use crate::jvm::jdk_tools::detector::{JdkToolsStatus, ToolStatus};
use crate::jvm::jdk_tools::executor::execute_command;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DiscoveredJvm {
    pub pid: u32,
    pub main_class: String,
}

pub async fn discover_local_jvms() -> Result<Vec<DiscoveredJvm>> {
    let status = JdkToolsStatus::detect();

    if let ToolStatus::Available { path, .. } = &status.jcmd {
        discover_via_jcmd(path).await
    } else if let ToolStatus::Available { path, .. } = &status.jps {
        discover_via_jps(path).await
    } else {
        Err(crate::error::AppError::Config(
            "No JDK tools available for JVM discovery".to_string(),
        ))
    }
}

async fn discover_via_jcmd(jcmd_path: &PathBuf) -> Result<Vec<DiscoveredJvm>> {
    let output = execute_command(
        jcmd_path.to_str().unwrap(),
        &["-l"],
        Some(std::time::Duration::from_secs(2)),
    )
    .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_jcmd_list(&stdout))
}

async fn discover_via_jps(jps_path: &PathBuf) -> Result<Vec<DiscoveredJvm>> {
    let output = execute_command(
        jps_path.to_str().unwrap(),
        &["-l"],
        Some(std::time::Duration::from_secs(2)),
    )
    .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_jps_list(&stdout))
}

fn parse_jcmd_list(output: &str) -> Vec<DiscoveredJvm> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                return None;
            }

            let pid = parts[0].parse::<u32>().ok()?;
            let main_class = parts[1].to_string();

            if should_filter(&main_class) {
                return None;
            }

            Some(DiscoveredJvm { pid, main_class })
        })
        .collect()
}

fn parse_jps_list(output: &str) -> Vec<DiscoveredJvm> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                return None;
            }

            let pid = parts[0].parse::<u32>().ok()?;
            let main_class = parts[1].to_string();

            if should_filter(&main_class) {
                return None;
            }

            Some(DiscoveredJvm { pid, main_class })
        })
        .collect()
}

fn should_filter(main_class: &str) -> bool {
    main_class.contains("jdk.jcmd")
        || main_class.contains("sun.tools.jcmd.JCmd")
        || main_class.contains("sun.tools.jps.Jps")
        || main_class.contains("sun.tools.jstat.Jstat")
        || main_class == "Jps"
        || main_class == "JCmd"
        || main_class == "Jstat"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jcmd_list() {
        let output = "46168 com.intellij.idea.Main
3852 jdk.jcmd/sun.tools.jcmd.JCmd -l
48127 /path/to/sonarlint-ls.jar -stdio
12345 MyApplication";

        let jvms = parse_jcmd_list(output);

        assert_eq!(jvms.len(), 3);
        assert_eq!(jvms[0].pid, 46168);
        assert_eq!(jvms[0].main_class, "com.intellij.idea.Main");
        assert_eq!(jvms[1].pid, 48127);
        assert_eq!(jvms[2].pid, 12345);
        assert_eq!(jvms[2].main_class, "MyApplication");
    }

    #[test]
    fn test_parse_jps_list() {
        let output = "12345 MyApplication
67890 com.example.Service
3852 Jps";

        let jvms = parse_jps_list(output);

        assert_eq!(jvms.len(), 2);
        assert_eq!(jvms[0].pid, 12345);
        assert_eq!(jvms[1].pid, 67890);
    }

    #[test]
    fn test_filter_jdk_tools() {
        assert!(should_filter("jdk.jcmd/sun.tools.jcmd.JCmd"));
        assert!(should_filter("sun.tools.jps.Jps"));
        assert!(should_filter("Jps"));
        assert!(should_filter("JCmd"));
        assert!(!should_filter("com.intellij.idea.Main"));
        assert!(!should_filter("MyApplication"));
    }

    #[tokio::test]
    async fn test_discover_local_jvms() {
        let jvms = discover_local_jvms().await.unwrap();
        println!("Discovered {} JVMs:", jvms.len());
        for jvm in &jvms {
            println!("  {} - {}", jvm.pid, jvm.main_class);
        }

        assert!(!jvms.iter().any(|j| should_filter(&j.main_class)));
    }
}
