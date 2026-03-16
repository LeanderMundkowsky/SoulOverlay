import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { HangarVehicle } from "@/bindings";

export type { HangarVehicle };

export const useHangarStore = defineStore("hangar", () => {
  const fleet = ref<HangarVehicle[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const stale = ref(false);

  async function loadFleet() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.hangarGetFleet();
      if (result.status === "error") throw result.error;
      const resp = result.data;
      if (resp.ok && resp.data) {
        fleet.value = resp.data;
        stale.value = resp.stale;
      } else {
        error.value = resp.error ?? "Unknown error";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load fleet:", e);
    } finally {
      loading.value = false;
    }
  }

  async function refreshFleet() {
    loading.value = true;
    error.value = null;
    try {
      await commands.cacheRefresh("fleet");
      const result = await commands.hangarGetFleet();
      if (result.status === "error") throw result.error;
      const resp = result.data;
      if (resp.ok && resp.data) {
        fleet.value = resp.data;
        stale.value = resp.stale;
      } else {
        error.value = resp.error ?? "Unknown error";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to refresh fleet:", e);
    } finally {
      loading.value = false;
    }
  }

  return {
    fleet,
    loading,
    error,
    stale,
    loadFleet,
    refreshFleet,
  };
});
