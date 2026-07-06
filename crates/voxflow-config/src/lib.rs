use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QualityMode {
    #[default]
    Hybrid,
    Economy,
    Balanced,
    Accurate,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DictationMode {
    #[default]
    PushToTalk,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub key_code: u16,
    pub modifiers: u32,
    pub label: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            key_code: 59, // Left Control — present on all Mac/PC keyboards
            modifiers: 0,
            label: "Left Control".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostControlConfig {
    pub monthly_minute_cap: Option<f32>,
    pub monthly_spend_cap_usd: Option<f32>,
    pub warn_at_percent: Vec<u8>,
    pub auto_local_after_cap: bool,
}

impl Default for CostControlConfig {
    fn default() -> Self {
        Self {
            monthly_minute_cap: None,
            monthly_spend_cap_usd: Some(5.0),
            warn_at_percent: vec![50, 80, 100],
            auto_local_after_cap: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub save_history: bool,
    pub auto_delete_days: Option<u32>,
    pub never_save_audio: bool,
    pub sensitive_app_blocklist: Vec<String>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            save_history: true,
            auto_delete_days: Some(30),
            never_save_audio: true,
            sensitive_app_blocklist: vec![
                "com.1password.1password".into(),
                "com.apple.keychainaccess".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppProfile {
    pub app_id: String,
    pub name: String,
    pub output_mode: OutputMode,
    pub disable_cloud: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    #[default]
    Balanced,
    PlainText,
    Markdown,
    Email,
    Casual,
    TerminalSafe,
    CodePreserve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub trigger: String,
    pub expansion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub term: String,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub key: Option<String>,
    pub tier: LicenseTier,
    pub validated_at: Option<String>,
}

impl Default for LicenseInfo {
    fn default() -> Self {
        Self {
            key: None,
            tier: LicenseTier::Free,
            validated_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTier {
    #[default]
    Free,
    Pro,
    Beta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub quality_mode: QualityMode,
    pub dictation_mode: DictationMode,
    pub hotkey: HotkeyConfig,
    pub microphone_device: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_model: String,
    pub local_model: String,
    pub session_cap_seconds: u32,
    pub clipboard_restore: bool,
    pub launch_at_login: bool,
    pub bar_position: BarPosition,
    pub cost_control: CostControlConfig,
    pub privacy: PrivacyConfig,
    pub crash_reporting_opt_in: bool,
    pub analytics_opt_in: bool,
    pub onboarding_complete: bool,
    pub beta_invite_code: Option<String>,
    pub license: LicenseInfo,
    pub app_profiles: Vec<AppProfile>,
    pub snippets: Vec<Snippet>,
    pub dictionary: Vec<DictionaryEntry>,
    pub cleanup_enabled: bool,
    pub cleanup_prompt: String,
    pub rewrite_commands: Vec<RewriteCommand>,
    pub history_limit_free: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteCommand {
    pub name: String,
    pub prompt: String,
    pub hotkey_suffix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BarPosition {
    #[default]
    BottomCenter,
    BottomRight,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            quality_mode: QualityMode::Hybrid,
            dictation_mode: DictationMode::PushToTalk,
            hotkey: HotkeyConfig::default(),
            microphone_device: None,
            openai_api_key: None,
            openai_model: "gpt-4o-mini-transcribe".into(),
            local_model: "tiny".into(),
            session_cap_seconds: 120,
            clipboard_restore: true,
            launch_at_login: false,
            bar_position: BarPosition::default(),
            cost_control: CostControlConfig::default(),
            privacy: PrivacyConfig::default(),
            crash_reporting_opt_in: false,
            analytics_opt_in: false,
            onboarding_complete: false,
            beta_invite_code: None,
            license: LicenseInfo::default(),
            app_profiles: default_app_profiles(),
            snippets: Vec::new(),
            dictionary: Vec::new(),
            cleanup_enabled: false,
            cleanup_prompt: "Fix punctuation and capitalization only. Keep meaning.".into(),
            rewrite_commands: Vec::new(),
            history_limit_free: 50,
        }
    }
}

fn default_app_profiles() -> Vec<AppProfile> {
    vec![
        AppProfile {
            app_id: "com.todesktop.230313mzl4w4u92".into(),
            name: "Cursor".into(),
            output_mode: OutputMode::CodePreserve,
            disable_cloud: false,
        },
        AppProfile {
            app_id: "com.microsoft.VSCode".into(),
            name: "VS Code".into(),
            output_mode: OutputMode::CodePreserve,
            disable_cloud: false,
        },
        AppProfile {
            app_id: "com.apple.Terminal".into(),
            name: "Terminal".into(),
            output_mode: OutputMode::TerminalSafe,
            disable_cloud: false,
        },
        AppProfile {
            app_id: "com.tinyspeck.slackmacgap".into(),
            name: "Slack".into(),
            output_mode: OutputMode::Casual,
            disable_cloud: false,
        },
    ]
}

impl Settings {
    pub fn is_pro(&self) -> bool {
        matches!(self.license.tier, LicenseTier::Pro | LicenseTier::Beta)
    }

    pub fn max_history_entries(&self) -> u32 {
        if self.is_pro() {
            u32::MAX
        } else {
            self.history_limit_free
        }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "maskedsyntax", "VoxFlow").map(|d| d.config_dir().to_path_buf())
}

/// Path used by early scripts/docs before `directories` canonical layout.
pub fn legacy_config_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|d| {
        d.home_dir()
            .join("Library/Application Support/maskedsyntax/VoxFlow")
    })
}

pub fn legacy_settings_path() -> Option<PathBuf> {
    legacy_config_dir().map(|d| d.join("settings.json"))
}

pub fn data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "maskedsyntax", "VoxFlow").map(|d| d.data_dir().to_path_buf())
}

pub fn settings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

pub fn load_settings() -> Result<Settings, ConfigError> {
    let Some(path) = settings_path() else {
        return Ok(Settings::default());
    };

    let legacy = legacy_settings_path().filter(|p| p.exists() && *p != path);
    let legacy_settings = legacy
        .as_ref()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|data| serde_json::from_str::<Settings>(&data).ok());

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(legacy_path) = &legacy {
            fs::copy(legacy_path, &path)?;
        } else {
            return Ok(Settings::default());
        }
    }

    let data = fs::read_to_string(&path)?;
    let mut settings: Settings = serde_json::from_str(&data)?;

    if let Some(legacy_settings) = legacy_settings {
        merge_legacy_settings(&mut settings, &legacy_settings);
        save_settings(&settings)?;
    }

    Ok(settings)
}

/// Prefer user-edited values from the legacy init-settings.sh path.
fn merge_legacy_settings(current: &mut Settings, legacy: &Settings) {
    if legacy
        .openai_api_key
        .as_ref()
        .is_some_and(|k| !k.trim().is_empty())
    {
        current.openai_api_key = legacy.openai_api_key.clone();
    }
    if legacy.hotkey != current.hotkey {
        current.hotkey = legacy.hotkey.clone();
    }
    if legacy.onboarding_complete {
        current.onboarding_complete = true;
    }
}

pub fn save_settings(settings: &Settings) -> Result<(), ConfigError> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(settings)?;
    fs::write(path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_hybrid_mode() {
        let s = Settings::default();
        assert_eq!(s.quality_mode, QualityMode::Hybrid);
        assert!(!s.is_pro());
    }
}
