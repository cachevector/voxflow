// Mirrors the Rust types in crates/voxflow-config, crates/voxflow-core, and
// crates/voxflow-cost. Keep in sync by hand for now — these crates are the
// source of truth (see #[derive(Serialize, Deserialize)] on each struct).

export type ProviderKind = "open_ai" | "groq" | "custom_endpoint";

export interface ProviderConfig {
  kind: ProviderKind;
  base_url: string | null;
  model: string;
  accurate_model: string | null;
  api_key_ref: string | null;
}

export type QualityMode = "hybrid" | "economy" | "balanced" | "accurate";
export type DictationMode = "push_to_talk" | "toggle";
export type BarPosition = "bottom_center" | "bottom_right";
export type OutputMode =
  | "balanced"
  | "plain_text"
  | "markdown"
  | "email"
  | "casual"
  | "terminal_safe"
  | "code_preserve";

export interface HotkeyConfig {
  key_code: number;
  modifiers: number;
  label: string;
}

export interface CostControlConfig {
  monthly_minute_cap: number | null;
  monthly_spend_cap_usd: number | null;
  warn_at_percent: number[];
  auto_downgrade_after_cap: boolean;
}

export interface PrivacyConfig {
  save_history: boolean;
  auto_delete_days: number | null;
  never_save_audio: boolean;
  sensitive_app_blocklist: string[];
}

export interface AppProfile {
  app_id: string;
  name: string;
  output_mode: OutputMode;
  disable_cloud: boolean;
}

export interface Snippet {
  trigger: string;
  expansion: string;
}

export interface DictionaryEntry {
  term: string;
  replacement: string | null;
}

export interface RewriteCommand {
  name: string;
  prompt: string;
  hotkey_suffix: string | null;
}

export interface WhisperConfig {
  model_id: string;
  prewarm_on_launch: boolean;
}

export interface Settings {
  schema_version: number;
  quality_mode: QualityMode;
  dictation_mode: DictationMode;
  hotkey: HotkeyConfig;
  microphone_device: string | null;
  whisper: WhisperConfig;
  transcription_provider: ProviderConfig;
  rewrite_provider: ProviderConfig;
  rewrite_enabled: boolean;
  rewrite_prompt: string;
  session_cap_seconds: number;
  clipboard_restore: boolean;
  launch_at_login: boolean;
  bar_position: BarPosition;
  cost_control: CostControlConfig;
  privacy: PrivacyConfig;
  crash_reporting_opt_in: boolean;
  analytics_opt_in: boolean;
  onboarding_complete: boolean;
  app_profiles: AppProfile[];
  snippets: Snippet[];
  dictionary: DictionaryEntry[];
  rewrite_commands: RewriteCommand[];
  history_limit_free: number;
}

export type DictationState =
  | "idle"
  | "listening"
  | "finalizing"
  | "transcribing"
  | "cleaning"
  | "inserting"
  | "copied"
  | "done"
  | "error";

export type UiState =
  | "idle"
  | "listening"
  | "cleaning"
  | "inserting"
  | "copied"
  | "done"
  | "error";

export interface StateEvent {
  state: DictationState;
  ui_state: UiState;
  message: string | null;
  transcript_preview: string | null;
}

export interface CostDashboard {
  minutes_transcribed: number;
  estimated_usd: number;
  estimated_inr: number;
  current_provider: string;
  current_model: string;
  average_daily_usd: number;
  projected_monthly_usd: number;
  projected_monthly_inr: number;
  self_hosted_percentage: number;
  cloud_percentage: number;
  cap_warnings: string[];
}

export interface HistoryEntry {
  id: string;
  text: string;
  created_at: string;
  provider: string;
  model: string;
  duration_raw_secs: number;
  duration_billable_secs: number;
  active_app_id: string | null;
  estimated_usd: number;
}

export interface AudioDeviceInfo {
  id: string;
  name: string;
  is_default: boolean;
}
