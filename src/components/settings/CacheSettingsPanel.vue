<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import IconRefresh from "@/components/icons/IconRefresh.vue";
import IconEdit from "@/components/icons/IconEdit.vue";
import IconClose from "@/components/icons/IconClose.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import { useCache } from "@/composables/useCache";
import { useSettingsStore } from "@/stores/settings";
import type { CollectionStatus } from "@/composables/useCache";

const cache = useCache();
const settingsStore = useSettingsStore();

// Ticks every second so formatAge re-evaluates reactively in the template.
const now = ref(Date.now());
let ticker: ReturnType<typeof setInterval> | null = null;
let unlistenCacheUpdated: (() => void) | null = null;

// Inline TTL editing state
const editingCollection = ref<string | null>(null);
const editTtlText = ref("");

onMounted(async () => {
  cache.fetchStatus();
  ticker = setInterval(() => { now.value = Date.now(); }, 1000);
  unlistenCacheUpdated = await listen("cache-updated", () => {
    cache.fetchStatus();
  });
});

onUnmounted(() => {
  if (ticker !== null) clearInterval(ticker);
  if (unlistenCacheUpdated !== null) unlistenCacheUpdated();
});

function formatAge(status: CollectionStatus): string {
  if (!status.cached_at) return "never";
  const diffSec = Math.floor((now.value - new Date(status.cached_at).getTime()) / 1000);

  if (diffSec < 60) return `${diffSec}s ago`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  return `${Math.floor(diffHr / 24)}d ago`;
}

function formatTtl(secs: number): string {
  if (secs >= 86400 && secs % 86400 === 0) return `${secs / 86400}d`;
  if (secs >= 3600 && secs % 3600 === 0) return `${secs / 3600}h`;
  if (secs >= 60 && secs % 60 === 0) return `${secs / 60}m`;
  return `${secs}s`;
}

/** Parse a duration string like "15m", "2h", "3d" or plain seconds into seconds. */
function parseTtl(input: string): number | null {
  const trimmed = input.trim().toLowerCase();
  if (!trimmed) return null;
  const match = trimmed.match(/^(\d+(?:\.\d+)?)\s*(m|h|d|s)?$/);
  if (!match) return null;
  const value = parseFloat(match[1]);
  if (isNaN(value) || value <= 0) return null;
  const unit = match[2] ?? "s";
  const multipliers: Record<string, number> = { s: 1, m: 60, h: 3600, d: 86400 };
  return Math.floor(value * multipliers[unit]);
}

function startEditing(col: CollectionStatus) {
  editingCollection.value = col.collection;
  editTtlText.value = formatTtl(col.ttl_secs);
}

function cancelEditing() {
  editingCollection.value = null;
}

async function saveTtl(collection: string) {
  const parsed = parseTtl(editTtlText.value);
  if (parsed === null) return;
  const clamped = Math.max(60, parsed);
  const updated = { ...settingsStore.settings };
  updated.cache_ttls = { ...updated.cache_ttls, [collection]: clamped };
  try {
    await settingsStore.saveSettings(updated);
    await cache.fetchStatus();
  } catch (e) {
    cache.error.value = String(e);
  }
  editingCollection.value = null;
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <label class="block text-white/60 text-xs font-medium uppercase tracking-wider">
        Data Cache
      </label>
      <button
        @click="cache.refreshAll()"
        :disabled="cache.refreshing.value"
        class="flex items-center gap-1.5 text-xs text-blue-400 hover:text-blue-300 disabled:text-white/20 transition-colors"
      >
        <IconRefresh
          class="w-3.5 h-3.5"
          :class="{ 'animate-spin': cache.refreshing.value }"
        />
        Refresh All
      </button>
    </div>

    <AlertBanner v-if="cache.error.value" variant="error" :message="cache.error.value" />

    <div class="space-y-1.5">
      <div
        v-for="col in cache.collections.value"
        :key="col.collection"
        class="bg-white/5 rounded-lg px-3 py-2"
      >
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-white/80 text-sm">{{ col.display_name }}</span>
              <span
                class="text-xs px-1.5 py-0.5 rounded-full"
                :class="col.is_expired
                  ? 'bg-yellow-500/20 text-yellow-400'
                  : 'bg-green-500/20 text-green-400'"
              >
                {{ col.is_expired ? "stale" : "fresh" }}
              </span>
            </div>
            <div class="text-white/30 text-xs mt-0.5">
              {{ col.entry_count }} entries
              <span class="mx-1">&middot;</span>
              {{ formatAge(col) }}
              <span class="mx-1">&middot;</span>
              TTL {{ formatTtl(col.ttl_secs) }}
            </div>
          </div>
          <div class="flex items-center gap-1 ml-2">
            <button
              @click="startEditing(col)"
              class="text-white/30 hover:text-white disabled:text-white/10 transition-colors p-1"
              :title="`Edit TTL for ${col.display_name}`"
            >
              <IconEdit class="w-3.5 h-3.5" />
            </button>
            <button
              @click="cache.refreshCollection(col.collection)"
              :disabled="cache.refreshing.value"
              class="text-white/30 hover:text-white disabled:text-white/10 transition-colors p-1"
              :title="`Refresh ${col.display_name}`"
            >
              <IconRefresh
                class="w-3.5 h-3.5"
                :class="{ 'animate-spin': cache.refreshing.value }"
              />
            </button>
          </div>
        </div>

        <!-- Inline TTL editor -->
        <div
          v-if="editingCollection === col.collection"
          class="flex items-center gap-2 mt-2 pt-2 border-t border-white/5"
        >
          <input
            v-model="editTtlText"
            type="text"
            placeholder="e.g. 600, 15m, 2h, 3d"
            class="flex-1 bg-white/5 border border-white/10 rounded px-2 py-1 text-white text-xs focus:outline-none focus:border-blue-500/50 transition-colors w-20"
            @keydown.enter="saveTtl(col.collection)"
            @keydown.escape="cancelEditing()"
          />
          <button
            @click="saveTtl(col.collection)"
            :disabled="parseTtl(editTtlText) === null"
            class="text-xs text-blue-400 hover:text-blue-300 disabled:text-white/20 transition-colors px-2 py-1"
          >
            Save
          </button>
          <button
            @click="cancelEditing()"
            class="text-white/30 hover:text-white transition-colors p-1"
          >
            <IconClose class="w-3 h-3" />
          </button>
        </div>
      </div>
    </div>

    <p class="text-white/20 text-xs">
      Expired collections are automatically refreshed in the background every 30s.
    </p>
  </div>
</template>
