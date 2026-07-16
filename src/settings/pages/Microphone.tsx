import { useEffect, useState } from "react";
import type { PageProps } from "../App";
import { Row, Section, Select } from "../components/Field";
import { commands } from "@/shared/tauri";
import type { AudioDeviceInfo } from "@/shared/types";

export default function Microphone({ settings, update }: PageProps) {
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    commands
      .listAudioDevices()
      .then(setDevices)
      .catch((e) => setError(String(e)));
  }, []);

  return (
    <Section title="Microphone">
      <Row label="Input device">
        <Select
          value={settings.microphone_device ?? ""}
          onChange={(v) => update({ microphone_device: v || null })}
          options={[
            { value: "", label: "System default" },
            ...devices.map((d) => ({ value: d.id, label: d.name })),
          ]}
        />
      </Row>
      {error && <p className="text-xs text-red-500">Couldn't list devices: {error}</p>}
    </Section>
  );
}
