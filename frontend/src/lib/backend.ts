import type { MemoryPakBackend } from './types';
import { createTauriBackend } from './tauriBackend';
import { createWebWasmBackend } from './webWasmBackend';

export async function createBackend(): Promise<MemoryPakBackend> {
  if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
    return createTauriBackend();
  }

  return createWebWasmBackend();
}
