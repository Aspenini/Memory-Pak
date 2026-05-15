<script lang="ts">
  import { AlertCircle, Download, ExternalLink, RefreshCw, X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { createEventDispatcher } from 'svelte';
  import type { UpdateStatus } from '../updates';

  export let status: UpdateStatus | null;
  export let checking = false;
  export let installing = false;

  const dispatch = createEventDispatcher<{
    check: void;
    dismiss: string;
    install: void;
  }>();

  $: visible = Boolean(status?.available || status?.error || checking || installing);
  $: version = status?.version ?? 'latest';
  $: actionLabel =
    status?.platform === 'web'
      ? 'Reload'
      : status?.platform === 'android' && !status.canInstallInApp
        ? 'Open Store'
        : 'Install';
  $: readyTitle =
    status?.platform === 'android' && !status.canInstallInApp
      ? 'Check Google Play for updates'
      : `Update ${version} is ready`;
</script>

{#if visible}
  <section class:error={Boolean(status?.error)} class="update-banner" transition:fade>
    <div class="update-banner-copy">
      {#if status?.error}
        <AlertCircle size={18} />
        <div>
          <strong>Update check failed</strong>
          <span>{status.error}</span>
        </div>
      {:else if checking}
        <RefreshCw size={18} />
        <div>
          <strong>Checking for updates</strong>
          <span>Looking for a newer build</span>
        </div>
      {:else if installing}
        <Download size={18} />
        <div>
          <strong>Installing update</strong>
          <span>This may take a moment</span>
        </div>
      {:else}
        <Download size={18} />
        <div>
          <strong>{readyTitle}</strong>
          <span>{status?.notes ?? 'A newer Memory Pak build is available'}</span>
        </div>
      {/if}
    </div>

    <div class="update-banner-actions">
      {#if status?.available}
        <button class="ghost-button" disabled={installing} on:click={() => dispatch('install')}>
          {#if status.platform === 'android' && !status.canInstallInApp}
            <ExternalLink size={16} />
          {:else}
            <Download size={16} />
          {/if}
          <span>{actionLabel}</span>
        </button>
        <button
          class="icon-button"
          aria-label="Dismiss update"
          disabled={installing}
          on:click={() => dispatch('dismiss', version)}
        >
          <X size={17} />
        </button>
      {:else}
        <button class="ghost-button" disabled={checking} on:click={() => dispatch('check')}>
          <RefreshCw size={16} />
          <span>Check</span>
        </button>
        {#if status?.error}
          <button
            class="icon-button"
            aria-label="Dismiss update message"
            on:click={() => dispatch('dismiss', version)}
          >
            <X size={17} />
          </button>
        {/if}
      {/if}
    </div>
  </section>
{/if}
