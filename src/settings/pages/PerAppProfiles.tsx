import type { PageProps } from "../App";
import { Row, Section, Select, Toggle } from "../components/Field";
import type { OutputMode } from "@/shared/types";

const OUTPUT_MODES: { value: OutputMode; label: string }[] = [
  { value: "balanced", label: "Balanced" },
  { value: "plain_text", label: "Plain text" },
  { value: "markdown", label: "Markdown" },
  { value: "email", label: "Email" },
  { value: "casual", label: "Casual" },
  { value: "terminal_safe", label: "Terminal-safe" },
  { value: "code_preserve", label: "Code-preserve" },
];

export default function PerAppProfiles({ settings, update }: PageProps) {
  const setProfile = (index: number, patch: Partial<PageProps["settings"]["app_profiles"][number]>) => {
    const next = settings.app_profiles.slice();
    next[index] = { ...next[index], ...patch };
    update({ app_profiles: next });
  };

  return (
    <Section title="Per-App Profiles">
      {settings.app_profiles.map((profile, i) => (
        <div key={profile.app_id} className="rounded-lg border border-neutral-200 p-3 dark:border-neutral-800">
          <div className="mb-2 text-sm font-medium">{profile.name}</div>
          <Row label="Output mode">
            <Select
              value={profile.output_mode}
              onChange={(v) => setProfile(i, { output_mode: v as OutputMode })}
              options={OUTPUT_MODES}
            />
          </Row>
          <Row label="Disable cloud for this app" hint="Route to the cheapest tier instead">
            <Toggle
              checked={profile.disable_cloud}
              onChange={(v) => setProfile(i, { disable_cloud: v })}
            />
          </Row>
        </div>
      ))}
    </Section>
  );
}
