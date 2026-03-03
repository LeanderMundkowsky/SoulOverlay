import { ref } from "vue";
import { commands } from "@/bindings";
import type { CollectionStatus, CacheRefreshResult } from "@/bindings";

export type { CollectionStatus, CacheRefreshResult };

/**
 * Composable for cache status and refresh operations.
 */
export function useCache() {
  const loading = ref(false);
  const refreshing = ref(false);
  const error = ref<string | null>(null);
  const collections = ref<CollectionStatus[]>([]);

  async function fetchStatus() {
    loading.value = true;
    error.value = null;

    try {
      const result = await commands.cacheStatus();
      if (result.status === "error") throw result.error;
      collections.value = result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to fetch cache status:", e);
    } finally {
      loading.value = false;
    }
  }

  async function refreshCollection(collection: string) {
    refreshing.value = true;
    error.value = null;

    try {
      const result = await commands.cacheRefresh(collection);
      if (result.status === "error") throw result.error;
      if (!result.data.ok && result.data.error) {
        error.value = result.data.error;
      }
      // Refresh status after update
      await fetchStatus();
    } catch (e) {
      error.value = String(e);
      console.error("Failed to refresh collection:", e);
    } finally {
      refreshing.value = false;
    }
  }

  async function refreshAll() {
    refreshing.value = true;
    error.value = null;

    try {
      const result = await commands.cacheRefreshAll();
      if (result.status === "error") throw result.error;
      const failures = result.data.filter((r) => !r.ok);
      if (failures.length > 0) {
        error.value = failures
          .map((f) => `${f.collection}: ${f.error}`)
          .join("; ");
      }
      // Refresh status after update
      await fetchStatus();
    } catch (e) {
      error.value = String(e);
      console.error("Failed to refresh all collections:", e);
    } finally {
      refreshing.value = false;
    }
  }

  return {
    loading,
    refreshing,
    error,
    collections,
    fetchStatus,
    refreshCollection,
    refreshAll,
  };
}
