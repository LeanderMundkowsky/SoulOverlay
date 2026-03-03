import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface HangarVehicle {
  id: string;
  id_vehicle: string;
  name: string;
  model_name: string;
  serial: string | null;
  description: string | null;
  organization_name: string | null;
  is_hidden: boolean;
  is_pledged: boolean;
  date_added: string;
}

interface ApiResponse<T> {
  ok: boolean;
  data: T | null;
  error: string | null;
  stale: boolean;
  total: number | null;
}

export const useHangarStore = defineStore("hangar", () => {
  const fleet = ref<HangarVehicle[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const stale = ref(false);

  async function loadFleet() {
    loading.value = true;
    error.value = null;
    try {
      const resp = await invoke<ApiResponse<HangarVehicle[]>>("hangar_get_fleet");
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

  return {
    fleet,
    loading,
    error,
    stale,
    loadFleet,
  };
});
