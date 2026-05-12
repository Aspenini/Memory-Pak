<script lang="ts">
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import {
    Boxes,
    Check,
    ChevronDown,
    Database,
    Download,
    Gamepad2,
    Heart,
    Menu,
    MoreVertical,
    Monitor,
    Package,
    Search,
    Star,
    Upload,
    X
  } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { onMount, tick } from 'svelte';
  import { createBackend } from './lib/backend';
  import type {
    CollectionStats,
    FilterId,
    InitialState,
    ItemKind,
    MemoryPakBackend,
    QueryInput,
    RowView,
    SetItemStatusInput,
    TabId
  } from './lib/types';
  import { isConsoleView, isGameView, isLegoView } from './lib/types';

  const tabs: Array<{ id: TabId; label: string; mobileLabel: string; icon: typeof Monitor }> = [
    { id: 'consoles', label: 'Consoles', mobileLabel: 'Consoles', icon: Monitor },
    { id: 'games', label: 'Games', mobileLabel: 'Games', icon: Gamepad2 },
    { id: 'lego', label: 'LEGO Dimensions', mobileLabel: 'LEGO', icon: Boxes },
    { id: 'skylanders', label: 'Skylanders', mobileLabel: 'Sky', icon: Package }
  ];

  const filters: Array<{ id: FilterId; label: string; mobileLabel: string }> = [
    { id: 'all', label: 'All', mobileLabel: 'All' },
    { id: 'owned', label: 'Owned', mobileLabel: 'Owned' },
    { id: 'favorites', label: 'Favorites', mobileLabel: 'Fav' },
    { id: 'wishlist', label: 'Wishlist', mobileLabel: 'Wish' },
    { id: 'notOwned', label: 'Not owned', mobileLabel: 'Missing' }
  ];
  const dropdownFade = { duration: 90 };

  let backend: MemoryPakBackend | null = null;
  let initial: InitialState | null = null;
  let stats: CollectionStats | null = null;
  let rows: RowView[] = [];
  let activeTab: TabId = 'consoles';
  let search = '';
  let filterBy: FilterId = 'all';
  let sortBy = 'title';
  let selectedConsole = 'all';
  let loading = true;
  let refreshing = false;
  let error = '';
  let navOpen = false;
  let mobileMenuOpen = false;
  let openSelect: 'console' | 'sort' | null = null;
  let detailRow: RowView | null = null;
  let scrollElement: HTMLDivElement;
  let filterPillsElement: HTMLDivElement;
  let filterIndicator = { left: 0, width: 0 };
  let rowCount = 0;
  let refreshSerial = 0;
  let lastQueryKey = '';
  let isMobile = false;
  let isShort = false;

  // Per-row in-flight notes; flushes to backend on blur.
  let pendingNotes: Record<string, string> = {};

  $: sortOptions = getSortOptions(activeTab);
  $: if (!sortOptions.some((option) => option.id === sortBy)) sortBy = sortOptions[0].id;
  $: rowHeight = estimatedRowHeight(activeTab, isMobile, isShort);
  $: rowVirtualizer = createVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollElement,
    estimateSize: () => rowHeight,
    overscan: 8
  });
  $: queryKey = JSON.stringify({ activeTab, search, filterBy, sortBy, selectedConsole });
  $: if (backend && initial && queryKey) {
    void refreshRows();
  }
  $: if (filterPillsElement && filterBy) {
    isMobile;
    void scheduleFilterIndicatorUpdate();
  }

  onMount(() => {
    const mqMobile = window.matchMedia('(max-width: 960px)');
    const mqShort = window.matchMedia('(max-height: 600px)');
    isMobile = mqMobile.matches;
    isShort = mqShort.matches;
    const onMobile = (event: MediaQueryListEvent) => {
      isMobile = event.matches;
    };
    const onShort = (event: MediaQueryListEvent) => {
      isShort = event.matches;
    };
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
    window.addEventListener('resize', updateFilterIndicator);

    void initBackend();

    return () => {
      mqMobile.removeEventListener('change', onMobile);
      mqShort.removeEventListener('change', onShort);
      document.removeEventListener('click', closeSelects);
      document.removeEventListener('keydown', closeSelectsOnEscape);
      window.removeEventListener('resize', updateFilterIndicator);
    };
  });

  async function scheduleFilterIndicatorUpdate(): Promise<void> {
    await tick();
    updateFilterIndicator();
  }

  function updateFilterIndicator(): void {
    if (!filterPillsElement) return;
    const activeButton = filterPillsElement.querySelector<HTMLElement>(`[data-filter="${filterBy}"]`);
    if (!activeButton) return;
    filterIndicator = {
      left: activeButton.offsetLeft,
      width: activeButton.offsetWidth
    };
  }

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
        consoleId: activeTab === 'games' ? selectedConsole : undefined
      };
      let nextRows: RowView[];

      if (activeTab === 'consoles') {
        nextRows = (await backend.queryConsoles(input)).items;
      } else if (activeTab === 'games') {
        nextRows = (await backend.queryGames(input)).items;
      } else if (activeTab === 'lego') {
        nextRows = (await backend.queryLego(input)).items;
      } else {
        nextRows = (await backend.querySkylanders(input)).items;
      }

      if (serial !== refreshSerial) return;
      rows = nextRows;
      rowCount = nextRows.length;
      pendingNotes = {};
      const queryChanged = queryKey !== lastQueryKey;
      if (queryChanged) {
        lastQueryKey = queryKey;
      }
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

  async function refreshStats(): Promise<void> {
    if (!backend) return;
    stats = await backend.getCollectionStats();
  }

  function setTab(tab: TabId): void {
    activeTab = tab;
    navOpen = false;
    mobileMenuOpen = false;
    openSelect = null;
    detailRow = null;
  }

  function toggleSelect(select: 'console' | 'sort'): void {
    mobileMenuOpen = false;
    openSelect = openSelect === select ? null : select;
  }

  function selectConsole(consoleId: string): void {
    selectedConsole = consoleId;
    openSelect = null;
  }

  function selectSort(sortId: string): void {
    sortBy = sortId;
    openSelect = null;
  }

  function toggleMobileMenu(): void {
    openSelect = null;
    mobileMenuOpen = !mobileMenuOpen;
  }

  function selectedConsoleLabel(): string {
    if (selectedConsole === 'all') return 'All consoles';
    return initial?.consoles.find((console) => console.id === selectedConsole)?.name ?? 'All consoles';
  }

  function sortLabel(sortId: string): string {
    return sortOptions.find((option) => option.id === sortId)?.label ?? sortOptions[0]?.label ?? 'Title';
  }

  async function toggleStatus(row: RowView, field: 'owned' | 'favorite' | 'wishlist'): Promise<void> {
    if (!backend) return;
    const next = !row.state[field];
    // Optimistic mutation so the chip flips instantly.
    row.state[field] = next;
    rows = rows;
    if (detailRow?.id === row.id && detailRow.kind === row.kind) {
      detailRow = row;
    }

    try {
      await backend.setItemStatus({
        kind: row.kind,
        id: row.id,
        [field]: next
      } as SetItemStatusInput);
      await refreshStats();
      // Filter changes a row's visibility; resync the list when filter is restrictive.
      if (filterBy !== 'all') {
        await refreshRows({ preserveScroll: true });
      }
    } catch (cause) {
      // Roll back on failure.
      row.state[field] = !next;
      rows = rows;
      if (detailRow?.id === row.id && detailRow.kind === row.kind) {
        detailRow = row;
      }
      error = cause instanceof Error ? cause.message : String(cause);
    }
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
      await backend.setItemNotes({ kind: row.kind, id: row.id, notes: next });
      row.state.notes = next;
      const { [row.id]: _drop, ...rest } = pendingNotes;
      pendingNotes = rest;
      rows = rows;
      if (detailRow?.id === row.id && detailRow.kind === row.kind) {
        detailRow = row;
      }
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function importCollection(): Promise<void> {
    mobileMenuOpen = false;
    if (!backend?.importFromFile) return;
    const imported = await backend.importFromFile();
    if (!imported) return;
    await refreshStats();
    await refreshRows();
  }

  async function exportCollection(): Promise<void> {
    mobileMenuOpen = false;
    if (backend?.exportToFile) {
      await backend.exportToFile();
      return;
    }
    if (backend) {
      await backend.exportJson();
    }
  }

  async function backupCollection(): Promise<void> {
    await exportCollection();
  }

  async function restoreCollection(): Promise<void> {
    await importCollection();
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

  function rowTitle(row: RowView): string {
    return isGameView(row) ? row.title : row.name;
  }

  function rowSubtitle(row: RowView): string {
    if (isConsoleView(row)) return `${row.manufacturer} / ${row.year}`;
    if (isGameView(row)) return `${row.consoleName} / ${row.publisher} / ${row.year || 'Unknown year'}`;
    if (isLegoView(row)) return `${row.category} / ${row.packId} / ${row.year}`;
    return `${row.game} / ${row.category} / ${row.baseColor}`;
  }

  function rowMobileSubtitle(row: RowView): string {
    if (isConsoleView(row)) return `${row.manufacturer} · ${row.year}`;
    if (isGameView(row)) return `${row.consoleName} · ${row.year || 'Unknown year'}`;
    if (isLegoView(row)) return `${row.category} · ${row.year}`;
    return `${row.game} · ${row.category}`;
  }

  function rowMeta(row: RowView): string | null {
    if (isConsoleView(row)) {
      return `${row.gameCounts.owned} owned / ${row.gameCounts.favorite} favorite / ${row.gameCounts.wishlist} wishlist`;
    }
    return null;
  }

  function rowMobileMeta(row: RowView): string | null {
    if (isConsoleView(row)) {
      return `${row.gameCounts.owned} owned · ${row.gameCounts.favorite} fav · ${row.gameCounts.wishlist} wish`;
    }
    const states = [
      row.state.owned ? 'owned' : 'not owned',
      row.state.favorite ? 'fav' : 'not fav',
      row.state.wishlist ? 'wish' : 'not wish'
    ];
    return states.join(' · ');
  }

  function notesValue(row: RowView): string {
    return pendingNotes[row.id] ?? row.state.notes;
  }

  function notePreview(row: RowView): string {
    return notesValue(row).trim();
  }

  function detailStats(row: RowView): Array<{ label: string; value: string | number }> {
    if (isConsoleView(row)) {
      return [
        { label: 'Owned', value: row.gameCounts.owned },
        { label: 'Favorite', value: row.gameCounts.favorite },
        { label: 'Wishlist', value: row.gameCounts.wishlist }
      ];
    }
    return [
      { label: 'Owned', value: row.state.owned ? 'Yes' : 'No' },
      { label: 'Favorite', value: row.state.favorite ? 'Yes' : 'No' },
      { label: 'Wishlist', value: row.state.wishlist ? 'Yes' : 'No' }
    ];
  }

  function getSortOptions(tab: TabId): Array<{ id: string; label: string }> {
    if (tab === 'lego') {
      return [
        { id: 'name', label: 'Name' },
        { id: 'category', label: 'Category' },
        { id: 'year', label: 'Year' },
        { id: 'pack', label: 'Pack' },
        { id: 'status', label: 'Status' }
      ];
    }
    if (tab === 'skylanders') {
      return [
        { id: 'name', label: 'Name' },
        { id: 'game', label: 'Game' },
        { id: 'baseColor', label: 'Base color' },
        { id: 'category', label: 'Category' },
        { id: 'status', label: 'Status' }
      ];
    }
    return [
      { id: 'title', label: 'Title' },
      { id: 'year', label: 'Year' },
      { id: 'status', label: 'Status' }
    ];
  }

  function tabCount(tab: TabId): number {
    if (!initial) return 0;
    if (tab === 'consoles') return initial.consoles.length;
    if (tab === 'games') return initial.totalGames;
    if (tab === 'lego') return initial.totalLegoDimensions;
    return initial.totalSkylanders;
  }

  function estimatedRowHeight(tab: TabId, mobile: boolean, short: boolean): number {
    if (mobile && short) {
      return 100;
    }
    if (mobile) {
      return 118;
    }
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
    if (tab === 'lego') {
      return {
        owned: currentStats.ownedLegoDimensions,
        favorite: 0,
        wishlist: 0,
        total: currentStats.totalLegoDimensions
      };
    }
    return {
      owned: currentStats.ownedSkylanders,
      favorite: 0,
      wishlist: 0,
      total: currentStats.totalSkylanders
    };
  }

  function activeKindLabel(kind: ItemKind): string {
    if (kind === 'console') return 'Consoles';
    if (kind === 'game') return 'Games';
    if (kind === 'lego') return 'Figures';
    return 'Skylanders';
  }

  $: summary = summaryForActiveTab(activeTab, stats);
  $: activeTotal = !initial
    ? 0
    : activeTab === 'consoles'
      ? initial.consoles.length
      : activeTab === 'games'
        ? initial.totalGames
        : activeTab === 'lego'
          ? initial.totalLegoDimensions
          : initial.totalSkylanders;
  $: ownershipPercent = activeTotal ? Math.min(100, Math.round((summary.owned / activeTotal) * 100)) : 0;
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
    <aside class:open={navOpen} class="sidebar">
      <div class="brand">
        <img src="./icons/icon-192.png" alt="" />
        <div>
          <strong>Memory Pak</strong>
        </div>
      </div>

      <nav aria-label="Collection sections">
        {#each tabs as tab}
          <button class:active={activeTab === tab.id} on:click={() => setTab(tab.id)}>
            <svelte:component this={tab.icon} size={20} />
            <span>{tab.label}</span>
            <small>{tabCount(tab.id).toLocaleString()}</small>
          </button>
        {/each}
      </nav>

      <div class="sidebar-footer">
        <small>v0.3</small>
      </div>
    </aside>

    {#if navOpen}
      <button aria-label="Close navigation" class="scrim" on:click={() => (navOpen = false)}></button>
    {/if}

    <main class="workspace">
      <header class="topbar">
        <button class="icon-button mobile-only" aria-label="Open navigation" on:click={() => (navOpen = true)}>
          <Menu size={22} />
        </button>
        <div class="topbar-title">
          <h1>{tabs.find((tab) => tab.id === activeTab)?.label}</h1>
          <p>
            {rows.length.toLocaleString()} shown
            <span class="mobile-inline-stats">
              · {summary.owned.toLocaleString()} owned
              {#if summary.favorite > 0 || activeTab === 'consoles' || activeTab === 'games'}
                · {summary.favorite.toLocaleString()} favorite
              {/if}
            </span>
            {#if refreshing}<span class="dim"> // syncing</span>{/if}
          </p>
          <div
            class="ownership-meter"
            style={`--meter: ${ownershipPercent}%`}
            aria-label={`${ownershipPercent}% owned`}
          >
            <span></span>
          </div>
        </div>
        <div class="topbar-summary" aria-label="Collection summary">
          <div>
            <Check size={14} />
            <strong>{summary.owned.toLocaleString()}</strong>
            <span>owned</span>
          </div>
          {#if summary.favorite > 0 || activeTab === 'consoles' || activeTab === 'games'}
            <div>
              <Star size={14} />
              <strong>{summary.favorite.toLocaleString()}</strong>
              <span>favorite</span>
            </div>
          {/if}
          {#if summary.wishlist > 0 || activeTab === 'consoles' || activeTab === 'games'}
            <div>
              <Heart size={14} />
              <strong>{summary.wishlist.toLocaleString()}</strong>
              <span>wishlist</span>
            </div>
          {/if}
        </div>
        <div class="top-actions">
          <button class="ghost-button" on:click={importCollection}>
            <Upload size={18} />
            <span>Import</span>
          </button>
          <button class="ghost-button" on:click={exportCollection}>
            <Download size={18} />
            <span>Export</span>
          </button>
        </div>
        <div class="mobile-menu-wrap">
          <button
            class="icon-button mobile-more"
            type="button"
            aria-label="Open collection actions"
            aria-expanded={mobileMenuOpen}
            on:click={toggleMobileMenu}
          >
            <MoreVertical size={20} />
          </button>

          {#if mobileMenuOpen}
            <div class="mobile-action-menu" role="menu" transition:fade={dropdownFade}>
              <button type="button" role="menuitem" on:click={importCollection}>Import</button>
              <button type="button" role="menuitem" on:click={exportCollection}>Export</button>
              <button type="button" role="menuitem" on:click={backupCollection}>Backup</button>
              <button type="button" role="menuitem" on:click={restoreCollection}>Restore</button>
            </div>
          {/if}
        </div>
      </header>

      <section class="toolbar" aria-label="Filters">
        <label class="search-box">
          <Search size={18} />
          <input bind:value={search} placeholder="Search collection" />
          {#if search}
            <button
              class="search-clear"
              type="button"
              aria-label="Clear search"
              on:click={() => (search = '')}
            >
              <X size={14} />
            </button>
          {/if}
        </label>

        <div
          bind:this={filterPillsElement}
          class="filter-pills"
          class:ready={filterIndicator.width > 0}
          style={`--filter-left: ${filterIndicator.left}px; --filter-width: ${filterIndicator.width}px`}
          role="tablist"
          aria-label="Filter by status"
        >
          {#each filters as filter}
            <button
              role="tab"
              data-filter={filter.id}
              aria-selected={filterBy === filter.id}
              class:active={filterBy === filter.id}
              on:click={() => (filterBy = filter.id)}
            >
              {isMobile ? filter.mobileLabel : filter.label}
            </button>
          {/each}
        </div>

        <div class="toolbar-selects">
          {#if activeTab === 'games'}
            <div class="select-wrap">
              <button
                class="select-trigger"
                type="button"
                aria-haspopup="listbox"
                aria-expanded={openSelect === 'console'}
                on:click={() => toggleSelect('console')}
              >
                <span class="select-label">Console</span>
                <strong>{selectedConsoleLabel()}</strong>
                <ChevronDown size={16} />
              </button>

              {#if openSelect === 'console' && !isMobile}
                <div
                  class="select-popover"
                  role="listbox"
                  aria-label="Console"
                  transition:fade={dropdownFade}
                >
                  <button
                    type="button"
                    role="option"
                    aria-selected={selectedConsole === 'all'}
                    class:selected={selectedConsole === 'all'}
                    on:click={() => selectConsole('all')}
                  >
                    All consoles
                  </button>
                  {#each initial?.consoles ?? [] as console}
                    <button
                      type="button"
                      role="option"
                      aria-selected={selectedConsole === console.id}
                      class:selected={selectedConsole === console.id}
                      on:click={() => selectConsole(console.id)}
                    >
                      {console.name}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <div class="select-wrap">
            <button
              class="select-trigger"
              type="button"
              aria-haspopup="listbox"
              aria-expanded={openSelect === 'sort'}
              on:click={() => toggleSelect('sort')}
            >
              <span class="select-label">Sort</span>
              <strong>{sortLabel(sortBy)}</strong>
              <ChevronDown size={16} />
            </button>

            {#if openSelect === 'sort' && !isMobile}
              <div
                class="select-popover compact"
                role="listbox"
                aria-label="Sort"
                transition:fade={dropdownFade}
              >
                {#each sortOptions as option}
                  <button
                    type="button"
                    role="option"
                    aria-selected={sortBy === option.id}
                    class:selected={sortBy === option.id}
                    on:click={() => selectSort(option.id)}
                  >
                    {option.label}
                  </button>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      </section>

      <section class="list-region">
        <div class="list-viewport" bind:this={scrollElement}>
          {#if rows.length === 0}
            <div class="empty" transition:fade>
              <Search size={28} />
              <p>No matching {activeKindLabel(activeTab === 'consoles' ? 'console' : activeTab === 'games' ? 'game' : activeTab === 'lego' ? 'lego' : 'skylander').toLowerCase()}</p>
              {#if search || filterBy !== 'all'}
                <button
                  class="ghost-button"
                  on:click={() => {
                    search = '';
                    filterBy = 'all';
                  }}
                >
                  Reset filters
                </button>
              {/if}
            </div>
          {:else}
            <div class="virtual-space" style={`height: ${$rowVirtualizer.getTotalSize()}px`}>
              {#each $rowVirtualizer.getVirtualItems() as virtualRow (rows[virtualRow.index].id)}
                {@const row = rows[virtualRow.index]}
                <div
                  class="row-slot"
                  style={`height: ${virtualRow.size}px; transform: translateY(${virtualRow.start}px)`}
                >
                  {#if isMobile}
                    <button
                      type="button"
                      class="mobile-card"
                      class:owned={row.state.owned}
                      class:favorite={row.state.favorite}
                      class:wishlist={row.state.wishlist}
                      data-kind={row.kind}
                      on:click={() => openDetails(row)}
                    >
                      <span class="mobile-card-top">
                        <strong title={rowTitle(row)}>{rowTitle(row)}</strong>
                        <span class="mobile-status-icons" aria-hidden="true">
                          <span class="mobile-status-icon owned" class:active={row.state.owned}>
                            <Check size={16} />
                          </span>
                          <span class="mobile-status-icon favorite" class:active={row.state.favorite}>
                            <Star size={16} />
                          </span>
                          <span class="mobile-status-icon wishlist" class:active={row.state.wishlist}>
                            <Heart size={16} />
                          </span>
                        </span>
                      </span>
                      <span class="mobile-card-subtitle" title={rowMobileSubtitle(row)}>
                        {rowMobileSubtitle(row)}
                      </span>
                      {#if rowMobileMeta(row)}
                        <span class="mobile-card-meta">{rowMobileMeta(row)}</span>
                      {/if}
                      {#if notePreview(row)}
                        <span class="mobile-card-note">Note: {notePreview(row)}</span>
                      {/if}
                    </button>
                  {:else}
                    <article
                      class="card"
                      class:owned={row.state.owned}
                      class:favorite={row.state.favorite}
                      class:wishlist={row.state.wishlist}
                      data-kind={row.kind}
                    >
                      <header class="card-head">
                        <div class="card-title">
                          <strong title={rowTitle(row)}>{rowTitle(row)}</strong>
                          <span title={rowSubtitle(row)}>{rowSubtitle(row)}</span>
                          {#if rowMeta(row)}
                            <small>{rowMeta(row)}</small>
                          {/if}
                        </div>
                        <div class="card-actions" role="group" aria-label="Status">
                          <button
                            type="button"
                            class:pressed={row.state.owned}
                            on:click={() => toggleStatus(row, 'owned')}
                            aria-pressed={row.state.owned}
                            title="Toggle owned"
                          >
                            <Check size={16} />
                            <span>Owned</span>
                          </button>
                          <button
                            type="button"
                            class:pressed={row.state.favorite}
                            on:click={() => toggleStatus(row, 'favorite')}
                            aria-pressed={row.state.favorite}
                            title="Toggle favorite"
                          >
                            <Star size={16} />
                            <span>Favorite</span>
                          </button>
                          <button
                            type="button"
                            class:pressed={row.state.wishlist}
                            on:click={() => toggleStatus(row, 'wishlist')}
                            aria-pressed={row.state.wishlist}
                            title="Toggle wishlist"
                          >
                            <Heart size={16} />
                            <span>Wishlist</span>
                          </button>
                        </div>
                      </header>
                      <textarea
                        class="card-notes"
                        placeholder="Notes"
                        rows="2"
                        value={notesValue(row)}
                        on:input={(event) => onNotesInput(row.id, event.currentTarget.value)}
                        on:blur={() => flushNotes(row)}
                      ></textarea>
                    </article>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </section>
    </main>

    <nav class="bottom-tabs" aria-label="Collection sections">
      {#each tabs as tab}
        <button aria-label={tab.label} class:active={activeTab === tab.id} on:click={() => setTab(tab.id)}>
          <svelte:component this={tab.icon} size={20} />
          <span>{tab.mobileLabel}</span>
        </button>
      {/each}
    </nav>

    {#if isMobile && openSelect}
      <button class="sheet-scrim" aria-label="Close selector" on:click={() => (openSelect = null)} transition:fade={dropdownFade}></button>
      <dialog
        class="bottom-sheet option-sheet"
        open
        aria-labelledby="option-sheet-title"
      >
        <span class="sheet-handle"></span>
        <header class="sheet-header">
          <h2 id="option-sheet-title">{openSelect === 'sort' ? 'Sort by' : 'Console'}</h2>
          <button class="icon-button" type="button" aria-label="Close selector" on:click={() => (openSelect = null)}>
            <X size={18} />
          </button>
        </header>

        {#if openSelect === 'sort'}
          <div class="sheet-options" role="listbox" aria-label="Sort by">
            {#each sortOptions as option}
              <button
                type="button"
                role="option"
                aria-selected={sortBy === option.id}
                class:selected={sortBy === option.id}
                on:click={() => selectSort(option.id)}
              >
                <span>{sortBy === option.id ? '●' : '○'}</span>
                {option.label}
              </button>
            {/each}
          </div>
        {:else}
          <div class="sheet-options" role="listbox" aria-label="Console">
            <button
              type="button"
              role="option"
              aria-selected={selectedConsole === 'all'}
              class:selected={selectedConsole === 'all'}
              on:click={() => selectConsole('all')}
            >
              <span>{selectedConsole === 'all' ? '●' : '○'}</span>
              All consoles
            </button>
            {#each initial?.consoles ?? [] as console}
              <button
                type="button"
                role="option"
                aria-selected={selectedConsole === console.id}
                class:selected={selectedConsole === console.id}
                on:click={() => selectConsole(console.id)}
              >
                <span>{selectedConsole === console.id ? '●' : '○'}</span>
                {console.name}
              </button>
            {/each}
          </div>
        {/if}
      </dialog>
    {/if}

    {#if detailRow}
      <button class="sheet-scrim" aria-label="Close details" on:click={() => closeDetails()} transition:fade={dropdownFade}></button>
      <dialog class="bottom-sheet detail-sheet" open aria-labelledby="detail-sheet-title">
        <span class="sheet-handle"></span>
        <header class="sheet-header">
          <div>
            <h2 id="detail-sheet-title">{rowTitle(detailRow)}</h2>
            <p>{rowMobileSubtitle(detailRow)}</p>
          </div>
          <button class="icon-button" type="button" aria-label="Close details" on:click={() => closeDetails()}>
            <X size={18} />
          </button>
        </header>

        <div class="sheet-actions" role="group" aria-label="Status">
          <button
            type="button"
            class:pressed={detailRow.state.owned}
            on:click={() => toggleStatus(detailRow!, 'owned')}
            aria-pressed={detailRow.state.owned}
          >
            <Check size={16} />
            Owned
          </button>
          <button
            type="button"
            class:pressed={detailRow.state.favorite}
            on:click={() => toggleStatus(detailRow!, 'favorite')}
            aria-pressed={detailRow.state.favorite}
          >
            <Star size={16} />
            Favorite
          </button>
          <button
            type="button"
            class:pressed={detailRow.state.wishlist}
            on:click={() => toggleStatus(detailRow!, 'wishlist')}
            aria-pressed={detailRow.state.wishlist}
          >
            <Heart size={16} />
            Wishlist
          </button>
        </div>

        <section class="sheet-section">
          <h3>Stats</h3>
          <dl class="detail-stats">
            {#each detailStats(detailRow) as stat}
              <div>
                <dt>{stat.label}</dt>
                <dd>{stat.value}</dd>
              </div>
            {/each}
          </dl>
        </section>

        <section class="sheet-section">
          <h3>Notes</h3>
          <textarea
            class="detail-notes"
            placeholder="No notes"
            rows="4"
            value={notesValue(detailRow)}
            on:input={(event) => onNotesInput(detailRow!.id, event.currentTarget.value)}
          ></textarea>
        </section>

        <button class="sheet-save" type="button" on:click={saveDetails}>Save</button>
      </dialog>
    {/if}
  </div>
{/if}
