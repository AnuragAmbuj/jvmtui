use super::JdkToolsError;
use std::process::Output;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn execute_command(
    tool: &str,
    args: &[&str],
    timeout_duration: Option<Duration>,
) -> Result<Output, JdkToolsError> {
    let timeout_duration = timeout_duration.unwrap_or(DEFAULT_TIMEOUT);

    let command = Command::new(tool).args(args).output();

    timeout(timeout_duration, command)
        .await
        .map_err(|_| JdkToolsError::Timeout {
            command: format!("{} {}", tool, args.join(" ")),
        })?
        .map_err(|e| JdkToolsError::ExecutionFailed {
            command: format!("{} {}", tool, args.join(" ")),
            source: e,
        })
}
