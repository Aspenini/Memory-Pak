<script lang="ts">
  import { ChevronDown, Search, X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { createEventDispatcher, tick } from 'svelte';
  import type { FilterBy, SortKey } from '../types';
  import type { SortOption } from '../sortOptions';

  interface GroupOption {
    id: string;
    label: string;
  }

  export let searchValue: string;
  export let filterBy: FilterBy;
  export let filters: Array<{ id: FilterBy; label: string; mobileLabel: string }>;
  export let sortBy: SortKey;
  export let sortOptions: SortOption[];
  export let sortLabel: string;
  export let group:
    | { label: string; items: GroupOption[]; selected: string; selectedLabel: string }
    | null = null;
  export let openSelect: 'group' | 'sort' | null;
  export let isMobile: boolean;

  const dropdownFade = { duration: 90 };
  const dispatch = createEventDispatcher<{
    search: string;
    setFilter: FilterBy;
    toggleSelect: 'group' | 'sort';
    selectGroup: string;
    selectSort: SortKey;
  }>();

  let filterPillsElement: HTMLDivElement;
  let filterIndicator = { left: 0, width: 0 };

  $: if (filterPillsElement && filterBy) {
    isMobile;
    void scheduleFilterIndicatorUpdate();
  }

  async function scheduleFilterIndicatorUpdate(): Promise<void> {
    await tick();
    updateFilterIndicator();
  }

  function updateFilterIndicator(): void {
    if (!filterPillsElement) return;
    const activeButton = filterPillsElement.querySelector<HTMLElement>(
      `[data-filter="${filterBy}"]`
    );
    if (!activeButton) return;
    filterIndicator = {
      left: activeButton.offsetLeft,
      width: activeButton.offsetWidth
    };
  }
</script>

<svelte:window on:resize={updateFilterIndicator} />

<section class="toolbar" aria-label="Filters">
  <label class="search-box">
    <Search size={18} />
    <input
      value={searchValue}
      on:input={(event) => dispatch('search', event.currentTarget.value)}
      placeholder="Search collection"
    />
    {#if searchValue}
      <button
        class="search-clear"
        type="button"
        aria-label="Clear search"
        on:click={() => dispatch('search', '')}
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
        on:click={() => dispatch('setFilter', filter.id)}
      >
        {isMobile ? filter.mobileLabel : filter.label}
      </button>
    {/each}
  </div>

  <div class="toolbar-selects">
    {#if group}
      <div class="select-wrap">
        <button
          class="select-trigger"
          type="button"
          aria-haspopup="listbox"
          aria-expanded={openSelect === 'group'}
          on:click={() => dispatch('toggleSelect', 'group')}
        >
          <span class="select-label">{group.label}</span>
          <strong>{group.selectedLabel}</strong>
          <ChevronDown size={16} />
        </button>

        {#if openSelect === 'group' && !isMobile}
          <div
            class="select-popover"
            role="listbox"
            aria-label={group.label}
            transition:fade={dropdownFade}
          >
            <button
              type="button"
              role="option"
              aria-selected={group.selected === 'all'}
              class:selected={group.selected === 'all'}
              on:click={() => dispatch('selectGroup', 'all')}
            >
              All {group.label.toLowerCase()}s
            </button>
            {#each group.items as item}
              <button
                type="button"
                role="option"
                aria-selected={group.selected === item.id}
                class:selected={group.selected === item.id}
                on:click={() => dispatch('selectGroup', item.id)}
              >
                {item.label}
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
        on:click={() => dispatch('toggleSelect', 'sort')}
      >
        <span class="select-label">Sort</span>
        <strong>{sortLabel}</strong>
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
              on:click={() => dispatch('selectSort', option.id)}
            >
              {option.label}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</section>
