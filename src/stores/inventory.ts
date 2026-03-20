import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { InventoryEntry, InventoryCollection, TransferResult } from "@/bindings";

export type { InventoryEntry, InventoryCollection, TransferResult };

export const useInventoryStore = defineStore("inventory", () => {
  const entries = ref<InventoryEntry[]>([]);
  const collections = ref<InventoryCollection[]>([]);
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
    collectionIds: number[];
  }) {
    const result = await commands.addInventoryEntry(
      params.entityId,
      params.entityName,
      params.entityKind,
      params.locationId,
      params.locationName,
      params.locationSlug,
      params.quantity,
      params.collectionIds,
    );
    if (result.status === "error") throw result.error;
    // Upsert into local state (backend merges on same entity+location)
    const idx = entries.value.findIndex((e) => e.id === result.data.id);
    if (idx >= 0) entries.value[idx] = result.data;
    else entries.value.push(result.data);
  }

  async function removeEntry(id: number) {
    const result = await commands.removeInventoryEntry(id);
    if (result.status === "error") throw result.error;
    entries.value = entries.value.filter((e) => e.id !== id);
  }

  async function removeQuantity(id: number, quantity: number) {
    const result = await commands.removeInventoryQuantity(id, quantity);
    if (result.status === "error") throw result.error;
    if (result.data === null) {
      entries.value = entries.value.filter((e) => e.id !== id);
    } else {
      const idx = entries.value.findIndex((e) => e.id === id);
      if (idx >= 0) entries.value[idx] = result.data;
    }
  }

  async function updateQuantity(id: number, quantity: number) {
    const result = await commands.updateInventoryQuantity(id, quantity);
    if (result.status === "error") throw result.error;
    const idx = entries.value.findIndex((e) => e.id === id);
    if (idx >= 0) entries.value[idx] = result.data;
  }

  async function updateEntry(params: {
    id: number;
    entityId: string;
    entityName: string;
    entityKind: string;
    locationId: string;
    locationName: string;
    locationSlug: string;
    quantity: number;
    collectionIds: number[];
  }) {
    const result = await commands.updateInventoryEntry(
      params.id,
      params.entityId,
      params.entityName,
      params.entityKind,
      params.locationId,
      params.locationName,
      params.locationSlug,
      params.quantity,
      params.collectionIds,
    );
    if (result.status === "error") throw result.error;
    const idx = entries.value.findIndex((e) => e.id === params.id);
    if (idx >= 0) entries.value[idx] = result.data;
  }

  async function transferEntry(params: {
    id: number;
    quantity: number;
    targetLocationId: string;
    targetLocationName: string;
    targetLocationSlug: string;
    targetCollectionIds: number[];
  }): Promise<TransferResult> {
    const result = await commands.transferInventory(
      params.id,
      params.quantity,
      params.targetLocationId,
      params.targetLocationName,
      params.targetLocationSlug,
      params.targetCollectionIds,
    );
    if (result.status === "error") throw result.error;
    const { source, target } = result.data;
    if (source === null) {
      entries.value = entries.value.filter((e) => e.id !== params.id);
    } else {
      const srcIdx = entries.value.findIndex((e) => e.id === params.id);
      if (srcIdx >= 0) entries.value[srcIdx] = source;
    }
    const tgtIdx = entries.value.findIndex((e) => e.id === target.id);
    if (tgtIdx >= 0) entries.value[tgtIdx] = target;
    else entries.value.push(target);
    return result.data;
  }

  // ── Collection CRUD ────────────────────────────────────────────────────────

  async function createCollection(name: string): Promise<InventoryCollection> {
    const result = await commands.inventoryCollectionCreate(name);
    if (result.status === "error") throw result.error;
    collections.value.push(result.data);
    collections.value.sort((a, b) => a.name.localeCompare(b.name));
    return result.data;
  }

  async function updateCollection(id: number, name: string): Promise<InventoryCollection> {
    const result = await commands.inventoryCollectionUpdate(id, name);
    if (result.status === "error") throw result.error;
    const idx = collections.value.findIndex((c) => c.id === id);
    if (idx >= 0) collections.value[idx] = result.data;
    return result.data;
  }

  async function deleteCollection(id: number) {
    const result = await commands.inventoryCollectionDelete(id);
    if (result.status === "error") throw result.error;
    collections.value = collections.value.filter((c) => c.id !== id);
  }

  /// Pull all entries + collections from the backend into Pinia state.
  /// Called after login (or register) so the user immediately sees their data
  /// even if they weren't logged in when the app started.
  async function syncFromBackend() {
    // Run one-time legacy migration first (no-op when legacy table is absent)
    const migResult = await commands.inventoryMigrateLegacy();
    if (migResult.status === "error") {
      console.error("Legacy inventory migration failed:", migResult.error);
    }

    const [syncResult, collResult] = await Promise.all([
      commands.inventorySyncFromBackend(),
      commands.getInventoryCollections(),
    ]);
    if (syncResult.status === "ok") entries.value = syncResult.data;
    if (collResult.status === "ok") collections.value = collResult.data;
  }

  // Cross-tab navigation: HangarTab sets this before switching to inventory tab
  const pendingLocationFilter = ref<{ id: string; name: string } | null>(null);

  return {
    entries,
    collections,
    loading,
    error,
    pendingLocationFilter,
    loadInventory,
    loadCollections,
    addEntry,
    removeEntry,
    removeQuantity,
    updateQuantity,
    updateEntry,
    transferEntry,
    createCollection,
    updateCollection,
    deleteCollection,
    syncFromBackend,
  };
});
