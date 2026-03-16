<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from "vue";
import { commands } from "@/bindings";
import type { WatchEntry } from "@/bindings";
import TabBar from "./components/layout/TabBar.vue";
import StatusBar from "./components/layout/StatusBar.vue";
import SearchTab from "./components/tabs/SearchTab.vue";
import DetailsTab from "./components/tabs/DetailsTab.vue";
import InventoryTab from "./components/tabs/InventoryTab.vue";
import HangarTab from "./components/tabs/HangarTab.vue";
import CZTab from "./components/tabs/CZTab.vue";
import ProfileTab from "./components/tabs/ProfileTab.vue";
import PlaceholderTab from "./components/tabs/PlaceholderTab.vue";
import FavoritesPanel from "./components/overlay/FavoritesPanel.vue";
import WatchListPanel from "./components/overlay/WatchListPanel.vue";
import SettingsPanel from "./components/panels/SettingsPanel.vue";
import DebugPanel from "./components/panels/DebugPanel.vue";
import ResizeHandle from "./components/ui/ResizeHandle.vue";
import KeybindsModal from "./components/ui/KeybindsModal.vue";
import { useGameStore } from "./stores/game";
import { useSettingsStore } from "./stores/settings";
import { useFavoritesStore } from "./stores/favorites";
import { useDetailsStore } from "./stores/details";
import { useUserStore } from "./stores/user";
import { useWatchlistStore } from "./stores/watchlist";
import { useInventoryStore } from "./stores/inventory";
import { useLogWatcher } from "./composables/useLogWatcher";
import { useOverlayEvents } from "./composables/useOverlayEvents";
import { matchesHotkey } from "./composables/useHotkeyMatch";
import { useDragDrop } from "./composables/useDragDrop";
import UpdateBanner from "./components/ui/UpdateBanner.vue";
import UpdateModal from "./components/ui/UpdateModal.vue";

const gameStore = useGameStore();
const settingsStore = useSettingsStore();
const favoritesStore = useFavoritesStore();
const detailsStore = useDetailsStore();
const userStore = useUserStore();
const watchlistStore = useWatchlistStore();
const inventoryStore = useInventoryStore();
const { dragging: isDragActive, payload: dragPayload, ghostX, ghostY, ghostLabel } = useDragDrop();
const activeTab = ref("search");
const showSettings = ref(false);
const showDebug = ref(false);
const showKeybinds = ref(false);
const showUpdateModal = ref(false);
const showFavorites = ref(true);
const showWatchlist = ref(false);
const scDetected = ref(false);
const searchTabRef = ref<InstanceType<typeof SearchTab> | null>(null);

const isAuthenticated = computed(() =>
  settingsStore.settings.uex_api_key.length > 0 &&
  settingsStore.settings.uex_secret_key.length > 0
);
const userAvatarUrl = computed(() => {
  const avatar = userStore.profile?.avatar;
  if (!avatar) return null;
  return avatar.replace(/^https?:\/\//, "http://uex-img.localhost/");
});

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
  // Settings already loaded in main.ts before mount; just apply layout values.
  leftPanelPx.value = settingsStore.settings.layout_widths.left_panel_px;
  settingsPanelPx.value = settingsStore.settings.layout_widths.settings_panel_px;
  document.documentElement.style.fontSize = settingsStore.settings.font_size + "px";
  await favoritesStore.loadFavorites();
  await watchlistStore.loadWatchlist();
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

  // In dev mode let Ctrl+Shift+I through for WebView2 devtools.
  if (import.meta.env.DEV && ctrl && e.shiftKey && e.code === "KeyI") return;

  // F-keys with browser meaning
  if (e.code === "F12") { e.preventDefault(); return; } // devtools
  if (e.code === "F5") { e.preventDefault(); return; } // reload
  if (e.code === "F3") { e.preventDefault(); return; } // find-next (browser)
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
      commands.hideOverlayCmd();
    }
    return;
  }

  if (matchesHotkey(e, settingsStore.settings.hotkey)) {
    e.preventDefault();
    commands.hideOverlayCmd();
  }
}

