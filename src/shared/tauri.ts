import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AudioDeviceInfo,
  CostDashboard,
  HistoryEntry,
  Settings,
  StateEvent,
} from "./types";

export const commands = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  setProviderKey: (keyRef: string, secret: string) =>
    invoke<void>("set_provider_key", { keyRef, secret }),
  deleteProviderKey: (keyRef: string) => invoke<void>("delete_provider_key", { keyRef }),
  getCostDashboard: () => invoke<CostDashboard>("get_cost_dashboard"),
  listHistory: (limit: number) => invoke<HistoryEntry[]>("list_history", { limit }),
  exportHistoryJson: (limit: number) => invoke<string>("export_history_json", { limit }),
  exportHistoryCsv: (limit: number) => invoke<string>("export_history_csv", { limit }),
  listAudioDevices: () => invoke<AudioDeviceInfo[]>("list_audio_devices"),
  getPermissionStatus: () =>
    invoke<{ accessibility_hint: string; microphone_hint: string }>("get_permission_status"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
  pasteText: (text: string) => invoke<{ success: boolean }>("paste_text", { text }),
  whisperModelReady: () => invoke<boolean>("whisper_model_ready"),
  downloadWhisperModel: () => invoke<string>("download_whisper_model"),
  completeOnboarding: () => invoke<void>("complete_onboarding"),
};

export const events = {
  onDictationState: (cb: (event: StateEvent) => void): Promise<UnlistenFn> =>
    listen<StateEvent>("dictation://state", (e) => cb(e.payload)),
  onDictationAmplitude: (cb: (level: number) => void): Promise<UnlistenFn> =>
    listen<number>("dictation://amplitude", (e) => cb(e.payload)),
};
