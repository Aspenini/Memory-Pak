export type ItemKind = 'console' | 'game' | 'collectible';
export type TabId = 'consoles' | 'games' | 'collectibles';
export type FilterBy = 'all' | 'owned' | 'favorites' | 'wishlist' | 'notOwned';
export type SortKey =
  | 'title'
  | 'name'
  | 'year'
  | 'status'
  | 'category'
  | 'group'
  | 'collection'
  | 'variant'
  | 'manufacturer';

/** Wire form of an EntryId; structurally `kind:locator`. */
export type EntryId = string;

export interface EntryState {
  owned: boolean;
  favorite: boolean;
  wishlist: boolean;
  notes: string;
}

export interface ConsoleCounts {
  owned: number;
  favorite: number;
  wishlist: number;
}

export interface ConsoleView {
  kind: 'console';
  id: EntryId;
  shortId: string;
  name: string;
  manufacturer: string;
  abbreviation: string;
  generation: number;
  state: EntryState;
  gameCounts: ConsoleCounts;
}

export interface GameView {
  kind: 'game';
  id: EntryId;
  title: string;
  year: number;
  developer: string;
  publisher: string;
  consoleId: EntryId;
  consoleName: string;
  state: EntryState;
}

export interface CollectibleView {
  kind: 'collectible';
  id: EntryId;
  collectionId: string;
  collectionName: string;
  name: string;
  category: string;
  group: string;
  variant: string;
  year: number;
  state: EntryState;
}

export interface CollectionView {
  id: string;
  name: string;
  manufacturer: string;
  kind: string;
  total: number;
  owned: number;
}

export type RowView = ConsoleView | GameView | CollectibleView;

export interface CollectionStats {
  totalConsoles: number;
  ownedConsoles: number;
  favoriteConsoles: number;
  wishlistConsoles: number;
  totalGames: number;
  ownedGames: number;
  favoriteGames: number;
  wishlistGames: number;
  totalCollectibles: number;
  ownedCollectibles: number;
  favoriteCollectibles: number;
  wishlistCollectibles: number;
}

export interface InitialState {
  stats: CollectionStats;
  consoles: ConsoleView[];
  /** Consoles that have at least one game (Games tab console filter only). */
  consolesWithGames: ConsoleView[];
  collections: CollectionView[];
  totalGames: number;
  totalCollectibles: number;
}

export interface QueryInput {
  search?: string;
  sortBy?: SortKey;
  filterBy?: FilterBy;
  consoleId?: EntryId | 'all';
  collectionId?: string | 'all';
  offset?: number;
  limit?: number;
}

export interface QueryResult<T> {
  total: number;
  items: T[];
}

export interface SetItemStatusInput {
  id: EntryId;
  owned?: boolean;
  favorite?: boolean;
  wishlist?: boolean;
}

export interface SetItemNotesInput {
  id: EntryId;
  notes: string;
}

export interface MutationResult {
  id: EntryId;
  state: EntryState;
  stats: CollectionStats;
}

export interface PersistedState {
  entries: Record<EntryId, EntryState>;
}

export interface MemoryPakBackend {
  loadInitialState(): Promise<InitialState>;
  queryConsoles(input: QueryInput): Promise<QueryResult<ConsoleView>>;
  queryGames(input: QueryInput): Promise<QueryResult<GameView>>;
  queryCollectibles(input: QueryInput): Promise<QueryResult<CollectibleView>>;
  setItemStatus(input: SetItemStatusInput): Promise<MutationResult>;
  setItemNotes(input: SetItemNotesInput): Promise<MutationResult>;
  importJson(json: string): Promise<CollectionStats>;
  exportJson(): Promise<string>;
  getCollectionStats(): Promise<CollectionStats>;
  importFromFile?(): Promise<CollectionStats | undefined>;
  exportToFile?(): Promise<void>;
}

export function isConsoleView(row: RowView): row is ConsoleView {
  return row.kind === 'console';
}

export function isGameView(row: RowView): row is GameView {
  return row.kind === 'game';
}

export function isCollectibleView(row: RowView): row is CollectibleView {
  return row.kind === 'collectible';
}
