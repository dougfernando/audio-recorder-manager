pub mod cli;
pub mod commands;
pub mod config;
pub mod devices;
pub mod domain;
pub mod error;
pub mod recorder;
pub mod status;
pub mod wasapi_loopback;
pub mod wasapi_microphone;

pub use error::Result;
