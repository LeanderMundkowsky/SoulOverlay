import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/stores/settings";

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

/**
 * Composable for UEX API interactions.
 */
export function useUex() {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const results = ref<UexResult[]>([]);
  const prices = ref<PriceEntry[]>([]);

  const settingsStore = useSettingsStore();

  async function search(query: string) {
    if (!query.trim()) {
      results.value = [];
      return;
    }

    loading.value = true;
    error.value = null;

    try {
      const apiKey = settingsStore.settings.uex_api_key;
      results.value = await invoke<UexResult[]>("uex_search", {
        query,
        apiKey,
      });
    } catch (e) {
      error.value = String(e);
      results.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function getPrices(commodityId: string) {
    loading.value = true;
    error.value = null;

    try {
      const apiKey = settingsStore.settings.uex_api_key;
      prices.value = await invoke<PriceEntry[]>("uex_prices", {
        commodity: commodityId,
        apiKey,
      });
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
    search,
    getPrices,
  };
}
