<script lang="ts">
  import { Check, Download, Heart, Menu, MoreVertical, Star, Upload } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { createEventDispatcher } from 'svelte';

  export let title: string;
  export let shownCount: number;
  export let summary: { owned: number; favorite: number; wishlist: number };
  export let showFavoriteSummary: boolean;
  export let showWishlistSummary: boolean;
  export let refreshing: boolean;
  export let ownershipPercent: number;
  export let mobileMenuOpen: boolean;

  const dropdownFade = { duration: 90 };
  const dispatch = createEventDispatcher<{
    openNav: void;
    import: void;
    export: void;
    backup: void;
    restore: void;
    toggleMobileMenu: void;
  }>();
</script>

<header class="topbar">
  <button
    class="icon-button mobile-only"
    aria-label="Open navigation"
    on:click={() => dispatch('openNav')}
  >
    <Menu size={22} />
  </button>
  <div class="topbar-title">
    <h1>{title}</h1>
    <p>
      {shownCount.toLocaleString()} shown
      <span class="mobile-inline-stats">
        · {summary.owned.toLocaleString()} owned
        {#if showFavoriteSummary}
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
    {#if showFavoriteSummary}
      <div>
        <Star size={14} />
        <strong>{summary.favorite.toLocaleString()}</strong>
        <span>favorite</span>
      </div>
    {/if}
    {#if showWishlistSummary}
      <div>
        <Heart size={14} />
        <strong>{summary.wishlist.toLocaleString()}</strong>
        <span>wishlist</span>
      </div>
    {/if}
  </div>
  <div class="top-actions">
    <button class="ghost-button" on:click={() => dispatch('import')}>
      <Upload size={18} />
      <span>Import</span>
    </button>
    <button class="ghost-button" on:click={() => dispatch('export')}>
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
      on:click={() => dispatch('toggleMobileMenu')}
    >
      <MoreVertical size={20} />
    </button>

    {#if mobileMenuOpen}
      <div class="mobile-action-menu" role="menu" transition:fade={dropdownFade}>
        <button type="button" role="menuitem" on:click={() => dispatch('import')}>Import</button>
        <button type="button" role="menuitem" on:click={() => dispatch('export')}>Export</button>
        <button type="button" role="menuitem" on:click={() => dispatch('backup')}>Backup</button>
        <button type="button" role="menuitem" on:click={() => dispatch('restore')}>Restore</button>
      </div>
    {/if}
  </div>
</header>
