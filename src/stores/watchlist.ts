import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { WatchEntry } from "@/bindings";

export type { WatchEntry };

export const useWatchlistStore = defineStore("watchlist", () => {
  const entries = ref<WatchEntry[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadWatchlist() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.getWatchlist();
      if (result.status === "error") throw result.error;
      entries.value = result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load watchlist:", e);
    } finally {
      loading.value = false;
    }
  }

  async function addEntry(entry: {
    entityId: string;
    entityName: string;
    entityKind: string;
    entitySlug: string;
    terminalId: string;
    terminalName: string;
    priceType: string;
  }) {
    try {
      const result = await commands.addWatchEntry(
        entry.entityId,
        entry.entityName,
        entry.entityKind,
        entry.entitySlug,
        entry.terminalId,
        entry.terminalName,
        entry.priceType,
      );
      if (result.status === "error") throw result.error;
      await loadWatchlist();
    } catch (e) {
      console.error("Failed to add watch entry:", e);
      throw e;
    }
  }

  async function removeEntry(entityId: string, terminalId: string, priceType: string) {
    try {
      const result = await commands.removeWatchEntry(entityId, terminalId, priceType);
      if (result.status === "error") throw result.error;
      await loadWatchlist();
    } catch (e) {
      console.error("Failed to remove watch entry:", e);
      throw e;
    }
  }

  function isWatched(entityId: string, terminalId: string, priceType: string): boolean {
    return entries.value.some(
      (w) => w.entity_id === entityId && w.terminal_id === terminalId && w.price_type === priceType,
    );
  }

  const highlightTarget = ref<{ terminalId: string; priceType: string } | null>(null);

  return {
    entries,
    loading,
    error,
    highlightTarget,
    loadWatchlist,
    addEntry,
    removeEntry,
    isWatched,
  };
});
