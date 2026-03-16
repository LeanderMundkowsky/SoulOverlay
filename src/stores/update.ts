import { defineStore } from "pinia";
import { ref } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface UpdateInfo {
  version: string;
  date: string | null;
  body: string | null;
}

export const useUpdateStore = defineStore("update", () => {
  const updateAvailable = ref(false);
  const updateInfo = ref<UpdateInfo | null>(null);
  const checking = ref(false);
  const installing = ref(false);
  const error = ref<string | null>(null);
  const dismissed = ref(false);

  let pendingUpdate: Update | null = null;

  async function checkForUpdates() {
    checking.value = true;
    error.value = null;
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateAvailable.value = true;
        updateInfo.value = {
          version: update.version,
          date: update.date ?? null,
          body: update.body ?? null,
        };
      } else {
        pendingUpdate = null;
        updateAvailable.value = false;
        updateInfo.value = null;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      checking.value = false;
    }
  }

  async function installUpdate() {
    if (!pendingUpdate) {
      error.value = "No update available to install";
      return;
    }

    installing.value = true;
    error.value = null;
    try {
      // Back up database + settings before installing
      await invoke("backup_before_update");

      // Download and install via the updater plugin
      await pendingUpdate.downloadAndInstall();

      // Restart the app
      await relaunch();
    } catch (e) {
      error.value = String(e);
      installing.value = false;
    }
  }

  function dismiss() {
    dismissed.value = true;
  }

  // Listen for backend startup update check event
  listen<UpdateInfo>("update-available", (event) => {
    updateAvailable.value = true;
    updateInfo.value = event.payload;
    // No pendingUpdate object from the plugin here — user must
    // click Install which will re-check via the plugin API.
  });

  return {
    updateAvailable,
    updateInfo,
    checking,
    installing,
    error,
    dismissed,
    checkForUpdates,
    installUpdate,
    dismiss,
  };
});
