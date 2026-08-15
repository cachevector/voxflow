import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { events } from "@/shared/tauri";
import type { StateEvent, UiState } from "@/shared/types";
import { Waveform } from "./components/Waveform";
import { StateLabel } from "./components/StateLabel";
import { Timer } from "./components/Timer";
import { LogoMark } from "./components/LogoMark";

export default function App() {
  const [event, setEvent] = useState<StateEvent | null>(null);
  const [amplitude, setAmplitude] = useState(0);

  useEffect(() => {
    const unlistenState = events.onDictationState(setEvent);
    const unlistenAmp = events.onDictationAmplitude(setAmplitude);
    return () => {
      unlistenState.then((f) => f());
      unlistenAmp.then((f) => f());
    };
  }, []);

  const visible = event !== null && event.ui_state !== "idle";
  const uiState: UiState = event?.ui_state ?? "idle";

  return (
    <div className="flex h-screen w-screen items-center justify-center bg-transparent">
      <AnimatePresence>
        {visible && (
          <motion.div
            initial={{ opacity: 0, scale: 0.9, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.9, y: 8 }}
            transition={{ duration: 0.12, ease: "easeOut" }}
            className="flex h-14 min-w-[220px] max-w-[420px] items-center gap-3 rounded-pill border border-white/10 bg-neutral-900/80 px-4 text-white shadow-lg backdrop-blur-md"
          >
            <LogoMark amplitude={amplitude} active={uiState === "listening"} />
            <Waveform amplitude={amplitude} active={uiState === "listening"} />
            <StateLabel state={uiState} message={event?.message ?? null} />
            {uiState === "listening" && <Timer />}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
