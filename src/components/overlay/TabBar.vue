<script setup lang="ts">
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
}>();

const emit = defineEmits<{
  (e: "update:activeTab", tab: string): void;
  (e: "close"): void;
  (e: "toggle-settings"): void;
  (e: "toggle-debug"): void;
}>();

const tabs: Tab[] = [
  { id: "search",   label: "SEARCH",   shortcut: "F3",  disabled: false, action: "switch" },
  { id: "close",    label: "CLOSE",    shortcut: null,  disabled: false, action: "close" },
  { id: "trade",    label: "TRADE",    shortcut: "F4",  disabled: true,  action: "switch" },
  { id: "mining",   label: "MINING",   shortcut: "F5",  disabled: true,  action: "switch" },
  { id: "market",   label: "MARKET",   shortcut: null,  disabled: true,  action: "switch" },
  { id: "hangar",   label: "HANGAR",   shortcut: "F8",  disabled: true,  action: "switch" },
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
        <!-- Tab icon -->
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
          <!-- Search icon -->
          <svg v-if="tab.id === 'search'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <!-- Close / X icon -->
          <svg v-else-if="tab.id === 'close'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
          <!-- Trade icon -->
          <svg v-else-if="tab.id === 'trade'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
            <line x1="8" y1="21" x2="16" y2="21"></line>
            <line x1="12" y1="17" x2="12" y2="21"></line>
          </svg>
          <!-- Mining icon -->
          <svg v-else-if="tab.id === 'mining'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 6l-1-2H5v17h2v-7h5l1 2h7V6h-6z"></path>
          </svg>
          <!-- Market icon -->
          <svg v-else-if="tab.id === 'market'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="1" x2="12" y2="23"></line>
            <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
          </svg>
          <!-- Hangar icon -->
          <svg v-else-if="tab.id === 'hangar'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
            <polyline points="9 22 9 12 15 12 15 22"></polyline>
          </svg>
          <!-- Org icon -->
          <svg v-else-if="tab.id === 'org'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
          </svg>
          <!-- Inventory icon -->
          <svg v-else-if="tab.id === 'inventory'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
            <line x1="12" y1="22.08" x2="12" y2="12"></line>
          </svg>
          <!-- Settings icon -->
          <svg v-else-if="tab.id === 'settings'" class="w-4 h-4 flex-shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"></path>
          </svg>

          <span class="text-xs font-semibold tracking-widest">{{ tab.label }}</span>
        </div>

        <!-- Shortcut badge -->
        <div
          v-if="tab.shortcut"
          class="mb-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold tracking-wide leading-none"
          :class="
            tab.disabled
              ? 'bg-white/10 text-white/40'
              : 'bg-teal-500 text-white'
          "
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

    <!-- SC status indicator pinned to the right -->
    <div class="flex items-center pr-5 flex-shrink-0">
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
  </div>
</template>
