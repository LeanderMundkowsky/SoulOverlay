import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface CollectionStatus {
  collection: string;
  display_name: string;
  cached_at: string | null;
  ttl_secs: number;
  is_expired: boolean;
  entry_count: number;
}

export interface CacheRefreshResult {
  ok: boolean;
  collection: string;
  error: string | null;
}

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
      collections.value = await invoke<CollectionStatus[]>("cache_status");
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
      const result = await invoke<CacheRefreshResult>("cache_refresh", { collection });
      if (!result.ok && result.error) {
        error.value = result.error;
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
      const results = await invoke<CacheRefreshResult[]>("cache_refresh_all");
      const failures = results.filter((r) => !r.ok);
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
