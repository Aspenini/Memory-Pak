<script lang="ts">
  import { X } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { createEventDispatcher } from 'svelte';
  import type { SortKey } from '../types';
  import type { SortOption } from '../sortOptions';

  interface GroupOption {
    id: string;
    label: string;
  }

  export let mode: 'group' | 'sort';
  export let title: string;
  export let sortOptions: SortOption[];
  export let sortBy: SortKey;
  export let groupItems: GroupOption[] = [];
  export let groupSelected: string = 'all';
  export let groupAllLabel: string = 'All';

  const dropdownFade = { duration: 90 };
  const sheetIn = { y: 36, duration: 190, easing: cubicOut, opacity: 0.98 };
  const sheetOut = { y: 20, duration: 130, easing: cubicOut, opacity: 0.98 };
  const dispatch = createEventDispatcher<{
    close: void;
    selectSort: SortKey;
    selectGroup: string;
  }>();
</script>

<button
  class="sheet-scrim"
  aria-label="Close selector"
  on:click={() => dispatch('close')}
  transition:fade={dropdownFade}
></button>
<dialog
  class="bottom-sheet option-sheet"
  open
  aria-labelledby="option-sheet-title"
  in:fly={sheetIn}
  out:fly={sheetOut}
>
  <span class="sheet-handle"></span>
  <header class="sheet-header">
    <h2 id="option-sheet-title">{title}</h2>
    <button
      class="icon-button"
      type="button"
      aria-label="Close selector"
      on:click={() => dispatch('close')}
    >
      <X size={18} />
    </button>
  </header>

  {#if mode === 'sort'}
    <div class="sheet-options" role="listbox" aria-label="Sort by">
      {#each sortOptions as option}
        <button
          type="button"
          role="option"
          aria-selected={sortBy === option.id}
          class:selected={sortBy === option.id}
          on:click={() => dispatch('selectSort', option.id)}
        >
          <span>{sortBy === option.id ? '●' : '○'}</span>
          {option.label}
        </button>
      {/each}
    </div>
  {:else}
    <div class="sheet-options" role="listbox" aria-label={title}>
      <button
        type="button"
        role="option"
        aria-selected={groupSelected === 'all'}
        class:selected={groupSelected === 'all'}
        on:click={() => dispatch('selectGroup', 'all')}
      >
        <span>{groupSelected === 'all' ? '●' : '○'}</span>
        {groupAllLabel}
      </button>
      {#each groupItems as item}
        <button
          type="button"
          role="option"
          aria-selected={groupSelected === item.id}
          class:selected={groupSelected === item.id}
          on:click={() => dispatch('selectGroup', item.id)}
        >
          <span>{groupSelected === item.id ? '●' : '○'}</span>
          {item.label}
        </button>
      {/each}
    </div>
  {/if}
</dialog>
