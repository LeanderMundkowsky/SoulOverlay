<script setup lang="ts">
import { computed, ref, watch } from "vue";
import AlertBanner from "@/components/ui/AlertBanner.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import IconPackage from "@/components/icons/IconPackage.vue";
import { useHangarStore } from "@/stores/hangar";
import { useDetailsStore } from "@/stores/details";
import { useInventoryStore } from "@/stores/inventory";
import { useBackendStore } from "@/stores/backend";
import { proxyImageUrl } from "@/utils/imageProxy";

const hangarStore = useHangarStore();
const detailsStore = useDetailsStore();
const inventoryStore = useInventoryStore();
const backendStore = useBackendStore();

const emit = defineEmits<{
  (e: "switchToInventory", locationId: string, locationName: string): void;
}>();

const importFeedback = ref<string | null>(null);

const canFetch = computed(() => backendStore.isLoggedIn);

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

async function importFromUex() {
  if (!canFetch.value) return;
  importFeedback.value = null;
  const result = await hangarStore.importFleet();
  if (result) {
    importFeedback.value = `Imported ${result.imported} ships (${result.created} new, ${result.updated} updated, ${result.removed} removed).`;
    setTimeout(() => { importFeedback.value = null; }, 5000);
  }
}

function viewVehicle(uexVehicleId: string, modelName: string) {
  detailsStore.openEntity({
    id: uexVehicleId,
    name: modelName,
    kind: "vehicle",
    slug: "",
    uuid: "",
    source: "uex",
  });
}

const shipsWithInventory = computed(() => {
  const ids = new Set<string>();
  for (const entry of inventoryStore.entries) {
    if (entry.location_slug === "fleet_vehicle") {
      ids.add(entry.location_id);
    }
  }
  return ids;
});

function viewShipInventory(shipId: number, modelName: string) {
  const locationId = `fleet_${shipId}`;
  const locationName = `[Ship] ${modelName}`;
  emit("switchToInventory", locationId, locationName);
}
</script>

<template>
  <div class="p-6 max-w-5xl mx-auto w-full space-y-4">
    <!-- Login warning -->
    <AlertBanner
      v-if="!backendStore.isLoggedIn"
      variant="warning"
      message="Log in to your SoulOverlay account to enable Hangar access."
    />

    <!-- API error -->
    <AlertBanner
      v-if="hangarStore.error"
      variant="error"
      :message="hangarStore.error"
    />

    <!-- Import feedback -->
    <AlertBanner
      v-if="importFeedback"
      variant="success"
      :message="importFeedback"
    />

    <!-- Header row -->
    <div v-if="canFetch" class="flex items-center justify-between">
      <h2 class="text-white/80 text-sm font-semibold uppercase tracking-wider">
        My Hangar
        <span v-if="hangarStore.fleet.length > 0" class="text-white/40 font-normal ml-2">
          ({{ hangarStore.fleet.length }})
        </span>
      </h2>
      <div class="flex items-center gap-2">
        <button
          @click="importFromUex"
          :disabled="hangarStore.importing || hangarStore.loading"
          class="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg bg-[#1e2028] text-white/50 hover:bg-[#272a33] hover:text-white/70 disabled:opacity-30 disabled:pointer-events-none border border-white/10 transition-colors"
        >
          <svg class="w-3.5 h-3.5" :class="{ 'animate-spin': hangarStore.importing }" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
          {{ hangarStore.importing ? "Importing..." : "Import from UEX" }}
        </button>
        <button
          @click="refresh"
          :disabled="hangarStore.loading || hangarStore.importing"
          class="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded-lg bg-[#172035] text-blue-400 hover:bg-[#1e2a48] hover:text-blue-300 disabled:opacity-30 disabled:pointer-events-none border border-blue-500/20 transition-colors"
        >
          <svg class="w-3.5 h-3.5" :class="{ 'animate-spin': hangarStore.loading }" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ hangarStore.loading ? "Loading..." : "Refresh" }}
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="(hangarStore.loading || hangarStore.importing) && hangarStore.fleet.length === 0" class="flex justify-center py-12">
      <LoadingSpinner />
    </div>

    <!-- Empty state -->
    <div
      v-if="canFetch && !hangarStore.loading && !hangarStore.importing && hangarStore.fleet.length === 0 && !hangarStore.error"
      class="text-center text-white/30 py-12 text-sm"
    >
      No ships in your fleet yet. Import from UEX to get started.
    </div>

    <!-- Fleet list -->
    <div v-if="hangarStore.fleet.length > 0" class="space-y-2">
      <div
        v-for="ship in hangarStore.fleet"
        :key="ship.id"
        class="bg-[#1a1d24] border border-white/10 rounded-lg px-4 py-3 hover:border-white/20 cursor-pointer transition-colors group"
        @click="viewVehicle(ship.uex_vehicle_id, ship.model_name)"
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

          <!-- Right: inventory button + arrow -->
          <div class="flex items-center gap-2 shrink-0 mt-1">
            <button
              v-if="shipsWithInventory.has(`fleet_${ship.id}`)"
              @click.stop="viewShipInventory(ship.id, ship.model_name)"
              class="flex items-center gap-1 text-xs px-2 py-1 rounded-lg text-blue-400/60 hover:text-blue-400 hover:bg-blue-400/10 transition-colors"
              title="View ship inventory"
            >
              <IconPackage class="w-3.5 h-3.5" />
              <span>Inventory</span>
            </button>
            <svg
              class="w-4 h-4 text-white/20 group-hover:text-white/50 transition-colors shrink-0"
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
  </div>
</template>
