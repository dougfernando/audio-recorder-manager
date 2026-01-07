use serde_json::json;

use crate::error::Result;

/// Execute the status command
pub async fn execute() -> Result<()> {
    #[cfg(windows)]
    {
        let result = json!({
            "status": "success",
            "data": {
                "message": "Using WASAPI for audio capture (default system audio + microphone)",
                "platform": "Windows"
            }
        });

        println!("{}", serde_json::to_string(&result)?);
    }

    #[cfg(not(windows))]
    {
        let result = json!({
            "status": "error",
            "error": "Recording is currently only supported on Windows. Cross-platform support is planned for a future release."
        });

        println!("{}", serde_json::to_string(&result)?);
    }

    Ok(())
}
