import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
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

export function createTauriBackend(): MemoryPakBackend {
  return {
    loadInitialState: () => invoke<InitialState>('load_initial_state'),
    queryConsoles: (input: QueryInput) => invoke<QueryResult<ConsoleView>>('query_consoles', { input }),
    queryGames: (input: QueryInput) => invoke<QueryResult<GameView>>('query_games', { input }),
    queryLego: (input: QueryInput) => invoke<QueryResult<LegoView>>('query_lego', { input }),
    querySkylanders: (input: QueryInput) =>
      invoke<QueryResult<SkylanderView>>('query_skylanders', { input }),
    setItemStatus: (input: SetItemStatusInput) =>
      invoke<PersistedState>('set_item_status', { input }),
    setItemNotes: (input: SetItemNotesInput) => invoke<PersistedState>('set_item_notes', { input }),
    importJson: (json: string) => invoke<PersistedState>('import_json', { json }),
    exportJson: () => invoke<string>('export_json'),
    getCollectionStats: () => invoke<CollectionStats>('get_collection_stats'),
    importFromFile: async () => {
      const path = await open({
        multiple: false,
        filters: [{ name: 'Memory Pak Export', extensions: ['json'] }]
      });
      if (typeof path !== 'string') return undefined;
      return invoke<PersistedState>('import_from_path', { path });
    },
    exportToFile: async () => {
      const path = await save({
        defaultPath: 'memory_pak_export.json',
        filters: [{ name: 'Memory Pak Export', extensions: ['json'] }]
      });
      if (!path) return;
      await invoke('export_to_path', { path });
    }
  };
}

