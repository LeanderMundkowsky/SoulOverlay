<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import SearchBar from "./components/overlay/SearchBar.vue";
import CommodityPanel from "./components/overlay/CommodityPanel.vue";
import StatusBar from "./components/overlay/StatusBar.vue";
import SettingsPanel from "./components/settings/SettingsPanel.vue";
import { useGameStore } from "./stores/game";
import { useSettingsStore } from "./stores/settings";
import { useLogWatcher } from "./composables/useLogWatcher";

const gameStore = useGameStore();
const settingsStore = useSettingsStore();
const showSettings = ref(false);
const scDetected = ref(false);
const selectedCommodity = ref<{ id: string; name: string } | null>(null);

// Initialize log watcher composable
useLogWatcher();

// Load settings on mount
onMounted(async () => {
  await settingsStore.loadSettings();

  // Listen for ESC key to hide overlay
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
});

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    if (showSettings.value) {
      showSettings.value = false;
    } else {
      invoke("hide_overlay_cmd");
    }
    return;
  }

  // Fallback hotkey handling: when the overlay is focused, the LL keyboard
  // hook may not fire (Windows limitation). Match the configured hotkey here
  // and hide the overlay from the frontend side.
  if (matchesHotkey(e, settingsStore.settings.hotkey)) {
    e.preventDefault();
    invoke("hide_overlay_cmd");
  }
}

/** Check if a KeyboardEvent matches a hotkey string like "F4" or "Alt+Shift+S". */
function matchesHotkey(e: KeyboardEvent, hotkey: string): boolean {
  const parts = hotkey.split("+").map((p) => p.trim().toLowerCase());
  const key = parts[parts.length - 1];
  const needCtrl = parts.includes("ctrl") || parts.includes("control");
  const needAlt = parts.includes("alt");
  const needShift = parts.includes("shift");

  if (e.ctrlKey !== needCtrl || e.altKey !== needAlt || e.shiftKey !== needShift) {
    return false;
  }

  // Use e.code (physical key) to avoid locale-dependent e.key values
  // when modifiers are held (e.g. Alt+Shift+S → "Í" on some layouts).
  const eventKey = codeToToken(e.code);
  return eventKey === key;
}

/** Map KeyboardEvent.code to the lowercase token format used in hotkey strings. */
function codeToToken(code: string): string {
  // "KeyA" → "a", "Digit0" → "0", "F4" → "f4"
  const letterMatch = code.match(/^Key([A-Z])$/);
  if (letterMatch) return letterMatch[1].toLowerCase();

  const digitMatch = code.match(/^Digit([0-9])$/);
  if (digitMatch) return digitMatch[1];

  const fMatch = code.match(/^F(\d+)$/);
  if (fMatch) return `f${fMatch[1]}`;

  const map: Record<string, string> = {
    Space: "space",
    Tab: "tab",
    Escape: "escape",
    Insert: "insert",
    Delete: "delete",
    Home: "home",
    End: "end",
    PageUp: "pageup",
    PageDown: "pagedown",
    ArrowUp: "up",
    ArrowDown: "down",
    ArrowLeft: "left",
    ArrowRight: "right",
  };
  return map[code] ?? code.toLowerCase();
}

// Listen for SC window events
let unlistenFound: (() => void) | null = null;
let unlistenLost: (() => void) | null = null;
let unlistenOpenSettings: (() => void) | null = null;

onMounted(async () => {
  unlistenFound = await listen("sc-window-found", () => {
    scDetected.value = true;
    gameStore.scConnected = true;
  });

  unlistenLost = await listen("sc-window-lost", () => {
    scDetected.value = false;
    gameStore.scConnected = false;
  });

  unlistenOpenSettings = await listen("open-settings", () => {
    showSettings.value = true;
  });
});

onUnmounted(() => {
  unlistenFound?.();
  unlistenLost?.();
  unlistenOpenSettings?.();
});

function onCommoditySelected(commodity: { id: string; name: string }) {
  selectedCommodity.value = commodity;
}

function closeCommodityPanel() {
  selectedCommodity.value = null;
}
</script>

<template>
  <div class="w-full h-full">
    <!-- Main overlay background -->
    <div class="w-full h-full flex flex-col" :style="{ backgroundColor: `rgba(0,0,0,${settingsStore.settings.overlay_opacity})` }">
      <!-- Top bar -->
      <div class="flex-shrink-0 flex items-center justify-between px-6 py-3 bg-black/40 border-b border-white/10">
        <div class="flex items-center gap-3">
          <h1 class="text-white font-bold text-lg tracking-wide">SoulOverlay</h1>
          <div
            class="flex items-center gap-1.5 text-xs"
            :class="scDetected ? 'text-green-400' : 'text-yellow-400'"
          >
            <span
              class="w-2 h-2 rounded-full"
              :class="scDetected ? 'bg-green-400' : 'bg-yellow-400'"
            ></span>
            {{ scDetected ? "Star Citizen Connected" : "Waiting for Star Citizen..." }}
          </div>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="showSettings = !showSettings"
            class="text-white/60 hover:text-white transition-colors p-2 rounded hover:bg-white/10"
            title="Settings"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"></path>
            </svg>
          </button>
          <button
            @click="invoke('hide_overlay_cmd')"
            class="text-white/60 hover:text-white transition-colors p-2 rounded hover:bg-white/10"
            title="Close overlay (ESC)"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>

      <!-- Content area -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Main content -->
        <div class="flex-1 flex flex-col p-6 gap-4 overflow-y-auto">
          <!-- First-run notice -->
          <div
            v-if="!scDetected"
            class="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-4 text-yellow-200 text-sm"
          >
            <p class="font-semibold mb-1">Star Citizen not detected</p>
            <p class="text-yellow-200/70">
              Make sure Star Citizen is running in <strong>Borderless Windowed</strong> mode.
              The overlay will automatically detect the game window.
            </p>
          </div>

          <!-- Search -->
          <SearchBar @select="onCommoditySelected" />

          <!-- Commodity prices -->
          <CommodityPanel
            v-if="selectedCommodity"
            :commodity-id="selectedCommodity.id"
            :commodity-name="selectedCommodity.name"
            @close="closeCommodityPanel"
          />

          <!-- Spacer -->
          <div class="flex-1"></div>
        </div>

        <!-- Settings panel (slide in from right, inside content area) -->
        <Transition name="slide">
          <SettingsPanel
            v-if="showSettings"
            @close="showSettings = false"
            class="w-96 flex-shrink-0"
          />
        </Transition>
      </div>

      <!-- Status bar -->
      <StatusBar />
    </div>
  </div>
</template>

<style>
.slide-enter-active,
.slide-leave-active {
  transition: margin-right 0.2s ease;
}
.slide-enter-from,
.slide-leave-to {
  margin-right: -384px; /* -w-96 */
}
.slide-enter-to,
.slide-leave-from {
  margin-right: 0;
}
</style>
