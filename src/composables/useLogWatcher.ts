import { onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useGameStore } from "@/stores/game";

/**
 * Composable that listens to game log events emitted from the Rust backend
 * and updates the game store accordingly.
 */
export function useLogWatcher() {
  const gameStore = useGameStore();

  let unlisteners: (() => void)[] = [];

  onMounted(async () => {
    const unlistenLocation = await listen<{ location: string }>(
      "sc-location",
      (event) => {
        gameStore.setLocation(event.payload.location);
      }
    );

    const unlistenDeath = await listen<{ killer: string }>(
      "sc-death",
      (event) => {
        gameStore.addDeath(event.payload.killer);
      }
    );

    const unlistenKill = await listen<{ victim: string }>(
      "sc-kill",
      (event) => {
        gameStore.addKill(event.payload.victim);
      }
    );

    const unlistenShip = await listen<{ ship: string }>(
      "sc-ship-changed",
      (event) => {
        gameStore.setShip(event.payload.ship);
      }
    );

    unlisteners = [unlistenLocation, unlistenDeath, unlistenKill, unlistenShip];
  });

  onUnmounted(() => {
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners = [];
  });
}
