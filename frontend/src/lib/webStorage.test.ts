import { describe, expect, it, beforeEach } from 'vitest';
import { loadPersistedState, savePersistedState } from './webStorage';
import type { PersistedState } from './types';

beforeEach(() => {
  localStorage.clear();
});

describe('web storage compatibility', () => {
  it('loads existing localStorage keys', () => {
    localStorage.setItem(
      'memory_pak_console_states',
      JSON.stringify([{ console_id: 'nes', owned: true, favorite: false, wishlist: false, notes: '' }])
    );
    localStorage.setItem(
      'memory_pak_state_nes',
      JSON.stringify([{ game_id: 'nes-aaaaaaaaaaaaaaaa', owned: false, favorite: true, wishlist: false, notes: '' }])
    );

    const state = loadPersistedState();

    expect(state.console_states.nes.owned).toBe(true);
    expect(state.game_states['nes-aaaaaaaaaaaaaaaa'].favorite).toBe(true);
  });

  it('saves the versioned snapshot and legacy split files', () => {
    const state: PersistedState = {
      console_states: {
        nes: { console_id: 'nes', owned: true, favorite: false, wishlist: false, notes: '' }
      },
      game_states: {
        'nes-aaaaaaaaaaaaaaaa': {
          game_id: 'nes-aaaaaaaaaaaaaaaa',
          owned: true,
          favorite: false,
          wishlist: false,
          notes: 'cart only'
        }
      },
      lego_dimensions_states: {},
      skylanders_states: {}
    };

    savePersistedState(state);

    expect(localStorage.getItem('memory_pak_state_v2')).toContain('cart only');
    expect(localStorage.getItem('memory_pak_state_nes')).toContain('cart only');
  });
});

