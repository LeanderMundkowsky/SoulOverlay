import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { commands } from "@/bindings";
import type { HomeLocationOption } from "@/bindings";

export type { HomeLocationOption };

export const useHomeLocationStore = defineStore("homeLocation", () => {
  const homeLocations = ref<HomeLocationOption[]>([]);
  const homeLocationId = ref<number | null>(null);
  const loading = ref(false);
  const promptDismissed = ref(false);

  /** The currently selected home location object, or null if unset / no longer active. */
  const homeLocation = computed(() =>
    homeLocations.value.find((l) => l.id === homeLocationId.value) ?? null,
  );

  /** True only when a home location is set and present in the active list. */
  const hasValidHomeLocation = computed(() => homeLocation.value !== null);

  /** Whether the post-login prompt should be shown. */
  const shouldPrompt = computed(
    () => !hasValidHomeLocation.value && !promptDismissed.value,
  );

  async function loadHomeLocations(): Promise<void> {
    loading.value = true;
    try {
      const result = await commands.backendGetHomeLocations();
      if (result.status === "ok") {
        // Only expose locations that have a UEX ID (required for inventory transfers)
        homeLocations.value = result.data.filter((l) => l.uex_id !== null);
      }
    } catch (e) {
      console.error("Failed to load home locations:", e);
    } finally {
      loading.value = false;
    }
  }

  async function loadHomeLocationId(): Promise<void> {
    const result = await commands.getHomeLocationId();
    if (result.status === "ok") {
      homeLocationId.value = result.data ?? null;
    }
  }

  async function setHomeLocation(id: number | null): Promise<string | null> {
    const result = await commands.setHomeLocationId(id);
    if (result.status === "error") return result.error;
    homeLocationId.value = id;
    return null;
  }

  function dismissPrompt(): void {
    promptDismissed.value = true;
  }

  /** Dropdown options formatted as "System → Name" for the SearchableDropdown. */
  const dropdownOptions = computed(() =>
    homeLocations.value.map((l) => ({
      id: String(l.id),
      label: `${l.system_name} → ${l.name}`,
      meta: l.type_name,
    })),
  );

  return {
    homeLocations,
    homeLocationId,
    loading,
    promptDismissed,
    homeLocation,
    hasValidHomeLocation,
    shouldPrompt,
    dropdownOptions,
    loadHomeLocations,
    loadHomeLocationId,
    setHomeLocation,
    dismissPrompt,
  };
});
