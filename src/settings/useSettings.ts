import { useCallback, useEffect, useState } from "react";
import { commands } from "@/shared/tauri";
import type { Settings } from "@/shared/types";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    commands
      .getSettings()
      .then(setSettings)
      .catch((e) => setError(String(e)));
  }, []);

  const update = useCallback((patch: Partial<Settings>) => {
    setSettings((prev) => (prev ? { ...prev, ...patch } : prev));
  }, []);

  const save = useCallback(async () => {
    if (!settings) return;
    setSaving(true);
    setError(null);
    try {
      await commands.saveSettings(settings);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }, [settings]);

  return { settings, update, save, saving, error };
}
