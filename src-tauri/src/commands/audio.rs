use voxflow_audio::{AudioCapture, AudioDeviceInfo};

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    AudioCapture::list_devices().map_err(|e| e.to_string())
}
