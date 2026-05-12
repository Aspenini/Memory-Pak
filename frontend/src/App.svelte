<script lang="ts">
  import { Database, Gamepad2, Monitor, Package, X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { onMount, tick } from 'svelte';
  import { createBackend } from './lib/backend';
  import { debounce } from './lib/debounce';
  import { getSortOptions, sortLabel as resolveSortLabel } from './lib/sortOptions';
  import type {
    CollectionStats,
    EntryState,
    FilterBy,
    InitialState,
    MemoryPakBackend,
    QueryInput,
    RowView,
    SortKey,
    TabId
  } from './lib/types';
  import BottomTabs from './lib/components/BottomTabs.svelte';
  import DetailSheet from './lib/components/DetailSheet.svelte';
  import OptionSheet from './lib/components/OptionSheet.svelte';
  import RowList from './lib/components/RowList.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import TopBar from './lib/components/TopBar.svelte';
  import Toolbar from './lib/components/Toolbar.svelte';

  const tabs = [
    { id: 'consoles' as TabId, label: 'Consoles', mobileLabel: 'Consoles', icon: Monitor },
    { id: 'games' as TabId, label: 'Games', mobileLabel: 'Games', icon: Gamepad2 },
    {
      id: 'collectibles' as TabId,
      label: 'Collectibles',
      mobileLabel: 'Collectibles',
      icon: Package
    }
  ];

  const filters: Array<{ id: FilterBy; label: string; mobileLabel: string }> = [
    { id: 'all', label: 'All', mobileLabel: 'All' },
    { id: 'owned', label: 'Owned', mobileLabel: 'Owned' },
    { id: 'favorites', label: 'Favorites', mobileLabel: 'Fav' },
    { id: 'wishlist', label: 'Wishlist', mobileLabel: 'Wish' },
    { id: 'notOwned', label: 'Not owned', mobileLabel: 'Missing' }
  ];

  let backend: MemoryPakBackend | null = null;
  let initial: InitialState | null = null;
  let stats: CollectionStats | null = null;
  let rows: RowView[] = [];
  let activeTab: TabId = 'consoles';
  let searchInput = '';
  let search = '';
  let filterBy: FilterBy = 'all';
  let sortBy: SortKey = 'name';
  let selectedConsole = 'all';
  let selectedCollection = 'all';
  let loading = true;
  let refreshing = false;
  let error = '';
  let navOpen = false;
  let mobileMenuOpen = false;
  let openSelect: 'group' | 'sort' | null = null;
  let detailRow: RowView | null = null;
  let scrollElement: HTMLDivElement | undefined;
  let rowCount = 0;
  let refreshSerial = 0;
  let lastQueryKey = '';
  let isMobile = false;
  let isShort = false;

  let pendingNotes: Record<string, string> = {};

  const commitSearch = debounce((value: string) => {
    search = value;
  }, 150);

  function onSearchChange(value: string): void {
    searchInput = value;
    if (value === '') {
      commitSearch.cancel();
      search = '';
    } else {
      commitSearch(value);
    }
  }

  $: sortOptions = getSortOptions(activeTab);
  $: if (!sortOptions.some((option) => option.id === sortBy)) sortBy = sortOptions[0].id;
  $: rowHeight = estimatedRowHeight(activeTab, isMobile, isShort);
  $: queryKey = JSON.stringify({
    activeTab,
    search,
    filterBy,
    sortBy,
    selectedConsole,
    selectedCollection
  });
  $: if (backend && initial && queryKey) {
    void refreshRows();
  }

  $: summary = summaryForActiveTab(activeTab, stats);
  $: activeTotal = activeTotalCount(activeTab, initial);
  $: ownershipPercent = activeTotal
    ? Math.min(100, Math.round((summary.owned / activeTotal) * 100))
    : 0;
  $: sortLabelText = resolveSortLabel(sortOptions, sortBy);
  $: tabCounts = computeTabCounts(initial);
  $: activeTitle = tabs.find((tab) => tab.id === activeTab)?.label ?? '';

  $: groupConfig = buildGroupConfig(activeTab, initial, selectedConsole, selectedCollection);

  $: if (
    initial &&
    selectedConsole !== 'all' &&
    !initial.consolesWithGames.some((c) => c.id === selectedConsole)
  ) {
    selectedConsole = 'all';
  }

  onMount(() => {
    const mqMobile = window.matchMedia('(max-width: 960px)');
    const mqShort = window.matchMedia('(max-height: 600px)');
    isMobile = mqMobile.matches;
    isShort = mqShort.matches;
    const onMobile = (event: MediaQueryListEvent) => (isMobile = event.matches);
    const onShort = (event: MediaQueryListEvent) => (isShort = event.matches);
    mqMobile.addEventListener('change', onMobile);
    mqShort.addEventListener('change', onShort);

    const closeSelects = (event: MouseEvent) => {
      if (event.target instanceof HTMLElement && event.target.closest('.select-wrap')) return;
      if (event.target instanceof HTMLElement && event.target.closest('.mobile-menu-wrap')) return;
      openSelect = null;
      mobileMenuOpen = false;
    };
    const closeSelectsOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        openSelect = null;
        mobileMenuOpen = false;
        detailRow = null;
      }
    };
    document.addEventListener('click', closeSelects);
    document.addEventListener('keydown', closeSelectsOnEscape);

    void initBackend();

    return () => {
      mqMobile.removeEventListener('change', onMobile);
      mqShort.removeEventListener('change', onShort);
      document.removeEventListener('click', closeSelects);
      document.removeEventListener('keydown', closeSelectsOnEscape);
      commitSearch.cancel();
    };
  });

  async function initBackend(): Promise<void> {
    try {
      backend = await createBackend();
      initial = await backend.loadInitialState();
      stats = initial.stats;
      loading = false;
      await refreshRows();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
      loading = false;
    }
  }

  async function refreshRows(options: { preserveScroll?: boolean } = {}): Promise<void> {
    if (!backend) return;
    const serial = ++refreshSerial;
    const scrollTop = options.preserveScroll ? scrollElement?.scrollTop : undefined;
    refreshing = true;

    try {
      const input: QueryInput = {
        search,
        filterBy,
        sortBy,
        consoleId: activeTab === 'games' ? selectedConsole : undefined,
        collectionId: activeTab === 'collectibles' ? selectedCollection : undefined
      };
      let nextRows: RowView[];

      if (activeTab === 'consoles') {
        nextRows = (await backend.queryConsoles(input)).items;
      } else if (activeTab === 'games') {
        nextRows = (await backend.queryGames(input)).items;
      } else {
        nextRows = (await backend.queryCollectibles(input)).items;
      }

      if (serial !== refreshSerial) return;
      rows = nextRows;
      rowCount = nextRows.length;
      pendingNotes = {};
      const queryChanged = queryKey !== lastQueryKey;
      if (queryChanged) lastQueryKey = queryKey;
      if (options.preserveScroll && scrollTop !== undefined) {
        await tick();
        scrollElement?.scrollTo({ top: scrollTop });
      } else if (queryChanged) {
        await tick();
        scrollElement?.scrollTo({ top: 0 });
      }
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (serial === refreshSerial) refreshing = false;
    }
  }

  function setTab(tab: TabId): void {
    activeTab = tab;
    navOpen = false;
    mobileMenuOpen = false;
    openSelect = null;
    detailRow = null;
  }

  function toggleSelect(select: 'group' | 'sort'): void {
    mobileMenuOpen = false;
    openSelect = openSelect === select ? null : select;
  }

  function selectGroup(id: string): void {
    if (activeTab === 'games') {
      selectedConsole = id;
    } else if (activeTab === 'collectibles') {
      selectedCollection = id;
    }
    openSelect = null;
  }

  function selectSort(sortId: SortKey): void {
    sortBy = sortId;
    openSelect = null;
  }

  function toggleMobileMenu(): void {
    openSelect = null;
    mobileMenuOpen = !mobileMenuOpen;
  }

  async function toggleStatus(
    row: RowView,
    field: 'owned' | 'favorite' | 'wishlist'
  ): Promise<void> {
    if (!backend) return;
    const next = !row.state[field];
    row.state[field] = next;
    rows = rows;
    if (detailRow?.id === row.id) detailRow = row;

    try {
      const result = await backend.setItemStatus({ id: row.id, [field]: next });
      stats = result.stats;
      row.state = result.state;
      rows = rows;
      if (detailRow?.id === row.id) detailRow = row;
      if (filterBy !== 'all' && !rowMatchesFilter(result.state, filterBy)) {
        await refreshRows({ preserveScroll: true });
      }
    } catch (cause) {
      row.state[field] = !next;
      rows = rows;
      if (detailRow?.id === row.id) detailRow = row;
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function rowMatchesFilter(state: EntryState, filter: FilterBy): boolean {
    if (filter === 'all') return true;
    if (filter === 'owned') return state.owned;
    if (filter === 'favorites') return state.favorite;
    if (filter === 'wishlist') return state.wishlist;
    return !state.owned;
  }

  function onNotesInput(rowId: string, value: string): void {
    pendingNotes = { ...pendingNotes, [rowId]: value };
  }

  async function flushNotes(row: RowView): Promise<void> {
    if (!backend) return;
    const next = pendingNotes[row.id];
    if (next === undefined || next === row.state.notes) {
      const { [row.id]: _drop, ...rest } = pendingNotes;
      pendingNotes = rest;
      return;
    }
    try {
      const result = await backend.setItemNotes({ id: row.id, notes: next });
      row.state = result.state;
      stats = result.stats;
      const { [row.id]: _drop, ...rest } = pendingNotes;
      pendingNotes = rest;
      rows = rows;
      if (detailRow?.id === row.id) detailRow = row;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function restoreCollection(): Promise<void> {
    mobileMenuOpen = false;
    if (!backend?.importFromFile) return;
    const importedStats = await backend.importFromFile();
    if (!importedStats) return;
    stats = importedStats;
    await refreshRows();
  }

  async function backupCollection(): Promise<void> {
    mobileMenuOpen = false;
    if (backend?.exportToFile) {
      await backend.exportToFile();
      return;
    }
    if (backend) await backend.exportJson();
  }

  function openDetails(row: RowView): void {
    if (!isMobile) return;
    mobileMenuOpen = false;
    openSelect = null;
    detailRow = row;
  }

  function closeDetails(discardPending = true): void {
    if (detailRow && discardPending) {
      const { [detailRow.id]: _drop, ...rest } = pendingNotes;
      pendingNotes = rest;
    }
    detailRow = null;
  }

  async function saveDetails(): Promise<void> {
    if (!detailRow) return;
    await flushNotes(detailRow);
    closeDetails(false);
  }

  function resetFilters(): void {
    onSearchChange('');
    filterBy = 'all';
  }

  function notesValueFor(row: RowView): string {
    return pendingNotes[row.id] ?? row.state.notes;
  }

  function estimatedRowHeight(tab: TabId, mobile: boolean, short: boolean): number {
    if (mobile && short) return 100;
    if (mobile) return 118;
    return tab === 'consoles' ? 208 : 188;
  }

  type TabSummary = { owned: number; favorite: number; wishlist: number; total: number };

  function summaryForActiveTab(tab: TabId, currentStats: CollectionStats | null): TabSummary {
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

  function activeTotalCount(tab: TabId, init: InitialState | null): number {
    if (!init) return 0;
    if (tab === 'consoles') return init.consoles.length;
    if (tab === 'games') return init.totalGames;
    return init.totalCollectibles;
  }

  function computeTabCounts(init: InitialState | null): Record<TabId, number> {
    return {
      consoles: init?.consoles.length ?? 0,
      games: init?.totalGames ?? 0,
      collectibles: init?.totalCollectibles ?? 0
    };
  }

  function buildGroupConfig(
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
</script>

{#if loading}
  <main class="boot" transition:fade>
    <Database size={42} />
    <p>Loading Memory Pak</p>
  </main>
{:else if error}
  <main class="boot error" transition:fade>
    <X size={42} />
    <p>{error}</p>
  </main>
{:else}
  <div class="app-shell">
    <Sidebar
      {tabs}
      {activeTab}
      counts={tabCounts}
      version="0.3"
      open={navOpen}
      on:select={(event) => setTab(event.detail)}
    />

    {#if navOpen}
      <button aria-label="Close navigation" class="scrim" on:click={() => (navOpen = false)}></button>
    {/if}

    <main class="workspace">
      <TopBar
        title={activeTitle}
        shownCount={rows.length}
        summary={{ owned: summary.owned, favorite: summary.favorite, wishlist: summary.wishlist }}
        showFavoriteSummary={summary.favorite > 0 ||
          activeTab === 'consoles' ||
          activeTab === 'games'}
        showWishlistSummary={summary.wishlist > 0 ||
          activeTab === 'consoles' ||
          activeTab === 'games'}
        {refreshing}
        {ownershipPercent}
        {mobileMenuOpen}
        on:openNav={() => (navOpen = true)}
        on:backup={backupCollection}
        on:restore={restoreCollection}
        on:toggleMobileMenu={toggleMobileMenu}
      />

      <Toolbar
        searchValue={searchInput}
        {filterBy}
        {filters}
        {sortBy}
        {sortOptions}
        sortLabel={sortLabelText}
        group={groupConfig
          ? {
              label: groupConfig.label,
              items: groupConfig.items,
              selected: groupConfig.selected,
              selectedLabel: groupConfig.selectedLabel
            }
          : null}
        {openSelect}
        {isMobile}
        on:search={(event) => onSearchChange(event.detail)}
        on:setFilter={(event) => (filterBy = event.detail)}
        on:toggleSelect={(event) => toggleSelect(event.detail)}
        on:selectGroup={(event) => selectGroup(event.detail)}
        on:selectSort={(event) => selectSort(event.detail)}
      />

      <RowList
        {rows}
        {rowCount}
        {rowHeight}
        {isMobile}
        {activeTab}
        {filterBy}
        searchValue={searchInput}
        {pendingNotes}
        bind:scrollElement
        on:toggle={(event) => toggleStatus(event.detail.row, event.detail.field)}
        on:notesInput={(event) => onNotesInput(event.detail.rowId, event.detail.value)}
        on:notesBlur={(event) => flushNotes(event.detail)}
        on:open={(event) => openDetails(event.detail)}
        on:resetFilters={resetFilters}
      />
    </main>

    <BottomTabs {tabs} {activeTab} on:select={(event) => setTab(event.detail)} />

    {#if isMobile && openSelect && (openSelect === 'sort' || groupConfig)}
      <OptionSheet
        mode={openSelect}
        title={openSelect === 'sort' ? 'Sort by' : (groupConfig?.label ?? '')}
        {sortOptions}
        {sortBy}
        groupItems={groupConfig?.items ?? []}
        groupSelected={groupConfig?.selected ?? 'all'}
        groupAllLabel={groupConfig?.allLabel ?? 'All'}
        on:close={() => (openSelect = null)}
        on:selectSort={(event) => selectSort(event.detail)}
        on:selectGroup={(event) => selectGroup(event.detail)}
      />
    {/if}

    {#if detailRow}
      <DetailSheet
        row={detailRow}
        notesValue={notesValueFor(detailRow)}
        on:close={() => closeDetails()}
        on:save={saveDetails}
        on:toggle={(event) => toggleStatus(event.detail.row, event.detail.field)}
        on:notesInput={(event) => onNotesInput(event.detail.rowId, event.detail.value)}
      />
    {/if}
  </div>
{/if}
