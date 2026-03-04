import { ref } from "vue";
import { defineStore } from "pinia";
import { commands } from "@/bindings";
import type { UexUserProfile } from "@/bindings";

export const useUserStore = defineStore("user", () => {
  const profile = ref<UexUserProfile | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const stale = ref(false);

  async function loadProfile() {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.userGetProfile();
      if (result.status === "error") throw result.error;
      const resp = result.data;
      if (resp.ok && resp.data) {
        profile.value = resp.data;
        stale.value = resp.stale;
      } else {
        error.value = resp.error ?? "Unknown error";
      }
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load user profile:", e);
    } finally {
      loading.value = false;
    }
  }

  return { profile, loading, error, stale, loadProfile };
});
