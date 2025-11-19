pub mod cli;
pub mod commands;
pub mod config;
pub mod devices;
pub mod domain;
pub mod error;
pub mod logging;
pub mod output;
pub mod recorder;
pub mod status;
pub mod transcription;
pub mod wasapi_loopback;
pub mod wasapi_microphone;

pub use error::Result;
pub use output::UserOutput;
