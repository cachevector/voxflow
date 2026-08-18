interface LogoMarkProps {
  /** Live mic level, 0..1 — drives the halo so the mark breathes with your voice. */
  amplitude: number;
  /** True while actively listening. */
  active: boolean;
}

/**
 * Widget-sized VoxFlow mark.
 *
 * The full website logo is a lockup — mic + wave swoosh + shield + speed lines
 * on a white plate — which turns to mush at the ~22px this pill allows, and
 * whose own wave motif would compete with the live waveform sitting next to it.
 * This is the reduced form: just the microphone silhouette, the logo's core
 * shape, in a single brand colour so it stays crisp on the dark pill.
 *
 * It doubles as the status light — saturated cyan with a voice-reactive halo
 * while listening, dimmed while the transcription is being processed.
 */
export function LogoMark({ amplitude, active }: LogoMarkProps) {
  // Keep the mark calm and let the waveform carry the live audio feedback.
  const glow = active ? 0.28 + Math.min(1, amplitude) * 0.18 : 0.18;

  return (
    <span
      className="relative flex h-6 w-6 shrink-0 items-center justify-center"
      style={{ color: active ? "#00b2c5" : "rgba(255,255,255,0.55)" }}
    >
      <span
        aria-hidden
        className="absolute inset-0 rounded-full transition-opacity"
        style={{
          background:
            "radial-gradient(circle, rgba(0,178,197,0.55) 0%, rgba(0,178,197,0) 70%)",
          opacity: glow,
          transform: "scale(1.08)",
        }}
      />
      <svg
        viewBox="0 0 24 24"
        role="img"
        aria-label="VoxFlow"
        className="relative h-[18px] w-[18px]"
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
      >
        {/* Mic capsule — filled so it still reads as a solid mark at small sizes. */}
        <rect x="9" y="2" width="6" height="11" rx="3" fill="currentColor" stroke="none" />
        {/* Pickup arc + stand. */}
        <path d="M6 11.5v.5a6 6 0 0 0 12 0v-.5" />
        <path d="M12 19.5V22" />
      </svg>
    </span>
  );
}
