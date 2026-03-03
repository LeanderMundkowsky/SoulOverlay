import { ref } from "vue";
import { commands } from "@/bindings";
import type { UexResult, PriceEntry, EntityInfo, ApiResponse, Result } from "@/bindings";

export type { UexResult, PriceEntry, EntityInfo };

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
      const result = await commands.apiSearch(query);
      if (result.status === "error") throw result.error;
      const resp = result.data;
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
      await commands.cacheRefreshExpired();
      // Re-run with whatever the user has typed by now.
      const q = latestQuery.value;
      if (!q.trim()) {
        stale.value = false;
        return;
      }
      const result = await commands.apiSearch(q);
      if (result.status === "ok") {
        const fresh = result.data;
        if (fresh.ok && fresh.data) {
          results.value = fresh.data;
          total.value = fresh.total ?? fresh.data.length;
          stale.value = false;
        }
      }
    } catch (e) {
      console.error("Background cache refresh failed:", e);
    } finally {
      refreshing = false;
    }
  }

  async function getPrices(commodityId: string) {
    return getEntityPrices("commodity", commodityId);
  }

  type PriceCommand = (id: string) => Promise<Result<ApiResponse<PriceEntry[]>, string>>;

  const priceCommandMap: Record<string, PriceCommand> = {
    commodity: (id) => commands.apiCommodityPrices(id),
    raw_commodity: (id) => commands.apiRawCommodityPrices(id),
    item: (id) => commands.apiItemPrices(id),
    vehicle: (id) => commands.apiVehiclePurchasePrices(id),
    "ground vehicle": (id) => commands.apiVehiclePurchasePrices(id),
    vehicle_rental: (id) => commands.apiVehicleRentalPrices(id),
    fuel: (id) => commands.apiFuelPrices(id),
    location: (id) => commands.apiFuelPrices(id),
  };

  async function getEntityPrices(kind: string, entityId: string) {
    loading.value = true;
    error.value = null;

    const command = priceCommandMap[kind];
    if (!command) {
      prices.value = [];
      loading.value = false;
      return;
    }

    try {
      const result = await command(entityId);
      if (result.status === "error") throw result.error;
      const resp = result.data;
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

  const entityInfo = ref<EntityInfo | null>(null);
  const entityInfoLoading = ref(false);
  const entityInfoError = ref<string | null>(null);

  async function getEntityInfo(kind: string, entityId: string) {
    entityInfoLoading.value = true;
    entityInfoError.value = null;
    entityInfo.value = null;

    try {
      const result = await commands.apiEntityInfo(kind, entityId);
      if (result.status === "error") throw result.error;
      const resp = result.data;
      if (resp.ok && resp.data) {
        entityInfo.value = resp.data;
      } else {
        entityInfoError.value = resp.error ?? "Unknown error";
      }
    } catch (e) {
      entityInfoError.value = String(e);
    } finally {
      entityInfoLoading.value = false;
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
    getEntityPrices,
    entityInfo,
    entityInfoLoading,
    entityInfoError,
    getEntityInfo,
  };
}
