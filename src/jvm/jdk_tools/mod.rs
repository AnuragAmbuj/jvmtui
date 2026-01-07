pub mod connector;
pub mod detector;
pub mod executor;
pub mod parsers;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum JdkToolsError {
    #[error("jcmd not found in PATH")]
    JcmdNotFound,

    #[error("jstat not found in PATH")]
    JstatNotFound,

    #[error("jps not found in PATH")]
    JpsNotFound,

    #[error("Failed to execute {command}: {source}")]
    ExecutionFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command timed out: {command}")]
    Timeout { command: String },

    #[error("Parse error: {0}")]
    ParseError(String),
}
