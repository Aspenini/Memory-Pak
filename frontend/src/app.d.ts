/// <reference types="svelte" />
/// <reference types="vite/client" />
/// <reference types="vite-plugin-pwa/client" />

interface ImportMetaEnv {
  readonly TAURI_ENV_PLATFORM?: string;
}

interface Window {
  __TAURI_INTERNALS__?: unknown;
}
