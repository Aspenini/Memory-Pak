<script lang="ts">
  import { Check, Heart, Star, X } from 'lucide-svelte';
  import { fade } from 'svelte/transition';
  import { createEventDispatcher } from 'svelte';
  import type { RowView } from '../types';
  import { isConsoleView } from '../types';
  import { rowMobileSubtitle, rowTitle } from '../rowFormat';

  export let row: RowView;
  export let notesValue: string;

  const dropdownFade = { duration: 90 };
  const dispatch = createEventDispatcher<{
    close: void;
    save: void;
    toggle: { row: RowView; field: 'owned' | 'favorite' | 'wishlist' };
    notesInput: { rowId: string; value: string };
  }>();

  function detailStats(r: RowView): Array<{ label: string; value: string | number }> {
    if (isConsoleView(r)) {
      return [
        { label: 'Owned', value: r.gameCounts.owned },
        { label: 'Favorite', value: r.gameCounts.favorite },
        { label: 'Wishlist', value: r.gameCounts.wishlist }
      ];
    }
    return [
      { label: 'Owned', value: r.state.owned ? 'Yes' : 'No' },
      { label: 'Favorite', value: r.state.favorite ? 'Yes' : 'No' },
      { label: 'Wishlist', value: r.state.wishlist ? 'Yes' : 'No' }
    ];
  }
</script>

<button
  class="sheet-scrim"
  aria-label="Close details"
  on:click={() => dispatch('close')}
  transition:fade={dropdownFade}
></button>
<dialog class="bottom-sheet detail-sheet" open aria-labelledby="detail-sheet-title">
  <span class="sheet-handle"></span>
  <header class="sheet-header">
    <div>
      <h2 id="detail-sheet-title">{rowTitle(row)}</h2>
      <p>{rowMobileSubtitle(row)}</p>
    </div>
    <button
      class="icon-button"
      type="button"
      aria-label="Close details"
      on:click={() => dispatch('close')}
    >
      <X size={18} />
    </button>
  </header>

  <div class="sheet-actions" role="group" aria-label="Status">
    <button
      type="button"
      class:pressed={row.state.owned}
      on:click={() => dispatch('toggle', { row, field: 'owned' })}
      aria-pressed={row.state.owned}
    >
      <Check size={16} />
      Owned
    </button>
    <button
      type="button"
      class:pressed={row.state.favorite}
      on:click={() => dispatch('toggle', { row, field: 'favorite' })}
      aria-pressed={row.state.favorite}
    >
      <Star size={16} />
      Favorite
    </button>
    <button
      type="button"
      class:pressed={row.state.wishlist}
      on:click={() => dispatch('toggle', { row, field: 'wishlist' })}
      aria-pressed={row.state.wishlist}
    >
      <Heart size={16} />
      Wishlist
    </button>
  </div>

  <section class="sheet-section">
    <h3>Stats</h3>
    <dl class="detail-stats">
      {#each detailStats(row) as stat}
        <div>
          <dt>{stat.label}</dt>
          <dd>{stat.value}</dd>
        </div>
      {/each}
    </dl>
  </section>

  <section class="sheet-section">
    <h3>Notes</h3>
    <textarea
      class="detail-notes"
      placeholder="No notes"
      rows="4"
      value={notesValue}
      on:input={(event) => dispatch('notesInput', { rowId: row.id, value: event.currentTarget.value })}
    ></textarea>
  </section>

  <button class="sheet-save" type="button" on:click={() => dispatch('save')}>Save</button>
</dialog>
