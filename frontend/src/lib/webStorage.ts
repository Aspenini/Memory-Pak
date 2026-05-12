import type { PersistedState } from './types';

const DB_NAME = 'memory-pak';
const DB_VERSION = 1;
const STORE = 'state';
const KEY = 'persisted';

let dbPromise: Promise<IDBDatabase> | null = null;

function openDb(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE)) {
        db.createObjectStore(STORE);
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
  return dbPromise;
}

async function withStore<T>(
  mode: IDBTransactionMode,
  fn: (store: IDBObjectStore) => IDBRequest<T>
): Promise<T> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, mode);
    const store = tx.objectStore(STORE);
    const request = fn(store);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

export async function loadPersistedState(): Promise<PersistedState> {
  try {
    const value = await withStore('readonly', (store) => store.get(KEY) as IDBRequest<unknown>);
    if (!value || typeof value !== 'object') return { entries: {} };
    const candidate = value as Partial<PersistedState>;
    return { entries: candidate.entries ?? {} };
  } catch (error) {
    console.warn('Memory Pak: failed to read persisted state', error);
    return { entries: {} };
  }
}

export async function savePersistedState(state: PersistedState): Promise<void> {
  try {
    await withStore('readwrite', (store) => store.put(state, KEY));
  } catch (error) {
    console.warn('Memory Pak: failed to write persisted state', error);
  }
}
