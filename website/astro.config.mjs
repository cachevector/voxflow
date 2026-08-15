// @ts-check
import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import mdx from "@astrojs/mdx";

// `site` + `base` drive canonical URLs, the sitemap, and every absolute OG/JSON-LD URL.
// Project Pages live at https://<owner>.github.io/<repo>, so `base` must be the repo name.
export default defineConfig({
  site: "https://cachevector.github.io",
  base: "/voxflow",
  trailingSlash: "never",
  integrations: [mdx(), sitemap()],
  build: { inlineStylesheets: "auto" },
  markdown: {
    shikiConfig: {
      // Both themes are emitted; DocsLayout's CSS picks one via [data-theme].
      themes: { light: "github-light", dark: "github-dark-dimmed" },
      wrap: true,
    },
  },
});
