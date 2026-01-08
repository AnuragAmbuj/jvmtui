use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub connections: Vec<ConnectionProfile>,
    #[serde(default)]
    pub advanced: AdvancedSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(
        default = "default_interval",
        deserialize_with = "deserialize_duration_string"
    )]
    pub default_interval: Duration,

    #[serde(default = "default_max_samples")]
    pub max_history_samples: usize,

    #[serde(default)]
    pub export_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConnectionProfile {
    Local {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pid: Option<u32>,
    },
    Jolokia {
        name: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    #[serde(rename = "ssh-jdk")]
    SshJdk {
        name: String,
        ssh_host: String,
        ssh_user: String,
        #[serde(default = "default_ssh_port")]
        ssh_port: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        ssh_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ssh_password: Option<String>,
        pid: u32,
    },
    #[serde(rename = "ssh-jolokia")]
    SshJolokia {
        name: String,
        ssh_host: String,
        ssh_user: String,
        #[serde(default = "default_ssh_port")]
        ssh_port: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        ssh_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ssh_password: Option<String>,
        jolokia_port: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        local_port: Option<u16>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    #[serde(default = "default_http_timeout")]
    pub http_timeout_ms: u64,

    #[serde(default = "default_ssh_timeout")]
    pub ssh_timeout_sec: u64,

    #[serde(default = "default_retry_attempts")]
    pub connection_retry_attempts: usize,

    #[serde(default = "default_retry_delay")]
    pub connection_retry_delay_ms: u64,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_interval: default_interval(),
            max_history_samples: default_max_samples(),
            export_directory: None,
        }
    }
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            http_timeout_ms: default_http_timeout(),
            ssh_timeout_sec: default_ssh_timeout(),
            connection_retry_attempts: default_retry_attempts(),
            connection_retry_delay_ms: default_retry_delay(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            preferences: Preferences::default(),
            connections: Vec::new(),
            advanced: AdvancedSettings::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        if let Some(path) = Self::find_config_file() {
            Self::load_from_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn load_from_file(path: &std::path::Path) -> Result<Self, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::ConfigLoad(format!("Failed to read config file: {}", e)))?;

        let mut config: Config = toml::from_str(&content)
            .map_err(|e| AppError::ConfigLoad(format!("Failed to parse config: {}", e)))?;

        config.expand_environment_variables();
        config.validate()?;

        Ok(config)
    }

    pub fn find_config_file() -> Option<PathBuf> {
        let config_locations = Self::config_search_paths();

        for path in config_locations {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    pub fn config_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Ok(custom_path) = std::env::var("JVM_TUI_CONFIG") {
            paths.push(PathBuf::from(custom_path));
        }

        paths.push(PathBuf::from("./config.toml"));
        paths.push(PathBuf::from("./jvm-tui.toml"));

        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("jvm-tui").join("config.toml"));
        }

        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".jvm-tui.toml"));
            paths.push(home_dir.join(".config").join("jvm-tui").join("config.toml"));
        }

        paths
    }

    fn expand_environment_variables(&mut self) {
        if let Some(ref mut export_dir) = self.preferences.export_directory {
            *export_dir = shellexpand::tilde(export_dir).to_string();
            *export_dir = shellexpand::env(export_dir)
                .unwrap_or_else(|_| export_dir.clone().into())
                .to_string();
        }

        for connection in &mut self.connections {
            match connection {
                ConnectionProfile::SshJdk { ssh_key, .. }
                | ConnectionProfile::SshJolokia { ssh_key, .. } => {
                    if let Some(ref mut key_path) = ssh_key {
                        *key_path = shellexpand::tilde(key_path).to_string();
                    }
                }
                _ => {}
            }
        }
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.preferences.max_history_samples == 0 {
            return Err(AppError::ConfigLoad(
                "max_history_samples must be greater than 0".to_string(),
            ));
        }

        if self.preferences.default_interval < Duration::from_millis(100) {
            return Err(AppError::ConfigLoad(
                "default_interval must be at least 100ms".to_string(),
            ));
        }

        for (idx, conn) in self.connections.iter().enumerate() {
            match conn {
                ConnectionProfile::Jolokia { url, .. } => {
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        return Err(AppError::ConfigLoad(format!(
                            "Connection '{}': Jolokia URL must start with http:// or https://",
                            idx
                        )));
                    }
                }
                ConnectionProfile::SshJdk { ssh_host, pid, .. } => {
                    if ssh_host.is_empty() {
                        return Err(AppError::ConfigLoad(format!(
                            "Connection '{}': ssh_host cannot be empty",
                            idx
                        )));
                    }
                    if *pid == 0 {
                        return Err(AppError::ConfigLoad(format!(
                            "Connection '{}': pid must be greater than 0",
                            idx
                        )));
                    }
                }
                ConnectionProfile::SshJolokia {
                    ssh_host,
                    jolokia_port,
                    ..
                } => {
                    if ssh_host.is_empty() {
                        return Err(AppError::ConfigLoad(format!(
                            "Connection '{}': ssh_host cannot be empty",
                            idx
                        )));
                    }
                    if *jolokia_port == 0 {
                        return Err(AppError::ConfigLoad(format!(
                            "Connection '{}': jolokia_port must be greater than 0",
                            idx
                        )));
                    }
                }
                ConnectionProfile::Local { .. } => {}
            }
        }

        Ok(())
    }

    pub fn get_connection(&self, name: &str) -> Option<&ConnectionProfile> {
        self.connections.iter().find(|c| match c {
            ConnectionProfile::Local { name: n, .. } => n == name,
            ConnectionProfile::Jolokia { name: n, .. } => n == name,
            ConnectionProfile::SshJdk { name: n, .. } => n == name,
            ConnectionProfile::SshJolokia { name: n, .. } => n == name,
        })
    }
}

