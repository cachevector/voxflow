import React from "react";
import {
  AbsoluteFill,
  Img,
  Sequence,
  interpolate,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { theme } from "./theme";

export const FPS = 30;
export const WIDTH = 1920;
export const HEIGHT = 1080;
export const DURATION = 22 * FPS;

const RAW = "um so can you uh check whether the the cache is warm before we ship";
const CLEAN = "Can you check whether the cache is warm before we ship?";

const clamp = { extrapolateLeft: "clamp" as const, extrapolateRight: "clamp" as const };

const fade = (frame: number, start: number, end: number, out = 10) =>
  interpolate(frame, [start, start + 12, end - out, end], [0, 1, 1, 0], clamp);

const Brand: React.FC<{ size?: number }> = ({ size = 44 }) => (
  <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
    <Img
      src={staticFile("logo.png")}
      style={{ width: size, height: size, borderRadius: Math.round(size * 0.22) }}
    />
    <span
      style={{
        fontFamily: theme.display,
        fontWeight: 700,
        fontSize: size * 0.64,
        letterSpacing: "-0.03em",
        color: theme.text,
      }}
    >
      VoxFlow
    </span>
    <span
      style={{
        fontFamily: theme.mono,
        fontSize: 12,
        letterSpacing: "0.14em",
        textTransform: "uppercase",
        color: theme.faint,
        border: `1px solid ${theme.line}`,
        borderRadius: 999,
        padding: "3px 8px 2px",
      }}
    >
      macOS
    </span>
  </div>
);

const Mic: React.FC = () => (
  <svg
    width={22}
    height={22}
    viewBox="0 0 24 24"
    fill="none"
    stroke={theme.signal}
    strokeWidth={2}
    strokeLinecap="round"
  >
    <rect x="9" y="2" width="6" height="11" rx="3" fill={theme.signal} stroke="none" />
    <path d="M6 11.5v.5a6 6 0 0 0 12 0v-.5" />
    <path d="M12 19.5V22" />
  </svg>
);

const Wave: React.FC<{ frame: number; talking?: boolean }> = ({ frame, talking = true }) => {
  const bars = 12;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 3, height: 28 }}>
      {new Array(bars).fill(0).map((_, i) => {
        const talk = talking ? 0.35 + 0.65 * Math.abs(Math.sin(frame * 0.23 + i * 0.55)) : 0.18;
        const env = 0.35 + 0.65 * Math.sin((i / (bars - 1)) * Math.PI);
        const h = 5 + talk * env * 22;
        return (
          <div
            key={i}
            style={{
              width: 3,
              height: h,
              borderRadius: 2,
              background: theme.signal,
            }}
          />
        );
      })}
    </div>
  );
};

const Pill: React.FC<{ frame: number; label: string; seconds: number }> = ({
  frame,
  label,
  seconds,
}) => (
  <div
    style={{
      display: "flex",
      alignItems: "center",
      gap: 14,
      height: 56,
      minWidth: 340,
      padding: "0 18px",
      borderRadius: 999,
      background: "rgba(17, 20, 24, 0.86)",
      border: "1px solid rgba(255,255,255,0.1)",
      boxShadow: "0 18px 40px -16px rgba(0,0,0,0.7)",
      color: "#fff",
    }}
  >
    <Mic />
    <Wave frame={frame} talking={label === "Listening"} />
    <span style={{ fontSize: 15, fontWeight: 600 }}>{label}</span>
    <span
      style={{
        marginLeft: "auto",
        fontFamily: theme.mono,
        fontSize: 12,
        color: theme.faint,
      }}
    >
      0:0{Math.min(9, Math.max(0, seconds))}
    </span>
  </div>
);

const Footer: React.FC = () => (
  <div
    style={{
      position: "absolute",
      left: 80,
      right: 80,
      bottom: 40,
      display: "flex",
      justifyContent: "space-between",
      fontFamily: theme.mono,
      fontSize: 15,
      letterSpacing: "0.12em",
      textTransform: "uppercase",
      color: theme.faint,
    }}
  >
    <span>hold · speak · release</span>
    <span>voxflow.cachevector.com</span>
  </div>
);

const Stage: React.FC<{ children: React.ReactNode; opacity: number }> = ({ children, opacity }) => (
  <AbsoluteFill style={{ opacity, padding: "64px 80px 48px" }}>{children}</AbsoluteFill>
);

const Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = spring({ frame, fps, config: { damping: 16, mass: 0.7 } });
  const glow = interpolate(frame, [0, 40], [0, 1], clamp);
  return (
    <Stage opacity={fade(frame, 0, 75, 12)}>
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `radial-gradient(50% 45% at 50% 42%, rgba(0,178,197,${0.22 * glow}), transparent 70%)`,
        }}
      />
      <div
        style={{
          height: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          transform: `translateY(${(1 - enter) * 28}px) scale(${0.92 + enter * 0.08})`,
        }}
      >
        <Img
          src={staticFile("logo.png")}
          style={{ width: 128, height: 128, borderRadius: 28, marginBottom: 28 }}
        />
        <div
          style={{
            fontFamily: theme.display,
            fontWeight: 700,
            fontSize: 72,
            letterSpacing: "-0.035em",
          }}
        >
          VoxFlow
        </div>
        <div
          style={{
            marginTop: 10,
            fontFamily: theme.mono,
            letterSpacing: "0.16em",
            textTransform: "uppercase",
            color: theme.faint,
            fontSize: 16,
          }}
        >
          dictation for macOS
        </div>
      </div>
    </Stage>
  );
};

const Tagline: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const a = spring({ frame, fps, config: { damping: 18 } });
  const b = spring({ frame: frame - 10, fps, config: { damping: 18 } });
  return (
    <Stage opacity={fade(frame, 0, 105, 12)}>
      <Brand />
      <div style={{ marginTop: 88 }}>
        <div
          style={{
            fontFamily: theme.display,
            fontWeight: 700,
            fontSize: 88,
            lineHeight: 0.96,
            letterSpacing: "-0.038em",
            opacity: a,
            transform: `translateY(${(1 - a) * 20}px)`,
          }}
        >
          Speak anywhere.
        </div>
        <div
          style={{
            fontFamily: theme.display,
            fontWeight: 700,
            fontSize: 88,
            lineHeight: 0.96,
            letterSpacing: "-0.038em",
            color: theme.signalBright,
            opacity: b,
            transform: `translateY(${(1 - b) * 20}px)`,
          }}
        >
          VoxFlow writes it for you.
        </div>
        <div
          style={{
            marginTop: 28,
            fontSize: 26,
            color: theme.muted,
            maxWidth: "28em",
            opacity: interpolate(frame, [20, 36], [0, 1], clamp),
          }}
        >
          Hold Option+Ctrl. Whisper transcribes on your Mac. Clean text lands at the cursor.
        </div>
      </div>
      <Footer />
    </Stage>
  );
};

const OverlayDemo: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const pill = spring({ frame: frame - 18, fps, config: { damping: 14, mass: 0.8 } });
  const seconds = Math.min(6, Math.floor(frame / fps));
  const label = frame < 130 ? "Listening" : "Transcribing";
  return (
    <Stage opacity={fade(frame, 0, 165, 12)}>
      <Brand />
      <div
        style={{
          marginTop: 36,
          flex: 1,
          position: "relative",
          height: 720,
          border: `1px solid ${theme.line}`,
          borderRadius: 18,
          background: "linear-gradient(180deg, #101820 0%, #0c131a 100%)",
          overflow: "hidden",
        }}
      >
        <div style={{ padding: "40px 48px 0", fontSize: 24, lineHeight: 1.55, color: "#6d8294" }}>
          The window that already had focus. Slack, Cursor, Mail, the terminal.
          <br />
          Nothing steals it. The pill never covers the line you are on.
        </div>
        <div
          style={{
            position: "absolute",
            left: "50%",
            bottom: 36,
            transform: `translateX(-50%) translateY(${(1 - pill) * 40}px)`,
            opacity: pill,
          }}
        >
          <Pill frame={frame} label={label} seconds={seconds} />
        </div>
      </div>
      <Footer />
    </Stage>
  );
};

const Rewrite: React.FC = () => {
  const frame = useCurrentFrame();
  const cleaned = frame > 48;
  const flash = interpolate(frame, [44, 52, 64], [0, 1, 0], clamp);
  return (
    <Stage opacity={fade(frame, 0, 120, 12)}>
      <Brand />
      <div style={{ marginTop: 80, maxWidth: 1400 }}>
        <div
          style={{
            fontFamily: theme.mono,
            fontSize: 14,
            letterSpacing: "0.16em",
            textTransform: "uppercase",
            color: cleaned ? theme.rewrite : theme.signalBright,
            marginBottom: 18,
          }}
        >
          {cleaned ? "rewrite" : "transcript"}
        </div>
        <div
          style={{
            fontFamily: theme.display,
            fontWeight: 600,
            fontSize: 48,
            lineHeight: 1.2,
            letterSpacing: "-0.025em",
            color: cleaned ? theme.text : theme.muted,
          }}
        >
          {cleaned ? CLEAN : RAW}
        </div>
      </div>
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `rgba(232, 163, 61, ${0.12 * flash})`,
          pointerEvents: "none",
        }}
      />
      <Footer />
    </Stage>
  );
};

