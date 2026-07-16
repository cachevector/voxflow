import type { PageProps } from "../App";
import { Row, Section, Select, Toggle } from "../components/Field";

export default function Privacy({ settings, update }: PageProps) {
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
