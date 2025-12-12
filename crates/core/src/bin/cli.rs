use audio_recorder_manager_core::cli;
use audio_recorder_manager_core::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = env::args().collect();
    cli::run(args).await
}
