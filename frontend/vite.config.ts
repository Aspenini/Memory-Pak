import { svelte } from '@sveltejs/vite-plugin-svelte';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { defineConfig, type Plugin } from 'vite';

const host = process.env.TAURI_DEV_HOST;
const isTauri = Boolean(process.env.TAURI_ENV_PLATFORM);

const ICONS_DIR = resolve(__dirname, '../icons/web');

const ICON_MIME: Record<string, string> = {
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.svg': 'image/svg+xml'
};

function sharedIconsPlugin(): Plugin {
  return {
    name: 'memory-pak-shared-icons',
    configureServer(server) {
      server.middlewares.use('/icons', (req, res, next) => {
        if (!req.url) return next();
        const url = req.url.split('?')[0];
        if (!url || url === '/' || url.includes('..')) return next();
        const filename = url.replace(/^\/+/, '');
        const file = join(ICONS_DIR, filename);
        if (!existsSync(file)) return next();
        const mime = ICON_MIME[extname(file).toLowerCase()];
        if (mime) res.setHeader('content-type', mime);
        res.end(readFileSync(file));
      });
    },
    generateBundle() {
      if (!existsSync(ICONS_DIR)) return;
      for (const entry of readdirSync(ICONS_DIR)) {
        if (!ICON_MIME[extname(entry).toLowerCase()]) continue;
        this.emitFile({
          type: 'asset',
          fileName: `icons/${entry}`,
          source: readFileSync(join(ICONS_DIR, entry))
        });
      }
    }
  };
}

export default defineConfig({
  plugins: [svelte(), sharedIconsPlugin()],
  clearScreen: false,
  base: isTauri ? './' : '/Memory-Pak/app/',
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**', '**/target/**']
    }
  },
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG)
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts']
  }
});
