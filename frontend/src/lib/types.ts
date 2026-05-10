export type ItemKind = 'console' | 'game' | 'lego' | 'skylander';
export type TabId = 'consoles' | 'games' | 'lego' | 'skylanders';
export type FilterId = 'all' | 'owned' | 'favorites' | 'wishlist' | 'notOwned';

export interface StatusState {
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
  id: string;
  name: string;
  manufacturer: string;
  year: number;
  variant?: string | null;
  state: StatusState;
  gameCounts: ConsoleCounts;
}

export interface GameView {
  id: string;
  title: string;
  year: number;
  publisher: string;
  consoleId: string;
  consoleName: string;
  state: StatusState;
}

export interface LegoView {
  id: string;
  name: string;
  category: string;
  year: number;
  packId: string;
  state: StatusState;
}

export interface SkylanderView {
  id: string;
  name: string;
  game: string;
  baseColor: string;
  category: string;
  state: StatusState;
}

export type RowView = ConsoleView | GameView | LegoView | SkylanderView;

export interface CollectionStats {
  totalConsoles: number;
  ownedConsoles: number;
  favoriteConsoles: number;
  wishlistConsoles: number;
  totalGames: number;
  ownedGames: number;
  favoriteGames: number;
  wishlistGames: number;
  totalLegoDimensions: number;
  ownedLegoDimensions: number;
  totalSkylanders: number;
  ownedSkylanders: number;
}

export interface InitialState {
  stats: CollectionStats;
  consoles: ConsoleView[];
  totalGames: number;
  totalLegoDimensions: number;
  totalSkylanders: number;
}

export interface QueryInput {
  search?: string;
  sortBy?: string;
  filterBy?: FilterId;
  consoleId?: string;
  offset?: number;
  limit?: number;
}

export interface QueryResult<T> {
  total: number;
  items: T[];
}

export interface SetItemStatusInput {
  kind: ItemKind;
  id: string;
  owned?: boolean;
  favorite?: boolean;
  wishlist?: boolean;
}

export interface SetItemNotesInput {
  kind: ItemKind;
  id: string;
  notes: string;
}

export interface ConsoleState {
  console_id: string;
  owned: boolean;
  favorite: boolean;
  wishlist: boolean;
  notes: string;
}

export interface GameState {
  game_id: string;
  owned: boolean;
  favorite: boolean;
  wishlist: boolean;
  notes: string;
}

export interface LegoDimensionState {
  figure_id: string;
  owned: boolean;
  favorite: boolean;
  wishlist: boolean;
  notes: string;
}

export interface SkylanderState {
  skylander_id: string;
  owned: boolean;
  favorite: boolean;
  wishlist: boolean;
  notes: string;
}

export interface PersistedState {
  console_states: Record<string, ConsoleState>;
  game_states: Record<string, GameState>;
  lego_dimensions_states: Record<string, LegoDimensionState>;
  skylanders_states: Record<string, SkylanderState>;
}

export interface MemoryPakBackend {
  loadInitialState(): Promise<InitialState>;
  queryConsoles(input: QueryInput): Promise<QueryResult<ConsoleView>>;
  queryGames(input: QueryInput): Promise<QueryResult<GameView>>;
  queryLego(input: QueryInput): Promise<QueryResult<LegoView>>;
  querySkylanders(input: QueryInput): Promise<QueryResult<SkylanderView>>;
  setItemStatus(input: SetItemStatusInput): Promise<PersistedState>;
  setItemNotes(input: SetItemNotesInput): Promise<PersistedState>;
  importJson(json: string): Promise<PersistedState>;
  exportJson(): Promise<string>;
  getCollectionStats(): Promise<CollectionStats>;
  importFromFile?(): Promise<PersistedState | undefined>;
  exportToFile?(): Promise<void>;
}

export function isConsoleView(row: RowView): row is ConsoleView {
  return 'manufacturer' in row;
}

export function isGameView(row: RowView): row is GameView {
  return 'publisher' in row;
}

export function isLegoView(row: RowView): row is LegoView {
  return 'packId' in row;
}

export function isSkylanderView(row: RowView): row is SkylanderView {
  return 'baseColor' in row;
}

