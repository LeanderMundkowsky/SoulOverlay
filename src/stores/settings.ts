import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface LayoutWidths {
  left_panel_px: number;
  settings_panel_px: number;
  search_split_pct: number;
  search_solo_pct: number;
}

export interface Settings {
  hotkey: string;
  uex_api_key: string;
  log_path: string | null;
  overlay_opacity: number;
  esc_closes_overlay: boolean;
  reset_on_open: boolean;
  max_search_results: number;
  cache_ttl_prices_secs: number;
  cache_ttl_catalog_secs: number;
  layout_widths: LayoutWidths;
}

const defaultSettings: Settings = {
  hotkey: "Alt+Shift+S",
  uex_api_key: "",
  log_path: null,
  overlay_opacity: 0.85,
  esc_closes_overlay: true,
  reset_on_open: true,
  max_search_results: 50,
  cache_ttl_prices_secs: 3600,
  cache_ttl_catalog_secs: 86400,
  layout_widths: {
    left_panel_px: 280,
    settings_panel_px: 448,
    search_split_pct: 50,
    search_solo_pct: 50,
  },
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
