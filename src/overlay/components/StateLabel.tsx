import { AnimatePresence, motion } from "framer-motion";
import type { UiState } from "@/shared/types";

const LABELS: Record<UiState, string> = {
  idle: "",
  listening: "Listening",
  transcribing: "Transcribing",
  cleaning: "Cleaning",
  inserting: "Inserting",
  copied: "Copied",
  done: "Done",
  error: "Something went wrong",
};

export function StateLabel({ state, message }: { state: UiState; message: string | null }) {
  const text = state === "error" && message ? message : LABELS[state];
  return (
    <div className="flex-1 overflow-hidden">
      <AnimatePresence mode="wait">
        <motion.span
          key={text}
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -4 }}
          transition={{ duration: 0.1 }}
          className="block truncate text-sm font-medium"
        >
          {text}
        </motion.span>
      </AnimatePresence>
    </div>
  );
}
