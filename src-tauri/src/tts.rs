//! Remote TTS engines for the assistant's spoken summaries.
//!
//! Two engines are handled here in Rust (audio fetched and played natively
//! via rodio, so playback works even when the panel webview is hidden):
//! - "openai": any OpenAI-compatible `/audio/speech` endpoint — OpenAI,
//!   Azure OpenAI (`https://{res}.openai.azure.com/openai/v1` or
//!   `cognitiveservices.azure.com/openai/v1`, model = deployment name),
//!   Groq, LocalAI, Kokoro-FastAPI, openai-edge-tts, etc.
//! - "elevenlabs": ElevenLabs `text-to-speech/{voice_id}` API.
//!
//! The third engine, "kokoro", runs fully locally in the panel webview
//! (kokoro-js, WebGPU) and never reaches this module.

use crate::settings::AppSettings;
use log::{debug, error};
use std::io::Cursor;
use tauri::AppHandle;

/// Fetch speech audio for `text` using the configured remote engine and play
/// it on the selected output device. Returns after playback finishes.
pub async fn speak_remote(app: &AppHandle, settings: &AppSettings, text: String) {
    let result = match settings.assistant_tts_engine.as_str() {
        "openai" => fetch_openai_speech(settings, &text).await,
        "elevenlabs" => fetch_elevenlabs_speech(settings, &text).await,
        other => Err(format!("Unknown TTS engine: {}", other)),
    };

    match result {
        Ok(audio_bytes) => {
            debug!("TTS audio fetched: {} KB", audio_bytes.len() / 1024);
            let volume = settings.audio_feedback_volume;
            let device = settings.selected_output_device.clone();
            // rodio playback blocks; run it off the async runtime.
            let _ = tauri::async_runtime::spawn_blocking(move || {
                if let Err(e) = play_audio_bytes(audio_bytes, device, volume) {
                    error!("TTS playback failed: {}", e);
                }
            })
            .await;
        }
        Err(e) => {
            error!("TTS request failed: {}", e);
            use tauri::Emitter;
            let _ = app.emit("assistant-error", format!("TTS failed: {}", e));
        }
    }
}

/// POST {base}/audio/speech — OpenAI-compatible shape.
async fn fetch_openai_speech(settings: &AppSettings, text: &str) -> Result<Vec<u8>, String> {
    let base = settings.assistant_tts_base_url.trim_end_matches('/');
    let url = format!("{}/audio/speech", base);

    let client = reqwest::Client::new();
    let mut request = client.post(&url).json(&serde_json::json!({
        "model": settings.assistant_tts_model,
        "input": text,
        "voice": settings.assistant_tts_remote_voice,
        "response_format": "mp3",
    }));

    let api_key = settings.assistant_tts_api_key.0.trim();
    if !api_key.is_empty() {
        request = request.bearer_auth(api_key);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("{}: {}", status, truncate(&body, 300)));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read audio: {}", e))
}

/// POST https://api.elevenlabs.io/v1/text-to-speech/{voice_id}
async fn fetch_elevenlabs_speech(settings: &AppSettings, text: &str) -> Result<Vec<u8>, String> {
    let voice_id = settings.assistant_tts_remote_voice.trim();
    if voice_id.is_empty() {
        return Err("No ElevenLabs voice ID configured".to_string());
    }
    let url = format!(
        "https://api.elevenlabs.io/v1/text-to-speech/{}?output_format=mp3_44100_64",
        voice_id
    );

    let model = if settings.assistant_tts_model.trim().is_empty()
        || settings.assistant_tts_model == "gpt-4o-mini-tts"
    {
        // Sensible default when the user hasn't set an ElevenLabs model.
        "eleven_flash_v2_5".to_string()
    } else {
        settings.assistant_tts_model.clone()
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("xi-api-key", settings.assistant_tts_api_key.0.trim())
        .json(&serde_json::json!({
            "text": text,
            "model_id": model,
        }))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("{}: {}", status, truncate(&body, 300)));
    }

    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read audio: {}", e))
}

/// Decode and play audio bytes (mp3/wav/ogg) on the selected output device.
fn play_audio_bytes(
    bytes: Vec<u8>,
    selected_device: Option<String>,
    volume: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    use cpal::traits::{DeviceTrait, HostTrait};
    use rodio::OutputStreamBuilder;

    let stream_builder = match selected_device {
        Some(name) if name != "Default" => {
            let host = crate::audio_toolkit::get_cpal_host();
            let device = host
                .output_devices()?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false));
            match device {
                Some(device) => OutputStreamBuilder::from_device(device)?,
                None => OutputStreamBuilder::from_default_device()?,
            }
        }
        _ => OutputStreamBuilder::from_default_device()?,
    };

    let stream_handle = stream_builder.open_stream()?;
    let sink = rodio::play(stream_handle.mixer(), Cursor::new(bytes))?;
    sink.set_volume(volume.max(0.1));
    sink.sleep_until_end();
    Ok(())
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    }
}
