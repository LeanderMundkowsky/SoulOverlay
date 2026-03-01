<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useUex, type PriceEntry } from "@/composables/useUex";

const props = defineProps<{
  commodityId: string;
  commodityName: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { loading, error, prices, getPrices } = useUex();

// Sort state
const sortKey = ref<keyof PriceEntry>("sell_price");
const sortAsc = ref(false);

const sortedPrices = ref<PriceEntry[]>([]);

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
      const aStr = String(aVal);
      const bStr = String(bVal);
      return sortAsc.value
        ? aStr.localeCompare(bStr)
        : bStr.localeCompare(aStr);
    });
    sortedPrices.value = sorted;
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
  return val.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

onMounted(() => {
  getPrices(props.commodityId);
});

watch(
  () => props.commodityId,
  (newId) => {
    getPrices(newId);
  }
);
</script>

<template>
  <div class="bg-white/5 border border-white/10 rounded-lg overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-white/5 border-b border-white/10">
      <h2 class="text-white font-semibold text-sm">
        {{ commodityName }} — Prices
      </h2>
      <button
        @click="emit('close')"
        class="text-white/40 hover:text-white transition-colors"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="px-4 py-8 text-center text-white/40 text-sm">
      <div class="w-6 h-6 border-2 border-blue-400/30 border-t-blue-400 rounded-full animate-spin mx-auto mb-2"></div>
      Loading prices...
    </div>

    <!-- Error -->
    <div v-else-if="error" class="px-4 py-4 text-red-400 text-sm">
      {{ error }}
    </div>

    <!-- Table -->
    <div v-else-if="sortedPrices.length > 0" class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-white/50 text-xs uppercase tracking-wider">
            <th
              @click="toggleSort('location')"
              class="text-left px-4 py-2 cursor-pointer hover:text-white/80 transition-colors"
            >
              Location{{ sortIndicator("location") }}
            </th>
            <th
              @click="toggleSort('terminal')"
              class="text-left px-4 py-2 cursor-pointer hover:text-white/80 transition-colors"
            >
              Terminal{{ sortIndicator("terminal") }}
            </th>
            <th
              @click="toggleSort('buy_price')"
              class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors"
            >
              Buy{{ sortIndicator("buy_price") }}
            </th>
            <th
              @click="toggleSort('sell_price')"
              class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors"
            >
              Sell{{ sortIndicator("sell_price") }}
            </th>
            <th
              @click="toggleSort('scu_available')"
              class="text-right px-4 py-2 cursor-pointer hover:text-white/80 transition-colors"
            >
              SCU{{ sortIndicator("scu_available") }}
            </th>
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
            <td class="px-4 py-2 text-right text-green-400">
              {{ formatPrice(entry.buy_price) }}
            </td>
            <td class="px-4 py-2 text-right text-blue-400">
              {{ formatPrice(entry.sell_price) }}
            </td>
            <td class="px-4 py-2 text-right text-white/50">
              {{ entry.scu_available !== null ? entry.scu_available : "-" }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Empty state -->
    <div v-else class="px-4 py-8 text-center text-white/40 text-sm">
      No price data available for this commodity.
    </div>
  </div>
</template>
