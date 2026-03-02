<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import IconClose from "@/components/icons/IconClose.vue";

// ── Types ──────────────────────────────────────────────────────────────────

interface CollectionDebugInfo {
  key: string;
  display_name: string;
  cached_at: string | null;
  expires_at: string | null;
  ttl_secs: number;
  is_expired: boolean;
  is_refreshing: boolean;
  entry_count: number;
}

interface LastUserAction {
  timestamp: string;
  kind: string;
  entity_id: string;
  collection: string;
  source: "fresh" | "stale" | "missing";
  row_count: number;
}

interface FetchEvent {
  timestamp: string;
  collection: string;
  endpoint: string;
  row_count: number;
  duration_ms: number;
  triggered_by: "startup" | "timer" | "manual";
  ok: boolean;
  error: string | null;
}

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
  esc_closes_overlay: boolean;
  reset_on_open: boolean;
  max_search_results: number;
  cache_ttl_prices_secs: number;
  cache_ttl_catalog_secs: number;
  log_watcher_active: boolean;
  hotkey_registered: boolean;
  refreshing_collections: string[];
  cache_total_keys: number;
  cache_collections: CollectionDebugInfo[];
  last_bg_check_at: string | null;
  next_bg_check_in_secs: number;
  last_bg_check_ago_secs: number | null;
  last_user_action: LastUserAction | null;
  fetch_log: FetchEvent[];
}

// ── State ──────────────────────────────────────────────────────────────────

defineEmits<{ (e: "close"): void }>();

const info = ref<DebugInfo | null>(null);
const error = ref<string | null>(null);
const lastUpdated = ref<Date | null>(null);
const activeSection = ref<"system" | "cache" | "activity">("system");

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

// ── Formatters ─────────────────────────────────────────────────────────────

function formatHwnd(hwnd: number | null): string {
  if (hwnd === null) return "—";
  return `0x${(hwnd >>> 0).toString(16).toUpperCase()}`;
}

