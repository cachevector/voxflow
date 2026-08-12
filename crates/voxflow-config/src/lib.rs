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
            key_code: 0,
            modifiers: 0,
            label: "Option+Ctrl".into(),
        }
    }
}

/// Which kind of OpenAI-API-compatible endpoint a `ProviderConfig` targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    #[default]
    OpenAi,
    Groq,
    CustomEndpoint,
}

impl ProviderKind {
    /// Default base URL for known cloud providers. `CustomEndpoint` has no
    /// default — the user must supply one.
    pub fn default_base_url(self) -> Option<&'static str> {
        match self {
            ProviderKind::OpenAi => Some("https://api.openai.com/v1"),
            ProviderKind::Groq => Some("https://api.groq.com/openai/v1"),
            ProviderKind::CustomEndpoint => None,
        }
    }
}

/// Config for a single OpenAI-compatible call target (transcription or rewrite).
/// The actual secret lives in the OS keychain, looked up by `api_key_ref` via
/// `voxflow-secrets` — never stored here in plaintext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    /// Required for `CustomEndpoint`; overrides the default for known providers if set.
    pub base_url: Option<String>,
    pub model: String,
    /// Optional higher-accuracy model used when the router picks the accurate tier.
    pub accurate_model: Option<String>,
    /// Keyring lookup key (e.g. "transcription", "rewrite"), not the secret itself.
    pub api_key_ref: Option<String>,
}

impl ProviderConfig {
    pub fn resolved_base_url(&self) -> String {
        self.base_url
            .clone()
            .or_else(|| self.kind.default_base_url().map(str::to_string))
            .unwrap_or_default()
    }

    pub fn default_transcription() -> Self {
        Self {
            kind: ProviderKind::OpenAi,
            base_url: None,
            model: "gpt-4o-mini-transcribe".into(),
            accurate_model: Some("gpt-4o-transcribe".into()),
            api_key_ref: Some("transcription".into()),
        }
    }

    pub fn default_rewrite() -> Self {
        Self {
            kind: ProviderKind::Groq,
            base_url: None,
            model: "llama-3.1-8b-instant".into(),
            accurate_model: None,
            api_key_ref: Some("rewrite".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostControlConfig {
    pub monthly_minute_cap: Option<f32>,
    pub monthly_spend_cap_usd: Option<f32>,
    pub warn_at_percent: Vec<u8>,
    pub auto_downgrade_after_cap: bool,
}

impl Default for CostControlConfig {
    fn default() -> Self {
        Self {
            monthly_minute_cap: None,
            monthly_spend_cap_usd: Some(5.0),
            warn_at_percent: vec![50, 80, 100],
            auto_downgrade_after_cap: true,
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
pub struct WhisperConfig {
    /// GGML model id without prefix, e.g. `small.en`.
    pub model_id: String,
    pub prewarm_on_launch: bool,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            model_id: "small.en".into(),
            prewarm_on_launch: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub schema_version: u32,
    pub quality_mode: QualityMode,
    pub dictation_mode: DictationMode,
    pub hotkey: HotkeyConfig,
    pub microphone_device: Option<String>,
    #[serde(default)]
    pub whisper: WhisperConfig,
    pub transcription_provider: ProviderConfig,
    pub rewrite_provider: ProviderConfig,
    /// AI rewrite pass runs on every transcript by default (not Pro-gated).
    pub rewrite_enabled: bool,
    pub rewrite_prompt: String,
    pub session_cap_seconds: u32,
    pub clipboard_restore: bool,
    pub launch_at_login: bool,
    pub bar_position: BarPosition,
    pub cost_control: CostControlConfig,
    pub privacy: PrivacyConfig,
    pub crash_reporting_opt_in: bool,
    pub analytics_opt_in: bool,
    pub onboarding_complete: bool,
    pub license: LicenseInfo,
    pub app_profiles: Vec<AppProfile>,
    pub snippets: Vec<Snippet>,
    pub dictionary: Vec<DictionaryEntry>,
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
            schema_version: 3,
            quality_mode: QualityMode::Hybrid,
            dictation_mode: DictationMode::PushToTalk,
            hotkey: HotkeyConfig::default(),
            microphone_device: None,
            whisper: WhisperConfig::default(),
            transcription_provider: ProviderConfig::default_transcription(),
            rewrite_provider: ProviderConfig::default_rewrite(),
            rewrite_enabled: true,
            rewrite_prompt: "You clean up dictated speech. Remove filler words and false starts (um, uh, hmm, er). Fix grammar, punctuation, and capitalization. Do NOT change meaning, add content, or rephrase for style. Return ONLY the corrected text with no quotes or markdown.".into(),
            session_cap_seconds: 120,
            clipboard_restore: true,
            launch_at_login: false,
            bar_position: BarPosition::default(),
            cost_control: CostControlConfig::default(),
            privacy: PrivacyConfig::default(),
            crash_reporting_opt_in: false,
            analytics_opt_in: false,
            onboarding_complete: false,
            license: LicenseInfo::default(),
            app_profiles: default_app_profiles(),
            snippets: Vec::new(),
            dictionary: Vec::new(),
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
    pub fn max_history_entries(&self) -> u32 {
        if matches!(self.license.tier, LicenseTier::Pro | LicenseTier::Beta) {
            u32::MAX
        } else {
            self.history_limit_free
        }
    }
}

pub fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "maskedsyntax", "VoxFlow").map(|d| d.config_dir().to_path_buf())
}

pub fn data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "maskedsyntax", "VoxFlow").map(|d| d.data_dir().to_path_buf())
}

pub fn settings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

/// Reads the legacy plaintext key (if a pre-v2 settings.json is still on disk)
/// so callers can migrate it into the keychain before the first save overwrites it.
pub fn legacy_plaintext_openai_key() -> Option<String> {
    let path = settings_path()?;
    let data = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&data).ok()?;
    value
        .get("openai_api_key")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

pub fn load_settings() -> Result<Settings, ConfigError> {
    let Some(path) = settings_path() else {
        return Ok(Settings::default());
    };

    if !path.exists() {
        return Ok(Settings::default());
    }

    let data = fs::read_to_string(&path)?;
    let settings: Settings = serde_json::from_str(&data)?;
    Ok(settings)
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
        assert!(s.rewrite_enabled);
        assert_eq!(s.max_history_entries(), 50);
    }

    #[test]
    fn custom_endpoint_has_no_default_base_url() {
        assert_eq!(ProviderKind::CustomEndpoint.default_base_url(), None);
        let mut cfg = ProviderConfig::default_transcription();
        cfg.kind = ProviderKind::CustomEndpoint;
        cfg.base_url = Some("http://raspberrypi.local:8080/v1".into());
        assert_eq!(cfg.resolved_base_url(), "http://raspberrypi.local:8080/v1");
    }
}
