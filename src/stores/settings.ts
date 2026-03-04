import { defineStore } from "pinia";
import { ref } from "vue";
import { commands } from "@/bindings";
import type { Settings, Keybinds, LayoutWidths } from "@/bindings";

export type { Settings, Keybinds, LayoutWidths };

export const useSettingsStore = defineStore("settings", () => {
  // Populated by loadSettings() in main.ts before app.mount(), so always non-null.
  const settings = ref({} as Settings);
  const defaults = ref({} as Settings);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadSettings() {
    loading.value = true;
    error.value = null;
    try {
      const [settingsResult, defaultsResult] = await Promise.all([
        commands.getSettings(),
        commands.getDefaultSettings(),
      ]);
      if (settingsResult.status === "error") throw settingsResult.error;
      if (defaultsResult.status === "error") throw defaultsResult.error;
      settings.value = settingsResult.data;
      defaults.value = defaultsResult.data;
    } catch (e) {
      error.value = String(e);
      console.error("Failed to load settings:", e);
    } finally {
      loading.value = false;
    }
  }

  async function saveSettings(newSettings: Settings) {
    loading.value = true;
    error.value = null;
    try {
      const result = await commands.saveSettings(newSettings);
      if (result.status === "error") throw result.error;
      // Deep-clone via JSON round-trip to strip any nested reactive proxies
      settings.value = JSON.parse(JSON.stringify(newSettings));
    } catch (e) {
      error.value = String(e);
      console.error("Failed to save settings:", e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    settings,
    defaults,
    loading,
    error,
    loadSettings,
    saveSettings,
  };
});
