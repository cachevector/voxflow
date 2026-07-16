import type { PageProps } from "../App";
import { Row, Section, Toggle } from "../components/Field";
import { ProviderConfigEditor } from "../components/ProviderConfigEditor";

export default function AIRewrite({ settings, update }: PageProps) {
  return (
    <Section title="AI Rewrite">
      <Row
        label="Rewrite every transcript"
        hint="On by default — turns rough speech into a clean, well-formed sentence"
      >
        <Toggle checked={settings.rewrite_enabled} onChange={(v) => update({ rewrite_enabled: v })} />
      </Row>

      <div>
        <label className="mb-1 block text-sm font-medium">Rewrite prompt</label>
        <textarea
          value={settings.rewrite_prompt}
          onChange={(e) => update({ rewrite_prompt: e.target.value })}
          rows={3}
          className="w-full rounded-lg border border-neutral-300 bg-transparent p-3 text-sm outline-none focus:border-accent dark:border-neutral-700"
        />
      </div>

      <ProviderConfigEditor
        config={settings.rewrite_provider}
        onChange={(rewrite_provider) => update({ rewrite_provider })}
      />
    </Section>
  );
}
