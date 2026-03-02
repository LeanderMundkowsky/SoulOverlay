<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import TabBar from "./components/layout/TabBar.vue";
import StatusBar from "./components/layout/StatusBar.vue";
import SearchTab from "./components/tabs/SearchTab.vue";
import InventoryTab from "./components/tabs/InventoryTab.vue";
import PlaceholderTab from "./components/tabs/PlaceholderTab.vue";
import SettingsPanel from "./components/panels/SettingsPanel.vue";
import DebugPanel from "./components/panels/DebugPanel.vue";
import { useGameStore } from "./stores/game";
import { useSettingsStore } from "./stores/settings";
import { useLogWatcher } from "./composables/useLogWatcher";
import { useOverlayEvents } from "./composables/useOverlayEvents";
import { matchesHotkey } from "./composables/useHotkeyMatch";

const gameStore = useGameStore();
const settingsStore = useSettingsStore();
const activeTab = ref("search");
const showSettings = ref(false);
const showDebug = ref(false);
const scDetected = ref(false);
const searchTabRef = ref<InstanceType<typeof SearchTab> | null>(null);

useLogWatcher();

onMounted(async () => {
  await settingsStore.loadSettings();
  document.addEventListener("keydown", handleKeyDown);
  document.addEventListener("keydown", blockBrowserShortcuts, true);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeyDown);
  document.removeEventListener("keydown", blockBrowserShortcuts, true);
});

useOverlayEvents({
  onWindowFound() {
    scDetected.value = true;
    gameStore.scConnected = true;
  },
  onWindowLost() {
    scDetected.value = false;
    gameStore.scConnected = false;
  },
  onOpenSettings() {
    showSettings.value = true;
  },
  onOverlayShown,
});

function onOverlayShown() {
  if (!settingsStore.settings.reset_on_open) return;
  activeTab.value = "search";
  showSettings.value = false;
  showDebug.value = false;
  nextTick(() => { searchTabRef.value?.focusInput(); });
}

// Capture-phase handler: block all browser built-in shortcuts that make no
// sense in an overlay context (devtools, reload, zoom, find, print, etc.).
function blockBrowserShortcuts(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey;

  // F-keys with browser meaning
  if (e.code === "F12") { e.preventDefault(); return; } // devtools
  if (e.code === "F5")  { e.preventDefault(); return; } // reload
  if (e.code === "F3")  { e.preventDefault(); return; } // find-next (browser)

  if (!ctrl) return;

  switch (e.code) {
    case "KeyR":  // reload
    case "KeyF":  // find
    case "KeyP":  // print
    case "KeyU":  // view source
    case "KeyS":  // save page
    case "KeyG":  // find next
    case "KeyH":  // history (browser)
    case "KeyJ":  // downloads (browser)
    case "KeyL":  // address bar (browser)
    case "Equal": // zoom in  (Ctrl++)
    case "Minus": // zoom out (Ctrl+-)
    case "Digit0": // reset zoom (Ctrl+0)
      e.preventDefault();
      break;
    default:
      if (e.shiftKey && e.code === "KeyI") e.preventDefault(); // Ctrl+Shift+I devtools
      if (e.shiftKey && e.code === "KeyJ") e.preventDefault(); // Ctrl+Shift+J console
      if (e.shiftKey && e.code === "KeyC") e.preventDefault(); // Ctrl+Shift+C inspector
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.code === "F12") {
    onToggleSettings();
    return;
  }

  if (e.key === "Escape") {
    if (showSettings.value) {
      showSettings.value = false;
    } else if (showDebug.value) {
      showDebug.value = false;
    } else if (settingsStore.settings.esc_closes_overlay) {
      invoke("hide_overlay_cmd");
    }
    return;
  }

  if (matchesHotkey(e, settingsStore.settings.hotkey)) {
    e.preventDefault();
    invoke("hide_overlay_cmd");
  }
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
    <!-- Background dimming layer -->
    <div
      class="absolute inset-0 pointer-events-none"
      :style="{ backgroundColor: `rgba(0,0,0,${settingsStore.settings.overlay_opacity})` }"
    ></div>

    <!-- UI layer -->
    <div class="relative w-full h-full flex flex-col">
      <TabBar
        :active-tab="activeTab"
        :sc-detected="scDetected"
        @update:active-tab="(t) => { activeTab = t; }"
        @close="onTabClose"
        @toggle-settings="onToggleSettings"
        @toggle-debug="onToggleDebug"
      />

      <!-- Main content + side panels -->
      <div class="flex-1 flex overflow-hidden">
        <div class="flex-1 overflow-y-auto">
          <SearchTab
            v-show="activeTab === 'search'"
            ref="searchTabRef"
            :sc-detected="scDetected"
          />
          <InventoryTab v-show="activeTab === 'inventory'" />
          <PlaceholderTab
            v-show="activeTab !== 'search' && activeTab !== 'inventory'"
          />
        </div>

        <!-- Settings side panel -->
        <Transition name="slide">
          <SettingsPanel
            v-if="showSettings"
            class="w-96 flex-shrink-0"
            @close="showSettings = false"
          />
        </Transition>

        <!-- Debug side panel -->
        <Transition name="slide">
          <DebugPanel
            v-if="showDebug"
            class="w-72 flex-shrink-0"
            @close="showDebug = false"
          />
        </Transition>
      </div>

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
