/**
 * Renders public/og.png (1200x630) from an inline SVG.
 *
 * Uses only system fonts, since sharp's SVG rasteriser resolves fonts through
 * fontconfig and will not see the site's webfonts. Run via `pnpm og`; the
 * build script runs it automatically.
 */
import { writeFile, mkdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import sharp from "sharp";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// Same envelope shape as the hero waveform: silence, speech, silence.
const bars = [3, 4, 3, 14, 30, 46, 38, 58, 70, 53, 79, 64, 49, 73, 86, 68, 52,
  76, 91, 63, 44, 70, 82, 57, 73, 97, 80, 61, 75, 52, 65, 79, 50, 37, 58, 42,
  27, 34, 20, 24, 13, 10, 5, 3];

const waveW = 1000;
const gap = 6;
const barW = (waveW - gap * (bars.length - 1)) / bars.length;
const waveTop = 470;
const waveMax = 96;

const wave = bars
  .map((h, i) => {
    const height = Math.max(3, (h / 100) * waveMax);
    const x = 100 + i * (barW + gap);
    const y = waveTop + (waveMax - height) / 2;
    const speech = i > 2 && i < bars.length - 3;
    const fill = speech ? "#00b2c5" : "#3d5060";
    return `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${barW.toFixed(1)}" height="${height.toFixed(1)}" rx="2.5" fill="${fill}"/>`;
  })
  .join("");

const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <defs>
    <radialGradient id="glow" cx="50%" cy="12%" r="70%">
      <stop offset="0%" stop-color="#00b2c5" stop-opacity="0.20"/>
      <stop offset="100%" stop-color="#00b2c5" stop-opacity="0"/>
    </radialGradient>
  </defs>
  <rect width="1200" height="630" fill="#0b1219"/>
  <rect width="1200" height="630" fill="url(#glow)"/>
  <rect y="626" width="1200" height="4" fill="#00b2c5"/>

  <g transform="translate(100,86)">
    <rect x="0" y="0" width="11" height="20" rx="5.5" fill="#00b2c5"/>
    <path d="M-4.5 19.5v1a10 10 0 0 0 20 0v-1" fill="none" stroke="#00b2c5" stroke-width="3.4" stroke-linecap="round"/>
    <path d="M5.5 31v4.5" fill="none" stroke="#00b2c5" stroke-width="3.4" stroke-linecap="round"/>
    <text x="34" y="30" font-family="Helvetica Neue, Helvetica, Arial" font-size="27" font-weight="700" fill="#eaf2f6" letter-spacing="-0.4">VoxFlow</text>
  </g>

  <text x="100" y="250" font-family="Helvetica Neue, Helvetica, Arial" font-size="76" font-weight="700" fill="#eaf2f6" letter-spacing="-2.6">Speak anywhere.</text>
  <text x="100" y="336" font-family="Helvetica Neue, Helvetica, Arial" font-size="76" font-weight="700" fill="#45dcec" letter-spacing="-2.6">VoxFlow writes it for you.</text>

  <text x="100" y="398" font-family="Helvetica Neue, Helvetica, Arial" font-size="26" fill="#93a8b8">Voice dictation for macOS. On-device Whisper, your own key, no subscription.</text>

  ${wave}

  <text x="100" y="602" font-family="Menlo, Courier New, monospace" font-size="19" fill="#64798a" letter-spacing="1.4">⌥ ⌃  HOLD · SPEAK · RELEASE</text>
  <text x="1100" y="602" text-anchor="end" font-family="Menlo, Courier New, monospace" font-size="19" fill="#64798a" letter-spacing="1.4">cachevector.github.io/voxflow</text>
</svg>`;

await mkdir(join(root, "public"), { recursive: true });
const png = await sharp(Buffer.from(svg)).png({ compressionLevel: 9 }).toBuffer();
await writeFile(join(root, "public", "og.png"), png);
// Also write it into dist/ when the build has already emitted, so a post-build
// run does not require a second `astro build`.
try {
  await writeFile(join(root, "dist", "og.png"), png);
} catch {
  /* dist/ may not exist yet on a bare `pnpm og` — public/og.png is enough. */
}
console.log(`og.png written (${(png.length / 1024).toFixed(1)} KB)`);
