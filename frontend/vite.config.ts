import { svelte } from '@sveltejs/vite-plugin-svelte';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { defineConfig, type Plugin } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

const host = process.env.TAURI_DEV_HOST;
const isTauriBuild = Boolean(process.env.TAURI_ENV_PLATFORM);

const GENERATED_WASM_DIR = resolve(__dirname, 'generated/wasm');
const ICONS_DIR = resolve(__dirname, '../icons/web');
const ICON_URL_PREFIXES = ['/icons', '/app/icons', '/Memory-Pak/app/icons'];

const ICON_MIME: Record<string, string> = {
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.svg': 'image/svg+xml'
};

function sharedIconsPlugin(): Plugin {
  return {
    name: 'memory-pak-shared-icons',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (!req.url) return next();
        const url = req.url.split('?')[0];
        if (!url || url.includes('..')) return next();
        const prefix = ICON_URL_PREFIXES.find((candidate) => url === candidate || url.startsWith(`${candidate}/`));
        if (!prefix) return next();
        const filename = url.slice(prefix.length).replace(/^\/+/, '');
        if (!filename) return next();
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

const pwa = VitePWA({
  // Tauri ships its own webview shell -- never inject a SW into the desktop build.
  disable: isTauriBuild,
  registerType: 'prompt',
  injectRegister: null,
  includeAssets: ['icons/favicon.ico', 'icons/apple-touch-icon.png'],
  manifest: {
    name: 'Memory Pak',
    short_name: 'Memory Pak',
    description: 'A fast offline-first tracker for retro game collections.',
    start_url: '.',
    scope: '.',
    display: 'standalone',
    orientation: 'any',
    background_color: '#121318',
    theme_color: '#121318',
    categories: ['games', 'utilities', 'productivity'],
    icons: [
      { src: './icons/icon-192.png', type: 'image/png', sizes: '192x192', purpose: 'any' },
      { src: './icons/icon-512.png', type: 'image/png', sizes: '512x512', purpose: 'any' },
      {
        src: './icons/icon-192-maskable.png',
        type: 'image/png',
        sizes: '192x192',
        purpose: 'maskable'
      },
      {
        src: './icons/icon-512-maskable.png',
        type: 'image/png',
        sizes: '512x512',
        purpose: 'maskable'
      }
    ]
  },
  workbox: {
    // Catalog payload + WASM puts us well above the default 2 MiB limit.
    maximumFileSizeToCacheInBytes: 10 * 1024 * 1024,
    globPatterns: ['**/*.{js,css,html,wasm,png,svg,ico,webp,woff2}'],
    navigateFallback: 'index.html',
    runtimeCaching: [
      {
        urlPattern: ({ request }) => request.mode === 'navigate',
        handler: 'NetworkFirst',
        options: {
          cacheName: 'memory-pak-html',
          networkTimeoutSeconds: 3,
          expiration: { maxEntries: 8 }
        }
      },
      {
        urlPattern: ({ request }) => ['style', 'script', 'worker'].includes(request.destination),
        handler: 'StaleWhileRevalidate',
        options: { cacheName: 'memory-pak-static' }
      }
    ]
  },
  devOptions: {
    enabled: false
  }
});

export default defineConfig({
  plugins: [svelte(), sharedIconsPlugin(), pwa],
  clearScreen: false,
  base: './',
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  resolve: {
    alias: {
      '@wasm': GENERATED_WASM_DIR
    }
  },
  // lucide-svelte ships .svelte sources; esbuild (optimizeDeps) has no Svelte loader.
  optimizeDeps: {
    exclude: ['lucide-svelte']
  },
  server: {
    port: 5173,
    strictPort: true,
    // Allow non-localhost Host headers (Docker, odd proxies). LAN IPv4 is already allowed by Vite’s host check.
    allowedHosts: true,
    // Plain `vite`: localhost only. `vite --host 0.0.0.0` (dev:tauri) listens on all interfaces for phones/emulators.
    host: host || false,
    // Tauri Android/iOS sets TAURI_DEV_HOST so the device can reach your PC. HMR must use the **same port**
    // as the HTTP server: a separate port (e.g. 1421) is blocked by CSP (different origin than :5173) and
    // often has no listener unless port-forwarding is set up — which can leave the WebView stuck on a white screen.
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 5173,
          clientPort: 5173
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
