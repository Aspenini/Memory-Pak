<script lang="ts">
  import Minus from 'lucide-svelte/icons/minus';
  import Square from 'lucide-svelte/icons/square';
  import X from 'lucide-svelte/icons/x';
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  let maximized = false;

  const appWindow = getCurrentWindow();

  async function syncMaximized(): Promise<void> {
    maximized = await appWindow.isMaximized();
  }

  onMount(() => {
    void syncMaximized();
    const unlisten = appWindow.onResized(() => {
      void syncMaximized();
    });
    return () => {
      void unlisten.then((dispose) => dispose());
    };
  });

  function onDragMouseDown(event: MouseEvent): void {
    if (event.buttons !== 1) return;

    if (event.detail === 2) {
      void appWindow.toggleMaximize();
      return;
    }

    const target = event.target;
    if (target instanceof Element && target.closest('.window-chrome-controls')) {
      return;
    }

    void appWindow.startDragging();
  }
</script>

<header class="window-chrome">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="window-chrome-drag" data-tauri-drag-region on:mousedown={onDragMouseDown}>
    <img src="./icons/icon-192.png" alt="" width="20" height="20" />
    <span>Memory Pak</span>
  </div>

  <div class="window-chrome-controls">
    <button type="button" aria-label="Minimize window" on:click={() => appWindow.minimize()}>
      <Minus size={15} />
    </button>
    <button
      type="button"
      aria-label={maximized ? 'Restore window' : 'Maximize window'}
      on:click={() => appWindow.toggleMaximize()}
    >
      <Square size={13} />
    </button>
    <button type="button" class="close" aria-label="Close window" on:click={() => appWindow.close()}>
      <X size={15} />
    </button>
  </div>
</header>
