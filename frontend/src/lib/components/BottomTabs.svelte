<script lang="ts">
  import { createEventDispatcher, type ComponentType } from 'svelte';
  import type { TabId } from '../types';

  export let tabs: Array<{ id: TabId; label: string; mobileLabel: string; icon: ComponentType }>;
  export let activeTab: TabId;

  const dispatch = createEventDispatcher<{ select: TabId }>();
</script>

<nav class="bottom-tabs" aria-label="Collection sections">
  {#each tabs as tab}
    <button
      aria-label={tab.label}
      class:active={activeTab === tab.id}
      on:click={() => dispatch('select', tab.id)}
    >
      <svelte:component this={tab.icon} size={20} />
      <span>{tab.mobileLabel}</span>
    </button>
  {/each}
</nav>
