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
}

/**
 * Composable for UEX API interactions.
 * Searches use the local cache (via api_search), with stale-data awareness.
 */
export function useUex() {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const results = ref<UexResult[]>([]);
  const prices = ref<PriceEntry[]>([]);
  const stale = ref(false);

  async function search(query: string) {
    if (!query.trim()) {
      results.value = [];
      stale.value = false;
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      const resp = await invoke<ApiResponse<UexResult[]>>("api_search", { query });
      if (resp.ok && resp.data) {
        results.value = resp.data;
        stale.value = resp.stale;
      } else {
        error.value = resp.error ?? "Unknown error";
        results.value = [];
        stale.value = false;
      }
    } catch (e) {
      error.value = String(e);
      results.value = [];
      stale.value = false;
    } finally {
      loading.value = false;
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
    prices,
    stale,
    search,
    getPrices,
  };
}
