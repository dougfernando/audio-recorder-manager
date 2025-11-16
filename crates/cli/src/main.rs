use audio_recorder_manager_core::cli;
use audio_recorder_manager_core::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    cli::run(args).await
}
