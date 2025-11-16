use audio_recorder_manager::transcription::{transcribe_audio, load_config};
use std::path::PathBuf;

#[tokio::test]
#[ignore] // Run with: cargo test --test transcription_test -- --ignored
async fn test_transcription_with_real_audio() {
    // Initialize logger for test
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // Path to the test audio file
    let audio_path = PathBuf::from("src-tauri/target/release/storage/recordings/recording_20251115_184335.m4a");

    // Verify file exists
    assert!(audio_path.exists(), "Test audio file not found at {:?}", audio_path);

    // Load configuration (will use saved API key)
    let mut config = load_config().expect("Failed to load transcription config");

    // Override with correct model name (use latest stable flash)
    config.model = "gemini-2.5-flash".to_string();

    // Verify API key is configured
    assert!(!config.api_key.is_empty(), "API key not configured. Please set it in Settings first.");

    println!("Test Configuration:");
    println!("  Audio file: {}", audio_path.display());
    println!("  Model: {}", config.model);
    println!("  Optimize: {}", config.optimize_audio);
    println!("  API key length: {} chars", config.api_key.len());
    println!();

    // Test session ID
    let session_id = "test_transcription_20251115";

    // Run transcription
    println!("Starting transcription...");
    let result = transcribe_audio(
        &audio_path,
        &config.api_key,
        &config.model,
        &config.prompt,
        config.optimize_audio,
        session_id,
    )
    .await;

    // Check result
    match result {
        Ok(transcript_result) => {
            println!("\n✅ Transcription successful!");
            println!("  Success: {}", transcript_result.success);
            println!("  Transcript file: {:?}", transcript_result.transcript_file);

            assert!(transcript_result.success, "Transcription should be successful");
            assert!(transcript_result.transcript_file.is_some(), "Transcript file path should be set");

            // Verify transcript file was created
            if let Some(ref path) = transcript_result.transcript_file {
                let transcript_path = PathBuf::from(path);
                assert!(transcript_path.exists(), "Transcript file should exist at {:?}", transcript_path);

                // Read and display transcript content
                let content = std::fs::read_to_string(&transcript_path)
                    .expect("Failed to read transcript file");

                println!("\n📄 Transcript content ({} chars, {} words):",
                    content.len(),
                    content.split_whitespace().count());
                println!("{}", "=".repeat(80));
                println!("{}", content);
                println!("{}", "=".repeat(80));

                assert!(!content.is_empty(), "Transcript should not be empty");
            }
        }
        Err(e) => {
            println!("\n❌ Transcription failed!");
            println!("  Error: {}", e);
            panic!("Transcription failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_config_loading() {
    // Test that configuration can be loaded
    let config_result = load_config();

    assert!(config_result.is_ok(), "Should be able to load config");

    let config = config_result.unwrap();
    println!("Configuration loaded:");
    println!("  Model: {}", config.model);
    println!("  API key configured: {}", !config.api_key.is_empty());
    println!("  Optimize audio: {}", config.optimize_audio);
    println!("  Prompt length: {} chars", config.prompt.len());
}

#[test]
fn test_audio_file_exists() {
    let audio_path = PathBuf::from("src-tauri/target/release/storage/recordings/recording_20251115_184335.m4a");

    if !audio_path.exists() {
        println!("⚠️  Warning: Test audio file not found at {:?}", audio_path);
        println!("   This test will be skipped until the file is available.");
    } else {
        let metadata = std::fs::metadata(&audio_path).unwrap();
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        println!("✅ Test audio file found:");
        println!("  Path: {}", audio_path.display());
        println!("  Size: {:.2} MB", size_mb);

        assert!(size_mb > 0.0, "Audio file should not be empty");
    }
}
