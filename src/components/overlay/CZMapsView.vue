<script setup lang="ts">
import { ref, onMounted } from "vue";
import { commands } from "@/bindings";
import type { CzMap } from "@/bindings";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";

const maps = ref<CzMap[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const fullscreenMap = ref<string | null>(null);

function toggleFullscreen(name: string) {
  fullscreenMap.value = fullscreenMap.value === name ? null : name;
}

async function fetchMaps() {
  loading.value = true;
  error.value = null;
  try {
    const result = await commands.czGetMaps();
    if (result.status === "ok" && result.data.ok && result.data.data) {
      maps.value = result.data.data;
    } else {
      error.value = result.status === "ok" ? (result.data.error ?? "Unknown error") : result.error;
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(fetchMaps);
</script>

<template>
  <div class="h-full overflow-y-auto px-4 py-3 select-none">
    <h2 class="text-xs font-semibold text-white/50 tracking-widest uppercase mb-3">
      Contested Zone Maps
    </h2>

    <!-- Loading -->
    <div v-if="loading" class="flex flex-col items-center py-12">
      <LoadingSpinner />
      <p class="text-white/40 text-xs mt-2">Loading maps...</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-center py-8">
      <p class="text-red-400 text-sm">{{ error }}</p>
      <button
        class="mt-2 px-3 py-1 text-xs bg-white/10 hover:bg-white/20 rounded transition-colors"
        @click="fetchMaps"
      >
        Retry
      </button>
    </div>

    <!-- Map grid (2 columns) -->
    <template v-else>
      <div class="grid grid-cols-2 gap-3">
        <div
          v-for="map in maps"
          :key="map.name"
          class="bg-[#1a1d24] border border-white/10 rounded-lg overflow-hidden cursor-pointer hover:border-white/20 transition-colors"
          @click="toggleFullscreen(map.name)"
        >
          <div class="px-3 py-2 border-b border-white/5">
            <h3 class="text-[10px] font-semibold text-white/70 tracking-wider">{{ map.name }}</h3>
          </div>
          <img
            :src="map.image_url"
            :alt="map.name"
            class="w-full object-contain"
            loading="lazy"
          />
        </div>
      </div>

      <!-- Attribution -->
      <div class="text-center text-[10px] text-white/20 py-3">
        Maps by Terada &amp; u/Kerast · via
        <a href="https://contestedzonetimers.com/contested-zone-maps" target="_blank" class="underline hover:text-white/40">
          contestedzonetimers.com
        </a>
      </div>
    </template>

    <!-- Fullscreen overlay -->
    <Transition name="map-fs">
      <div
        v-if="fullscreenMap"
        class="fixed inset-0 z-[100] bg-black/90 flex items-center justify-center cursor-pointer"
        @click="fullscreenMap = null"
      >
        <img
          :src="maps.find(m => m.name === fullscreenMap)?.image_url"
          :alt="fullscreenMap"
          class="max-w-full max-h-full object-contain"
        />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.map-fs-enter-active,
.map-fs-leave-active {
  transition: opacity 0.2s ease;
}
.map-fs-enter-from,
.map-fs-leave-to {
  opacity: 0;
}
</style>
