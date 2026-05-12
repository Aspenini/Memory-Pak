import type { SortKey, TabId } from './types';

export interface SortOption {
  id: SortKey;
  label: string;
}

export function getSortOptions(tab: TabId): SortOption[] {
  if (tab === 'collectibles') {
    return [
      { id: 'name', label: 'Name' },
      { id: 'collection', label: 'Collection' },
      { id: 'category', label: 'Category' },
      { id: 'group', label: 'Pack / Game' },
      { id: 'variant', label: 'Variant' },
      { id: 'year', label: 'Year' },
      { id: 'status', label: 'Status' }
    ];
  }
  if (tab === 'consoles') {
    return [
      { id: 'name', label: 'Name' },
      { id: 'manufacturer', label: 'Manufacturer' },
      { id: 'status', label: 'Status' }
    ];
  }
  return [
    { id: 'title', label: 'Title' },
    { id: 'year', label: 'Year' },
    { id: 'status', label: 'Status' }
  ];
}

export function sortLabel(options: SortOption[], id: SortKey): string {
  return options.find((option) => option.id === id)?.label ?? options[0]?.label ?? 'Name';
}
