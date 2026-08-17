import { motion } from "framer-motion";

/** A quiet three-dot progress animation used while audio is being processed. */
export function BufferingIndicator() {
  return (
    <span aria-label="Processing dictation" className="flex w-8 shrink-0 items-center justify-center gap-1">
      {[0, 0.16, 0.32].map((delay) => (
        <motion.span
          key={delay}
          aria-hidden
          className="h-1.5 w-1.5 rounded-full bg-white/70"
          animate={{ opacity: [0.28, 1, 0.28], scale: [0.72, 1, 0.72] }}
          transition={{ duration: 0.72, delay, repeat: Infinity, ease: "easeInOut" }}
        />
      ))}
    </span>
  );
}
