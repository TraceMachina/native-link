import { defineConfig, passthroughImageService } from "astro/config";

import { rehypeHeadingIds } from "@astrojs/markdown-remark";
import react from "@astrojs/react";
import sitemap from "@astrojs/sitemap";
import starlight from "@astrojs/starlight";

import deno from "@deno/astro-adapter";
import qwik from "@qwikdev/astro";
import tailwindcss from "@tailwindcss/vite";
// import partytown from "@astrojs/partytown";

import { rehypeMermaid } from "@beoe/rehype-mermaid"; // "rehype-mermaid";
import rehypeAutolinkHeadings from "rehype-autolink-headings";

import { starlightConfig } from "./starlight.conf";

// https://astro.build/config
export default defineConfig({
  site: "https://nativelink.com",
  output: "server",
  image: {
    service: passthroughImageService(),
  },
  adapter: deno({
    port: 8881,
    hostname: "localhost",
  }),
  redirects: {
    "/blog/case-study%3A-samsung-internet's-integration-with-nativelink": {
      status: 301,
      destination: "/resources/blog/case-study-samsung",
    },
  },
  integrations: [
    qwik({
      include: ["**/components/qwik/**/*"],
    }),
    react({
      include: ["**/components/react/*"],
    }),
    starlight(starlightConfig),
    sitemap(),
  ],
  markdown: {
    rehypePlugins: [
      rehypeHeadingIds,
      [
        rehypeAutolinkHeadings,
        {
          behavior: "wrap",
        },
      ],
      [
        rehypeMermaid,
        {
          class: "not-content",
          strategy: "img-class-dark-mode",
        },
      ],
    ],
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
