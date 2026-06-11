//! Tauri commands for the assistant panel and assistant settings.

use crate::assistant::{self, AssistantConversation};
use crate::llm_client::ChatMessage;
use crate::settings::{get_settings, write_settings};
use tauri::{AppHandle, Manager};

/// Send a typed message to the assistant (keyboard alternative to voice).
#[tauri::command]
#[specta::specta]
pub async fn assistant_send_text(app: AppHandle, text: String) -> Result<(), String> {
    assistant::run_assistant_turn(app, text, None).await;
    Ok(())
}

/// Send a typed message with a screenshot of the current screen attached.
#[tauri::command]
#[specta::specta]
pub async fn assistant_send_text_with_screen(app: AppHandle, text: String) -> Result<(), String> {
    let settings = get_settings(&app);
    let screenshot = if settings.assistant_screenshot_enabled {
        match tauri::async_runtime::spawn_blocking(crate::screenshot::capture_screen_data_url)
            .await
        {
            Ok(Ok(url)) => Some(url),
            Ok(Err(e)) => {
                use tauri::Emitter;
                let _ = app.emit("assistant-error", format!("Screen capture failed: {}", e));
                None
            }
            Err(e) => {
                use tauri::Emitter;
                let _ = app.emit("assistant-error", format!("Screen capture failed: {}", e));
                None
            }
        }
    } else {
        None
    };
    assistant::run_assistant_turn(app, text, screenshot).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn assistant_get_conversation(app: AppHandle) -> Result<Vec<ChatMessage>, String> {
    let conversation = app.state::<AssistantConversation>();
    let history = conversation
        .messages
        .lock()
        .map_err(|e| format!("Conversation lock poisoned: {}", e))?;
    Ok(history.clone())
}

#[tauri::command]
#[specta::specta]
pub fn assistant_clear_conversation(app: AppHandle) -> Result<(), String> {
    let conversation = app.state::<AssistantConversation>();
    conversation
        .messages
        .lock()
        .map_err(|e| format!("Conversation lock poisoned: {}", e))?
        .clear();
    assistant::emit_conversation(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_assistant_panel(app: AppHandle) -> Result<(), String> {
    assistant::toggle_assistant_panel(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn hide_assistant_panel(app: AppHandle) -> Result<(), String> {
    assistant::hide_assistant_panel(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_provider(app: AppHandle, provider_id: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    if !settings
        .post_process_providers
        .iter()
        .any(|p| p.id == provider_id)
    {
        return Err(format!("Unknown provider: {}", provider_id));
    }
    settings.assistant_provider_id = provider_id;
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_assistant_model_setting(
    app: AppHandle,
    provider_id: String,
    model: String,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_models.insert(provider_id, model);
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_assistant_system_prompt_setting(
    app: AppHandle,
    prompt: String,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_system_prompt = prompt;
    write_settings(&app, settings);
    Ok(())
}

/// Notify the panel (a separate webview) that assistant settings changed.
fn emit_settings_changed(app: &AppHandle) {
    use tauri::Emitter;
    let _ = app.emit("assistant-settings-changed", ());
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_screenshot_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_screenshot_enabled = enabled;
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_tts_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_tts_enabled = enabled;
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_tts_voice(app: AppHandle, voice: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_tts_voice = voice;
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_assistant_tts_prompt_setting(app: AppHandle, prompt: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_tts_prompt = prompt;
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_panel_opacity(app: AppHandle, opacity: f64) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_panel_opacity = opacity.clamp(0.5, 1.0);
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_font_size(app: AppHandle, size: String) -> Result<(), String> {
    if !matches!(size.as_str(), "small" | "medium" | "large") {
        return Err(format!("Unknown font size: {}", size));
    }
    let mut settings = get_settings(&app);
    settings.assistant_font_size = size;
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_assistant_accent(app: AppHandle, accent: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.assistant_accent = accent;
    write_settings(&app, settings);
    emit_settings_changed(&app);
    Ok(())
}
