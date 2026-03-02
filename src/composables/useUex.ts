import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface UexResult {
  id: string;
  name: string;
  kind: string;
  slug: string;
}

export interface PriceEntry {
  location: string;
  terminal: string;
  buy_price: number;
  sell_price: number;
  scu_available: number | null;
  date_updated: string;
}

interface ApiResponse<T> {
  ok: boolean;
  data: T | null;
  error: string | null;
  stale: boolean;
  total: number | null;
}

/**
 * Composable for UEX API interactions.
 * Searches use the local cache (via api_search), with stale-data awareness.
 * When stale data is returned, a background cache refresh is triggered
 * automatically and the search is re-run with the latest query.
 */
export function useUex() {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const results = ref<UexResult[]>([]);
  const total = ref<number | null>(null);
  const prices = ref<PriceEntry[]>([]);
  const stale = ref(false);

  // Track the latest query so the post-refresh re-search uses current input.
  const latestQuery = ref("");
  // Prevent concurrent background refreshes.
  let refreshing = false;

  async function search(query: string) {
    latestQuery.value = query;

    if (!query.trim()) {
      results.value = [];
      total.value = null;
      stale.value = false;
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      const resp = await invoke<ApiResponse<UexResult[]>>("api_search", { query });
      if (resp.ok && resp.data) {
        results.value = resp.data;
        total.value = resp.total ?? resp.data.length;
        stale.value = resp.stale;

        if (resp.stale && !refreshing) {
          refreshInBackground();
        }
      } else {
        error.value = resp.error ?? "Unknown error";
        results.value = [];
        total.value = null;
        stale.value = false;
      }
    } catch (e) {
      error.value = String(e);
      results.value = [];
      total.value = null;
      stale.value = false;
    } finally {
      loading.value = false;
    }
  }

  async function refreshInBackground() {
    refreshing = true;
    try {
      await invoke("cache_refresh_expired");
      // Re-run with whatever the user has typed by now.
      const q = latestQuery.value;
      if (!q.trim()) {
        stale.value = false;
        return;
      }
      const fresh = await invoke<ApiResponse<UexResult[]>>("api_search", { query: q });
      if (fresh.ok && fresh.data) {
        results.value = fresh.data;
        total.value = fresh.total ?? fresh.data.length;
        stale.value = false;
      }
    } catch (e) {
      console.error("Background cache refresh failed:", e);
    } finally {
      refreshing = false;
    }
  }

  async function getPrices(commodityId: string) {
    loading.value = true;
    error.value = null;

    try {
      const resp = await invoke<ApiResponse<PriceEntry[]>>("api_commodity_prices", {
        commodityId,
      });
      if (resp.ok && resp.data) {
        prices.value = resp.data;
        stale.value = resp.stale;
      } else {
        error.value = resp.error ?? "Unknown error";
        prices.value = [];
      }
    } catch (e) {
      error.value = String(e);
      prices.value = [];
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    results,
    total,
    prices,
    stale,
    search,
    getPrices,
  };
}
