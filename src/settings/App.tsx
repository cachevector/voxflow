import { useState } from "react";
import logo from "../assets/logo.png";
import { useSettings } from "./useSettings";
import General from "./pages/General";
import Hotkey from "./pages/Hotkey";
import Microphone from "./pages/Microphone";
import Transcription from "./pages/Transcription";
import AIRewrite from "./pages/AIRewrite";
import Providers from "./pages/Providers";
import CostControl from "./pages/CostControl";
import PerAppProfiles from "./pages/PerAppProfiles";
import Privacy from "./pages/Privacy";
import Advanced from "./pages/Advanced";
import History from "./pages/History";
import Dictionary from "./pages/Dictionary";

const PAGES = [
  { id: "general", label: "General", Component: General },
  { id: "hotkey", label: "Hotkey", Component: Hotkey },
  { id: "microphone", label: "Microphone", Component: Microphone },
  { id: "history", label: "History", Component: History },
  { id: "transcription", label: "Speech (Whisper)", Component: Transcription },
  { id: "dictionary", label: "Vocabulary", Component: Dictionary },
  { id: "ai-rewrite", label: "AI Cleanup (Groq)", Component: AIRewrite },
  { id: "providers", label: "Providers", Component: Providers },
  { id: "cost-control", label: "Cost Control", Component: CostControl },
  { id: "per-app", label: "Per-App Profiles", Component: PerAppProfiles },
  { id: "privacy", label: "Privacy", Component: Privacy },
  { id: "advanced", label: "Advanced", Component: Advanced },
] as const;

export default function App() {
  const [activeId, setActiveId] = useState<(typeof PAGES)[number]["id"]>("general");
  const { settings, update, save, saving, error } = useSettings();

  const active = PAGES.find((p) => p.id === activeId)!;

  return (
    <div className="flex h-screen bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <nav className="w-56 shrink-0 border-r border-neutral-200 p-4 dark:border-neutral-800">
        <div className="mb-6 flex items-center gap-2 px-2">
          <img src={logo} alt="" className="h-6 w-6 rounded-md" />
          <span className="font-semibold">VoxFlow</span>
        </div>
        <ul className="space-y-1">
          {PAGES.map((p) => (
            <li key={p.id}>
              <button
                onClick={() => setActiveId(p.id)}
                className={`w-full rounded-lg px-3 py-2 text-left text-sm transition-colors ${
                  p.id === activeId
                    ? "bg-accent/10 text-accent font-medium"
                    : "text-neutral-600 hover:bg-neutral-100 dark:text-neutral-400 dark:hover:bg-neutral-900"
                }`}
              >
                {p.label}
              </button>
            </li>
          ))}
        </ul>
      </nav>

      <main className="flex-1 overflow-y-auto p-8">
        {!settings ? (
          <p className="text-sm text-neutral-500">
            {error ? `Failed to load settings: ${error}` : "Loading settings…"}
          </p>
        ) : (
          <div className="mx-auto max-w-2xl">
            <active.Component settings={settings} update={update} />
            <div className="mt-8 flex items-center gap-3 border-t border-neutral-200 pt-4 dark:border-neutral-800">
              <button
                onClick={save}
                disabled={saving}
                className="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white disabled:opacity-50"
              >
                {saving ? "Saving…" : "Save changes"}
              </button>
              {error && <span className="text-sm text-red-500">{error}</span>}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export type PageProps = {
  settings: import("@/shared/types").Settings;
  update: (patch: Partial<import("@/shared/types").Settings>) => void;
};
