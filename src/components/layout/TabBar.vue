<script setup lang="ts">
import IconSearch from "@/components/icons/IconSearch.vue";
import IconClose from "@/components/icons/IconClose.vue";
import IconMonitor from "@/components/icons/IconMonitor.vue";
import IconFlag from "@/components/icons/IconFlag.vue";
import IconDollarSign from "@/components/icons/IconDollarSign.vue";
import IconHome from "@/components/icons/IconHome.vue";
import IconUsers from "@/components/icons/IconUsers.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import IconSun from "@/components/icons/IconSun.vue";
import IconInfoCircle from "@/components/icons/IconInfoCircle.vue";
import IconUser from "@/components/icons/IconUser.vue";
import IconHeart from "@/components/icons/IconHeart.vue";
import IconEye from "@/components/icons/IconEye.vue";

interface Tab {
  id: string;
  label: string;
  shortcut: string | null;
  disabled: boolean;
  action: "switch" | "close" | "toggle-settings" | "toggle-debug";
}

defineProps<{
  activeTab: string;
  scDetected: boolean;
  showFavorites: boolean;
  showWatchlist: boolean;
  isAuthenticated: boolean;
  userAvatarUrl: string | null;
}>();

const emit = defineEmits<{
  (e: "update:activeTab", tab: string): void;
  (e: "close"): void;
  (e: "toggle-settings"): void;
  (e: "toggle-debug"): void;
  (e: "toggle-favorites"): void;
  (e: "toggle-watchlist"): void;
}>();

const tabs: Tab[] = [
  { id: "search",    label: "SEARCH",    shortcut: "F3",  disabled: false, action: "switch" },
  { id: "details",   label: "DETAILS",   shortcut: null,  disabled: false, action: "switch" },
  { id: "close",     label: "CLOSE",     shortcut: null,  disabled: false, action: "close" },
  { id: "trade",     label: "TRADE",     shortcut: "F4",  disabled: true,  action: "switch" },
  { id: "mining",    label: "MINING",    shortcut: "F5",  disabled: true,  action: "switch" },
  { id: "market",    label: "MARKET",    shortcut: null,  disabled: true,  action: "switch" },
  { id: "hangar",    label: "HANGAR",    shortcut: "F8",  disabled: false, action: "switch" },
  { id: "org",       label: "ORG",       shortcut: null,  disabled: true,  action: "switch" },
  { id: "inventory", label: "INVENTORY", shortcut: null,  disabled: false, action: "switch" },
  { id: "settings",  label: "SETTINGS",  shortcut: "F12", disabled: false, action: "toggle-settings" },
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
        class="relative flex flex-col items-center justify-center px-7 py-0 cursor-pointer group min-w-[80px]"
        :class="[
          tab.disabled
            ? 'cursor-not-allowed opacity-30'
            : tab.id === 'close'
              ? 'hover:bg-red-500/10'
              : 'hover:bg-white/5',
        ]"
        @click="handleTab(tab)"
      >
        <!-- Tab icon + label -->
        <div
          class="flex items-center gap-2 pt-3 pb-1"
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
          <IconMonitor   v-else-if="tab.id === 'trade'"     class="w-4 h-4 flex-shrink-0" />
          <IconFlag      v-else-if="tab.id === 'mining'"    class="w-4 h-4 flex-shrink-0" />
          <IconDollarSign v-else-if="tab.id === 'market'"   class="w-4 h-4 flex-shrink-0" />
          <IconHome      v-else-if="tab.id === 'hangar'"    class="w-4 h-4 flex-shrink-0" />
          <IconUsers     v-else-if="tab.id === 'org'"       class="w-4 h-4 flex-shrink-0" />
          <IconPackage   v-else-if="tab.id === 'inventory'" class="w-4 h-4 flex-shrink-0" />
          <IconSun       v-else-if="tab.id === 'settings'"  class="w-4 h-4 flex-shrink-0" />

          <span class="text-xs font-semibold tracking-widest">{{ tab.label }}</span>
        </div>

        <!-- Shortcut badge -->
        <div
          v-if="tab.shortcut"
          class="mb-1.5 px-1.5 py-0.5 rounded text-xs font-bold tracking-wide leading-none"
          :class="tab.disabled ? 'bg-white/10 text-white/40' : 'bg-teal-500 text-white'"
        >
          {{ tab.shortcut }}
        </div>
        <div v-else class="mb-1.5 h-[18px]"></div>

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

    <!-- SC status indicator -->
    <div class="flex items-center pr-3 flex-shrink-0">
      <div
        class="flex items-center gap-1.5 text-xs"
        :class="scDetected ? 'text-green-400' : 'text-yellow-400/70'"
      >
        <span
          class="w-2 h-2 rounded-full flex-shrink-0"
          :class="scDetected ? 'bg-green-400' : 'bg-yellow-400/70'"
        ></span>
        <span class="whitespace-nowrap">{{ scDetected ? "SC" : "No SC" }}</span>
      </div>
    </div>

    <!-- User profile icon -->
    <div
      class="flex items-center pr-5 flex-shrink-0 cursor-pointer"
      :class="isAuthenticated ? 'opacity-100' : 'opacity-30 cursor-not-allowed'"
      :title="isAuthenticated ? 'View profile' : 'Set API key & secret key in Settings to view profile'"
      @click="isAuthenticated && emit('update:activeTab', 'profile')"
    >
      <div
        class="relative w-6 h-6 rounded-full overflow-hidden flex items-center justify-center transition-all"
        :class="[
          activeTab === 'profile' ? 'ring-2 ring-blue-500' : '',
          isAuthenticated ? 'hover:ring-2 hover:ring-white/30' : '',
        ]"
      >
        <img
          v-if="isAuthenticated && userAvatarUrl"
          :src="userAvatarUrl"
          alt="Profile"
          class="w-full h-full object-cover"
          @error="($event.target as HTMLImageElement).style.display = 'none'"
        />
        <IconUser
          v-else
          class="w-4 h-4"
          :class="isAuthenticated
            ? (activeTab === 'profile' ? 'text-white' : 'text-white/50 hover:text-white/80')
            : 'text-white/30'"
        />
      </div>
    </div>
  </div>
</template>
