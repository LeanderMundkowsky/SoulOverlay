import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { Favorite } from "@/bindings";

export type { Favorite };

export const useFavoritesStore = defineStore("favorites", () => {
  const favorites = ref<Favorite[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadFavorites() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.getFavorites();
      if (result.status === "error") throw result.error;
      favorites.value = result.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load favorites:", e);
    } finally {
      loading.value = false;
    }
  }

  async function addFavorite(entity: {
    id: string;
    name: string;
    kind: string;
    slug: string;
    uuid?: string;
  }) {
    try {
      const result = await commands.addFavorite(
        entity.id,
        entity.name,
        entity.kind,
        entity.slug,
        entity.uuid ?? "",
      );
      if (result.status === "error") throw result.error;
      await loadFavorites();
    } catch (e) {
      console.error("Failed to add favorite:", e);
      throw e;
    }
  }

  async function removeFavorite(id: string, kind: string) {
    try {
      const result = await commands.removeFavorite(id, kind);
      if (result.status === "error") throw result.error;
      await loadFavorites();
    } catch (e) {
      console.error("Failed to remove favorite:", e);
      throw e;
    }
  }

  function isFavorite(id: string, kind: string): boolean {
    return favorites.value.some((f) => f.id === id && f.kind === kind);
  }

  async function toggleFavorite(entity: {
    id: string;
    name: string;
    kind: string;
    slug: string;
    uuid?: string;
  }) {
    if (isFavorite(entity.id, entity.kind)) {
      await removeFavorite(entity.id, entity.kind);
    } else {
      await addFavorite(entity);
    }
  }

  return {
    favorites,
    loading,
    error,
    loadFavorites,
    addFavorite,
    removeFavorite,
    isFavorite,
    toggleFavorite,
  };
});
