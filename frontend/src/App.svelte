<script lang="ts">
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import {
    Boxes,
    Check,
    Database,
    Download,
    Gamepad2,
    Heart,
    Menu,
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

  const tabs: Array<{ id: TabId; label: string; icon: typeof Monitor }> = [
    { id: 'consoles', label: 'Consoles', icon: Monitor },
    { id: 'games', label: 'Games', icon: Gamepad2 },
    { id: 'lego', label: 'LEGO Dimensions', icon: Boxes },
    { id: 'skylanders', label: 'Skylanders', icon: Package }
  ];

  const filters: Array<{ id: FilterId; label: string }> = [
    { id: 'all', label: 'All' },
    { id: 'owned', label: 'Owned' },
    { id: 'favorites', label: 'Favorites' },
    { id: 'wishlist', label: 'Wishlist' },
    { id: 'notOwned', label: 'Not owned' }
  ];

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
  let scrollElement: HTMLDivElement;
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
    count: rows.length,
    getScrollElement: () => scrollElement,
    estimateSize: () => rowHeight,
    overscan: 8
  });
  $: queryKey = JSON.stringify({ activeTab, search, filterBy, sortBy, selectedConsole });
  $: if (backend && initial && queryKey) {
    void refreshRows();
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

    void initBackend();

    return () => {
      mqMobile.removeEventListener('change', onMobile);
      mqShort.removeEventListener('change', onShort);
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

  async function refreshRows(): Promise<void> {
    if (!backend) return;
    const serial = ++refreshSerial;
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
      pendingNotes = {};
      if (queryKey !== lastQueryKey) {
        lastQueryKey = queryKey;
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
  }

  async function toggleStatus(row: RowView, field: 'owned' | 'favorite' | 'wishlist'): Promise<void> {
    if (!backend) return;
    const next = !row.state[field];
    // Optimistic mutation so the chip flips instantly.
    row.state[field] = next;
    rows = rows;

    try {
      await backend.setItemStatus({
        kind: row.kind,
        id: row.id,
        [field]: next
      } as SetItemStatusInput);
      await refreshStats();
      // Filter changes a row's visibility; resync the list when filter is restrictive.
      if (filterBy !== 'all') {
        await refreshRows();
      }
    } catch (cause) {
      // Roll back on failure.
      row.state[field] = !next;
      rows = rows;
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
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function importCollection(): Promise<void> {
    if (!backend?.importFromFile) return;
    const imported = await backend.importFromFile();
    if (!imported) return;
    await refreshStats();
    await refreshRows();
  }

  async function exportCollection(): Promise<void> {
    if (backend?.exportToFile) {
      await backend.exportToFile();
      return;
    }
    if (backend) {
      await backend.exportJson();
    }
  }

  function rowTitle(row: RowView): string {
    return isGameView(row) ? row.title : row.name;
  }

  function rowSubtitle(row: RowView): string {
    if (isConsoleView(row)) return `${row.manufacturer} · ${row.year}`;
    if (isGameView(row)) return `${row.consoleName} · ${row.publisher} · ${row.year || 'Unknown year'}`;
    if (isLegoView(row)) return `${row.category} · ${row.packId} · ${row.year}`;
    return `${row.game} · ${row.category} · ${row.baseColor}`;
  }

  function rowMeta(row: RowView): string | null {
    if (isConsoleView(row)) {
      return `${row.gameCounts.owned} owned · ${row.gameCounts.favorite} favorite · ${row.gameCounts.wishlist} wishlist`;
    }
    return null;
  }

  function notesValue(row: RowView): string {
    return pendingNotes[row.id] ?? row.state.notes;
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
    // Cards include a notes textarea + 3 status buttons. Consoles also
    // render an extra meta line (owned/favorite/wishlist counts). Heights
    // include a small inter-card gap controlled by .row-slot padding.
    if (mobile && short) {
      // Landscape phones / short windows: tighter rows, side-by-side head.
      return tab === 'consoles' ? 168 : 148;
    }
    if (mobile) {
      return tab === 'consoles' ? 280 : 250;
    }
    return tab === 'consoles' ? 208 : 188;
  }

  type TabSummary = { owned: number; favorite: number; wishlist: number; total: number };

  function summaryForActiveTab(): TabSummary {
    if (!stats) return { owned: 0, favorite: 0, wishlist: 0, total: 0 };
    if (activeTab === 'consoles') {
      return {
        owned: stats.ownedConsoles,
        favorite: stats.favoriteConsoles,
        wishlist: stats.wishlistConsoles,
        total: stats.totalConsoles
      };
    }
    if (activeTab === 'games') {
      return {
        owned: stats.ownedGames,
        favorite: stats.favoriteGames,
        wishlist: stats.wishlistGames,
        total: stats.totalGames
      };
    }
    if (activeTab === 'lego') {
      return {
        owned: stats.ownedLegoDimensions,
        favorite: 0,
        wishlist: 0,
        total: stats.totalLegoDimensions
      };
    }
    return {
      owned: stats.ownedSkylanders,
      favorite: 0,
      wishlist: 0,
      total: stats.totalSkylanders
    };
  }

  function activeKindLabel(kind: ItemKind): string {
    if (kind === 'console') return 'Consoles';
    if (kind === 'game') return 'Games';
    if (kind === 'lego') return 'Figures';
    return 'Skylanders';
  }

  $: summary = summaryForActiveTab();
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
          <span>{summary.owned.toLocaleString()} / {summary.total.toLocaleString()} owned</span>
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
        <small>Memory Pak · v0.3</small>
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
            {#if refreshing}<span class="dim">· syncing</span>{/if}
          </p>
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

        <div class="filter-pills" role="tablist" aria-label="Filter by status">
          {#each filters as filter}
            <button
              role="tab"
              aria-selected={filterBy === filter.id}
              class:active={filterBy === filter.id}
              on:click={() => (filterBy = filter.id)}
            >
              {filter.label}
            </button>
          {/each}
        </div>

        <div class="toolbar-selects">
          {#if activeTab === 'games'}
            <label class="select-wrap">
              <span>Console</span>
              <select bind:value={selectedConsole} aria-label="Console">
                <option value="all">All consoles</option>
                {#each initial?.consoles ?? [] as console}
                  <option value={console.id}>{console.name}</option>
                {/each}
              </select>
            </label>
          {/if}

          <label class="select-wrap">
            <span>Sort</span>
            <select bind:value={sortBy} aria-label="Sort">
              {#each sortOptions as option}
                <option value={option.id}>{option.label}</option>
              {/each}
            </select>
          </label>
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
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </section>
    </main>

    <nav class="bottom-tabs" aria-label="Collection sections">
      {#each tabs as tab}
        <button class:active={activeTab === tab.id} on:click={() => setTab(tab.id)}>
          <svelte:component this={tab.icon} size={20} />
          <span>{tab.label}</span>
        </button>
      {/each}
    </nav>
  </div>
{/if}
