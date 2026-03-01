<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import TabBar from "./components/overlay/TabBar.vue";
import SearchBar from "./components/overlay/SearchBar.vue";
import CommodityPanel from "./components/overlay/CommodityPanel.vue";
import DebugPanel from "./components/overlay/DebugPanel.vue";
import StatusBar from "./components/overlay/StatusBar.vue";
import SettingsPanel from "./components/settings/SettingsPanel.vue";
import { useGameStore } from "./stores/game";
import { useSettingsStore } from "./stores/settings";
import { useLogWatcher } from "./composables/useLogWatcher";

const gameStore = useGameStore();
const settingsStore = useSettingsStore();
const activeTab = ref("search");
const showSettings = ref(false);
const showDebug = ref(false);
const scDetected = ref(false);
const selectedCommodity = ref<{ id: string; name: string } | null>(null);

useLogWatcher();

onMounted(async () => {
  await settingsStore.loadSettings();
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
});

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    if (showSettings.value) {
      showSettings.value = false;
    } else if (showDebug.value) {
      showDebug.value = false;
    } else {
      invoke("hide_overlay_cmd");
    }
    return;
  }

  if (matchesHotkey(e, settingsStore.settings.hotkey)) {
    e.preventDefault();
    invoke("hide_overlay_cmd");
  }
}

function matchesHotkey(e: KeyboardEvent, hotkey: string): boolean {
  const parts = hotkey.split("+").map((p) => p.trim().toLowerCase());
  const key = parts[parts.length - 1];
  const needCtrl = parts.includes("ctrl") || parts.includes("control");
  const needAlt = parts.includes("alt");
  const needShift = parts.includes("shift");

  if (e.ctrlKey !== needCtrl || e.altKey !== needAlt || e.shiftKey !== needShift) {
    return false;
  }

  const eventKey = codeToToken(e.code);
  return eventKey === key;
}

function codeToToken(code: string): string {
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

  try {
    const gs = await invoke<{ sc_detected: boolean }>("get_game_state");
    scDetected.value = gs.sc_detected;
    gameStore.scConnected = gs.sc_detected;
  } catch (e) {
    console.error("get_game_state failed:", e);
  }
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

function onTabClose() {
  invoke("hide_overlay_cmd");
}

function onToggleSettings() {
  showSettings.value = !showSettings.value;
  if (showSettings.value) showDebug.value = false;
}

function onToggleDebug() {
  showDebug.value = !showDebug.value;
  if (showDebug.value) showSettings.value = false;
}
</script>

<template>
  <div class="w-full h-full">
    <!-- Background dimming layer — opacity-controlled, sits behind all UI -->
    <div
      class="absolute inset-0 pointer-events-none"
      :style="{ backgroundColor: `rgba(0,0,0,${settingsStore.settings.overlay_opacity})` }"
    ></div>

    <!-- UI layer — always full opacity, positioned above the background -->
    <div class="relative w-full h-full flex flex-col">
      <!-- Tab bar -->
      <TabBar
        :active-tab="activeTab"
        :sc-detected="scDetected"
        @update:active-tab="(t) => { activeTab = t; selectedCommodity = null; }"
        @close="onTabClose"
        @toggle-settings="onToggleSettings"
        @toggle-debug="onToggleDebug"
      />

      <!-- Main content + side panels -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Tab content area -->
        <div class="flex-1 overflow-y-auto">

          <!-- SEARCH tab -->
          <div v-if="activeTab === 'search'" class="p-6 grid grid-cols-1 gap-4 max-w-4xl mx-auto w-full">
            <!-- SC not detected notice — pill style -->
            <div
              v-if="!scDetected"
              class="flex items-start gap-3 bg-yellow-500/10 border border-yellow-500/30 rounded-xl px-4 py-3 text-yellow-200 text-sm"
            >
              <svg class="w-4 h-4 mt-0.5 flex-shrink-0 text-yellow-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
              </svg>
              <div>
                <p class="font-semibold">Star Citizen not detected</p>
                <p class="text-yellow-200/70 text-xs mt-0.5">
                  Make sure Star Citizen is running in <strong>Borderless Windowed</strong> mode.
                </p>
              </div>
            </div>

            <!-- Search card -->
            <div class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-visible">
              <SearchBar @select="onCommoditySelected" />
            </div>

            <!-- Commodity prices card -->
            <div v-if="selectedCommodity" class="bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden">
              <CommodityPanel
                :commodity-id="selectedCommodity.id"
                :commodity-name="selectedCommodity.name"
                @close="closeCommodityPanel"
              />
            </div>
          </div>

          <!-- Placeholder tabs -->
          <div v-else class="p-6 max-w-4xl mx-auto w-full">
            <div class="bg-[#1a1d24] border border-white/10 rounded-xl flex flex-col items-center justify-center py-16 gap-3 text-white/20 select-none">
              <svg class="w-10 h-10" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
              </svg>
              <p class="text-sm uppercase tracking-widest font-semibold">Coming soon</p>
              <p class="text-xs">This tab will be implemented in a future update.</p>
            </div>
          </div>
        </div>

        <!-- Settings panel (slide in from right) -->
        <Transition name="slide">
          <SettingsPanel
            v-if="showSettings"
            @close="showSettings = false"
            class="w-96 flex-shrink-0"
          />
        </Transition>

        <!-- Debug panel (slide in from right) -->
        <Transition name="slide">
          <DebugPanel
            v-if="showDebug"
            @close="showDebug = false"
            class="w-72 flex-shrink-0"
          />
        </Transition>
      </div>

      <!-- Status bar -->
      <StatusBar :sc-detected="scDetected" @toggle-debug="onToggleDebug" />
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
  margin-right: -384px;
}
.slide-enter-to,
.slide-leave-from {
  margin-right: 0;
}
</style>
