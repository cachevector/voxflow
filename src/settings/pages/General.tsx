import { useEffect, useState } from "react";
import type { PageProps } from "../App";
import { Row, Section, Select, Toggle } from "../components/Field";
import { commands } from "@/shared/tauri";

export default function General({ settings, update }: PageProps) {
  const [perm, setPerm] = useState<{
    accessibility_hint: string;
    accessibility_granted: boolean;
    microphone_hint: string;
    input_monitoring_hint: string;
    input_monitoring_granted: boolean;
  } | null>(null);
  const [modelReady, setModelReady] = useState<boolean | null>(null);
  const [downloading, setDownloading] = useState(false);

  useEffect(() => {
    commands.getPermissionStatus().then(setPerm).catch(console.error);
    commands.whisperModelReady().then(setModelReady).catch(console.error);
  }, []);

  const finishOnboarding = async () => {
    await commands.completeOnboarding();
    update({ onboarding_complete: true });
  };

  const downloadModel = async () => {
    setDownloading(true);
    try {
      await commands.downloadWhisperModel();
      setModelReady(true);
    } finally {
      setDownloading(false);
    }
  };

  return (
    <>
      {!settings.onboarding_complete && (
        <Section title="Welcome to VoxFlow">
          <ol className="list-decimal space-y-3 pl-5 text-sm text-neutral-600 dark:text-neutral-400">
            <li>{perm?.microphone_hint ?? "Allow microphone access when prompted."}</li>
            <li>{perm?.accessibility_hint ?? "Enable Accessibility for Option+Ctrl and paste."}</li>
            <li>
              Add your Groq API key under <strong>AI Cleanup</strong> for grammar/filler cleanup.
            </li>
            <li>
              {modelReady
                ? "Whisper model is ready."
                : "Download the local Whisper model (~500MB for small.en)."}
            </li>
          </ol>
          <div className="mt-4 flex flex-wrap gap-2">
            <button
              type="button"
              className="rounded-lg bg-neutral-100 px-3 py-2 text-sm dark:bg-neutral-800"
              onClick={() => commands.openAccessibilitySettings()}
            >
              Open Accessibility
            </button>
            {!modelReady && (
              <button
                type="button"
                disabled={downloading}
                className="rounded-lg bg-accent px-3 py-2 text-sm text-white disabled:opacity-50"
                onClick={downloadModel}
              >
                {downloading ? "Downloading…" : "Download Whisper model"}
              </button>
            )}
            <button
              type="button"
              className="rounded-lg border border-neutral-200 px-3 py-2 text-sm dark:border-neutral-700"
              onClick={finishOnboarding}
            >
              Done
            </button>
          </div>
        </Section>
      )}

      <Section title="General">
        <Row label="Bar position">
          <Select
            value={settings.bar_position}
            onChange={(v) => update({ bar_position: v as PageProps["settings"]["bar_position"] })}
            options={[{ value: "bottom_center", label: "Bottom center (Wispr-style)" }]}
          />
        </Row>
        <Row label="Launch at login">
          <Toggle
            checked={settings.launch_at_login}
            onChange={(v) => update({ launch_at_login: v })}
          />
        </Row>
      </Section>
    </>
  );
}
