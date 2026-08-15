/** Single source of truth for site-wide metadata, reused by SEO tags and JSON-LD. */
export const SITE = {
  name: "VoxFlow",
  tagline: "Speak anywhere. VoxFlow writes it for you.",
  description:
    "VoxFlow is a BYOK voice dictation app for macOS. Hold Option+Ctrl, speak, and get clean text at your cursor — local Whisper transcription, an AI cleanup pass, and no subscription.",
  url: "https://cachevector.github.io/voxflow",
  repo: "https://github.com/cachevector/voxflow",
  download: "https://github.com/cachevector/voxflow/releases/latest/download/VoxFlow-macos-arm64.dmg",
  releases: "https://github.com/cachevector/voxflow/releases/latest",
  studio: "MaskedSyntax",
  studioUrl: "https://maskedsyntax.com",
  ogImage: "/og.png",
  keywords: [
    "AI dictation",
    "voice typing",
    "speech to text",
    "macOS dictation app",
    "Whisper dictation",
    "BYOK transcription",
    "local transcription",
    "Wispr Flow alternative",
    "system-wide dictation",
    "voice input for developers",
  ],
} as const;

export const NAV = [
  { href: "/#how", label: "How it works" },
  { href: "/#byok", label: "Bring your own key" },
  { href: "/#pricing", label: "Pricing" },
  { href: "/docs", label: "Docs" },
] as const;

/**
 * Prefix an in-site path with the Astro `base` (required on GitHub project Pages).
 * Absolute http(s) URLs and bare hashes are left alone.
 */
export function withBase(href: string): string {
  if (/^(https?:)?\/\//.test(href) || href.startsWith("mailto:") || href.startsWith("#")) {
    return href;
  }
  const base = (import.meta.env.BASE_URL ?? "/").replace(/\/$/, "");
  if (!href.startsWith("/")) return `${base}/${href}`;
  return `${base}${href}`;
}
