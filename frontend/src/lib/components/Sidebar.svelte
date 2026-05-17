<script lang="ts">
  import Download from 'lucide-svelte/icons/download';
  import RefreshCw from 'lucide-svelte/icons/refresh-cw';
  import Upload from 'lucide-svelte/icons/upload';
  import { createEventDispatcher, type ComponentType } from 'svelte';
  import type { TabId } from '../types';

  export let tabs: Array<{ id: TabId; label: string; icon: ComponentType }>;
  export let activeTab: TabId;
  export let counts: Record<TabId, number>;
  export let version: string;
  export let open = false;
  export let showUpdates = true;

  const dispatch = createEventDispatcher<{
    backup: void;
    checkUpdates: void;
    restore: void;
    select: TabId;
  }>();
</script>

<aside class:open class="sidebar">
  <div class="brand">
    <img src="./icons/icon-192.png" alt="" />
    <div>
      <strong>Memory Pak</strong>
    </div>
  </div>

  <nav aria-label="Collection sections">
    {#each tabs as tab}
      <button class:active={activeTab === tab.id} on:click={() => dispatch('select', tab.id)}>
        <svelte:component this={tab.icon} size={20} />
        <span>{tab.label}</span>
        <small>{counts[tab.id]?.toLocaleString() ?? '0'}</small>
      </button>
    {/each}
  </nav>

  <section class="sidebar-actions" aria-label="Collection actions">
    {#if showUpdates}
      <button type="button" on:click={() => dispatch('checkUpdates')}>
        <RefreshCw size={17} />
        <span>Updates</span>
      </button>
    {/if}
    <button type="button" on:click={() => dispatch('backup')}>
      <Download size={17} />
      <span>Backup</span>
    </button>
    <button type="button" on:click={() => dispatch('restore')}>
      <Upload size={17} />
      <span>Restore</span>
    </button>
  </section>

  <div class="sidebar-footer">
    <small>v{version}</small>
  </div>
</aside>
