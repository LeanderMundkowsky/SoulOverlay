<script setup lang="ts">
import IconSearch from "@/components/icons/IconSearch.vue";
import IconClose from "@/components/icons/IconClose.vue";
import IconHome from "@/components/icons/IconHome.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import IconSun from "@/components/icons/IconSun.vue";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";
import IconUser from "@/components/icons/IconUser.vue";
import IconHeart from "@/components/icons/IconHeart.vue";
import IconEye from "@/components/icons/IconEye.vue";
import IconShield from "@/components/icons/IconShield.vue";
import IconDollarSign from "@/components/icons/IconDollarSign.vue";

interface Tab {
  id: string;
  label: string;
  shortcut: string | null;
  disabled: boolean;
  action: "switch" | "close" | "toggle-settings" | "toggle-debug";
}

defineProps<{
  activeTab: string;
  showFavorites: boolean;
  showWatchlist: boolean;
  isLoggedIn: boolean;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", tab: string): void;
  (e: "close"): void;
  (e: "toggle-settings"): void;
  (e: "toggle-debug"): void;
  (e: "toggle-favorites"): void;
  (e: "toggle-watchlist"): void;
  (e: "open-auth-modal"): void;
}>();

const tabs: Tab[] = [
  { id: "search",    label: "SEARCH",    shortcut: "F3",  disabled: false, action: "switch" },
  { id: "details",   label: "DETAILS",   shortcut: null,  disabled: false, action: "switch" },
  { id: "inventory", label: "INVENTORY", shortcut: null,  disabled: false, action: "switch" },
  { id: "hangar",    label: "HANGAR",    shortcut: "F8",  disabled: false, action: "switch" },
  { id: "wikelo",    label: "WIKELO",    shortcut: null,  disabled: false, action: "switch" },
  { id: "cz",        label: "CZ",        shortcut: null,  disabled: false, action: "switch" },
  { id: "settings",  label: "SETTINGS",  shortcut: "F12", disabled: false, action: "toggle-settings" },
  { id: "close",     label: "CLOSE",     shortcut: null,  disabled: false, action: "close" },
];

function handleTab(tab: Tab) {
  if (tab.disabled) return;
  if (tab.action === "close") {
    emit("close");
  } else if (tab.action === "toggle-settings") {
    emit("toggle-settings");
  } else if (tab.action === "toggle-debug") {
    emit("toggle-debug");
  } else {
    emit("update:activeTab", tab.id);
  }
}
</script>

<template>
  <div class="flex-shrink-0 flex items-stretch bg-[#111318] border-b border-white/10 select-none">
    <!-- Favorites toggle (far left) -->
    <div
      class="flex items-center pl-5 gap-3 flex-shrink-0"
    >
      <div
        class="cursor-pointer"
        @click="emit('toggle-favorites')"
        title="Toggle favorites panel"
      >
        <IconHeart
          :filled="showFavorites"
          class="w-4 h-4 transition-colors"
          :class="showFavorites ? 'text-red-400' : 'text-white/30 hover:text-red-400'"
        />
      </div>
      <div
        class="cursor-pointer"
        @click="emit('toggle-watchlist')"
        title="Toggle watch list panel"
      >
        <IconEye
          class="w-4 h-4 transition-colors"
          :class="showWatchlist ? 'text-blue-400' : 'text-white/30 hover:text-blue-400'"
        />
      </div>
    </div>

    <!-- Centered tab list -->
    <div class="flex items-stretch justify-center w-full">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="relative flex items-center justify-center px-7 py-3 cursor-pointer group min-w-[80px]"
        :class="[
          tab.disabled
            ? 'cursor-not-allowed opacity-30'
            : tab.id === 'close'
              ? 'hover:bg-red-500/10'
              : 'hover:bg-white/5',
        ]"
        @click="handleTab(tab)"
      >
        <!-- Tab icon + label + shortcut -->
        <div
          class="flex items-center gap-2"
          :class="[
            tab.id === 'close'
              ? 'text-red-400'
              : activeTab === tab.id
                ? 'text-white'
                : 'text-white/50 group-hover:text-white/80',
          ]"
        >
          <IconSearch    v-if="tab.id === 'search'"    class="w-4 h-4 flex-shrink-0" />
          <IconInfoCircle v-else-if="tab.id === 'details'" class="w-4 h-4 flex-shrink-0" />
          <IconClose     v-else-if="tab.id === 'close'"     class="w-4 h-4 flex-shrink-0" />
          <IconHome      v-else-if="tab.id === 'hangar'"    class="w-4 h-4 flex-shrink-0" />
          <IconPackage   v-else-if="tab.id === 'inventory'" class="w-4 h-4 flex-shrink-0" />
          <IconShield    v-else-if="tab.id === 'cz'"        class="w-4 h-4 flex-shrink-0" />
          <IconDollarSign v-else-if="tab.id === 'wikelo'"   class="w-4 h-4 flex-shrink-0" />
          <IconSun       v-else-if="tab.id === 'settings'"  class="w-4 h-4 flex-shrink-0" />

          <span class="text-xs font-semibold tracking-widest">{{ tab.label }}</span>

          <span
            v-if="tab.shortcut"
            class="px-1.5 py-0.5 rounded text-[10px] font-bold leading-none bg-teal-500 text-white"
          >
            {{ tab.shortcut }}
          </span>
        </div>

        <!-- Active underline -->
        <div
          class="absolute bottom-0 left-0 right-0 h-0.5 transition-colors duration-150"
          :class="
            !tab.disabled && tab.action === 'switch' && activeTab === tab.id
              ? 'bg-blue-500'
              : 'bg-transparent'
          "
        ></div>
      </div>
    </div>

    <!-- User profile / Login button (far right) -->
    <div class="flex items-center pr-5 flex-shrink-0">
      <!-- Login button when not authenticated -->
      <button
        v-if="!isLoggedIn"
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold tracking-wider text-white/60 hover:text-white bg-white/5 hover:bg-white/10 border border-white/10 hover:border-white/20 rounded-lg transition-colors"
        title="Sign in to your SoulOverlay account"
        @click="emit('open-auth-modal')"
      >
        <IconUser class="w-3.5 h-3.5 flex-shrink-0" />
        LOGIN
      </button>
      <!-- User icon when authenticated -->
      <div
        v-else
        class="cursor-pointer"
        title="View profile"
        @click="emit('update:activeTab', 'profile')"
      >
        <div
          class="relative w-6 h-6 rounded-full overflow-hidden flex items-center justify-center transition-all"
          :class="activeTab === 'profile' ? 'ring-2 ring-blue-500' : 'hover:ring-2 hover:ring-white/30'"
        >
          <IconUser
            class="w-4 h-4"
            :class="activeTab === 'profile' ? 'text-white' : 'text-white/50 hover:text-white/80'"
          />
        </div>
      </div>
    </div>
  </div>
</template>
