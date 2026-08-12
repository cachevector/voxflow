import type { PageProps } from "../App";
import { Row, Section, Toggle } from "../components/Field";

export default function Advanced({ settings, update }: PageProps) {
  return (
    <Section title="Advanced">
      <Row label="Session cap" hint="Stop recording automatically after this many seconds">
        <input
          type="number"
          value={settings.session_cap_seconds}
          onChange={(e) => update({ session_cap_seconds: Number(e.target.value) })}
          className="w-24 rounded-lg border border-neutral-300 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-accent dark:border-neutral-700"
        />
      </Row>
      <Row label="Restore clipboard after paste">
        <Toggle
          checked={settings.clipboard_restore}
          onChange={(v) => update({ clipboard_restore: v })}
        />
      </Row>
      <Row label="Crash reporting" hint="Off by default">
        <Toggle
          checked={settings.crash_reporting_opt_in}
          onChange={(v) => update({ crash_reporting_opt_in: v })}
        />
      </Row>
      <Row label="Anonymous analytics" hint="Off by default">
        <Toggle
          checked={settings.analytics_opt_in}
          onChange={(v) => update({ analytics_opt_in: v })}
        />
      </Row>
    </Section>
  );
}
