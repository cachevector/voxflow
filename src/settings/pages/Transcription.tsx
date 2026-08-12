import type { PageProps } from "../App";
import { Row, Section, Toggle } from "../components/Field";

export default function Transcription({ settings, update }: PageProps) {
  return (
    <Section title="Local speech (Whisper)">
      <Row label="Model">
        <select
          className="rounded-lg border border-neutral-200 bg-white px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-900"
          value={settings.whisper.model_id}
          onChange={(e) =>
            update({ whisper: { ...settings.whisper, model_id: e.target.value } })
          }
        >
          <option value="tiny.en">tiny.en (fastest)</option>
          <option value="base.en">base.en</option>
          <option value="small.en">small.en (recommended)</option>
        </select>
      </Row>
      <Row label="Prewarm on launch" hint="Load the model at startup for lower first-dictation latency.">
        <Toggle
          checked={settings.whisper.prewarm_on_launch}
          onChange={(prewarm_on_launch) =>
            update({ whisper: { ...settings.whisper, prewarm_on_launch } })
          }
        />
      </Row>
      <p className="text-xs text-neutral-500">
        Speech-to-text runs locally via whisper.cpp (Metal on Apple Silicon). The model downloads
        on first use (~500MB for small.en).
      </p>
    </Section>
  );
}
