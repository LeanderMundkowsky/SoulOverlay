import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { InventoryEntry } from "@/bindings";

export type { InventoryEntry };

export const useInventoryStore = defineStore("inventory", () => {
  const entries = ref<InventoryEntry[]>([]);
  const collections = ref<string[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadInventory() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.getInventory();
      if (result.status === "error") throw result.error;
      entries.value = result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load inventory:", e);
    } finally {
      loading.value = false;
    }
  }

  async function loadCollections() {
    try {
      const result = await commands.getInventoryCollections();
      if (result.status === "error") throw result.error;
      collections.value = result.data;
    } catch (e) {
      console.error("Failed to load collections:", e);
    }
  }

  async function addEntry(params: {
    entityId: string;
    entityName: string;
    entityKind: string;
    locationId: string;
    locationName: string;
    locationSlug: string;
    quantity: number;
    collection: string;
  }) {
    try {
      const result = await commands.addInventoryEntry(
        params.entityId,
        params.entityName,
        params.entityKind,
        params.locationId,
        params.locationName,
        params.locationSlug,
        params.quantity,
        params.collection,
      );
      if (result.status === "error") throw result.error;
      await loadInventory();
      await loadCollections();
    } catch (e) {
      console.error("Failed to add inventory entry:", e);
      throw e;
    }
  }

  async function removeEntry(id: number) {
    try {
      const result = await commands.removeInventoryEntry(id);
      if (result.status === "error") throw result.error;
      await loadInventory();
      await loadCollections();
    } catch (e) {
      console.error("Failed to remove inventory entry:", e);
      throw e;
    }
  }

  async function removeQuantity(id: number, quantity: number) {
    try {
      const result = await commands.removeInventoryQuantity(id, quantity);
      if (result.status === "error") throw result.error;
      await loadInventory();
    } catch (e) {
      console.error("Failed to remove inventory quantity:", e);
      throw e;
    }
  }

  async function updateQuantity(id: number, quantity: number) {
    try {
      const result = await commands.updateInventoryQuantity(id, quantity);
      if (result.status === "error") throw result.error;
      await loadInventory();
    } catch (e) {
      console.error("Failed to update inventory quantity:", e);
      throw e;
    }
  }

  async function transferEntry(params: {
    id: number;
    quantity: number;
    targetLocationId: string;
    targetLocationName: string;
    targetLocationSlug: string;
    targetCollection: string;
  }) {
    try {
      const result = await commands.transferInventory(
        params.id,
        params.quantity,
        params.targetLocationId,
        params.targetLocationName,
        params.targetLocationSlug,
        params.targetCollection,
      );
      if (result.status === "error") throw result.error;
      await loadInventory();
      await loadCollections();
    } catch (e) {
      console.error("Failed to transfer inventory:", e);
      throw e;
    }
  }

  return {
    entries,
    collections,
    loading,
    error,
    loadInventory,
    loadCollections,
    addEntry,
    removeEntry,
    removeQuantity,
    updateQuantity,
    transferEntry,
  };
});
