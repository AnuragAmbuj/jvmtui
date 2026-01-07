use super::JdkToolsError;
use std::path::PathBuf;
use std::process::Command;

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

impl ToolStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, ToolStatus::Available { .. })
    }

    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            ToolStatus::Available { path, .. } => Some(path),
            ToolStatus::NotExecutable { path } => Some(path),
            ToolStatus::NotFound => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Capabilities {
    pub can_discover: bool,
    pub can_heap_info: bool,
    pub can_gc_stats: bool,
    pub can_thread_dump: bool,
    pub can_class_histogram: bool,
    pub can_trigger_gc: bool,
}

impl JdkToolsStatus {
    pub fn detect() -> Self {
        let java_home = std::env::var("JAVA_HOME").ok().map(PathBuf::from);

        Self {
            jcmd: detect_tool("jcmd", &java_home),
            jstat: detect_tool("jstat", &java_home),
            jps: detect_tool("jps", &java_home),
            java_home,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.jcmd.is_available() || (self.jps.is_available() && self.jstat.is_available())
    }

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

    pub fn validate(&self) -> Result<(), JdkToolsError> {
        if !self.is_usable() {
            if !self.jcmd.is_available() {
                return Err(JdkToolsError::JcmdNotFound);
            }
            if !self.jstat.is_available() {
                return Err(JdkToolsError::JstatNotFound);
            }
            if !self.jps.is_available() {
                return Err(JdkToolsError::JpsNotFound);
            }
        }
        Ok(())
    }

    pub fn installation_guidance(&self) -> String {
        let platform = if cfg!(target_os = "macos") {
            "macOS"
        } else if cfg!(target_os = "linux") {
            "Linux"
        } else if cfg!(target_os = "windows") {
            "Windows"
        } else {
            "Unknown"
        };

        let mut guidance = format!("JDK tools detection failed on {}\n\n", platform);

        if !self.jcmd.is_available() {
            guidance.push_str("❌ jcmd not found\n");
        }
        if !self.jstat.is_available() {
            guidance.push_str("❌ jstat not found\n");
        }
        if !self.jps.is_available() {
            guidance.push_str("❌ jps not found\n");
        }

        guidance.push_str("\nInstallation Instructions:\n\n");

        match platform {
            "macOS" => {
                guidance.push_str("Using Homebrew:\n");
                guidance.push_str("  brew install openjdk@21\n");
                guidance.push_str("  echo 'export PATH=\"/opt/homebrew/opt/openjdk@21/bin:$PATH\"' >> ~/.zshrc\n\n");
                guidance.push_str("Or download from:\n");
                guidance.push_str("  https://adoptium.net/\n");
            }
            "Linux" => {
                guidance.push_str("Ubuntu/Debian:\n");
                guidance.push_str("  sudo apt update\n");
                guidance.push_str("  sudo apt install openjdk-21-jdk\n\n");
                guidance.push_str("RHEL/CentOS/Fedora:\n");
                guidance.push_str("  sudo dnf install java-21-openjdk-devel\n");
            }
            "Windows" => {
                guidance.push_str("Download and install:\n");
                guidance.push_str("  https://adoptium.net/\n\n");
                guidance.push_str("Then add to PATH:\n");
                guidance.push_str("  System Properties > Environment Variables > Path\n");
                guidance.push_str("  Add: C:\\Program Files\\Eclipse Adoptium\\jdk-21\\bin\n");
            }
            _ => {
                guidance.push_str("Please install a JDK (version 11 or higher).\n");
                guidance.push_str("Visit: https://adoptium.net/\n");
            }
        }

        if let Some(java_home) = &self.java_home {
            guidance.push_str(&format!(
                "\n💡 JAVA_HOME is set to: {}\n",
                java_home.display()
            ));
            guidance.push_str("Make sure this JDK includes the required tools.\n");
        } else {
            guidance.push_str("\n💡 JAVA_HOME is not set.\n");
            guidance.push_str("Set it to your JDK installation directory.\n");
        }

        guidance
    }
}

fn detect_tool(name: &str, java_home: &Option<PathBuf>) -> ToolStatus {
    let candidates: Vec<PathBuf> = java_home
        .iter()
        .map(|h| {
            let mut path = h.join("bin").join(name);
            if cfg!(target_os = "windows") && !name.ends_with(".exe") {
                path.set_extension("exe");
            }
            path
        })
        .chain(std::iter::once(PathBuf::from(name)))
        .collect();

    for path in candidates {
        match try_execute(&path) {
            Ok(version) => {
                return ToolStatus::Available { path, version };
            }
            Err(TryExecuteError::NotExecutable) => {
                return ToolStatus::NotExecutable { path };
            }
            Err(TryExecuteError::NotFound) => continue,
        }
    }

    ToolStatus::NotFound
}

enum TryExecuteError {
    NotFound,
    NotExecutable,
}

fn try_execute(path: &PathBuf) -> Result<String, TryExecuteError> {
    let output = Command::new(path).arg("-h").output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            TryExecuteError::NotFound
        } else {
            TryExecuteError::NotExecutable
        }
    })?;

    if output.status.success() || output.status.code() == Some(1) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        let version = combined
            .lines()
            .find(|line| line.contains("version") || line.contains("JDK"))
            .unwrap_or("unknown")
            .to_string();

        Ok(version)
    } else {
        Err(TryExecuteError::NotExecutable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tools() {
        let status = JdkToolsStatus::detect();
        println!("Detection result: {:#?}", status);

        if !status.is_usable() {
            println!("\n{}", status.installation_guidance());
        }
    }

    #[test]
    fn test_capabilities() {
        let status = JdkToolsStatus::detect();
        let caps = status.capabilities();
        println!("Capabilities: {:#?}", caps);
    }
}
