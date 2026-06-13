import { defineConfig } from 'vite';

// Dev-mode proxy: `npm run dev` serves the UI with hot reload while a
// locally running `straymark-loom` (default port 7700) answers /api.
export default defineConfig({
  build: { outDir: 'dist', emptyOutDir: true },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:7700',
        ws: true,
      },
    },
  },
});
