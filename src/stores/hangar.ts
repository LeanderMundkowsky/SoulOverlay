import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { HangarVehicle, FleetImportResult } from "@/bindings";

export type { HangarVehicle, FleetImportResult };

export const useHangarStore = defineStore("hangar", () => {
  const fleet = ref<HangarVehicle[]>([]);
  const loading = ref(false);
  const importing = ref(false);
  const error = ref<string | null>(null);

  async function loadFleet() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.hangarGetFleet();
      if (result.status === "error") throw result.error;
      fleet.value = result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load fleet:", e);
    } finally {
      loading.value = false;
    }
  }

  async function importFleet(): Promise<FleetImportResult | null> {
    importing.value = true;
    error.value = null;
    try {
      const result = await commands.hangarImportFleet();
      if (result.status === "error") throw result.error;
      fleet.value = result.data.fleet;
      return result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to import fleet:", e);
      return null;
    } finally {
      importing.value = false;
    }
  }

  async function updateVehicle(id: number, name: string | null, description: string | null): Promise<HangarVehicle | null> {
    try {
      const result = await commands.hangarUpdateVehicle(id, name, description);
      if (result.status === "error") throw result.error;
      const idx = fleet.value.findIndex((v) => v.id === id);
      if (idx >= 0) fleet.value[idx] = result.data;
      return result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to update vehicle:", e);
      return null;
    }
  }

  async function deleteVehicle(id: number) {
    try {
      const result = await commands.hangarDeleteVehicle(id);
      if (result.status === "error") throw result.error;
      fleet.value = fleet.value.filter((v) => v.id !== id);
    } catch (e) {
      error.value = String(e);
      console.error("Failed to delete vehicle:", e);
    }
  }

  async function addVehicle(
    modelName: string,
    uexVehicleId: string | null,
    name: string | null,
    serial: string | null,
    description: string | null,
    isPledged: boolean,
    isHidden: boolean,
  ): Promise<HangarVehicle | null> {
    try {
      const result = await commands.hangarAddVehicle(modelName, uexVehicleId, name, serial, description, isPledged, isHidden);
      if (result.status === "error") throw result.error;
      fleet.value.push(result.data);
      return result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to add vehicle:", e);
      return null;
    }
  }

  return {
    fleet,
    loading,
    importing,
    error,
    loadFleet,
    importFleet,
    updateVehicle,
    deleteVehicle,
    addVehicle,
  };
});
