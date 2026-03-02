<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import IconClose from "@/components/icons/IconClose.vue";
import LoadingSpinner from "@/components/ui/LoadingSpinner.vue";
import { useUex, type PriceEntry } from "@/composables/useUex";

const props = defineProps<{
  entityId: string;
  entityName: string;
  entityKind: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { loading, error, prices, getEntityPrices } = useUex();

const sortKey = ref<keyof PriceEntry>("sell_price");
const sortAsc = ref(false);
const sortedPrices = ref<PriceEntry[]>([]);

const showRentColumn = ref(false);
const showScuColumn = ref(false);

const bestBuy = ref(0);
const bestSell = ref(0);
const locationCount = ref(0);

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
  [prices, sortKey, sortAsc],
  () => {
    const sorted = [...prices.value];
    sorted.sort((a, b) => {
      const aVal = a[sortKey.value] ?? 0;
      const bVal = b[sortKey.value] ?? 0;
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortAsc.value ? aVal - bVal : bVal - aVal;
      }
      return sortAsc.value
        ? String(aVal).localeCompare(String(bVal))
        : String(bVal).localeCompare(String(aVal));
    });
    sortedPrices.value = sorted;

    showRentColumn.value = sorted.some((p) => p.rent_price > 0);
    showScuColumn.value = sorted.some((p) => p.scu_available !== null && p.scu_available > 0);

    const buys = sorted.map((p) => p.buy_price).filter((v) => v > 0);
    const sells = sorted.map((p) => p.sell_price).filter((v) => v > 0);
    bestBuy.value = buys.length > 0 ? Math.min(...buys) : 0;
    bestSell.value = sells.length > 0 ? Math.max(...sells) : 0;
    locationCount.value = new Set(sorted.map((p) => p.terminal_id)).size;
  },
  { immediate: true }
);

function toggleSort(key: keyof PriceEntry) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value;
  } else {
    sortKey.value = key;
    sortAsc.value = false;
  }
}

function sortIndicator(key: keyof PriceEntry): string {
  if (sortKey.value !== key) return "";
  return sortAsc.value ? " ^" : " v";
}

function formatPrice(val: number): string {
  if (val === 0) return "-";
  return val.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function fetchPrices() {
  getEntityPrices(props.entityKind, props.entityId);
}

onMounted(() => { fetchPrices(); });
watch(() => props.entityId, () => { fetchPrices(); });
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-white/5 border-b border-white/10">
      <h2 class="text-white font-semibold text-sm">{{ entityName }} — Prices</h2>
      <button @click="emit('close')" class="text-white/40 hover:text-white transition-colors">
        <IconClose class="w-4 h-4" />
      </button>
    </div>

    <!-- Item Info -->
    <div v-if="!loading && !error && sortedPrices.length > 0" class="px-4 py-2 bg-white/[0.03] border-b border-white/10 flex items-center gap-3 text-xs">
      <span class="px-2 py-0.5 rounded bg-white/10 text-white/70 font-medium uppercase tracking-wide">{{ kindLabels[entityKind] ?? entityKind }}</span>
      <span v-if="bestBuy > 0" class="text-white/50">Best Buy <span class="text-green-400 font-medium">{{ formatPrice(bestBuy) }}</span></span>
      <span v-if="bestSell > 0" class="text-white/50">Best Sell <span class="text-blue-400 font-medium">{{ formatPrice(bestSell) }}</span></span>
      <span class="text-white/50">{{ locationCount }} {{ locationCount === 1 ? "location" : "locations" }}</span>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="px-4 py-8 flex justify-center">
      <LoadingSpinner text="Loading prices..." />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="px-4 py-4 text-red-400 text-sm">{{ error }}</div>

    <!-- Table -->
    <div v-else-if="sortedPrices.length > 0" class="overflow-x-auto overflow-y-auto flex-1">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-white/50 text-xs uppercase tracking-wider sticky top-0 bg-[#1a1d24]">
            <th @click="toggleSort('location')"  class="text-left px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Location{{ sortIndicator("location") }}</th>
            <th @click="toggleSort('terminal')"  class="text-left px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Terminal{{ sortIndicator("terminal") }}</th>
            <th @click="toggleSort('buy_price')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Buy{{ sortIndicator("buy_price") }}</th>
            <th @click="toggleSort('sell_price')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Sell{{ sortIndicator("sell_price") }}</th>
            <th v-if="showRentColumn" @click="toggleSort('rent_price')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">Rent{{ sortIndicator("rent_price") }}</th>
            <th v-if="showScuColumn" @click="toggleSort('scu_available')" class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors">SCU{{ sortIndicator("scu_available") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(entry, idx) in sortedPrices"
            :key="idx"
            class="border-t border-white/5 hover:bg-white/5 transition-colors"
          >
            <td class="px-4 py-2 text-white/80">{{ entry.location }}</td>
            <td class="px-4 py-2 text-white/60">{{ entry.terminal }}</td>
            <td class="px-4 py-2 text-right text-green-400">{{ formatPrice(entry.buy_price) }}</td>
            <td class="px-4 py-2 text-right text-blue-400">{{ formatPrice(entry.sell_price) }}</td>
            <td v-if="showRentColumn" class="px-4 py-2 text-right text-yellow-400">{{ formatPrice(entry.rent_price) }}</td>
            <td v-if="showScuColumn" class="px-4 py-2 text-right text-white/50">{{ entry.scu_available !== null ? entry.scu_available : "-" }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="px-4 py-8 text-center text-white/40 text-sm">
      No price data available.
    </div>
  </div>
</template>
