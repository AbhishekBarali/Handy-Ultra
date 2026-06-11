//! Screen capture for assistant vision: grabs the monitor under the cursor
//! (fallback: primary) and returns a downscaled JPEG as a data URL ready to
//! embed in an OpenAI-compatible `image_url` content part.

use base64::Engine;
use image::imageops::FilterType;
use image::DynamicImage;
use log::debug;
use std::io::Cursor;
use xcap::Monitor;

/// Longest edge of the captured image sent to the model. Keeps token /
/// payload cost reasonable while UI text stays readable.
const MAX_DIMENSION: u32 = 1568;
const JPEG_QUALITY: u8 = 82;

#[cfg(target_os = "windows")]
fn cursor_position() -> Option<(i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok()? };
    Some((point.x, point.y))
}

#[cfg(not(target_os = "windows"))]
fn cursor_position() -> Option<(i32, i32)> {
    None
}

fn pick_monitor() -> Result<Monitor, String> {
    if let Some((x, y)) = cursor_position() {
        if let Ok(monitor) = Monitor::from_point(x, y) {
            return Ok(monitor);
        }
    }
    let monitors = Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {}", e))?;
    monitors
        .into_iter()
        .next()
        .ok_or_else(|| "No monitors found".to_string())
}

/// Capture the active monitor and return a `data:image/jpeg;base64,...` URL.
pub fn capture_screen_data_url() -> Result<String, String> {
    let start = std::time::Instant::now();

    let monitor = pick_monitor()?;
    let rgba = monitor
        .capture_image()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    let mut img = DynamicImage::ImageRgba8(rgba);
    let (w, h) = (img.width(), img.height());
    if w.max(h) > MAX_DIMENSION {
        let scale = MAX_DIMENSION as f32 / w.max(h) as f32;
        img = img.resize(
            (w as f32 * scale) as u32,
            (h as f32 * scale) as u32,
            FilterType::Triangle,
        );
    }

    // JPEG keeps the payload small; UI text survives fine at this quality.
    let rgb = DynamicImage::ImageRgb8(img.to_rgb8());
    let mut buf = Vec::new();
    rgb.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
        Cursor::new(&mut buf),
        JPEG_QUALITY,
    ))
    .map_err(|e| format!("Failed to encode screenshot: {}", e))?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(&buf);
    debug!(
        "Captured screen {}x{} -> {} KB jpeg in {:?}",
        rgb.width(),
        rgb.height(),
        buf.len() / 1024,
        start.elapsed()
    );

    Ok(format!("data:image/jpeg;base64,{}", encoded))
}
