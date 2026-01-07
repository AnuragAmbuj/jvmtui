use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub polling_interval: Duration,

    #[serde(default = "default_history_size")]
    pub history_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polling_interval: default_interval(),
            history_size: default_history_size(),
        }
    }
}

fn default_interval() -> Duration {
    Duration::from_secs(1)
}

fn default_history_size() -> usize {
    300
}
