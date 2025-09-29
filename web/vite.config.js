import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    allowedHosts: ['grn.mauderer.work'],
    host: true,
    strictPort: false,
  },
});
