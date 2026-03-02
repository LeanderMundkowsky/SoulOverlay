import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface Favorite {
  id: string;
  name: string;
  kind: string;
  slug: string;
  uuid: string;
  added_at: string;
}

export const useFavoritesStore = defineStore("favorites", () => {
  const favorites = ref<Favorite[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadFavorites() {
    loading.value = true;
    error.value = null;
    try {
      favorites.value = await invoke<Favorite[]>("get_favorites");
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
    uuid: string;
  }) {
    try {
      await invoke("add_favorite", {
        id: entity.id,
        name: entity.name,
        kind: entity.kind,
        slug: entity.slug,
        uuid: entity.uuid,
      });
      await loadFavorites();
    } catch (e) {
      console.error("Failed to add favorite:", e);
      throw e;
    }
  }

  async function removeFavorite(id: string, kind: string) {
    try {
      await invoke("remove_favorite", { id, kind });
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
    uuid: string;
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
