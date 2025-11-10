#[cfg(not(windows))]
use anyhow::{Context, Result};
#[cfg(not(windows))]
use cpal::traits::{DeviceTrait, HostTrait};
#[cfg(not(windows))]
use cpal::{Device, Host};
#[cfg(not(windows))]
use serde::{Deserialize, Serialize};

#[cfg(not(windows))]
#[derive(Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default_input: bool,
    pub is_default_output: bool,
    #[serde(skip)]
    device: Option<Device>,
}

#[cfg(not(windows))]
impl std::fmt::Debug for AudioDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioDevice")
            .field("name", &self.name)
            .field("is_default_input", &self.is_default_input)
            .field("is_default_output", &self.is_default_output)
            .finish()
    }
}

#[cfg(not(windows))]
impl AudioDevice {
    pub fn new(device: Device, name: String, is_default_input: bool, is_default_output: bool) -> Self {
        Self {
            name,
            is_default_input,
            is_default_output,
            device: Some(device),
        }
    }

    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }
}

#[cfg(not(windows))]
pub struct DeviceManager {
    host: Host,
}

#[cfg(not(windows))]
impl DeviceManager {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        Ok(Self { host })
    }

    pub fn list_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();

        let default_input = self.host.default_input_device();
        let default_output = self.host.default_output_device();

        let default_input_name = default_input.as_ref()
            .and_then(|d| d.name().ok());
        let default_output_name = default_output.as_ref()
            .and_then(|d| d.name().ok());

        // List all input devices
        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                let is_default_input = default_input_name.as_ref()
                    .map(|dn| dn == &name)
                    .unwrap_or(false);
                let is_default_output = default_output_name.as_ref()
                    .map(|dn| dn == &name)
                    .unwrap_or(false);

                devices.push(AudioDevice::new(device, name, is_default_input, is_default_output));
            }
        }

        Ok(devices)
    }

    fn get_default_input_device(&self) -> Result<AudioDevice> {
        let device = self.host.default_input_device()
            .context("No default input device available")?;

        let name = device.name().context("Failed to get device name")?;
        Ok(AudioDevice::new(device, name, true, false))
    }

    pub fn get_best_recording_device(&self) -> Result<AudioDevice> {
        // On Windows, try to get the loopback device (system audio)
        // For other platforms, use default input
        #[cfg(target_os = "windows")]
        {
            // Try to find a loopback device
            let devices = self.list_devices()?;
            for device in devices.iter() {
                if device.name.to_lowercase().contains("stereo mix")
                    || device.name.to_lowercase().contains("loopback") {
                    return Ok(device.clone());
                }
            }
        }

        // Fallback to default input device
        self.get_default_input_device()
    }
}

#[cfg(not(windows))]
impl Default for DeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create DeviceManager")
    }
}
