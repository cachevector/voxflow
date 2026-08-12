use std::path::PathBuf;
use thiserror::Error;
use voxflow_config::data_dir;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("no app data directory")]
    NoDataDir,
    #[error("download failed: {0}")]
    Download(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

const HF_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

pub fn default_model_filename(model_id: &str) -> String {
    format!("ggml-{model_id}.bin")
}

pub fn model_path_for_id(model_id: &str) -> Result<PathBuf, ModelError> {
    let data = data_dir().ok_or(ModelError::NoDataDir)?;
    Ok(data.join("models").join(default_model_filename(model_id)))
}

pub fn model_download_url(model_id: &str) -> String {
    format!("{HF_BASE}/{}", default_model_filename(model_id))
}

pub async fn ensure_model_downloaded(model_id: &str) -> Result<PathBuf, ModelError> {
    let path = model_path_for_id(model_id)?;
    if path.exists() {
        return Ok(path);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let url = model_download_url(model_id);
    tracing::info!(%url, "downloading whisper model (this may take a few minutes)");
    let response = reqwest::get(&url)
        .await
        .map_err(|e| ModelError::Download(e.to_string()))?;
    if !response.status().is_success() {
        return Err(ModelError::Download(format!(
            "HTTP {} for {url}",
            response.status()
        )));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| ModelError::Download(e.to_string()))?;
    std::fs::write(&path, &bytes)?;
    tracing::info!(path = ?path, "whisper model ready");
    Ok(path)
}

pub fn model_exists(model_id: &str) -> bool {
    model_path_for_id(model_id)
        .map(|p| p.exists())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn models_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("models"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filename_small_en() {
        assert_eq!(default_model_filename("small.en"), "ggml-small.en.bin");
    }
}
