<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import TabBar from "./components/layout/TabBar.vue";
import StatusBar from "./components/layout/StatusBar.vue";
import SearchTab from "./components/tabs/SearchTab.vue";
import DetailsTab from "./components/tabs/DetailsTab.vue";
import InventoryTab from "./components/tabs/InventoryTab.vue";
import PlaceholderTab from "./components/tabs/PlaceholderTab.vue";
import FavoritesPanel from "./components/overlay/FavoritesPanel.vue";
import SettingsPanel from "./components/panels/SettingsPanel.vue";
import DebugPanel from "./components/panels/DebugPanel.vue";
import ResizeHandle from "./components/ui/ResizeHandle.vue";
import KeybindsModal from "./components/ui/KeybindsModal.vue";
import { useGameStore } from "./stores/game";
import { useSettingsStore } from "./stores/settings";
import { useFavoritesStore } from "./stores/favorites";
import { useDetailsStore } from "./stores/details";
import { useLogWatcher } from "./composables/useLogWatcher";
import { useOverlayEvents } from "./composables/useOverlayEvents";
import { matchesHotkey } from "./composables/useHotkeyMatch";

const gameStore = useGameStore();
const settingsStore = useSettingsStore();
const favoritesStore = useFavoritesStore();
const detailsStore = useDetailsStore();
const activeTab = ref("search");
const showSettings = ref(false);
const showDebug = ref(false);
const showKeybinds = ref(false);
const showFavorites = ref(true);
const scDetected = ref(false);
const searchTabRef = ref<InstanceType<typeof SearchTab> | null>(null);

// Panel widths — loaded from settings, updated on drag
const leftPanelPx = ref(280);
const settingsPanelPx = ref(448);

let saveDebounce: ReturnType<typeof setTimeout> | null = null;
function scheduleSave() {
  if (saveDebounce) clearTimeout(saveDebounce);
  saveDebounce = setTimeout(() => {
    const s = settingsStore.settings;
    settingsStore.saveSettings({
      ...s,
      layout_widths: {
        ...s.layout_widths,
        left_panel_px: leftPanelPx.value,
        settings_panel_px: settingsPanelPx.value,
      },
    });
  }, 500);
}

function onLeftResize(newPx: number) {
  leftPanelPx.value = Math.min(500, Math.max(180, newPx));
  scheduleSave();
}
function onLeftReset() {
  leftPanelPx.value = 280;
  scheduleSave();
}
function onSettingsResize(newPx: number) {
  settingsPanelPx.value = Math.min(700, Math.max(300, newPx));
  scheduleSave();
}
function onSettingsReset() {
  settingsPanelPx.value = 448;
  scheduleSave();
}

useLogWatcher();

watch(() => settingsStore.settings.font_size, (px) => {
  document.documentElement.style.fontSize = px + "px";
});

// Watch for details store tab-switch requests
watch(() => detailsStore.requestTabSwitch, (shouldSwitch) => {
  if (shouldSwitch) {
    activeTab.value = "details";
    detailsStore.clearTabSwitchRequest();
  }
});

onMounted(async () => {
  await settingsStore.loadSettings();
  leftPanelPx.value = settingsStore.settings.layout_widths.left_panel_px;
  settingsPanelPx.value = settingsStore.settings.layout_widths.settings_panel_px;
  document.documentElement.style.fontSize = settingsStore.settings.font_size + "px";
  await favoritesStore.loadFavorites();
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
  if (e.code === "F11") { e.preventDefault(); return; } // browser fullscreen

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
  if (matchesHotkey(e, settingsStore.settings.keybinds.toggle_settings)) {
    onToggleSettings();
    return;
  }

  if (matchesHotkey(e, settingsStore.settings.keybinds.toggle_debug)) {
    onToggleDebug();
    return;
  }

  if (e.key === "Escape") {
    // On the search tab, let the search bar consume ESC first
    // (deselect row → clear query → only then close overlay)
    if (activeTab.value === "search" && searchTabRef.value?.handleEsc()) {
      return;
    }
    if (settingsStore.settings.esc_closes_overlay) {
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
}

function onToggleDebug() {
  showDebug.value = !showDebug.value;
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
        :show-favorites="showFavorites && (activeTab === 'search' || activeTab === 'details')"
        @update:active-tab="(t) => { activeTab = t; }"
        @close="onTabClose"
        @toggle-settings="onToggleSettings"
        @toggle-debug="onToggleDebug"
        @toggle-favorites="showFavorites = !showFavorites"
      />

      <!-- Main content + side panels -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Left column: Favorites + Debug stacked -->
        <Transition name="slide-left">
          <div
            v-if="showDebug || (showFavorites && (activeTab === 'search' || activeTab === 'details'))"
            class="relative flex-shrink-0 flex flex-col gap-4 py-4 pl-4"
            :style="{ width: leftPanelPx + 'px' }"
          >
            <FavoritesPanel
              v-if="showFavorites && (activeTab === 'search' || activeTab === 'details')"
              class="flex-1 min-h-0"
            />
            <DebugPanel
              v-if="showDebug"
              class="flex-1 min-h-0"
              @close="showDebug = false"
            />
            <ResizeHandle :default-px="280" @resize="onLeftResize" @reset="onLeftReset" />
          </div>
        </Transition>

        <div class="flex-1 overflow-y-auto">
          <SearchTab
            v-show="activeTab === 'search'"
            ref="searchTabRef"
            :sc-detected="scDetected"
          />
          <DetailsTab v-show="activeTab === 'details'" />
          <InventoryTab v-show="activeTab === 'inventory'" />
          <PlaceholderTab
            v-show="activeTab !== 'search' && activeTab !== 'details' && activeTab !== 'inventory'"
          />
        </div>

        <!-- Settings side panel -->
        <Transition name="slide">
          <div
            v-if="showSettings"
            class="relative z-50 flex-shrink-0 h-full"
            :style="{ width: settingsPanelPx + 'px' }"
          >
            <ResizeHandle
              side="left"
              :default-px="448"
              @resize="onSettingsResize"
              @reset="onSettingsReset"
            />
            <SettingsPanel
              class="w-full"
              @close="showSettings = false"
              @open-keybinds="showKeybinds = true"
            />
          </div>
        </Transition>
      </div>

      <StatusBar :sc-detected="scDetected" @toggle-debug="onToggleDebug" />
    </div>

    <!-- Keybinds modal (Teleports to body, z-40; settings panel is z-50) -->
    <KeybindsModal v-if="showKeybinds" @close="showKeybinds = false" />
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
.slide-left-enter-active,
.slide-left-leave-active {
  transition: margin-left 0.2s ease;
}
.slide-left-enter-from,
.slide-left-leave-to {
  margin-left: -236px;
}
.slide-left-enter-to,
.slide-left-leave-from {
  margin-left: 0;
}
</style>
