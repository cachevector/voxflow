import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { commands, events } from "@/shared/tauri";
import type { StateEvent, UiState, VocabularySuggestion } from "@/shared/types";
import { Waveform } from "./components/Waveform";
import { Timer } from "./components/Timer";
import { LogoMark } from "./components/LogoMark";
import { BufferingIndicator } from "./components/BufferingIndicator";

export default function App() {
  const [event, setEvent] = useState<StateEvent | null>(null);
  const [amplitude, setAmplitude] = useState(0);
  const [suggestion, setSuggestion] = useState<VocabularySuggestion | null>(null);
  const visible = suggestion !== null || (event !== null && event.ui_state !== "idle");
  const uiState: UiState = event?.ui_state ?? "idle";

  useEffect(() => {
    const unlistenState = events.onDictationState(setEvent);
    const unlistenAmp = events.onDictationAmplitude(setAmplitude);
    const unlistenSuggestion = events.onVocabularySuggestion(setSuggestion);
    const unlistenSuggestionCleared = events.onVocabularySuggestionCleared(() => setSuggestion(null));
    return () => {
      unlistenState.then((f) => f());
      unlistenAmp.then((f) => f());
      unlistenSuggestion.then((f) => f());
      unlistenSuggestionCleared.then((f) => f());
    };
  }, []);

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-transparent">
      <AnimatePresence>
        {visible && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 8 }}
            transition={{ duration: 0.16, ease: [0.22, 1, 0.36, 1] }}
            className="flex h-[52px] w-full items-center justify-center gap-2.5 rounded-pill border border-white/[0.08] bg-neutral-950/90 px-3.5 text-white shadow-lg backdrop-blur-xl"
          >
            {suggestion ? (
              <>
                <span className="text-xs text-neutral-300">
                  Learn “{suggestion.term}” → “{suggestion.replacement}”?
                </span>
                <button
                  type="button"
                  className="rounded bg-accent px-2 py-1 text-xs font-medium text-white"
                  onClick={() => {
                    commands.respondToEditLearningSuggestion(true).finally(() => setSuggestion(null));
                  }}
                >
                  Learn
                </button>
                <button
                  type="button"
                  className="text-xs text-neutral-400"
                  onClick={() => {
                    commands.respondToEditLearningSuggestion(false).finally(() => setSuggestion(null));
                  }}
                >
                  Not now <span className="sr-only">(Escape)</span>
                </button>
              </>
            ) : (
              <>
                <LogoMark amplitude={amplitude} active={uiState === "listening"} />
                <Waveform amplitude={amplitude} active={uiState === "listening"} />
                {uiState === "listening" ? <Timer /> : <BufferingIndicator />}
              </>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
