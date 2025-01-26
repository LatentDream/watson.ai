import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import tailwind from '@astrojs/tailwind';

export default defineConfig({
  integrations: [
    react({
      include: ['**/react/*']
    }),
    tailwind({
      applyBaseStyles: false,
    }),
  ],
  vite: {
    ssr: {
      external: ['svgo', '@iconify/tools'],
      noExternal: ['react', 'react-dom']
    },
  },
});
