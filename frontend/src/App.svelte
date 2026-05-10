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
  import { fade, fly } from 'svelte/transition';
  import { onMount } from 'svelte';
  import { createBackend } from './lib/backend';
  import type {
    CollectionStats,
    ConsoleView,
    FilterId,
    GameView,
    InitialState,
    ItemKind,
    LegoView,
    MemoryPakBackend,
    QueryInput,
    RowView,
    SetItemStatusInput,
    SkylanderView,
    TabId
  } from './lib/types';
  import { isConsoleView, isGameView, isLegoView, isSkylanderView } from './lib/types';

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
  let selected: RowView | null = null;
  let detailNotes = '';
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

  $: sortOptions = getSortOptions(activeTab);
  $: if (!sortOptions.some((option) => option.id === sortBy)) sortBy = sortOptions[0].id;
  $: rowHeight = activeTab === 'games' ? 104 : 96;
  $: rowVirtualizer = createVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollElement,
    estimateSize: () => rowHeight,
    overscan: 10
  });
  $: queryKey = JSON.stringify({ activeTab, search, filterBy, sortBy, selectedConsole });
  $: if (backend && initial && queryKey) {
    void refreshRows();
  }

  onMount(async () => {
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
  });

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
      selected = selected ? rows.find((row) => row.id === selected?.id) ?? null : null;
      detailNotes = selected?.state.notes ?? '';
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
    selected = null;
    navOpen = false;
  }

  function selectRow(row: RowView): void {
    selected = row;
    detailNotes = row.state.notes;
  }

  async function toggleStatus(row: RowView, field: 'owned' | 'favorite' | 'wishlist'): Promise<void> {
    if (!backend) return;
    const input = {
      kind: kindForRow(row),
      id: row.id,
      [field]: !row.state[field]
    } as SetItemStatusInput;

    await backend.setItemStatus(input);
    await refreshStats();
    await refreshRows();
  }

  async function saveNotes(): Promise<void> {
    if (!backend || !selected || detailNotes === selected.state.notes) return;
    await backend.setItemNotes({
      kind: kindForRow(selected),
      id: selected.id,
      notes: detailNotes
    });
    await refreshRows();
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

  function kindForRow(row: RowView): ItemKind {
    if (isConsoleView(row)) return 'console';
    if (isGameView(row)) return 'game';
    if (isLegoView(row)) return 'lego';
    return 'skylander';
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

  function totalOwnedForActiveTab(): string {
    if (!stats) return '0';
    if (activeTab === 'consoles') return `${stats.ownedConsoles}/${stats.totalConsoles}`;
    if (activeTab === 'games') return `${stats.ownedGames}/${stats.totalGames}`;
    if (activeTab === 'lego') return `${stats.ownedLegoDimensions}/${stats.totalLegoDimensions}`;
    return `${stats.ownedSkylanders}/${stats.totalSkylanders}`;
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
    <aside class:open={navOpen} class="sidebar">
      <div class="brand">
        <img src="./icons/icon-192.png" alt="" />
        <div>
          <strong>Memory Pak</strong>
          <span>{totalOwnedForActiveTab()} owned</span>
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
    </aside>

    {#if navOpen}
      <button aria-label="Close navigation" class="scrim" on:click={() => (navOpen = false)}></button>
    {/if}

    <main class="workspace">
      <header class="topbar">
        <button class="icon-button mobile-only" aria-label="Open navigation" on:click={() => (navOpen = true)}>
          <Menu size={22} />
        </button>
        <div>
          <h1>{tabs.find((tab) => tab.id === activeTab)?.label}</h1>
          <p>{rows.length.toLocaleString()} shown {refreshing ? '· syncing' : ''}</p>
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

      <section class="stats-grid" aria-label="Collection stats">
        <div>
          <strong>{stats?.ownedGames.toLocaleString()}</strong>
          <span>Owned games</span>
        </div>
        <div>
          <strong>{stats?.favoriteGames.toLocaleString()}</strong>
          <span>Favorite games</span>
        </div>
        <div>
          <strong>{stats?.wishlistGames.toLocaleString()}</strong>
          <span>Wishlist games</span>
        </div>
      </section>

      <section class="toolbar" aria-label="Filters">
        <label class="search-box">
          <Search size={18} />
          <input bind:value={search} placeholder="Search collection" />
        </label>

        {#if activeTab === 'games'}
          <select bind:value={selectedConsole} aria-label="Console">
            <option value="all">All consoles</option>
            {#each initial?.consoles ?? [] as console}
              <option value={console.id}>{console.name}</option>
            {/each}
          </select>
        {/if}

        <select bind:value={filterBy} aria-label="Filter">
          {#each filters as filter}
            <option value={filter.id}>{filter.label}</option>
          {/each}
        </select>

        <select bind:value={sortBy} aria-label="Sort">
          {#each sortOptions as option}
            <option value={option.id}>{option.label}</option>
          {/each}
        </select>
      </section>

      <section class="content-split">
        <div class="list-viewport" bind:this={scrollElement}>
          {#if rows.length === 0}
            <div class="empty" transition:fade>No matching items</div>
          {:else}
            <div class="virtual-space" style={`height: ${$rowVirtualizer.getTotalSize()}px`}>
              {#each $rowVirtualizer.getVirtualItems() as virtualRow (virtualRow.key)}
                {@const row = rows[virtualRow.index]}
                <button
                  class:active={selected?.id === row.id}
                  class="collection-row"
                  style={`height: ${virtualRow.size}px; transform: translateY(${virtualRow.start}px)`}
                  on:click={() => selectRow(row)}
                >
                  <div class="row-main">
                    <strong>{rowTitle(row)}</strong>
                    <span>{rowSubtitle(row)}</span>
                    {#if isConsoleView(row)}
                      <small>
                        {row.gameCounts.owned} owned · {row.gameCounts.favorite} favorite · {row.gameCounts.wishlist}
                        wishlist
                      </small>
                    {/if}
                  </div>
                  <div class="row-status">
                    {#if row.state.owned}<span class="status-pill owned">Owned</span>{/if}
                    {#if row.state.favorite}<span class="status-pill favorite">Favorite</span>{/if}
                    {#if row.state.wishlist}<span class="status-pill wishlist">Wishlist</span>{/if}
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        {#if selected}
          <aside class="detail-drawer" transition:fly={{ x: 28, duration: 160 }}>
            <button class="icon-button close-detail" aria-label="Close details" on:click={() => (selected = null)}>
              <X size={18} />
            </button>
            <span class="detail-kind">{kindForRow(selected)}</span>
            <h2>{rowTitle(selected)}</h2>
            <p>{rowSubtitle(selected)}</p>

            <div class="status-actions">
              <button
                class:pressed={selected.state.owned}
                on:click={() => toggleStatus(selected as RowView, 'owned')}
              >
                <Check size={18} />
                <span>Owned</span>
              </button>
              <button
                class:pressed={selected.state.favorite}
                on:click={() => toggleStatus(selected as RowView, 'favorite')}
              >
                <Star size={18} />
                <span>Favorite</span>
              </button>
              <button
                class:pressed={selected.state.wishlist}
                on:click={() => toggleStatus(selected as RowView, 'wishlist')}
              >
                <Heart size={18} />
                <span>Wishlist</span>
              </button>
            </div>

            <label class="notes">
              <span>Notes</span>
              <textarea bind:value={detailNotes} on:blur={saveNotes} rows="8"></textarea>
            </label>
          </aside>
        {/if}
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