impl ConnectionProfile {
    pub fn name(&self) -> &str {
        match self {
            ConnectionProfile::Local { name, .. } => name,
            ConnectionProfile::Jolokia { name, .. } => name,
            ConnectionProfile::SshJdk { name, .. } => name,
            ConnectionProfile::SshJolokia { name, .. } => name,
        }
    }

    pub fn connection_type(&self) -> &str {
        match self {
            ConnectionProfile::Local { .. } => "Local",
            ConnectionProfile::Jolokia { .. } => "Jolokia (HTTP)",
            ConnectionProfile::SshJdk { .. } => "SSH + JDK Tools",
            ConnectionProfile::SshJolokia { .. } => "SSH + Jolokia",
        }
    }
}

fn default_interval() -> Duration {
    Duration::from_secs(1)
}

fn default_max_samples() -> usize {
    300
}

fn default_ssh_port() -> u16 {
    22
}

fn default_http_timeout() -> u64 {
    5000
}

fn default_ssh_timeout() -> u64 {
    10
}

fn default_retry_attempts() -> usize {
    3
}

fn default_retry_delay() -> u64 {
    1000
}

fn deserialize_duration_string<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    humantime::parse_duration(&s).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.preferences.default_interval, Duration::from_secs(1));
        assert_eq!(config.preferences.max_history_samples, 300);
        assert!(config.connections.is_empty());
    }

    #[test]
    fn test_parse_local_connection() {
        let toml = r#"
            [[connections]]
            name = "Test Local"
            type = "local"
            pid = 12345
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.connections.len(), 1);

        match &config.connections[0] {
            ConnectionProfile::Local { name, pid } => {
                assert_eq!(name, "Test Local");
                assert_eq!(*pid, Some(12345));
            }
            _ => panic!("Expected Local connection"),
        }
    }

    #[test]
    fn test_parse_jolokia_connection() {
        let toml = r#"
            [[connections]]
            name = "Test Jolokia"
            type = "jolokia"
            url = "http://localhost:8778/jolokia"
            username = "admin"
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.connections.len(), 1);

        match &config.connections[0] {
            ConnectionProfile::Jolokia {
                name,
                url,
                username,
                ..
            } => {
                assert_eq!(name, "Test Jolokia");
                assert_eq!(url, "http://localhost:8778/jolokia");
                assert_eq!(username.as_deref(), Some("admin"));
            }
            _ => panic!("Expected Jolokia connection"),
        }
    }

    #[test]
    fn test_parse_ssh_jolokia_connection() {
        let toml = r#"
            [[connections]]
            name = "Test SSH"
            type = "ssh-jolokia"
            ssh_host = "example.com"
            ssh_user = "deploy"
            ssh_key = "~/.ssh/id_rsa"
            jolokia_port = 8778
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.connections.len(), 1);

        match &config.connections[0] {
            ConnectionProfile::SshJolokia {
                name,
                ssh_host,
                ssh_user,
                ssh_key,
                jolokia_port,
                ..
            } => {
                assert_eq!(name, "Test SSH");
                assert_eq!(ssh_host, "example.com");
                assert_eq!(ssh_user, "deploy");
                assert_eq!(ssh_key.as_deref(), Some("~/.ssh/id_rsa"));
                assert_eq!(*jolokia_port, 8778);
            }
            _ => panic!("Expected SshJolokia connection"),
        }
    }

    #[test]
    fn test_validation_rejects_invalid_interval() {
        let mut config = Config::default();
        config.preferences.default_interval = Duration::from_millis(50);

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_rejects_zero_samples() {
        let mut config = Config::default();
        config.preferences.max_history_samples = 0;

        assert!(config.validate().is_err());
    }
}
