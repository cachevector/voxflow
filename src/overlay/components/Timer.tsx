import { useEffect, useState } from "react";

export function Timer() {
  const [elapsedMs, setElapsedMs] = useState(0);

  useEffect(() => {
    const start = performance.now();
    const id = setInterval(() => setElapsedMs(performance.now() - start), 100);
    return () => clearInterval(id);
  }, []);

  const totalSeconds = elapsedMs / 1000;
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = (totalSeconds % 60).toFixed(1).padStart(4, "0");

  return (
    <span className="shrink-0 font-mono text-xs tabular-nums text-white/70">
      {minutes}:{seconds}
    </span>
  );
}