function formatTime(d: Date | null): string {
  if (!d) return "—";
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

function relTime(iso: string | null): string {
  if (!iso) return "—";
  const t = new Date(iso).getTime();
  if (isNaN(t)) return "—";
  const ago = Math.floor((Date.now() - t) / 1000);
  if (ago < 5) return "just now";
  if (ago < 60) return `${ago}s ago`;
  if (ago < 3600) return `${Math.floor(ago / 60)}m ago`;
  return `${Math.floor(ago / 3600)}h ago`;
}

function countdown(secs: number): string {
  if (secs <= 0) return "now";
  if (secs < 60) return `${secs}s`;
  return `${Math.floor(secs / 60)}m ${secs % 60}s`;
}

function ttlRemaining(info: CollectionDebugInfo): string {
  if (!info.cached_at) return "—";
  const cachedMs = new Date(info.cached_at).getTime();
  if (isNaN(cachedMs)) return "—";
  const expiresMs = cachedMs + info.ttl_secs * 1000;
  const remaining = Math.floor((expiresMs - Date.now()) / 1000);
  if (remaining <= 0) return "expired";
  if (remaining < 60) return `${remaining}s`;
  if (remaining < 3600) return `${Math.floor(remaining / 60)}m`;
  return `${Math.floor(remaining / 3600)}h ${Math.floor((remaining % 3600) / 60)}m`;
}

function ttlRemainingColor(info: CollectionDebugInfo): string {
  if (!info.cached_at) return "text-white/30";
  const cachedMs = new Date(info.cached_at).getTime();
  const expiresMs = cachedMs + info.ttl_secs * 1000;
  const pct = (expiresMs - Date.now()) / (info.ttl_secs * 1000);
  if (pct <= 0) return "text-red-400";
  if (pct < 0.2) return "text-yellow-400";
  return "text-green-400";
}

function triggerBadge(by: string): string {
  if (by === "startup") return "bg-blue-500/20 text-blue-300";
  if (by === "timer") return "bg-purple-500/20 text-purple-300";
  return "bg-orange-500/20 text-orange-300";
}

function sourceBadge(source: string): string {
  if (source === "fresh") return "text-green-400";
  if (source === "stale") return "text-yellow-400";
  return "text-red-400";
}

function fmtMs(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}
</script>

<template>
  <div class="w-full flex flex-col bg-[#1a1d24] border border-white/10 rounded-xl overflow-hidden text-xs font-mono">
    <!-- Header -->
    <div class="px-3 py-2 border-b border-white/10 flex items-center justify-between flex-shrink-0">
      <span class="text-xs font-sans font-semibold text-white/50 uppercase tracking-widest">Debug</span>
      <div class="flex items-center gap-2">
        <span class="text-white/20 text-xs">{{ lastUpdated ? formatTime(lastUpdated) : "—" }}</span>
        <button @click="$emit('close')" class="text-white/30 hover:text-white/70 transition-colors">
          <IconClose class="w-3 h-3" />
        </button>
      </div>
    </div>

    <!-- Section tabs -->
    <div class="flex border-b border-white/10 flex-shrink-0">
      <button v-for="s in (['system', 'cache', 'activity'] as const)" :key="s"
        @click="activeSection = s"
        class="flex-1 py-1 text-xs uppercase tracking-wider transition-colors font-sans"
        :class="activeSection === s ? 'text-blue-400 border-b border-blue-400' : 'text-white/25 hover:text-white/50'">
        {{ s }}
      </button>
    </div>

    <!-- Error -->
    <div v-if="error" class="p-2 text-red-400 text-xs">{{ error }}</div>
    <div v-else-if="!info" class="p-2 text-white/30 text-xs">Loading...</div>

    <!-- ══ SYSTEM ══════════════════════════════════════════════════════════ -->
    <div v-else-if="activeSection === 'system'" class="flex-1 overflow-y-auto p-2 space-y-2.5">

      <!-- Game Window -->
      <div>
        <p class="text-white/25 uppercase tracking-widest text-xs mb-1 font-sans">Game</p>
        <div class="space-y-0.5">
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Detected</span>
            <span :class="info.sc_detected ? 'text-green-400' : 'text-red-400'">{{ info.sc_detected ? "yes" : "no" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Focused</span>
            <span :class="info.sc_focused ? 'text-green-400' : 'text-white/30'">{{ info.sc_focused ? "yes" : "no" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">HWND</span>
            <span class="text-white/55">{{ formatHwnd(info.sc_hwnd) }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Pos</span>
            <span class="text-white/55">{{ info.sc_window_x }}, {{ info.sc_window_y }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Size</span>
            <span class="text-white/55">{{ info.sc_window_w }}×{{ info.sc_window_h }}</span>
          </div>
        </div>
      </div>

      <div class="border-t border-white/[0.06]"></div>

      <!-- Services -->
      <div>
        <p class="text-white/25 uppercase tracking-widest text-xs mb-1 font-sans">Services</p>
        <div class="space-y-0.5">
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Log watcher</span>
            <span :class="info.log_watcher_active ? 'text-green-400' : 'text-red-400'">{{ info.log_watcher_active ? "on" : "off" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Hotkey</span>
            <span :class="info.hotkey_registered ? 'text-green-400' : 'text-red-400'">{{ info.hotkey_registered ? "on" : "off" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Cache keys</span>
            <span class="text-white/55">{{ info.cache_total_keys }}</span>
          </div>
          <div v-if="info.refreshing_collections.length > 0" class="flex justify-between gap-1">
            <span class="text-white/40">Refreshing</span>
            <span class="text-yellow-400 truncate text-right" :title="info.refreshing_collections.join(', ')">{{ info.refreshing_collections.length }}</span>
          </div>
        </div>
      </div>

      <div class="border-t border-white/[0.06]"></div>

      <!-- Background Timer -->
      <div>
        <p class="text-white/25 uppercase tracking-widest text-xs mb-1 font-sans">BG Timer (30s)</p>
        <div class="space-y-0.5">
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Last</span>
            <span class="text-white/55">{{ relTime(info.last_bg_check_at) }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Next</span>
            <span class="text-white/55">{{ countdown(info.next_bg_check_in_secs) }}</span>
          </div>
          <div class="h-1 bg-white/10 rounded-full overflow-hidden mt-1">
            <div class="h-full bg-blue-500/50 rounded-full transition-all duration-1000"
              :style="{ width: (100 - (info.next_bg_check_in_secs / 30) * 100) + '%' }"></div>
          </div>
        </div>
      </div>

      <div class="border-t border-white/[0.06]"></div>

      <!-- Settings -->
      <div>
        <p class="text-white/25 uppercase tracking-widest text-xs mb-1 font-sans">Settings</p>
        <div class="space-y-0.5">
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Hotkey</span>
            <span class="text-white/60 truncate text-right" :title="info.hotkey">{{ info.hotkey }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Opacity</span>
            <span class="text-white/55">{{ (info.overlay_opacity * 100).toFixed(0) }}%</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">API key</span>
            <span :class="info.uex_api_key_set ? 'text-green-400' : 'text-yellow-400'">{{ info.uex_api_key_set ? "set" : "unset" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">ESC close</span>
            <span class="text-white/55">{{ info.esc_closes_overlay ? "yes" : "no" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Reset open</span>
            <span class="text-white/55">{{ info.reset_on_open ? "yes" : "no" }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Max results</span>
            <span class="text-white/55">{{ info.max_search_results }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">TTL prices</span>
            <span class="text-white/55">{{ info.cache_ttl_prices_secs }}s</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">TTL catalog</span>
            <span class="text-white/55">{{ info.cache_ttl_catalog_secs }}s</span>
          </div>
        </div>
      </div>

      <!-- Last User Action -->
      <div v-if="info.last_user_action">
        <div class="border-t border-white/[0.06] mb-2.5"></div>
        <p class="text-white/25 uppercase tracking-widest text-xs mb-1 font-sans">Last Lookup</p>
        <div class="bg-white/[0.04] rounded-lg p-1.5 space-y-0.5">
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Kind</span>
            <span class="text-white/60">{{ info.last_user_action.kind }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">ID</span>
            <span class="text-white/55">#{{ info.last_user_action.entity_id }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Source</span>
            <span :class="sourceBadge(info.last_user_action.source)">{{ info.last_user_action.source }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">Rows</span>
            <span class="text-white/55">{{ info.last_user_action.row_count }}</span>
          </div>
          <div class="flex justify-between gap-1">
            <span class="text-white/40">When</span>
            <span class="text-white/30">{{ relTime(info.last_user_action.timestamp) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- ══ CACHE ═══════════════════════════════════════════════════════════ -->
    <div v-else-if="activeSection === 'cache'" class="flex-1 overflow-y-auto">
      <div v-for="c in info!.cache_collections" :key="c.key"
        class="px-2 py-1.5 border-b border-white/[0.05] hover:bg-white/[0.03]">
        <!-- Name + status dot -->
        <div class="flex items-center gap-1.5 mb-0.5">
          <span v-if="c.is_refreshing" class="w-1.5 h-1.5 rounded-full bg-yellow-400 animate-pulse shrink-0"></span>
          <span v-else-if="c.is_expired" class="w-1.5 h-1.5 rounded-full bg-red-500/60 shrink-0"></span>
          <span v-else class="w-1.5 h-1.5 rounded-full bg-green-500/60 shrink-0"></span>
          <span class="text-white/65 truncate font-sans text-xs">{{ c.display_name }}</span>
        </div>
        <!-- Stats row -->
        <div class="flex items-center justify-between text-xs pl-3">
          <span class="text-white/30">{{ c.entry_count > 0 ? c.entry_count.toLocaleString() + " rows" : "empty" }}</span>
          <span :class="ttlRemainingColor(c)">{{ ttlRemaining(c) }}</span>
        </div>
        <div class="text-white/20 text-xs pl-3">{{ relTime(c.cached_at) }}</div>
      </div>
    </div>

    <!-- ══ ACTIVITY ════════════════════════════════════════════════════════ -->
    <div v-else-if="activeSection === 'activity'" class="flex-1 overflow-y-auto">
      <div v-if="info!.fetch_log.length === 0" class="p-3 text-center text-white/25 text-xs">
        No events yet...
      </div>
      <div v-else class="divide-y divide-white/[0.04]">
        <div v-for="(ev, idx) in info!.fetch_log" :key="idx"
          class="px-2 py-1.5 hover:bg-white/[0.03]">
          <div class="flex items-center gap-1.5 mb-0.5">
            <span class="px-1 rounded text-xs font-sans" :class="triggerBadge(ev.triggered_by)">{{ ev.triggered_by }}</span>
            <span v-if="!ev.ok" class="text-red-400 text-xs">ERR</span>
            <span class="text-white/25 text-xs ml-auto">{{ fmtMs(ev.duration_ms) }}</span>
          </div>
          <div class="text-white/55 truncate font-sans text-xs">{{ ev.collection }}</div>
          <div class="text-white/25 truncate text-xs">{{ ev.endpoint }}</div>
          <div class="flex items-center justify-between text-xs mt-0.5">
            <span v-if="ev.error" class="text-red-400/70 truncate">{{ ev.error }}</span>
            <span v-else class="text-white/20">{{ ev.row_count > 0 ? ev.row_count.toLocaleString() + " rows" : "" }}</span>
            <span class="text-white/20 shrink-0">{{ relTime(ev.timestamp) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