const Pillars: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const items = [
    { k: "transcribe", t: "On device", d: "Audio never leaves the Mac." },
    { k: "rewrite", t: "Your key", d: "Groq, OpenAI, or turn it off." },
    { k: "price", t: "Free", d: "The app is free. Forever." },
  ];
  return (
    <Stage opacity={fade(frame, 0, 90, 12)}>
      <Brand />
      <div
        style={{
          marginTop: 72,
          fontFamily: theme.display,
          fontWeight: 700,
          fontSize: 64,
          letterSpacing: "-0.035em",
        }}
      >
        Your voice stays home.
      </div>
      <div style={{ display: "flex", gap: 20, marginTop: 48 }}>
        {items.map((item, i) => {
          const enter = spring({
            frame: frame - i * 8,
            fps,
            config: { damping: 16 },
          });
          return (
            <div
              key={item.k}
              style={{
                flex: 1,
                border: `1px solid ${theme.line}`,
                borderRadius: 16,
                background: theme.surface,
                padding: "32px 28px",
                opacity: enter,
                transform: `translateY(${(1 - enter) * 24}px)`,
              }}
            >
              <div
                style={{
                  fontFamily: theme.mono,
                  fontSize: 13,
                  letterSpacing: "0.14em",
                  textTransform: "uppercase",
                  color: theme.signalBright,
                }}
              >
                {item.k}
              </div>
              <div
                style={{
                  fontFamily: theme.display,
                  fontSize: 36,
                  letterSpacing: "-0.03em",
                  marginTop: 14,
                }}
              >
                {item.t}
              </div>
              <div style={{ marginTop: 12, fontSize: 20, color: theme.muted, lineHeight: 1.4 }}>
                {item.d}
              </div>
            </div>
          );
        })}
      </div>
      <Footer />
    </Stage>
  );
};

const EndCard: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const enter = spring({ frame, fps, config: { damping: 16 } });
  return (
    <Stage opacity={interpolate(frame, [0, 12], [0, 1], clamp)}>
      <div
        style={{
          height: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          transform: `translateY(${(1 - enter) * 20}px)`,
        }}
      >
        <Img src={staticFile("logo.png")} style={{ width: 96, height: 96, borderRadius: 22 }} />
        <div
          style={{
            fontFamily: theme.display,
            fontWeight: 700,
            fontSize: 64,
            letterSpacing: "-0.035em",
            marginTop: 22,
          }}
        >
          The app is free.
        </div>
        <div style={{ marginTop: 14, fontSize: 26, color: theme.muted }}>
          Never for the minutes.
        </div>
        <div
          style={{
            marginTop: 36,
            fontFamily: theme.mono,
            letterSpacing: "0.14em",
            textTransform: "uppercase",
            color: theme.signalBright,
            fontSize: 18,
          }}
        >
          voxflow.cachevector.com
        </div>
      </div>
    </Stage>
  );
};

export const VoxFlowPromo: React.FC = () => {
  return (
    <AbsoluteFill style={{ background: theme.ground, fontFamily: theme.body, color: theme.text }}>
      <AbsoluteFill
        style={{
          background:
            "radial-gradient(60% 55% at 50% 0%, rgba(0,178,197,0.16), transparent 70%)",
        }}
      />
      <Sequence from={0} durationInFrames={75}>
        <Intro />
      </Sequence>
      <Sequence from={63} durationInFrames={105}>
        <Tagline />
      </Sequence>
      <Sequence from={156} durationInFrames={165}>
        <OverlayDemo />
      </Sequence>
      <Sequence from={309} durationInFrames={120}>
        <Rewrite />
      </Sequence>
      <Sequence from={417} durationInFrames={90}>
        <Pillars />
      </Sequence>
      <Sequence from={495} durationInFrames={165}>
        <EndCard />
      </Sequence>
    </AbsoluteFill>
  );
};
