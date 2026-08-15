import { useEffect, useRef } from "react";

interface WaveformProps {
  /** Live mic input level, 0..1. */
  amplitude: number;
  /** True while actively listening; drives the resting animation. */
  active: boolean;
}

const WIDTH = 120;
const HEIGHT = 28;
/** Brand cyan (sampled from the logo) so the wave and the mark read as one system. */
const COLOR = "rgba(0, 178, 197, 0.95)";

/**
 * A continuously animated voice waveform. Two travelling sine components with a
 * centered envelope give an organic "wave" that ripples along the pill, and its
 * height tracks the live mic level (smoothed) so it visibly grows when the user
 * speaks and settles to a gentle idle ripple when they're quiet.
 */
export function Waveform({ amplitude, active }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef(0);
  // Latest inputs read by the animation loop, so it always sees fresh values
  // without the effect re-subscribing on every amplitude update.
  const targetRef = useRef(0);
  const activeRef = useRef(active);
  const levelRef = useRef(0);

  targetRef.current = active ? amplitude : 0;
  activeRef.current = active;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = WIDTH * dpr;
    canvas.height = HEIGHT * dpr;
    ctx.scale(dpr, dpr);

    const start = performance.now();

    const render = (now: number) => {
      const t = (now - start) / 1000;

      // Ease the drawn level toward the live target so the wave is fluid, not jittery.
      levelRef.current += (targetRef.current - levelRef.current) * 0.2;
      const level = levelRef.current;
      const isActive = activeRef.current;

      ctx.clearRect(0, 0, WIDTH, HEIGHT);

      const mid = HEIGHT / 2;
      const maxAmp = HEIGHT / 2 - 2;
      // A small baseline ripple keeps it a living wave rather than a flat line.
      const base = isActive ? 0.14 : 0.05;
      const amp = Math.min(1, base + level * 0.9) * maxAmp;
      const speed = isActive ? 6 : 2.5;

      ctx.beginPath();
      for (let x = 0; x <= WIDTH; x++) {
        const nx = x / WIDTH;
        const envelope = Math.sin(nx * Math.PI); // taper to zero at both ends
        const wave =
          Math.sin(nx * Math.PI * 2 * 2 - t * speed) * 0.6 +
          Math.sin(nx * Math.PI * 2 * 3.5 + t * (speed * 0.7)) * 0.4;
        const y = mid + wave * amp * envelope;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.strokeStyle = COLOR;
      ctx.lineWidth = 2;
      ctx.lineJoin = "round";
      ctx.lineCap = "round";
      ctx.stroke();

      rafRef.current = requestAnimationFrame(render);
    };

    rafRef.current = requestAnimationFrame(render);
    return () => cancelAnimationFrame(rafRef.current);
  }, []);

  return <canvas ref={canvasRef} style={{ width: WIDTH, height: HEIGHT }} />;
}
