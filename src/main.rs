mod cli;
mod commands;
mod config;
mod devices;
mod domain;
mod error;
mod recorder;
mod status;
mod wasapi_loopback;
mod wasapi_microphone;

use error::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    cli::run(args).await
}
