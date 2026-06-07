import { isTauri } from '@tauri-apps/api/core';

const MOBILE_TAURI_PLATFORMS = new Set(['android', 'ios']);

export function isTauriDesktop(): boolean {
  if (!isTauri()) {
    return false;
  }

  const platform = import.meta.env.TAURI_ENV_PLATFORM;
  if (!platform) {
    return true;
  }

  return !MOBILE_TAURI_PLATFORMS.has(platform);
}
