import { useEffect, useRef } from "react";

interface WaveformProps {
  /** Live mic input level, 0..1. */
  amplitude: number;
  active: boolean;
}

const WIDTH = 94;
const HEIGHT = 28;
const BAR_COUNT = 15;
const BAR_WIDTH = 3;
const BAR_GAP = 3.2;
const SILENCE_FLOOR = 0.035;

const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, value));

function roundedBar(
  ctx: CanvasRenderingContext2D,
  x: number,
  height: number,
  opacity: number,
) {
  const y = (HEIGHT - height) / 2;
  const radius = Math.min(BAR_WIDTH / 2, height / 2);

  ctx.beginPath();
  ctx.roundRect(x, y, BAR_WIDTH, height, radius);
  ctx.fillStyle = `rgba(255, 255, 255, ${opacity})`;
  ctx.fill();
}

/**
 * A voice-reactive capsule meter. Unlike the old travelling sine wave, these
 * bars describe the microphone level directly: they settle into quiet dots,
 * jump quickly with speech, and fall away a little more slowly so syllables
 * remain readable without jitter.
 */
export function Waveform({ amplitude, active }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const frameRef = useRef(0);
  const amplitudeRef = useRef(0);
  const activeRef = useRef(active);

  amplitudeRef.current = active ? clamp(amplitude, 0, 1) : 0;
  activeRef.current = active;

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = WIDTH * dpr;
    canvas.height = HEIGHT * dpr;
    ctx.scale(dpr, dpr);

    const levels = Array.from({ length: BAR_COUNT }, () => 2);
    const seeds = Array.from({ length: BAR_COUNT }, (_, index) =>
      Math.sin(index * 91.17) * 0.5 + 0.5,
    );
    let displayLevel = 0;
    let lastFrame = performance.now();

    const render = (now: number) => {
      const delta = Math.min(32, now - lastFrame) / 16.67;
      lastFrame = now;

      const incoming = amplitudeRef.current;
      const levelEase = incoming > displayLevel ? 0.34 : 0.13;
      displayLevel += (incoming - displayLevel) * levelEase * delta;

      ctx.clearRect(0, 0, WIDTH, HEIGHT);

      const speaking = activeRef.current && displayLevel > SILENCE_FLOOR;
      const normalized = speaking
        ? clamp((displayLevel - SILENCE_FLOOR) / (1 - SILENCE_FLOOR), 0, 1)
        : 0;
      const energy = Math.pow(normalized, 0.58);
      const totalWidth = BAR_COUNT * BAR_WIDTH + (BAR_COUNT - 1) * BAR_GAP;
      const startX = (WIDTH - totalWidth) / 2;

      for (let index = 0; index < BAR_COUNT; index += 1) {
        const distanceFromCenter = Math.abs(index - (BAR_COUNT - 1) / 2);
        const envelope = 1 - (distanceFromCenter / (BAR_COUNT / 2)) * 0.48;
        const pulse =
          0.72 +
          Math.sin(now * (0.009 + seeds[index] * 0.004) + seeds[index] * 8) * 0.18 +
          Math.sin(now * 0.017 + index * 1.7) * 0.1;
        const target = speaking
          ? 3 + energy * 22 * envelope * clamp(pulse, 0.5, 1.05)
          : 2;
        const barEase = target > levels[index] ? 0.38 : 0.16;
        levels[index] += (target - levels[index]) * barEase * delta;

        roundedBar(
          ctx,
          startX + index * (BAR_WIDTH + BAR_GAP),
          clamp(levels[index], 2, HEIGHT - 2),
          speaking ? 0.92 : activeRef.current ? 0.38 : 0.2,
        );
      }

      frameRef.current = requestAnimationFrame(render);
    };

    frameRef.current = requestAnimationFrame(render);
    return () => cancelAnimationFrame(frameRef.current);
  }, []);

  return (
    <canvas
      ref={canvasRef}
      aria-label="Live microphone level"
      role="img"
      style={{ width: WIDTH, height: HEIGHT }}
    />
  );
}
