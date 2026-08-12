import type { PageProps } from "../App";
import { Row, Section, Select } from "../components/Field";
import { commands } from "@/shared/tauri";

export default function Hotkey({ settings, update }: PageProps) {
  return (
    <Section title="Hotkey">
      <Row label="Dictation mode">
        <Select
          value={settings.dictation_mode}
          onChange={(v) => update({ dictation_mode: v as PageProps["settings"]["dictation_mode"] })}
          options={[
            { value: "push_to_talk", label: "Push to talk — hold Option+Ctrl" },
            { value: "toggle", label: "Toggle — press Option+Ctrl to start/stop" },
          ]}
        />
      </Row>
      <Row label="Binding">
        <span className="rounded-lg bg-neutral-100 px-3 py-2 font-mono text-sm dark:bg-neutral-800">
          {settings.hotkey.label}
        </span>
      </Row>
      <p className="text-xs text-neutral-500">
        Option+Ctrl uses a global event tap on macOS. Enable VoxFlow under{" "}
        <strong>Privacy &amp; Security → Accessibility</strong> so the hotkey and automatic paste
        work system-wide.
      </p>
      <button
        type="button"
        className="text-sm text-accent underline"
        onClick={() => commands.openAccessibilitySettings()}
      >
        Open Accessibility settings
      </button>
    </Section>
  );
}
