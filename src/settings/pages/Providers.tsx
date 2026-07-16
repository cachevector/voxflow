import { useState } from "react";
import type { PageProps } from "../App";
import { Row, Section } from "../components/Field";
import { commands } from "@/shared/tauri";

export default function Providers({ settings }: PageProps) {
  const [message, setMessage] = useState<string | null>(null);

  const keyRefs = [
    { ref: settings.transcription_provider.api_key_ref, label: "Transcription key" },
    { ref: settings.rewrite_provider.api_key_ref, label: "AI Rewrite key" },
  ].filter((k): k is { ref: string; label: string } => Boolean(k.ref));

  const remove = async (keyRef: string) => {
    await commands.deleteProviderKey(keyRef);
    setMessage(`Removed key for "${keyRef}" from the OS keychain.`);
  };

  return (
    <Section title="Providers">
      <p className="text-sm text-neutral-500">
        Keys are entered per call type on the Transcription and AI Rewrite pages and stored in the
        OS keychain (macOS Keychain / Windows Credential Manager) — never in plaintext settings.
      </p>
      {keyRefs.map((k) => (
        <Row key={k.ref} label={k.label} hint={`keychain ref: ${k.ref}`}>
          <button
            onClick={() => remove(k.ref)}
            className="rounded-lg border border-neutral-300 px-3 py-1.5 text-sm dark:border-neutral-700"
          >
            Remove
          </button>
        </Row>
      ))}
      {message && <p className="text-xs text-neutral-500">{message}</p>}
    </Section>
  );
}
