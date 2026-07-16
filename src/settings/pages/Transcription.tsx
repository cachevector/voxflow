import type { PageProps } from "../App";
import { Section } from "../components/Field";
import { ProviderConfigEditor } from "../components/ProviderConfigEditor";

export default function Transcription({ settings, update }: PageProps) {
  return (
    <Section title="Transcription">
      <ProviderConfigEditor
        config={settings.transcription_provider}
        onChange={(transcription_provider) => update({ transcription_provider })}
      />
    </Section>
  );
}
