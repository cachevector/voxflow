// The Tauri app at the repo root has its own postcss.config.js with Tailwind.
// Without this file, Vite walks up and applies that config to the website's CSS.
// The site uses plain CSS with custom properties, so no plugins are needed.
export default { plugins: {} };
