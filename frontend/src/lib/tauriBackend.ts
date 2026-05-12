import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type {
  CollectibleView,
  CollectionStats,
  ConsoleView,
  GameView,
  InitialState,
  MemoryPakBackend,
  MutationResult,
  QueryInput,
  QueryResult,
  SetItemNotesInput,
  SetItemStatusInput
} from './types';

export function createTauriBackend(): MemoryPakBackend {
  return {
    loadInitialState: () => invoke<InitialState>('load_initial_state'),
    queryConsoles: (input: QueryInput) =>
      invoke<QueryResult<ConsoleView>>('query_consoles', { input }),
    queryGames: (input: QueryInput) => invoke<QueryResult<GameView>>('query_games', { input }),
    queryCollectibles: (input: QueryInput) =>
      invoke<QueryResult<CollectibleView>>('query_collectibles', { input }),
    setItemStatus: (input: SetItemStatusInput) =>
      invoke<MutationResult>('set_item_status', { input }),
    setItemNotes: (input: SetItemNotesInput) => invoke<MutationResult>('set_item_notes', { input }),
    importJson: (json: string) => invoke<CollectionStats>('import_json', { json }),
    exportJson: () => invoke<string>('export_json'),
    getCollectionStats: () => invoke<CollectionStats>('get_collection_stats'),
    importFromFile: async () => {
      const path = await open({
        multiple: false,
        filters: [{ name: 'Memory Pak Export', extensions: ['json'] }]
      });
      if (typeof path !== 'string') return undefined;
      return invoke<CollectionStats>('import_from_path', { path });
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
