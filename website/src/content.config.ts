import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
// Imported directly rather than via astro:content, whose `z` re-export is deprecated.
import { z } from "zod";

const docs = defineCollection({
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs" }),
  schema: z.object({
    title: z.string(),
    /** Used verbatim as the page meta description — write it for search results. */
    description: z.string(),
    /** Sidebar section heading. */
    group: z.string(),
    /** Sort order within the whole sidebar. */
    order: z.number(),
    /** Optional shorter label for the sidebar. */
    sidebarLabel: z.string().optional(),
  }),
});

export const collections = { docs };
