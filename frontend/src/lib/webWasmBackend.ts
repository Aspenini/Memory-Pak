import type {
  CollectionStats,
  ConsoleView,
  GameView,
  InitialState,
  LegoView,
  MemoryPakBackend,
  PersistedState,
  QueryInput,
  QueryResult,
  SetItemNotesInput,
  SetItemStatusInput,
  SkylanderView
} from './types';
import { loadPersistedState, savePersistedState } from './webStorage';

export async function createWebWasmBackend(): Promise<MemoryPakBackend> {
  const module = await import('../wasm/memory_pak_wasm.js');
  await module.default();
  const app = new module.WasmMemoryPak(JSON.stringify(loadPersistedState()));

  const saveSnapshot = (state: PersistedState): PersistedState => {
    savePersistedState(state);
    return state;
  };

  return {
    loadInitialState: () => Promise.resolve(app.loadInitialState() as InitialState),
    queryConsoles: (input: QueryInput) =>
      Promise.resolve(app.queryConsoles(input) as QueryResult<ConsoleView>),
    queryGames: (input: QueryInput) => Promise.resolve(app.queryGames(input) as QueryResult<GameView>),
    queryLego: (input: QueryInput) => Promise.resolve(app.queryLego(input) as QueryResult<LegoView>),
    querySkylanders: (input: QueryInput) =>
      Promise.resolve(app.querySkylanders(input) as QueryResult<SkylanderView>),
    setItemStatus: (input: SetItemStatusInput) =>
      Promise.resolve(saveSnapshot(app.setItemStatus(input) as PersistedState)),
    setItemNotes: (input: SetItemNotesInput) =>
      Promise.resolve(saveSnapshot(app.setItemNotes(input) as PersistedState)),
    importJson: (json: string) => Promise.resolve(saveSnapshot(app.importJson(json) as PersistedState)),
    exportJson: () => Promise.resolve(app.exportJson() as string),
    getCollectionStats: () => Promise.resolve(app.getCollectionStats() as CollectionStats),
    importFromFile: () => importJsonFile((json) => saveSnapshot(app.importJson(json) as PersistedState)),
    exportToFile: async () => {
      downloadJson(app.exportJson() as string, 'memory_pak_export.json');
    }
  };
}

async function importJsonFile(apply: (json: string) => PersistedState): Promise<PersistedState | undefined> {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = 'application/json,.json';

  const file = await new Promise<File | undefined>((resolve) => {
    input.addEventListener(
      'change',
      () => {
        resolve(input.files?.[0]);
      },
      { once: true }
    );
    input.click();
  });

  if (!file) return undefined;
  return apply(await file.text());
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

