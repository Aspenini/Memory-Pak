import type {
  CollectibleView,
  CollectionStats,
  ConsoleView,
  GameView,
  InitialState,
  MemoryPakBackend,
  MutationResult,
  PersistedState,
  QueryInput,
  QueryResult,
  SetItemNotesInput,
  SetItemStatusInput
} from './types';
import { loadPersistedState, savePersistedState } from './webStorage';

const SAVE_DEBOUNCE_MS = 250;

interface WasmInstance {
  loadInitialState(): InitialState;
  queryConsoles(input: QueryInput): QueryResult<ConsoleView>;
  queryGames(input: QueryInput): QueryResult<GameView>;
  queryCollectibles(input: QueryInput): QueryResult<CollectibleView>;
  setItemStatus(input: SetItemStatusInput): MutationResult;
  setItemNotes(input: SetItemNotesInput): MutationResult;
  importJson(json: string): CollectionStats;
  exportJson(): string;
  getCollectionStats(): CollectionStats;
  snapshotStateJson(): string;
}

export async function createWebWasmBackend(): Promise<MemoryPakBackend> {
  const module = await import('@wasm/memory_pak_wasm.js');
  await module.default();
  const persisted = await loadPersistedState();
  const app = new module.WasmMemoryPak(JSON.stringify(persisted)) as WasmInstance;

  const queueSave = makeDebouncedSave(app);

  return {
    loadInitialState: () => Promise.resolve(app.loadInitialState()),
    queryConsoles: (input) => Promise.resolve(app.queryConsoles(input)),
    queryGames: (input) => Promise.resolve(app.queryGames(input)),
    queryCollectibles: (input) => Promise.resolve(app.queryCollectibles(input)),
    setItemStatus: (input) => {
      const result = app.setItemStatus(input);
      queueSave();
      return Promise.resolve(result);
    },
    setItemNotes: (input) => {
      const result = app.setItemNotes(input);
      queueSave();
      return Promise.resolve(result);
    },
    importJson: (json) => {
      const stats = app.importJson(json);
      queueSave();
      return Promise.resolve(stats);
    },
    exportJson: () => Promise.resolve(app.exportJson()),
    getCollectionStats: () => Promise.resolve(app.getCollectionStats()),
    importFromFile: async () => {
      const file = await pickJsonFile();
      if (!file) return undefined;
      const stats = app.importJson(await file.text());
      queueSave();
      return stats;
    },
    exportToFile: async () => {
      downloadJson(app.exportJson(), 'memory_pak_export.json');
    }
  };
}

function makeDebouncedSave(app: WasmInstance): () => void {
  let timer: ReturnType<typeof setTimeout> | null = null;

  const flush = (): void => {
    timer = null;
    const snapshot = JSON.parse(app.snapshotStateJson()) as PersistedState;
    void savePersistedState(snapshot);
  };

  if (typeof window !== 'undefined') {
    window.addEventListener('beforeunload', () => {
      if (timer) {
        clearTimeout(timer);
        flush();
      }
    });
  }

  return () => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(flush, SAVE_DEBOUNCE_MS);
  };
}

async function pickJsonFile(): Promise<File | undefined> {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = 'application/json,.json';

  return new Promise((resolve) => {
    input.addEventListener(
      'change',
      () => {
        resolve(input.files?.[0]);
      },
      { once: true }
    );
    input.click();
  });
}

function downloadJson(json: string, filename: string): void {
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = filename;
  anchor.style.display = 'none';
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}
