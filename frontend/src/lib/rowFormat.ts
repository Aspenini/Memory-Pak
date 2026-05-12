import type { RowView } from './types';
import { isCollectibleView, isConsoleView, isGameView } from './types';

export function rowTitle(row: RowView): string {
  return isGameView(row) ? row.title : row.name;
}

export function rowSubtitle(row: RowView): string {
  if (isConsoleView(row)) {
    const generation = row.generation > 0 ? ` / Gen ${row.generation}` : '';
    return `${row.manufacturer}${generation}`;
  }
  if (isGameView(row)) {
    const dev = row.developer ? `${row.developer} / ` : '';
    return `${row.consoleName} / ${dev}${row.publisher || 'Unknown publisher'} / ${row.year || 'Unknown year'}`;
  }
  if (isCollectibleView(row)) {
    const parts = [row.collectionName, row.category, row.group, row.variant].filter(Boolean);
    return parts.join(' / ');
  }
  return '';
}

export function rowMobileSubtitle(row: RowView): string {
  if (isConsoleView(row)) return `${row.manufacturer}`;
  if (isGameView(row)) return `${row.consoleName} \u00B7 ${row.year || 'Unknown year'}`;
  if (isCollectibleView(row)) {
    return `${row.collectionName} \u00B7 ${row.category || row.group || ''}`.trim();
  }
  return '';
}

export function rowMeta(row: RowView): string | null {
  if (isConsoleView(row)) {
    return `${row.gameCounts.owned} owned / ${row.gameCounts.favorite} favorite / ${row.gameCounts.wishlist} wishlist`;
  }
  return null;
}

export function rowMobileMeta(row: RowView): string | null {
  if (isConsoleView(row)) {
    return `${row.gameCounts.owned} owned \u00B7 ${row.gameCounts.favorite} fav \u00B7 ${row.gameCounts.wishlist} wish`;
  }
  const states = [
    row.state.owned ? 'owned' : 'not owned',
    row.state.favorite ? 'fav' : 'not fav',
    row.state.wishlist ? 'wish' : 'not wish'
  ];
  return states.join(' \u00B7 ');
}
