import { useState } from "react";
import { Row, Select, TextInput } from "./Field";
import { commands } from "@/shared/tauri";
import type { ProviderConfig, ProviderKind } from "@/shared/types";

const KIND_OPTIONS: { value: ProviderKind; label: string }[] = [
  { value: "open_ai", label: "OpenAI" },
  { value: "groq", label: "Groq" },
  { value: "custom_endpoint", label: "Custom Endpoint (self-hosted)" },
];

export function ProviderConfigEditor({
  config,
  onChange,
}: {
  config: ProviderConfig;
  onChange: (config: ProviderConfig) => void;
}) {
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [testResult, setTestResult] = useState<string | null>(null);

  const saveKey = async () => {
    if (!config.api_key_ref || !apiKeyInput.trim()) return;
    await commands.setProviderKey(config.api_key_ref, apiKeyInput.trim());
    setApiKeyInput("");
    setTestResult("Key saved to OS keychain.");
  };

  return (
    <div className="space-y-3 rounded-lg border border-neutral-200 p-4 dark:border-neutral-800">
      <Row label="Provider">
        <Select
          value={config.kind}
          onChange={(v) => onChange({ ...config, kind: v as ProviderKind })}
          options={KIND_OPTIONS}
        />
      </Row>

      {config.kind === "custom_endpoint" && (
        <Row label="Base URL" hint="e.g. http://raspberrypi.tailnet-name.ts.net:8080/v1">
          <TextInput
            value={config.base_url ?? ""}
            onChange={(e) => onChange({ ...config, base_url: e.target.value })}
            placeholder="http://raspberrypi.local:8080/v1"
            className="w-72"
          />
        </Row>
      )}

      <Row label="Model">
        <TextInput
          value={config.model}
          onChange={(e) => onChange({ ...config, model: e.target.value })}
          className="w-56"
        />
      </Row>

      <Row label="Accurate-tier model" hint="Used when the router picks the accurate tier">
        <TextInput
          value={config.accurate_model ?? ""}
          onChange={(e) => onChange({ ...config, accurate_model: e.target.value || null })}
          className="w-56"
        />
      </Row>

      <Row label="API key" hint="Stored in the OS keychain, never shown again after saving">
        <div className="flex gap-2">
          <TextInput
            type="password"
            value={apiKeyInput}
            onChange={(e) => setApiKeyInput(e.target.value)}
            placeholder="sk-…"
            className="w-56"
          />
          <button
            onClick={saveKey}
            className="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white"
          >
            Save
          </button>
        </div>
      </Row>
      {testResult && <p className="text-xs text-neutral-500">{testResult}</p>}
    </div>
  );
}
