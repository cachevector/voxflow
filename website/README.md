# VoxFlow website

Marketing site and documentation for VoxFlow, built with [Astro](https://astro.build).
Deployed to GitHub Pages at [voxflow.cachevector.com](https://voxflow.cachevector.com).

This is a standalone project — it has its own `package.json` and lockfile and is not part of
the root pnpm workspace or the Tauri app's build.

## Commands

Run from this directory:

```bash
pnpm install
pnpm dev
```

| Command | What it does |
|---|---|
| `pnpm dev` | Dev server on `localhost:4321` |
| `pnpm build` | Static build into `dist/`, then regenerates `public/og.png` |
| `pnpm preview` | Serve the built `dist/` locally |
| `pnpm check` | Astro + TypeScript diagnostics |
| `pnpm og` | Regenerate the Open Graph image only |

## Layout

```
src/
  site.ts              Site-wide metadata — name, URL, description, keywords, nav
  components/          Seo, Nav, Footer, Logo, DictationTrace (the hero animation)
  layouts/             BaseLayout (head + chrome), DocsLayout (sidebar, TOC, pager)
  content/docs/        Documentation pages, one Markdown file each
  pages/               index.astro, 404.astro, docs/[...slug].astro
  styles/global.css    Design tokens and layout primitives
scripts/make-og.mjs    Renders public/og.png from an inline SVG
```

## Adding a documentation page

Drop a Markdown file into `src/content/docs/`. The filename becomes the URL
(`install.md` → `/docs/install`), and `index.md` is `/docs`. Frontmatter is validated by
`src/content.config.ts`:

```yaml
---
title: Page title
description: One sentence — this is also the page's meta description.
group: Start here      # sidebar section heading
order: 2               # position in the sidebar
sidebarLabel: Install  # optional shorter label
---
```

The sidebar, breadcrumbs, on-this-page TOC, previous/next links, and sitemap entry are all
generated from that.

## SEO

- `site` in `astro.config.mjs` drives canonical URLs, the sitemap, and absolute OG URLs.
  The custom domain is `voxflow.cachevector.com` (CNAME in `public/CNAME`).
  Push to `master` (or run the **GitHub Pages** workflow) to publish.
- `src/components/Seo.astro` emits title, description, canonical, robots, Open Graph, and
  Twitter card tags for every page.
- JSON-LD: `Organization` site-wide, `SoftwareApplication` and `FAQPage` on the homepage,
  `TechArticle` and `BreadcrumbList` on each docs page.
- `sitemap-index.xml` is generated at build time; `public/robots.txt` points at it.

## Theming

Light and dark are both real, driven by custom properties in `src/styles/global.css` and a
`data-theme` attribute on `<html>`. The choice is stored in `localStorage` and applied in an
inline script before first paint so it never flashes. All text colours meet WCAG AA against
their background in both themes.
