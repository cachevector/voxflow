import type { PageProps } from "../App";
import { useEffect, useState } from "react";
import { Row, Section, Select, Toggle } from "../components/Field";
import { commands } from "@/shared/tauri";

export default function Privacy({ settings, update }: PageProps) {
  const [monitoring, setMonitoring] = useState<boolean | null>(null);
  const [accessibility, setAccessibility] = useState<boolean | null>(null);
  const [manualEditLearningBlocked, setManualEditLearningBlocked] = useState(false);

  useEffect(() => {
    commands
      .getPermissionStatus()
      .then((status) => {
        setMonitoring(status.input_monitoring_granted);
        setAccessibility(status.accessibility_granted);
      })
      .catch(() => {
        setMonitoring(false);
        setAccessibility(false);
      });
  }, []);

  const toggleManualEditLearning = (enabled: boolean) => {
    if (!enabled || (monitoring && accessibility)) {
      setManualEditLearningBlocked(false);
      update({
        privacy: { ...settings.privacy, learn_from_manual_edits: enabled },
      });
    } else {
      setManualEditLearningBlocked(true);
    }
  };

  return (
    <Section title="Privacy">
      <Row label="Save history locally">
        <Toggle
          checked={settings.privacy.save_history}
          onChange={(v) => update({ privacy: { ...settings.privacy, save_history: v } })}
        />
      </Row>
      <Row label="Auto-delete history after">
        <Select
          value={String(settings.privacy.auto_delete_days ?? "")}
          onChange={(v) =>
            update({
              privacy: { ...settings.privacy, auto_delete_days: v ? Number(v) : null },
            })
          }
          options={[
            { value: "", label: "Never" },
            { value: "1", label: "1 day" },
            { value: "7", label: "7 days" },
            { value: "30", label: "30 days" },
          ]}
        />
      </Row>
      <Row label="Never save audio" hint="Only the transcript text is stored, never raw audio">
        <Toggle
          checked={settings.privacy.never_save_audio}
          onChange={(v) => update({ privacy: { ...settings.privacy, never_save_audio: v } })}
        />
      </Row>
      <Row
        label="Learn from manual edits"
        hint="For 15 seconds after VoxFlow inserts text, detect a small correction in the same field. VoxFlow never records keystrokes or saves field text. macOS only."
      >
        <Toggle
          checked={settings.privacy.learn_from_manual_edits}
          onChange={toggleManualEditLearning}
        />
      </Row>
      {(manualEditLearningBlocked || settings.privacy.learn_from_manual_edits) &&
        !(monitoring && accessibility) && (
        <div className="rounded-lg border border-amber-400/40 bg-amber-400/10 p-3 text-xs text-neutral-700 dark:text-neutral-200">
          <p>Manual-edit learning is off until both macOS permissions are enabled.</p>
          <div className="mt-2 flex gap-2">
            {!accessibility && (
              <button
                type="button"
                className="rounded bg-neutral-100 px-2 py-1 dark:bg-neutral-800"
                onClick={() => commands.openAccessibilitySettings()}
              >
                Open Accessibility
              </button>
            )}
            {!monitoring && (
              <button
                type="button"
                className="rounded bg-neutral-100 px-2 py-1 dark:bg-neutral-800"
                onClick={() => commands.openInputMonitoringSettings()}
              >
                Open Input Monitoring
              </button>
            )}
          </div>
        </div>
      )}
      <div>
        <label className="mb-1 block text-sm font-medium">Sensitive app blocklist</label>
        <textarea
          value={settings.privacy.sensitive_app_blocklist.join("\n")}
          onChange={(e) =>
            update({
              privacy: {
                ...settings.privacy,
                sensitive_app_blocklist: e.target.value.split("\n").filter(Boolean),
              },
            })
          }
          rows={4}
          className="w-full rounded-lg border border-neutral-300 bg-transparent p-3 font-mono text-xs outline-none focus:border-accent dark:border-neutral-700"
        />
        <p className="mt-1 text-xs text-neutral-500">
          One app bundle/process ID per line. Dictation is refused entirely for these apps.
        </p>
      </div>
    </Section>
  );
}
