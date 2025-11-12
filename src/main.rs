// CLI entry point for Audio Recorder Manager

mod cli;

use audio_recorder_manager::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    cli::run(args).await
}
