// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

import sitemap from "@astrojs/sitemap";

// https://astro.build/config
export default defineConfig({
  site: "https://sqlitex.pages.dev",
  integrations: [
    starlight({
      title: "Sqlitex",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/Nareshix/sqlitex",
        },
      ],
      sidebar: [
        {
          label: "Guides",
          items: [
            // Each item here is one entry in the navigation menu.
            { label: "Example Guide", slug: "guides/example" },
          ],
        },
        {
          label: "Reference",
          items: [{ autogenerate: { directory: "reference" } }],
        },
      ],
    }),
    sitemap(),
  ],
});