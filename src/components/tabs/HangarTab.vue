<script setup lang="ts">
import { ref, watch } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import { useHangarStore } from "@/stores/hangar";
import { useSettingsStore } from "@/stores/settings";
import { useDetailsStore } from "@/stores/details";
import { proxyImageUrl } from "@/utils/imageProxy";

const hangarStore = useHangarStore();
const settingsStore = useSettingsStore();
const detailsStore = useDetailsStore();

const hasApiKey = ref(false);
const hasSecretKey = ref(false);
const canFetch = ref(false);

// Reactively track key availability — settings load async after mount
watch(
  () => settingsStore.settings,
  (s) => {
    hasApiKey.value = s.uex_api_key.length > 0;
    hasSecretKey.value = s.uex_secret_key.length > 0;
    canFetch.value = hasApiKey.value && hasSecretKey.value;
  },
  { immediate: true, deep: true },
);

// Auto-load fleet once both keys become available (immediate because settings load pre-mount)
watch(canFetch, (ready) => {
  if (ready && hangarStore.fleet.length === 0 && !hangarStore.loading) {
    hangarStore.loadFleet();
  }
}, { immediate: true });

function refresh() {
  if (canFetch.value) {
    hangarStore.loadFleet();
  }
}

function viewVehicle(idVehicle: string, modelName: string) {
  detailsStore.openEntity({
    id: idVehicle,
    name: modelName,
    kind: "vehicle",
    slug: "",
    uuid: "",
  });
}
</script>

<template>
  <div class="p-6 max-w-5xl mx-auto w-full space-y-4">
    <!-- Missing keys warnings -->
    <AlertBanner
      v-if="!hasApiKey"
      variant="warning"
      message="UEX API key not configured. Set it in Settings → UEX API Key. Required for hangar access."
    />
    <AlertBanner
      v-if="!hasSecretKey"
      variant="warning"
      message="UEX secret key not configured. Set it in Settings → UEX Secret Key. Required for hangar access."
    />

    <!-- API error -->
    <AlertBanner
      v-if="hangarStore.error"
      variant="error"
      :message="hangarStore.error"
    />

    <!-- Header row -->
    <div v-if="canFetch" class="flex items-center justify-between">
      <h2 class="text-white/80 text-sm font-semibold uppercase tracking-wider">
        My Hangar
        <span v-if="hangarStore.fleet.length > 0" class="text-white/40 font-normal ml-2">
          ({{ hangarStore.fleet.length }})
        </span>
      </h2>
      <button
        @click="refresh"
        :disabled="hangarStore.loading"
        class="text-xs text-blue-400 hover:text-blue-300 disabled:text-white/20 transition-colors"
      >
        <span v-if="hangarStore.stale" class="text-yellow-400 mr-1">⟳</span>
        {{ hangarStore.loading ? "Refreshing..." : "Refresh" }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="hangarStore.loading && hangarStore.fleet.length === 0" class="flex justify-center py-12">
      <LoadingSpinner />
    </div>

    <!-- Empty state -->
    <div
      v-if="canFetch && !hangarStore.loading && hangarStore.fleet.length === 0 && !hangarStore.error"
      class="text-center text-white/30 py-12 text-sm"
    >
      No ships found in your hangar.
    </div>

    <!-- Fleet list -->
    <div v-if="hangarStore.fleet.length > 0" class="space-y-2">
      <div
        v-for="ship in hangarStore.fleet"
        :key="ship.id"
        class="bg-[#1a1d24] border border-white/10 rounded-lg px-4 py-3 hover:border-white/20 cursor-pointer transition-colors group"
        @click="viewVehicle(ship.id_vehicle, ship.model_name)"
      >
        <div class="flex items-start justify-between gap-4">
          <!-- Left: ship photo + info -->
          <div class="flex items-start gap-3 min-w-0 flex-1">
            <img
              v-if="ship.url_photo"
              :src="proxyImageUrl(ship.url_photo)"
              :alt="ship.model_name"
              class="w-20 h-14 object-cover rounded-lg shrink-0 bg-white/5"
              @error="($event.target as HTMLImageElement).style.display = 'none'"
            />
            <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-white text-sm font-medium truncate">
                {{ ship.model_name }}
              </span>
              <span
                v-if="ship.is_pledged"
                class="text-xs px-1.5 py-0.5 rounded bg-yellow-500/20 text-yellow-400 shrink-0"
              >
                Pledged
              </span>
              <span
                v-if="ship.is_hidden"
                class="text-xs px-1.5 py-0.5 rounded bg-white/10 text-white/40 shrink-0"
              >
                Hidden
              </span>
            </div>
            <div v-if="ship.name && ship.name !== ship.model_name" class="text-white/50 text-xs mt-0.5 truncate">
              "{{ ship.name }}"
            </div>
            <div class="flex items-center gap-3 mt-1 text-white/30 text-xs">
              <span v-if="ship.serial">SN: {{ ship.serial }}</span>
              <span v-if="ship.organization_name">Org: {{ ship.organization_name }}</span>
            </div>
            </div>
          </div>

          <!-- Right: arrow indicator -->
          <svg
            class="w-4 h-4 text-white/20 group-hover:text-white/50 transition-colors shrink-0 mt-1"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>
