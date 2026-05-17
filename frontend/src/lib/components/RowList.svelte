<script lang="ts">
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import Search from 'lucide-svelte/icons/search';
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { fade } from 'svelte/transition';
  import { createEventDispatcher } from 'svelte';
  import type { FilterBy, RowView, TabId } from '../types';
  import DesktopRowCard from './DesktopRowCard.svelte';
  import MobileRowCard from './MobileRowCard.svelte';

  export let rows: RowView[];
  export let rowCount: number;
  export let rowHeight: number;
  export let isMobile: boolean;
  export let activeTab: TabId;
  export let filterBy: FilterBy;
  export let searchValue: string;
  export let pendingNotes: Record<string, string>;
  export let scrollElement: HTMLDivElement | undefined = undefined;

  const rowVirtualizer = createVirtualizer<HTMLDivElement, HTMLDivElement>({
    count: 0,
    getScrollElement: () => scrollElement ?? null,
    estimateSize: () => rowHeight,
    getItemKey: (index) => rows[index]?.id ?? index,
    overscan: 8
  });

  const dispatch = createEventDispatcher<{
    toggle: { row: RowView; field: 'owned' | 'favorite' | 'wishlist' };
    notesInput: { rowId: string; value: string };
    notesBlur: RowView;
    open: RowView;
    resetFilters: void;
  }>();

  let listMotion = false;
  let motionTimer: ReturnType<typeof setTimeout> | undefined;
  let lastRowSignature = '';

  $: updateVirtualizerOptions(rowCount, rowHeight, rows, scrollElement);
  $: updateListMotion(activeTab, rowHeight, rows);

  onDestroy(() => {
    if (motionTimer) clearTimeout(motionTimer);
  });

  function updateVirtualizerOptions(
    count: number,
    height: number,
    currentRows: RowView[],
    scroller: HTMLDivElement | undefined
  ): void {
    get(rowVirtualizer).setOptions({
      count,
      getScrollElement: () => scroller ?? null,
      estimateSize: () => height,
      getItemKey: (index) => currentRows[index]?.id ?? index,
      overscan: 8
    });
  }

  function updateListMotion(tab: TabId, height: number, currentRows: RowView[]): void {
    const visibleSignature = currentRows
      .slice(0, 80)
      .map((row) => row.id)
      .join('|');
    const nextSignature = `${tab}:${height}:${currentRows.length}:${visibleSignature}`;
    if (nextSignature === lastRowSignature) return;
    lastRowSignature = nextSignature;
    listMotion = true;
    if (motionTimer) clearTimeout(motionTimer);
    motionTimer = setTimeout(() => {
      listMotion = false;
    }, 240);
  }

  $: rowVirtualItems = $rowVirtualizer.getVirtualItems();
  $: totalSize = $rowVirtualizer.getTotalSize();

  function virtualRowStyle(start: number, size: number): string {
    return `height: ${size}px; transform: translateY(${start}px)`;
  }

  function rowForIndex(index: number): RowView | undefined {
    return rows[index];
  }

  function notesValue(row: RowView): string {
    return pendingNotes[row.id] ?? row.state.notes;
  }

  function activeKindLabel(): string {
    if (activeTab === 'consoles') return 'consoles';
    if (activeTab === 'games') return 'games';
    return 'collectibles';
  }
</script>

<section class="list-region">
  <div class="list-viewport" class:list-motion={listMotion} bind:this={scrollElement}>
    {#if rows.length === 0}
      <div class="empty" transition:fade>
        <Search size={28} />
        <p>No matching {activeKindLabel()}</p>
        {#if searchValue || filterBy !== 'all'}
          <button class="ghost-button" on:click={() => dispatch('resetFilters')}>
            Reset filters
          </button>
        {/if}
      </div>
    {:else}
      <div class="virtual-space" style={`height: ${totalSize}px`}>
        {#each rowVirtualItems as virtualRow (virtualRow.key)}
          {@const row = rowForIndex(virtualRow.index)}
          {#if row}
            <div class="row-slot" style={virtualRowStyle(virtualRow.start, virtualRow.size)}>
              {#if isMobile}
                <MobileRowCard
                  {row}
                  notePreview={notesValue(row).trim()}
                  on:open
                />
              {:else}
                <DesktopRowCard
                  {row}
                  notesValue={notesValue(row)}
                  on:toggle
                  on:notesInput
                  on:notesBlur
                />
              {/if}
            </div>
          {/if}
        {/each}
      </div>
    {/if}
  </div>
</section>
