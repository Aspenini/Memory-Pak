<script lang="ts">
  import { Check, Heart, Star } from 'lucide-svelte';
  import { createEventDispatcher } from 'svelte';
  import type { RowView } from '../types';
  import { rowMeta, rowSubtitle, rowTitle } from '../rowFormat';

  export let row: RowView;
  export let notesValue: string;

  const dispatch = createEventDispatcher<{
    toggle: { row: RowView; field: 'owned' | 'favorite' | 'wishlist' };
    notesInput: { rowId: string; value: string };
    notesBlur: RowView;
  }>();
</script>

<article
  class="card"
  class:owned={row.state.owned}
  class:favorite={row.state.favorite}
  class:wishlist={row.state.wishlist}
  data-kind={row.kind}
>
  <header class="card-head">
    <div class="card-title">
      <strong title={rowTitle(row)}>{rowTitle(row)}</strong>
      <span title={rowSubtitle(row)}>{rowSubtitle(row)}</span>
      {#if rowMeta(row)}
        <small>{rowMeta(row)}</small>
      {/if}
    </div>
    <div class="card-actions" role="group" aria-label="Status">
      <button
        type="button"
        class:pressed={row.state.owned}
        on:click={() => dispatch('toggle', { row, field: 'owned' })}
        aria-pressed={row.state.owned}
        title="Toggle owned"
      >
        <Check size={16} />
        <span>Owned</span>
      </button>
      <button
        type="button"
        class:pressed={row.state.favorite}
        on:click={() => dispatch('toggle', { row, field: 'favorite' })}
        aria-pressed={row.state.favorite}
        title="Toggle favorite"
      >
        <Star size={16} />
        <span>Favorite</span>
      </button>
      <button
        type="button"
        class:pressed={row.state.wishlist}
        on:click={() => dispatch('toggle', { row, field: 'wishlist' })}
        aria-pressed={row.state.wishlist}
        title="Toggle wishlist"
      >
        <Heart size={16} />
        <span>Wishlist</span>
      </button>
    </div>
  </header>
  <textarea
    class="card-notes"
    placeholder="Notes"
    rows="2"
    value={notesValue}
    on:input={(event) => dispatch('notesInput', { rowId: row.id, value: event.currentTarget.value })}
    on:blur={() => dispatch('notesBlur', row)}
  ></textarea>
</article>
