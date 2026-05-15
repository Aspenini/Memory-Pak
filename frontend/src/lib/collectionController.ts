import { getSortOptions } from './sortOptions';
import type {
  CollectionStats,
  EntryState,
  FilterBy,
  InitialState,
  QueryInput,
  RowView,
  SortKey,
  TabId
} from './types';

export const filters: Array<{ id: FilterBy; label: string; mobileLabel: string }> = [
  { id: 'all', label: 'All', mobileLabel: 'All' },
  { id: 'owned', label: 'Owned', mobileLabel: 'Owned' },
  { id: 'favorites', label: 'Favorites', mobileLabel: 'Fav' },
  { id: 'wishlist', label: 'Wishlist', mobileLabel: 'Wish' },
  { id: 'notOwned', label: 'Not owned', mobileLabel: 'Missing' }
];

export type TabSummary = { owned: number; favorite: number; wishlist: number; total: number };

export function buildQueryKey(input: {
  activeTab: TabId;
  search: string;
  filterBy: FilterBy;
  sortBy: SortKey;
  selectedConsole: string;
  selectedCollection: string;
}): string {
  return JSON.stringify(input);
}

export function buildQueryInput(
  activeTab: TabId,
  search: string,
  filterBy: FilterBy,
  sortBy: SortKey,
  selectedConsole: string,
  selectedCollection: string
): QueryInput {
  return {
    search,
    filterBy,
    sortBy,
    consoleId: activeTab === 'games' ? selectedConsole : undefined,
    collectionId: activeTab === 'collectibles' ? selectedCollection : undefined
  };
}

export function normalizeSortForTab(activeTab: TabId, sortBy: SortKey): SortKey {
  const options = getSortOptions(activeTab);
  return options.some((option) => option.id === sortBy) ? sortBy : options[0].id;
}

export function rowMatchesFilter(state: EntryState, filter: FilterBy): boolean {
  if (filter === 'all') return true;
  if (filter === 'owned') return state.owned;
  if (filter === 'favorites') return state.favorite;
  if (filter === 'wishlist') return state.wishlist;
  return !state.owned;
}

export function estimatedRowHeight(tab: TabId, mobile: boolean, short: boolean): number {
  if (mobile && short) return 100;
  if (mobile) return 118;
  return tab === 'consoles' ? 208 : 188;
}

export function summaryForActiveTab(
  tab: TabId,
  currentStats: CollectionStats | null
): TabSummary {
  if (!currentStats) return { owned: 0, favorite: 0, wishlist: 0, total: 0 };
  if (tab === 'consoles') {
    return {
      owned: currentStats.ownedConsoles,
      favorite: currentStats.favoriteConsoles,
      wishlist: currentStats.wishlistConsoles,
      total: currentStats.totalConsoles
    };
  }
  if (tab === 'games') {
    return {
      owned: currentStats.ownedGames,
      favorite: currentStats.favoriteGames,
      wishlist: currentStats.wishlistGames,
      total: currentStats.totalGames
    };
  }
  return {
    owned: currentStats.ownedCollectibles,
    favorite: currentStats.favoriteCollectibles,
    wishlist: currentStats.wishlistCollectibles,
    total: currentStats.totalCollectibles
  };
}

export function activeTotalCount(tab: TabId, init: InitialState | null): number {
  if (!init) return 0;
  if (tab === 'consoles') return init.consoles.length;
  if (tab === 'games') return init.totalGames;
  return init.totalCollectibles;
}

export function computeTabCounts(init: InitialState | null): Record<TabId, number> {
  return {
    consoles: init?.consoles.length ?? 0,
    games: init?.totalGames ?? 0,
    collectibles: init?.totalCollectibles ?? 0
  };
}

export function buildGroupConfig(
  tab: TabId,
  init: InitialState | null,
  consoleId: string,
  collectionId: string
): {
  label: string;
  allLabel: string;
  items: Array<{ id: string; label: string }>;
  selected: string;
  selectedLabel: string;
} | null {
  if (!init) return null;
  if (tab === 'games') {
    const items = init.consolesWithGames.map((c) => ({ id: c.id, label: c.name }));
    const selectedLabel =
      consoleId === 'all'
        ? 'All consoles'
        : (items.find((c) => c.id === consoleId)?.label ?? 'All consoles');
    return {
      label: 'Console',
      allLabel: 'All consoles',
      items,
      selected: consoleId,
      selectedLabel
    };
  }
  if (tab === 'collectibles') {
    const items = init.collections.map((c) => ({ id: c.id, label: c.name }));
    const selectedLabel =
      collectionId === 'all'
        ? 'All collections'
        : (items.find((c) => c.id === collectionId)?.label ?? 'All collections');
    return {
      label: 'Collection',
      allLabel: 'All collections',
      items,
      selected: collectionId,
      selectedLabel
    };
  }
  return null;
}

export function notesValueFor(
  row: RowView,
  pendingNotes: Record<string, string>
): string {
  return pendingNotes[row.id] ?? row.state.notes;
}
