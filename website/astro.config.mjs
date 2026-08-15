// @ts-check
import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import mdx from "@astrojs/mdx";

// `site` drives canonical URLs, the sitemap, and every absolute OG/JSON-LD URL.
// Custom domain is voxflow.cachevector.com (CNAME to cachevector.github.io).
export default defineConfig({
  site: "https://voxflow.cachevector.com",
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
