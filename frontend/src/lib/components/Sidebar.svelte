<script lang="ts">
  import { createEventDispatcher, type ComponentType } from 'svelte';
  import type { TabId } from '../types';

  export let tabs: Array<{ id: TabId; label: string; icon: ComponentType }>;
  export let activeTab: TabId;
  export let counts: Record<TabId, number>;
  export let version: string;
  export let open = false;

  const dispatch = createEventDispatcher<{ select: TabId }>();
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

  <div class="sidebar-footer">
    <small>v{version}</small>
  </div>
</aside>
