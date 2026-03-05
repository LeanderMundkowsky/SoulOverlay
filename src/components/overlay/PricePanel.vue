<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import IconClose from "@/components/icons/IconClose.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import CommodityPriceView from "@/components/overlay/CommodityPriceView.vue";
import SimplePriceView from "@/components/overlay/SimplePriceView.vue";
import { useUex } from "@/composables/useUex";
import type { PriceEntry } from "@/bindings";

const props = defineProps<{
  entityId: string;
  entityName: string;
  entityKind: string;
  entitySlug?: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
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

function fetchPrices() {
  getEntityPrices(props.entityKind, props.entityId);
}

function hasData(): boolean {
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

    <!-- Rich data (commodity/raw_commodity) -->
    <CommodityPriceView
      v-else-if="hasData() && hasRichData"
      :buy-entries="buyEntries"
      :sell-entries="sellEntries"
      :entity-id="entityId"
      :entity-name="entityName"
      :entity-kind="entityKind"
      :entity-slug="entitySlug"
    />

    <!-- Simple data (vehicle/item/fuel) -->
    <SimplePriceView
      v-else-if="hasData()"
      :buy-entries="buyEntries"
      :sell-entries="sellEntries"
    />

    <!-- Empty state -->
    <div v-else-if="!loading" class="px-4 py-8 text-center text-white/40 text-sm">
      No price data available.
    </div>
  </div>
</template>
