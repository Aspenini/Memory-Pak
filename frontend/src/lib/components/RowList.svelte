<script lang="ts">
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  import { Search } from 'lucide-svelte';
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

  const dispatch = createEventDispatcher<{
    toggle: { row: RowView; field: 'owned' | 'favorite' | 'wishlist' };
    notesInput: { rowId: string; value: string };
    notesBlur: RowView;
    open: RowView;
    resetFilters: void;
  }>();

  $: rowVirtualizer = createVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollElement ?? null,
    estimateSize: () => rowHeight,
    overscan: 8
  });

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
  <div class="list-viewport" bind:this={scrollElement}>
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
      <div class="virtual-space" style={`height: ${$rowVirtualizer.getTotalSize()}px`}>
        {#each $rowVirtualizer.getVirtualItems() as virtualRow (rows[virtualRow.index].id)}
          {@const row = rows[virtualRow.index]}
          <div
            class="row-slot"
            style={`height: ${virtualRow.size}px; transform: translateY(${virtualRow.start}px)`}
          >
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
        {/each}
      </div>
    {/if}
  </div>
</section>
