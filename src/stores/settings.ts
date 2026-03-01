import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  hotkey: string;
  uex_api_key: string;
  log_path: string | null;
  overlay_opacity: number;
  esc_closes_overlay: boolean;
  reset_on_open: boolean;
}

const defaultSettings: Settings = {
  hotkey: "Alt+Shift+S",
  uex_api_key: "",
  log_path: null,
  overlay_opacity: 0.85,
  esc_closes_overlay: true,
  reset_on_open: true,
};

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings>({ ...defaultSettings });
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadSettings() {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Settings>("get_settings");
      settings.value = result;
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
      await invoke("save_settings", { newSettings });
      settings.value = { ...newSettings };
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
    loading,
    error,
    loadSettings,
    saveSettings,
  };
});
