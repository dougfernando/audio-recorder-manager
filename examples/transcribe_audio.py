import google.generativeai as genai
import os
import time
import json
from pathlib import Path
import argparse

# Timeout in seconds (adjust as needed)
REQUEST_TIMEOUT = 300  # 5 minutes

def write_status(status_file_path, step: int, total_steps: int, step_name: str, message: str = ""):
    """Write progress status to a file for UI monitoring."""
    if not status_file_path:
        return

    try:
        progress = int((step / total_steps) * 100)
        status_data = {
            "step": step_name,
            "step_number": step,
            "total_steps": total_steps,
            "progress": progress,
            "message": message,
        }
        Path(status_file_path).parent.mkdir(parents=True, exist_ok=True)
        Path(status_file_path).write_text(json.dumps(status_data), encoding="utf-8")
    except Exception as e:
        print(f"[Status] Error writing status file to {status_file_path}: {e}", file=__import__('sys').stderr)


def main():
    """Main function to execute transcription."""
    parser = argparse.ArgumentParser(description="Transcribes an audio file using the Google Gemini API.")
    parser.add_argument("audio_file", help="Path to the audio file to be transcribed.")
    parser.add_argument("--api-key", required=True, help="Google Gemini API Key.")
    parser.add_argument("--model", default="models/gemini-1.5-flash", help="Model to be used for transcription.")
    parser.add_argument("--optimize", action="store_true", help="Optimizes the audio before sending.")
    parser.add_argument("--output-file", help="Path to save the transcription file.")
    parser.add_argument("--session-id", help="Session ID for progress tracking.")
    args = parser.parse_args()

    # Set the audio file path from arguments
    AUDIO_FILE_PATH = args.audio_file
    API_KEY = args.api_key
    MODEL_NAME = args.model

    # Prepare status file path for progress tracking
    STATUS_FILE = None
    if args.session_id:
        # Get project path from current working directory
        project_path = Path.cwd()
        status_dir = project_path / "storage" / "status"
        STATUS_FILE = str(status_dir / f"{args.session_id}.json")

    try:
        # Check file information before starting
        file_path = Path(AUDIO_FILE_PATH)
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {AUDIO_FILE_PATH}")

        original_size_mb = file_path.stat().st_size / (1024 * 1024)
        print(f"Original file: {file_path.name}")
        print(f"Size: {original_size_mb:.2f} MB")
        print(f"Format: {file_path.suffix}")
        print()

        upload_file_path = AUDIO_FILE_PATH

        # 1. Configure API Key in the library
        genai.configure(api_key=API_KEY)

        # 2. Upload audio file to the Gemini API
        write_status(STATUS_FILE, 1, 4, "uploading", "Uploading file to Gemini API...")
        print(f"[1/4] Uploading file...")
        upload_start = time.time()

        # Try upload with retry
        max_retries = 3
        for attempt in range(max_retries):
            try:
                audio_file = genai.upload_file(path=upload_file_path)
                upload_time = time.time() - upload_start
                print(f"      Upload completed in {upload_time:.1f}s")
                print(f"      URI: {audio_file.uri}")
                break
            except Exception as e:
                if attempt < max_retries - 1:
                    wait_time = (attempt + 1) * 2  # 2s, 4s, 6s
                    print(f"      Connection error (attempt {attempt + 1}/{max_retries})")
                    print(f"      Waiting {wait_time}s before trying again...")
                    time.sleep(wait_time)
                else:
                    raise

        print()

        # Wait for file processing
        write_status(STATUS_FILE, 2, 4, "processing", "Processing file by Gemini API...")
        print(f"[2/4] Waiting for file processing...")
        process_start = time.time()
        while audio_file.state.name == "PROCESSING":
            print("      Processing...", end="\r")
            time.sleep(2)
            audio_file = genai.get_file(audio_file.name)

        process_time = time.time() - process_start

        if audio_file.state.name == "FAILED":
            raise Exception(f"File processing failed: {audio_file.state.name}")

        print(f"      Processing completed in {process_time:.1f}s")
        print()

        # 3. Select the model
        model = genai.GenerativeModel(MODEL_NAME)

        # Configuration with timeout
        generation_config = {
            "temperature": 0.1,  # Low temperature for more accurate transcription
            "top_p": 0.95,
            "top_k": 40,
            "max_output_tokens": 8192,
        }

        # 4. Send the audio and text prompt together
        write_status(STATUS_FILE, 3, 4, "transcribing", "Generating transcription from audio...")
        print(f"[3/4] Sending transcription request to the model...")
        prompt = '''Please process the attached audio file and provide the following two sections in markdown format:

**1. Raw Transcription:**

*   Detect the language spoken in the audio.
*   Transcribe the audio verbatim in the detected language, word for word, exactly as spoken.
*   Use appropriate punctuation.
*   Indicate long pauses with [...].
*   If there are multiple speakers, label them as "Speaker 1:", "Speaker 2:", etc.

**2. Key Topics Discussed:**

*   Analyze the raw transcription.
*   Identify the main subjects, decisions, and action items.
*   Organize these points into a summary with clear headings for each topic.
*   Describe the key topics in the same language as identified in the raw transcription as long it is Spanish, Portuguese or English; otherwise, use English.
*   Ensure no critical information is lost.

Your entire response should be a single markdown document.'''

        transcription_start = time.time()
        response = model.generate_content(
            [prompt, audio_file],
            generation_config=generation_config,
            request_options={"timeout": REQUEST_TIMEOUT}
        )
        transcription_time = time.time() - transcription_start
        print(f"      Transcription completed in {transcription_time:.1f}s")
        print()

        # 5. Print the transcription result and save
        write_status(STATUS_FILE, 4, 4, "saving", "Saved transcript to file")
        print("[4/4] Result:")
        print("\n" + "="*80)
        print("TRANSCRIPTION")
        print("="*80)
        print(response.text)
        print("="*80)
        print()

        # Timing summary
        total_time = upload_time + process_time + transcription_time
        print(f"Total time: {total_time:.1f}s")
        print(f"  - Upload: {upload_time:.1f}s")
        print(f"  - Processing: {process_time:.1f}s")
        print(f"  - Transcription: {transcription_time:.1f}s")
        print()

        # Save the transcription to a file, if specified
        if args.output_file:
            output_path = Path(args.output_file)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(response.text, encoding="utf-8")
            print(f"Transcription saved to: {output_path}")

            # Return JSON with success and file path
            print(json.dumps({
                "success": True,
                "transcript_file": str(output_path)
            }))
        else:
            print(json.dumps({"success": True}))

    except FileNotFoundError as e:
        print(json.dumps({"success": False, "error": str(e)}))
    except TimeoutError as e:
        print(json.dumps({"success": False, "error": f"Timeout: {e}"}))
    except Exception as e:
        print(json.dumps({"success": False, "error": f"{type(e).__name__}: {e}"}))

if __name__ == "__main__":
    main()
