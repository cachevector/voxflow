import type { PageProps } from "../App";
import { Row, Section, Select } from "../components/Field";

export default function Hotkey({ settings, update }: PageProps) {
  return (
    <Section title="Hotkey">
      <Row label="Dictation mode">
        <Select
          value={settings.dictation_mode}
          onChange={(v) => update({ dictation_mode: v as PageProps["settings"]["dictation_mode"] })}
          options={[
            { value: "push_to_talk", label: "Push to talk (hold)" },
            { value: "toggle", label: "Toggle (press to start/stop)" },
          ]}
        />
      </Row>
      <Row label="Current binding" hint={settings.hotkey.label}>
        <span className="text-xs text-neutral-500">
          Re-binding UI coming soon — edit settings.json directly for now.
        </span>
      </Row>
      <p className="text-xs text-neutral-500">
        Bare-modifier bindings (like plain Left Control) need the advanced raw-hotkey mode and, on
        macOS, Input Monitoring permission. Combo bindings (e.g. Cmd+Shift+Space) don't need it.
      </p>
    </Section>
  );
}
