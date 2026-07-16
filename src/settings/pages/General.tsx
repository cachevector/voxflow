import type { PageProps } from "../App";
import { Row, Section, Select, Toggle } from "../components/Field";

export default function General({ settings, update }: PageProps) {
  return (
    <Section title="General">
      <Row label="Quality mode" hint="How aggressively to prefer cheap vs. accurate transcription">
        <Select
          value={settings.quality_mode}
          onChange={(v) => update({ quality_mode: v as PageProps["settings"]["quality_mode"] })}
          options={[
            { value: "hybrid", label: "Hybrid (recommended)" },
            { value: "economy", label: "Economy" },
            { value: "balanced", label: "Balanced" },
            { value: "accurate", label: "Accurate" },
          ]}
        />
      </Row>
      <Row label="Bar position">
        <Select
          value={settings.bar_position}
          onChange={(v) => update({ bar_position: v as PageProps["settings"]["bar_position"] })}
          options={[
            { value: "bottom_center", label: "Bottom center" },
            { value: "bottom_right", label: "Bottom right" },
          ]}
        />
      </Row>
      <Row label="Launch at login">
        <Toggle checked={settings.launch_at_login} onChange={(v) => update({ launch_at_login: v })} />
      </Row>
    </Section>
  );
}
