import { describe, expect, it, beforeEach, vi } from 'vitest';

beforeEach(async () => {
  // Install a fresh in-memory IndexedDB for each test...
  const { IDBFactory } = await import('fake-indexeddb');
  Object.defineProperty(globalThis, 'indexedDB', {
    value: new IDBFactory(),
    configurable: true,
    writable: true
  });
  // ...and force webStorage to be re-imported so its cached connection
  // points at the new IndexedDB instance.
  vi.resetModules();
});

describe('web storage (IndexedDB)', () => {
  it('returns an empty state when nothing is persisted', async () => {
    const { loadPersistedState } = await import('./webStorage');
    const state = await loadPersistedState();
    expect(state.entries).toEqual({});
  });

  it('round-trips entries through IndexedDB', async () => {
    const { loadPersistedState, savePersistedState } = await import('./webStorage');
    await savePersistedState({
      entries: {
        'console:nes': { owned: true, favorite: false, wishlist: false, notes: '' },
        'game:nes/super-mario-bros': {
          owned: true,
          favorite: true,
          wishlist: false,
          notes: 'cart only'
        }
      }
    });

    const state = await loadPersistedState();
    expect(state.entries['console:nes']?.owned).toBe(true);
    expect(state.entries['game:nes/super-mario-bros']?.notes).toBe('cart only');
  });
});
