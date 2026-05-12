<script lang="ts">
  import { createEventDispatcher, onMount, tick, type ComponentType } from 'svelte';
  import type { TabId } from '../types';

  export let tabs: Array<{ id: TabId; label: string; mobileLabel: string; icon: ComponentType }>;
  export let activeTab: TabId;

  const dispatch = createEventDispatcher<{ select: TabId }>();

  let navEl: HTMLElement | undefined;
  let indicator = { left: 0, width: 0 };

  function updateIndicator(): void {
    if (!navEl) return;
    const activeBtn = navEl.querySelector<HTMLButtonElement>('button.active');
    if (!activeBtn) {
      indicator = { left: 0, width: 0 };
      return;
    }
    indicator = {
      left: activeBtn.offsetLeft,
      width: activeBtn.offsetWidth
    };
  }

  async function scheduleIndicatorUpdate(): Promise<void> {
    await tick();
    updateIndicator();
  }

  $: activeTab, navEl, void scheduleIndicatorUpdate();
  $: tabs, navEl, void scheduleIndicatorUpdate();

  onMount(() => {
    void scheduleIndicatorUpdate();
    window.addEventListener('resize', updateIndicator);
    return () => window.removeEventListener('resize', updateIndicator);
  });
</script>

<nav
  bind:this={navEl}
  class="bottom-tabs"
  class:ready={indicator.width > 0}
  style={`--bottom-tab-left: ${indicator.left}px; --bottom-tab-width: ${indicator.width}px`}
  aria-label="Collection sections"
>
  {#each tabs as tab}
    <button
      type="button"
      aria-label={tab.label}
      aria-current={activeTab === tab.id ? 'page' : undefined}
      class:active={activeTab === tab.id}
      on:click={() => dispatch('select', tab.id)}
    >
      <svelte:component this={tab.icon} size={20} />
      <span>{tab.mobileLabel}</span>
    </button>
  {/each}
</nav>
