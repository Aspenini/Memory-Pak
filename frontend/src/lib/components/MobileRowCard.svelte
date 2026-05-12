<script lang="ts">
  import { Check, Heart, Star } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';
  import type { RowView } from '../types';
  import { rowMobileMeta, rowMobileSubtitle, rowTitle } from '../rowFormat';

  export let row: RowView;
  export let notePreview: string;

  const dispatch = createEventDispatcher<{ open: RowView }>();
</script>

<button
  type="button"
  class="mobile-card"
  class:owned={row.state.owned}
  class:favorite={row.state.favorite}
  class:wishlist={row.state.wishlist}
  data-kind={row.kind}
  on:click={() => dispatch('open', row)}
>
  <span class="mobile-card-top">
    <strong title={rowTitle(row)}>{rowTitle(row)}</strong>
    <span class="mobile-status-icons" aria-hidden="true">
      <span class="mobile-status-icon owned" class:active={row.state.owned}>
        <Check size={16} />
      </span>
      <span class="mobile-status-icon favorite" class:active={row.state.favorite}>
        <Star size={16} />
      </span>
      <span class="mobile-status-icon wishlist" class:active={row.state.wishlist}>
        <Heart size={16} />
      </span>
    </span>
  </span>
  <span class="mobile-card-subtitle" title={rowMobileSubtitle(row)}>
    {rowMobileSubtitle(row)}
  </span>
  {#if rowMobileMeta(row)}
    <span class="mobile-card-meta">{rowMobileMeta(row)}</span>
  {/if}
  {#if notePreview}
    <span class="mobile-card-note">Note: {notePreview}</span>
  {/if}
</button>
