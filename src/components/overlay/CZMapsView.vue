<script setup lang="ts">
import { ref, onMounted } from "vue";
import { commands } from "@/bindings";
import type { CzMap } from "@/bindings";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";

const maps = ref<CzMap[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

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
  <div class="h-full overflow-y-auto px-4 py-3 space-y-4 select-none">
    <h2 class="text-sm font-semibold text-white/60 tracking-widest uppercase mb-3">
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

    <!-- Map list -->
    <template v-else>
      <div v-for="map in maps" :key="map.name" class="bg-[#1a1d24] border border-white/10 rounded-lg overflow-hidden">
        <div class="px-4 py-2.5 border-b border-white/5">
          <h3 class="text-xs font-semibold text-white/70 tracking-wider">{{ map.name }}</h3>
        </div>
        <img
          :src="map.image_url"
          :alt="map.name"
          class="w-full object-contain max-h-[500px]"
          loading="lazy"
        />
      </div>

      <!-- Attribution -->
      <div class="text-center text-[10px] text-white/20 pb-2">
        Maps by Terada &amp; u/Kerast · via
        <a href="https://contestedzonetimers.com/contested-zone-maps" target="_blank" class="underline hover:text-white/40">
          contestedzonetimers.com
        </a>
      </div>
    </template>
  </div>
</template>
