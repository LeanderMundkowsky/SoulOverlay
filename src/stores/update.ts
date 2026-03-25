import { defineStore } from "pinia";
import { ref } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/bindings";

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
    pendingUpdate = null;
    try {
      // Use the Rust command so release notes are fetched from the GitHub
      // release body rather than the auto-generated latest.json body.
      const result = await commands.checkForUpdate();
      if (result.status === "error") throw result.error;
      const info = result.data;
      if (info) {
        updateAvailable.value = true;
        updateInfo.value = {
          version: info.version,
          date: info.date ?? null,
          body: info.body ?? null,
        };
      } else {
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
    // Fetch the Update object from the plugin (needed for downloadAndInstall).
    if (!pendingUpdate) {
      try {
        pendingUpdate = await check();
      } catch (e) {
        error.value = `Update check failed: ${String(e)}`;
        return;
      }
      if (!pendingUpdate) {
        error.value = "No update available to install";
        return;
      }
    }

    installing.value = true;
    error.value = null;
    try {
      await invoke("backup_before_update");
      await pendingUpdate.downloadAndInstall();
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
    // No pendingUpdate object here — installUpdate() will re-check via the plugin.
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
