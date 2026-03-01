<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import IconClose from "@/components/icons/IconClose.vue";

interface DebugInfo {
  sc_detected: boolean;
  sc_focused: boolean;
  sc_hwnd: number | null;
  sc_window_x: number;
  sc_window_y: number;
  sc_window_w: number;
  sc_window_h: number;
  hotkey: string;
  log_path: string | null;
  overlay_opacity: number;
  uex_api_key_set: boolean;
  uex_cache_entries: number;
  log_watcher_active: boolean;
}

defineEmits<{ (e: "close"): void }>();

const info = ref<DebugInfo | null>(null);
const error = ref<string | null>(null);
const lastUpdated = ref<Date | null>(null);

let intervalId: ReturnType<typeof setInterval> | null = null;

async function refresh() {
  try {
    info.value = await invoke<DebugInfo>("get_debug_info");
    lastUpdated.value = new Date();
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(() => {
  refresh();
  intervalId = setInterval(refresh, 1000);
});

onUnmounted(() => {
  if (intervalId !== null) {
    clearInterval(intervalId);
    intervalId = null;
  }
});

function formatHwnd(hwnd: number | null): string {
  if (hwnd === null) return "—";
  return `0x${(hwnd >>> 0).toString(16).toUpperCase()}`;
}

function formatTime(d: Date | null): string {
  if (!d) return "—";
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}
</script>

<template>
  <div class="flex flex-col h-full bg-[#111318] border-l border-white/10 text-xs font-mono">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2 bg-[#0d0e11] border-b border-white/10 flex-shrink-0">
      <span class="text-white/80 font-sans font-semibold text-sm">Debug</span>
      <div class="flex items-center gap-3">
        <span class="text-white/30">{{ lastUpdated ? formatTime(lastUpdated) : "—" }}</span>
        <button
          @click="$emit('close')"
          class="text-white/40 hover:text-white transition-colors"
          title="Close debug panel"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Error state -->
    <div v-if="error" class="p-4 text-red-400">{{ error }}</div>

    <!-- Loading -->
    <div v-else-if="!info" class="p-4 text-white/40">Loading...</div>

    <!-- Content -->
    <div v-else class="flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Game Window -->
      <section>
        <p class="text-white/30 uppercase tracking-wider mb-2 font-sans text-[10px]">Game Window</p>
        <div class="space-y-1">
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Detected</span>
            <span :class="info.sc_detected ? 'text-green-400' : 'text-red-400'">{{ info.sc_detected ? "yes" : "no" }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Focused</span>
            <span :class="info.sc_focused ? 'text-green-400' : 'text-white/60'">{{ info.sc_focused ? "yes" : "no" }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">HWND</span>
            <span class="text-white/80">{{ formatHwnd(info.sc_hwnd) }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Position</span>
            <span class="text-white/80">{{ info.sc_window_x }}, {{ info.sc_window_y }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Size</span>
            <span class="text-white/80">{{ info.sc_window_w }} × {{ info.sc_window_h }}</span>
          </div>
        </div>
      </section>

      <div class="border-t border-white/10"></div>

      <!-- Settings -->
      <section>
        <p class="text-white/30 uppercase tracking-wider mb-2 font-sans text-[10px]">Settings</p>
        <div class="space-y-1">
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Hotkey</span>
            <span class="text-white/80">{{ info.hotkey }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Opacity</span>
            <span class="text-white/80">{{ (info.overlay_opacity * 100).toFixed(0) }}%</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">API Key</span>
            <span :class="info.uex_api_key_set ? 'text-green-400' : 'text-yellow-400'">{{ info.uex_api_key_set ? "set" : "not set" }}</span>
          </div>
          <div class="flex justify-between gap-4 min-w-0">
            <span class="text-white/50 flex-shrink-0">Log Path</span>
            <span class="text-white/60 truncate text-right" :title="info.log_path ?? 'default'">{{ info.log_path ?? "default" }}</span>
          </div>
        </div>
      </section>

      <div class="border-t border-white/10"></div>

      <!-- Services -->
      <section>
        <p class="text-white/30 uppercase tracking-wider mb-2 font-sans text-[10px]">Services</p>
        <div class="space-y-1">
          <div class="flex justify-between gap-4">
            <span class="text-white/50">Log Watcher</span>
            <span :class="info.log_watcher_active ? 'text-green-400' : 'text-red-400'">{{ info.log_watcher_active ? "active" : "inactive" }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span class="text-white/50">UEX Cache</span>
            <span class="text-white/80">{{ info.uex_cache_entries }} entries</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
