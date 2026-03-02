<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import IconRefresh from "@/components/icons/IconRefresh.vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import { useCache } from "@/composables/useCache";
import type { CollectionStatus } from "@/composables/useCache";

const cache = useCache();

// Ticks every second so formatAge re-evaluates reactively in the template.
const now = ref(Date.now());
let ticker: ReturnType<typeof setInterval> | null = null;
let unlistenCacheUpdated: (() => void) | null = null;

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
  if (secs < 60) return `${secs}s`;
  const min = Math.floor(secs / 60);
  if (min < 60) return `${min}m`;
  const hr = Math.floor(min / 60);
  return `${hr}h`;
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
        class="flex items-center justify-between bg-white/5 rounded-lg px-3 py-2"
      >
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-white/80 text-sm">{{ col.display_name }}</span>
            <span
              class="text-[10px] px-1.5 py-0.5 rounded-full"
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
        <button
          @click="cache.refreshCollection(col.collection)"
          :disabled="cache.refreshing.value"
          class="text-white/30 hover:text-white disabled:text-white/10 transition-colors ml-2 p-1"
          :title="`Refresh ${col.display_name}`"
        >
          <IconRefresh
            class="w-3.5 h-3.5"
            :class="{ 'animate-spin': cache.refreshing.value }"
          />
        </button>
      </div>
    </div>

    <p class="text-white/20 text-xs">
      Expired collections are automatically refreshed in the background every 30s.
    </p>
  </div>
</template>
