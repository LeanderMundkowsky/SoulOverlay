import type { PriceEntry } from "@/bindings";

export interface SortOption {
  key: keyof PriceEntry;
  label: string;
  defaultAsc: boolean;
}

export const richSortOptions: SortOption[] = [
  { key: "price_last", label: "Price", defaultAsc: false },
  { key: "scu_last", label: "SCU", defaultAsc: false },
  { key: "terminal", label: "Terminal", defaultAsc: true },
  { key: "date_updated", label: "Age", defaultAsc: false },
];

export const simpleSortOptions: SortOption[] = [
  { key: "buy_price", label: "Buy Price", defaultAsc: false },
  { key: "sell_price", label: "Sell Price", defaultAsc: false },
  { key: "terminal", label: "Terminal", defaultAsc: true },
  { key: "rent_price", label: "Rent Price", defaultAsc: false },
];

export const itemSortOptions: SortOption[] = [
  { key: "buy_price", label: "Buy Price", defaultAsc: false },
  { key: "sell_price", label: "Sell Price", defaultAsc: false },
  { key: "terminal", label: "Terminal", defaultAsc: true },
  { key: "rent_price", label: "Rent Price", defaultAsc: false },
];

export const vehicleSortOptions: SortOption[] = [
  { key: "buy_price", label: "Buy Price", defaultAsc: false },
  { key: "terminal", label: "Terminal", defaultAsc: true },
  { key: "rent_price", label: "Rent Price", defaultAsc: false },
];

export const fuelSortOptions: SortOption[] = [
  { key: "buy_price", label: "Price", defaultAsc: true },
  { key: "location", label: "Location", defaultAsc: true },
];

export const terminalSortOptions: SortOption[] = [
  { key: "entity_name", label: "Name", defaultAsc: true },
  { key: "buy_price", label: "Buy Price", defaultAsc: false },
  { key: "sell_price", label: "Sell Price", defaultAsc: false },
];

export function sortEntries(source: PriceEntry[], key: keyof PriceEntry, asc: boolean): PriceEntry[] {
  const sorted = [...source];
  sorted.sort((a, b) => {
    const aVal = a[key] ?? 0;
    const bVal = b[key] ?? 0;
    if (typeof aVal === "number" && typeof bVal === "number") {
      return asc ? aVal - bVal : bVal - aVal;
    }
    return asc
      ? String(aVal).localeCompare(String(bVal))
      : String(bVal).localeCompare(String(aVal));
  });
  return sorted;
}

export function sortLabel(key: keyof PriceEntry, options: SortOption[]): string {
  const found = options.find((o) => o.key === key);
  return found ? found.label : "Price";
}
