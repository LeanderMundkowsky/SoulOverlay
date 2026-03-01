import { onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface OverlayEventCallbacks {
  onWindowFound: () => void;
  onWindowLost: () => void;
  onOpenSettings: () => void;
  onOverlayShown: () => void;
}

/** Sets up all Tauri event listeners needed by the overlay shell. Handles
 *  onMounted registration and onUnmounted cleanup automatically. */
export function useOverlayEvents(callbacks: OverlayEventCallbacks): void {
  let unlistenFound: (() => void) | null = null;
  let unlistenLost: (() => void) | null = null;
  let unlistenOpenSettings: (() => void) | null = null;
  let unlistenOverlayShown: (() => void) | null = null;

  onMounted(async () => {
    unlistenFound = await listen("sc-window-found", callbacks.onWindowFound);
    unlistenLost = await listen("sc-window-lost", callbacks.onWindowLost);
    unlistenOpenSettings = await listen("open-settings", callbacks.onOpenSettings);
    unlistenOverlayShown = await listen("overlay-shown", callbacks.onOverlayShown);

    try {
      const gs = await invoke<{ sc_detected: boolean }>("get_game_state");
      if (gs.sc_detected) {
        callbacks.onWindowFound();
      }
    } catch (e) {
      console.error("get_game_state failed:", e);
    }
  });

  onUnmounted(() => {
    unlistenFound?.();
    unlistenLost?.();
    unlistenOpenSettings?.();
    unlistenOverlayShown?.();
  });
}