function onTabClose() {
  commands.hideOverlayCmd();
}

function onToggleSettings() {
  showSettings.value = !showSettings.value;
}

function onToggleDebug() {
  showDebug.value = !showDebug.value;
}

function onFavoriteSelect(fav: { id: string; name: string; kind: string; slug: string; uuid: string }) {
  if (activeTab.value === "search") {
    searchTabRef.value?.selectEntity(fav);
  } else {
    detailsStore.currentEntity = { id: fav.id, name: fav.name, kind: fav.kind, slug: fav.slug, uuid: fav.uuid };
  }
}

function onFavoritePin(fav: { id: string; name: string; kind: string; slug: string }) {
  if (activeTab.value !== "search") {
    activeTab.value = "search";
  }
  searchTabRef.value?.pinLocation(fav);
}

function onWatchSelect(entry: WatchEntry) {
  const entity = { id: entry.entity_id, name: entry.entity_name, kind: entry.entity_kind, slug: entry.entity_slug };
  watchlistStore.highlightTarget = { terminalId: entry.terminal_id, priceType: entry.price_type };
  if (activeTab.value === "search") {
    searchTabRef.value?.selectEntity(entity);
  } else {
    detailsStore.currentEntity = { ...entity, uuid: "" };
  }
}

function onToggleFavorites() {
  showFavorites.value = !showFavorites.value;
  if (showFavorites.value) showWatchlist.value = false;
}

function onToggleWatchlist() {
  showWatchlist.value = !showWatchlist.value;
  if (showWatchlist.value) showFavorites.value = false;
}

function onSwitchToInventory(locationId: string, locationName: string) {
  inventoryStore.pendingLocationFilter = { id: locationId, name: locationName };
  activeTab.value = "inventory";
}

// Temporarily show the target panel while dragging, restore on drop
let dragPanelSnapshot: { favorites: boolean; watchlist: boolean } | null = null;
watch(isDragActive, (active) => {
  if (active && dragPayload.value) {
    dragPanelSnapshot = { favorites: showFavorites.value, watchlist: showWatchlist.value };
    if (dragPayload.value.type === "entity") {
      showFavorites.value = true;
      showWatchlist.value = false;
    } else if (dragPayload.value.type === "price") {
      showWatchlist.value = true;
      showFavorites.value = false;
    }
  } else if (!active && dragPanelSnapshot) {
    showFavorites.value = dragPanelSnapshot.favorites;
    showWatchlist.value = dragPanelSnapshot.watchlist;
    dragPanelSnapshot = null;
  }
});
</script>

