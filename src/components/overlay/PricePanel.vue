<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import IconClose from "@/components/icons/IconClose.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import CommodityPriceView from "@/components/overlay/CommodityPriceView.vue";
import SimplePriceView from "@/components/overlay/SimplePriceView.vue";
import LocationTerminalsView from "@/components/overlay/LocationTerminalsView.vue";
import { useUex } from "@/composables/useUex";
import type { PriceEntry } from "@/bindings";

interface PinnedLocation {
  id: string;
  name: string;
  kind: string;
  slug?: string;
}

const props = defineProps<{
  entityId: string;
  entityName: string;
  entityKind: string;
  entitySlug?: string;
  pinnedLocation?: PinnedLocation | null;
  active?: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "select-entity", entity: { id: string; name: string; kind: string; slug: string }): void;
}>();

const { loading, error, prices, getEntityPrices } = useUex();

const buyEntries = ref<PriceEntry[]>([]);
const sellEntries = ref<PriceEntry[]>([]);
const hasRichData = ref(false);

const kindLabels: Record<string, string> = {
  commodity: "Commodity",
  raw_commodity: "Raw Commodity",
  item: "Item",
  vehicle: "Vehicle",
  "ground vehicle": "Ground Vehicle",
  vehicle_rental: "Vehicle Rental",
  fuel: "Fuel",
  location: "Location",
};

watch(
  prices,
  () => {
    const all = [...prices.value];
    buyEntries.value = all.filter((p) => p.buy_price > 0);
    sellEntries.value = all.filter((p) => p.sell_price > 0);
    hasRichData.value = props.entityKind === "commodity" || props.entityKind === "raw_commodity";
  },
  { immediate: true }
);

function matchesPinnedLocation(entry: PriceEntry, pin: PinnedLocation): boolean {
  const pinName = pin.name.replace(/^\[.*?\]\s*/, "").trim().toLowerCase();
  switch (pin.slug) {
    case "terminal":
      return entry.terminal_id === pin.id;
    case "star_system":
      return (entry.system ?? "").toLowerCase() === pinName;
    case "planet":
    case "moon":
      return entry.location.toLowerCase().includes(pinName);
    case "orbit":
      return (entry.orbit ?? "").toLowerCase().includes(pinName);
    case "space_station":
    case "outpost":
    case "city":
    case "poi":
      return entry.terminal.toLowerCase().includes(pinName) ||
             entry.location.toLowerCase().includes(pinName);
    default:
      return true;
  }
}

const filteredBuyEntries = computed(() => {
  if (!props.pinnedLocation) return buyEntries.value;
  return buyEntries.value.filter(e => matchesPinnedLocation(e, props.pinnedLocation!));
});

const filteredSellEntries = computed(() => {
  if (!props.pinnedLocation) return sellEntries.value;
  return sellEntries.value.filter(e => matchesPinnedLocation(e, props.pinnedLocation!));
});

const pinnedDisplayName = computed(() => {
  if (!props.pinnedLocation) return "";
  return props.pinnedLocation.name.replace(/^\[.*?\]\s*/, "");
});

const isLocationView = computed(() => {
  return props.entityKind === "location" && props.entitySlug !== "terminal"
    && props.entitySlug !== "faction" && props.entitySlug !== "company";
});

function fetchPrices() {
  if (!isLocationView.value) {
    getEntityPrices(props.entityKind, props.entityId);
  }
}

function hasData(): boolean {
  return filteredBuyEntries.value.length > 0 || filteredSellEntries.value.length > 0;
}

function hasUnfilteredData(): boolean {
  return buyEntries.value.length > 0 || sellEntries.value.length > 0;
}

onMounted(() => { fetchPrices(); });
watch(() => props.entityId, () => { fetchPrices(); });
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-white/5 border-b border-white/10">
      <div class="flex items-center gap-2 min-w-0">
        <span class="px-1.5 py-0.5 rounded bg-white/10 text-white/70 text-xs font-medium uppercase tracking-wide shrink-0">{{ kindLabels[entityKind] ?? entityKind }}</span>
        <h2 class="text-white font-semibold text-sm truncate">{{ entityName }}</h2>
      </div>
      <button @click="emit('close')" class="text-white/40 hover:text-white transition-colors shrink-0 ml-2">
        <IconClose class="w-4 h-4" />
      </button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="px-4 py-8 flex justify-center">
      <LoadingSpinner text="Loading prices..." />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="px-4 py-4 text-red-400 text-sm">{{ error }}</div>

    <!-- Location terminals view (non-terminal locations) -->
    <LocationTerminalsView
      v-else-if="isLocationView"
      :entity-id="entityId"
      :entity-slug="entitySlug ?? ''"
      @select-terminal="(t) => emit('select-entity', t)"
    />

    <!-- Rich data (commodity/raw_commodity) -->
    <CommodityPriceView
      v-else-if="hasData() && hasRichData"
      :buy-entries="filteredBuyEntries"
      :sell-entries="filteredSellEntries"
      :entity-id="entityId"
      :entity-name="entityName"
      :entity-kind="entityKind"
      :entity-slug="entitySlug"
      :active="active"
    />

    <!-- Simple data (vehicle/item/fuel) -->
    <SimplePriceView
      v-else-if="hasData()"
      :buy-entries="filteredBuyEntries"
      :sell-entries="filteredSellEntries"
    />

    <!-- No results after pin filtering -->
    <div v-else-if="!loading && hasUnfilteredData() && pinnedLocation" class="px-4 py-8 text-center text-white/40 text-sm">
      No prices found at <span class="text-green-300">{{ pinnedDisplayName }}</span>
    </div>

    <!-- Empty state -->
    <div v-else-if="!loading" class="px-4 py-8 text-center text-white/40 text-sm">
      No price data available.
    </div>
  </div>
</template>
