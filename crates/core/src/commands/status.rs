#[cfg(not(windows))]
use anyhow::Context;
use serde_json::json;

#[cfg(not(windows))]
use crate::devices::DeviceManager;
use crate::error::Result;

/// Execute the status command
pub async fn execute() -> Result<()> {
    #[cfg(not(windows))]
    {
        let device_manager = DeviceManager::new().context("Failed to create device manager")?;

        let devices = device_manager.list_devices()?;

        let device_list: Vec<_> = devices
            .iter()
            .map(|d| {
                json!({
                    "name": d.name,
                    "is_default_input": d.is_default_input,
                    "is_default_output": d.is_default_output,
                })
            })
            .collect();

        let result = json!({
            "status": "success",
            "data": {
                "devices_count": devices.len(),
                "devices": device_list
            }
        });

        println!("{}", serde_json::to_string(&result)?);
    }

    #[cfg(windows)]
    {
        let result = json!({
            "status": "success",
            "data": {
                "message": "Device listing not supported on Windows. Using WASAPI for recording."
            }
        });

        println!("{}", serde_json::to_string(&result)?);
    }

    Ok(())
}
