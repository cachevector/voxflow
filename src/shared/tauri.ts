import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AudioDeviceInfo,
  CostDashboard,
  HistoryEntry,
  HistoryCorrectionResult,
  Settings,
  StateEvent,
  VocabularySuggestion,
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
  correctHistoryEntry: (id: string, correctedText: string) =>
    invoke<HistoryCorrectionResult>("correct_history_entry", { id, correctedText }),
  acceptVocabularySuggestion: (suggestion: VocabularySuggestion) =>
    invoke<void>("accept_vocabulary_suggestion", { suggestion }),
  dismissVocabularySuggestion: (suggestion: VocabularySuggestion) =>
    invoke<void>("dismiss_vocabulary_suggestion", { suggestion }),
  restoreVocabularySuggestion: (suggestion: VocabularySuggestion) =>
    invoke<void>("restore_vocabulary_suggestion", { suggestion }),
  respondToEditLearningSuggestion: (accepted: boolean) =>
    invoke<void>("respond_to_edit_learning_suggestion", { accepted }),
  listAudioDevices: () => invoke<AudioDeviceInfo[]>("list_audio_devices"),
  getPermissionStatus: () =>
    invoke<{
      accessibility_hint: string;
      accessibility_granted: boolean;
      microphone_hint: string;
      input_monitoring_hint: string;
      input_monitoring_granted: boolean;
    }>("get_permission_status"),
  openAccessibilitySettings: () => invoke<void>("open_accessibility_settings"),
  openInputMonitoringSettings: () => invoke<void>("open_input_monitoring_settings"),
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
  onVocabularySuggestion: (cb: (suggestion: VocabularySuggestion) => void): Promise<UnlistenFn> =>
    listen<VocabularySuggestion>("dictation://vocabulary-suggestion", (e) => cb(e.payload)),
  onVocabularySuggestionCleared: (cb: () => void): Promise<UnlistenFn> =>
    listen("dictation://vocabulary-suggestion-cleared", cb),
};