<template>
  <div class="w-full h-full">
    <!-- Background dimming layer -->
    <div class="absolute inset-0 pointer-events-none"
      :style="{ backgroundColor: `rgba(0,0,0,${settingsStore.settings.overlay_opacity})` }">
    </div>

    <!-- UI layer -->
    <div class="relative w-full h-full flex flex-col">
      <UpdateBanner @open-update-modal="showUpdateModal = true" />

      <TabBar :active-tab="activeTab"
        :show-favorites="showFavorites && (activeTab === 'search' || activeTab === 'details')"
        :show-watchlist="showWatchlist && (activeTab === 'search' || activeTab === 'details')"
        :is-authenticated="isAuthenticated"
        :user-avatar-url="userAvatarUrl"
        @update:active-tab="(t) => { activeTab = t; }"
        @close="onTabClose"
        @toggle-settings="onToggleSettings"
        @toggle-debug="onToggleDebug"
        @toggle-favorites="onToggleFavorites"
        @toggle-watchlist="onToggleWatchlist" />

      <!-- Main content + side panels -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Left column: Favorites / Watch list + Debug stacked -->
        <Transition name="slide-left">
          <div
            v-if="showDebug || ((showFavorites || showWatchlist) && (activeTab === 'search' || activeTab === 'details'))"
            class="relative shrink-0 flex flex-col gap-4 py-4 pl-4"
            :style="{ width: leftPanelPx + 'px', '--panel-w': leftPanelPx + 'px' }">
            <FavoritesPanel
              v-if="showFavorites && (activeTab === 'search' || activeTab === 'details')"
              class="flex-1 min-h-0"
              @select="onFavoriteSelect"
              @pin="onFavoritePin" />
            <WatchListPanel
              v-if="showWatchlist && (activeTab === 'search' || activeTab === 'details')"
              class="flex-1 min-h-0"
              @select="onWatchSelect" />
            <DebugPanel v-if="showDebug"
              class="flex-1 min-h-0"
              @close="showDebug = false" />
            <ResizeHandle :default-px="280"
              @resize="onLeftResize"
              @reset="onLeftReset" />
          </div>
        </Transition>

        <div class="flex-1 overflow-y-auto">
          <SearchTab v-show="activeTab === 'search'"
            ref="searchTabRef"
            :active="activeTab === 'search'" />
          <DetailsTab v-show="activeTab === 'details'"
            :active="activeTab === 'details'" />
          <InventoryTab v-show="activeTab === 'inventory'" />
          <HangarTab v-show="activeTab === 'hangar'" @switch-to-inventory="onSwitchToInventory" />
          <CZTab v-show="activeTab === 'cz'" :active="activeTab === 'cz'" />
          <ProfileTab v-show="activeTab === 'profile'" />
          <PlaceholderTab
            v-show="activeTab !== 'search' && activeTab !== 'details' && activeTab !== 'inventory' && activeTab !== 'hangar' && activeTab !== 'cz' && activeTab !== 'profile'" />
        </div>

        <!-- Keybinds side panel (left of settings) -->
        <Transition name="slide">
          <div v-if="showKeybinds && showSettings"
            class="relative z-50 shrink-0 h-full"
            :style="{ width: '300px', '--panel-w': '300px' }">
            <KeybindsModal @close="showKeybinds = false" />
          </div>
        </Transition>

        <!-- Settings side panel -->
        <Transition name="slide">
          <div v-if="showSettings"
            class="relative z-50 shrink-0 h-full"
            :style="{ width: settingsPanelPx + 'px', '--panel-w': settingsPanelPx + 'px' }">
            <ResizeHandle side="left"
              :default-px="448"
              @resize="onSettingsResize"
              @reset="onSettingsReset" />
            <SettingsPanel class="w-full"
              @close="showSettings = false; showKeybinds = false"
              @open-keybinds="showKeybinds = !showKeybinds"
              @open-update-modal="showUpdateModal = true; showSettings = false" />
          </div>
        </Transition>
      </div>

      <StatusBar :sc-detected="scDetected"
        @toggle-debug="onToggleDebug" />
    </div>

    <!-- Drag ghost -->
    <div
      v-if="isDragActive && ghostLabel"
      class="fixed z-[9999] pointer-events-none px-3 py-1.5 rounded-lg bg-blue-500/20 border border-blue-500/40 text-blue-200 text-xs font-medium whitespace-nowrap backdrop-blur-sm"
      :style="{ left: ghostX + 12 + 'px', top: ghostY + 12 + 'px' }"
    >
      {{ ghostLabel }}
    </div>

    <!-- Update modal -->
    <UpdateModal v-if="showUpdateModal" @close="showUpdateModal = false" />
  </div>
</template>

<style>
.slide-enter-active,
.slide-leave-active {
  transition: margin-right 0.2s ease;
}

.slide-enter-from,
.slide-leave-to {
  margin-right: calc(-1 * var(--panel-w, 384px));
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
  margin-left: calc(-1 * var(--panel-w, 280px));
}

.slide-left-enter-to,
.slide-left-leave-from {
  margin-left: 0;
}
</style>
