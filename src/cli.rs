use clap::Parser;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "jvm-tui")]
#[command(author = "Anurag Ambuj")]
#[command(version)]
#[command(about = "A beautiful TUI for JVM monitoring", long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Attach to specific JVM process ID")]
    pub pid: Option<u32>,

    #[arg(
        short = 'i',
        long,
        help = "Polling interval (e.g. 500ms, 1s, 2s)",
        value_parser = parse_duration
    )]
    pub interval: Option<Duration>,
}

fn parse_duration(s: &str) -> Result<Duration, humantime::DurationError> {
    humantime::parse_duration(s)
}
