<script setup lang="ts">
import { ref, onMounted } from "vue";
import { commands } from "@/bindings";
import type { CzShip } from "@/bindings";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";

const ships = ref<CzShip[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

async function fetchShips() {
  loading.value = true;
  error.value = null;
  try {
    const result = await commands.czGetShips();
    if (result.status === "ok" && result.data.ok && result.data.data) {
      ships.value = result.data.data;
    } else {
      error.value = result.status === "ok" ? (result.data.error ?? "Unknown error") : result.error;
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(fetchShips);
</script>

<template>
  <div class="h-full overflow-y-auto px-4 py-3 space-y-4 select-none">
    <h2 class="text-sm font-semibold text-white/60 tracking-widest uppercase mb-3">
      Executive Hangar Ships
    </h2>

    <!-- Loading -->
    <div v-if="loading" class="flex flex-col items-center py-12">
      <LoadingSpinner />
      <p class="text-white/40 text-xs mt-2">Loading ships...</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-center py-8">
      <p class="text-red-400 text-sm">{{ error }}</p>
      <button
        class="mt-2 px-3 py-1 text-xs bg-white/10 hover:bg-white/20 rounded transition-colors"
        @click="fetchShips"
      >
        Retry
      </button>
    </div>

    <!-- Ship list -->
    <template v-else>
      <div v-for="ship in ships" :key="ship.name" class="bg-[#1a1d24] border border-white/10 rounded-lg overflow-hidden">
        <img
          :src="ship.image_url"
          :alt="ship.name"
          class="w-full object-contain max-h-[400px]"
          loading="lazy"
        />
        <div class="px-4 py-3 border-t border-white/5">
          <h3 class="text-sm font-bold text-white">{{ ship.name }}</h3>
          <p v-if="ship.ship_type" class="text-xs text-white/50 italic mt-0.5">
            {{ ship.ship_type }}
          </p>
          <div class="flex gap-3 mt-2">
            <a
              v-if="ship.wiki_url"
              :href="ship.wiki_url"
              target="_blank"
              class="text-[10px] text-blue-400/60 hover:text-blue-400 underline transition-colors"
            >
              Wiki ↗
            </a>
            <a
              v-if="ship.pledge_url"
              :href="ship.pledge_url"
              target="_blank"
              class="text-[10px] text-blue-400/60 hover:text-blue-400 underline transition-colors"
            >
              Pledge ↗
            </a>
          </div>
          <p v-if="ship.credit" class="text-[10px] text-amber-400/50 mt-1">
            {{ ship.credit }}
          </p>
        </div>
      </div>

      <!-- Attribution -->
      <div class="text-center text-[10px] text-white/20 pb-2">
        Ship data from
        <a href="https://contestedzonetimers.com/ships" target="_blank" class="underline hover:text-white/40">
          contestedzonetimers.com
        </a>
      </div>
    </template>
  </div>
</template>
