import { useEffect, useRef } from "react";

interface WaveformProps {
  amplitude: number;
  active: boolean;
  history: number[];
}

const BAR_COUNT = 24;
const BAR_WIDTH = 3;
const BAR_GAP = 2;
const HEIGHT = 28;

export function Waveform({ history, active }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = BAR_COUNT * (BAR_WIDTH + BAR_GAP);
    canvas.width = width * devicePixelRatio;
    canvas.height = HEIGHT * devicePixelRatio;
    ctx.scale(devicePixelRatio, devicePixelRatio);
    ctx.clearRect(0, 0, width, HEIGHT);

    const samples = history.slice(-BAR_COUNT);
    for (let i = 0; i < BAR_COUNT; i++) {
      const level = samples[i] ?? 0;
      const barHeight = active ? Math.max(2, Math.min(HEIGHT, level * HEIGHT)) : 3;
      const x = i * (BAR_WIDTH + BAR_GAP);
      const y = (HEIGHT - barHeight) / 2;
      const edgeFade = 1 - Math.abs(i - BAR_COUNT / 2) / (BAR_COUNT / 2);
      ctx.fillStyle = `rgba(140, 127, 240, ${0.35 + 0.65 * edgeFade})`;
      ctx.beginPath();
      ctx.roundRect(x, y, BAR_WIDTH, barHeight, 1.5);
      ctx.fill();
    }
  }, [history, active]);

  return (
    <canvas
      ref={canvasRef}
      style={{ width: BAR_COUNT * (BAR_WIDTH + BAR_GAP), height: HEIGHT }}
    />
  );
}
