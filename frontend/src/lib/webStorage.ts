import type {
  ConsoleState,
  GameState,
  LegoDimensionState,
  PersistedState,
  SkylanderState
} from './types';

const SNAPSHOT_KEY = 'memory_pak_state_v2';
const CONSOLES_KEY = 'memory_pak_console_states';
const LEGO_KEY = 'memory_pak_lego_dimensions_states';
const SKYLANDERS_KEY = 'memory_pak_skylanders_states';
const GAME_PREFIX = 'memory_pak_state_';

export function loadPersistedState(): PersistedState {
  const snapshot = readJson<PersistedState>(SNAPSHOT_KEY);
  if (snapshot) return withDefaults(snapshot);

  return {
    console_states: fromArray(readJson<ConsoleState[]>(CONSOLES_KEY), 'console_id'),
    game_states: loadLegacyGameStates(),
    lego_dimensions_states: fromArray(readJson<LegoDimensionState[]>(LEGO_KEY), 'figure_id'),
    skylanders_states: fromArray(readJson<SkylanderState[]>(SKYLANDERS_KEY), 'skylander_id')
  };
}

export function savePersistedState(state: PersistedState): void {
  const snapshot = withDefaults(state);
  localStorage.setItem(SNAPSHOT_KEY, JSON.stringify(snapshot));
  localStorage.setItem(CONSOLES_KEY, JSON.stringify(Object.values(snapshot.console_states)));
  localStorage.setItem(LEGO_KEY, JSON.stringify(Object.values(snapshot.lego_dimensions_states)));
  localStorage.setItem(SKYLANDERS_KEY, JSON.stringify(Object.values(snapshot.skylanders_states)));

  const gamesByConsole = new Map<string, GameState[]>();
  for (const state of Object.values(snapshot.game_states)) {
    const consoleId = getConsoleFromGameId(state.game_id);
    if (!consoleId) continue;
    const bucket = gamesByConsole.get(consoleId) ?? [];
    bucket.push(state);
    gamesByConsole.set(consoleId, bucket);
  }

  for (const [consoleId, states] of gamesByConsole) {
    states.sort((a, b) => a.game_id.localeCompare(b.game_id));
    localStorage.setItem(`${GAME_PREFIX}${consoleId}`, JSON.stringify(states));
  }
}

function loadLegacyGameStates(): Record<string, GameState> {
  const states: Record<string, GameState> = {};

  for (let index = 0; index < localStorage.length; index += 1) {
    const key = localStorage.key(index);
    if (!key?.startsWith(GAME_PREFIX)) continue;
    const games = readJson<GameState[]>(key) ?? [];
    for (const state of games) {
      states[state.game_id] = state;
    }
  }

  return states;
}

function fromArray<T extends Record<K, string>, K extends keyof T>(
  values: T[] | null,
  key: K
): Record<string, T> {
  const out: Record<string, T> = {};
  for (const value of values ?? []) {
    out[value[key]] = value;
  }
  return out;
}

function readJson<T>(key: string): T | null {
  const raw = localStorage.getItem(key);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as T;
  } catch (error) {
    console.warn(`Ignoring invalid Memory Pak localStorage value ${key}`, error);
    return null;
  }
}

function withDefaults(state: PersistedState): PersistedState {
  return {
    console_states: state.console_states ?? {},
    game_states: state.game_states ?? {},
    lego_dimensions_states: state.lego_dimensions_states ?? {},
    skylanders_states: state.skylanders_states ?? {}
  };
}

function getConsoleFromGameId(gameId: string): string {
  const splitAt = gameId.lastIndexOf('-');
  return splitAt > 0 ? gameId.slice(0, splitAt) : '';
}

